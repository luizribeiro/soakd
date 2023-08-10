use futures::future::abortable;
use futures::StreamExt;
use paho_mqtt as mqtt;
use rust_gpiozero::*;
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, panic, path::Path, process, sync::Arc, time::Duration};

#[derive(Deserialize, Clone)]
struct Configuration {
    mqtt: MQTTConfig,
    pump: PumpConfig,
    zones: Vec<ZoneConfig>,
    plans: Vec<SprinklerPlan>,
}

#[derive(Deserialize, Clone)]
struct MQTTConfig {
    broker: String,
    port: u16,
}

#[derive(Deserialize, Clone, Copy)]
struct PumpConfig {
    pin: u8,
    delay: u64,
}

#[derive(Deserialize, Clone)]
struct ZoneConfig {
    zone: u8,
    pin: u8,
}

#[derive(Deserialize, Clone)]
struct SprinklerPlan {
    name: String,
    zone_durations: Vec<SprinklerZone>,
}

#[derive(Deserialize, Clone)]
struct SprinklerZone {
    zone: u8,
    duration: u16,
}

#[derive(Serialize, Deserialize, Clone)]
struct WaterZonePayload {
    duration: u16,
}

#[derive(Debug)]
enum SprinklerError {
    YAML(serde_yaml::Error),
    IO(std::io::Error),
}

impl From<serde_yaml::Error> for SprinklerError {
    fn from(error: serde_yaml::Error) -> Self {
        SprinklerError::YAML(error)
    }
}

impl From<std::io::Error> for SprinklerError {
    fn from(error: std::io::Error) -> Self {
        SprinklerError::IO(error)
    }
}

fn read_config<P: AsRef<Path>>(path: P) -> Result<Configuration, SprinklerError> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

async fn activate_zone(pump_config: Arc<PumpConfig>, zone: ZoneConfig, duration: u64) {
    let mut output_device = OutputDevice::new(zone.pin.into());
    let mut pump_device = OutputDevice::new(pump_config.pin.into());
    output_device.on();
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    pump_device.on();
    tokio::time::sleep(Duration::from_secs(duration * 60 - 2 * pump_config.delay)).await;
    pump_device.off();
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    output_device.off();
}

fn set_cleanup_on_exit(config: &Configuration) {
    let config = config.clone();
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        cleanup(&config);
        process::exit(1);
    }));
}

fn cleanup(config: &Configuration) {
    let mut pump_device = OutputDevice::new(config.pump.pin.into());
    pump_device.off();
    for zone in &config.zones {
        let mut output_device = OutputDevice::new(zone.pin.into());
        output_device.off();
    }
}

async fn start_mqtt_client(config: &Configuration) -> Result<mqtt::AsyncClient, std::io::Error> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(format!("tcp://{}:{}", config.mqtt.broker, config.mqtt.port))
        .client_id("sprinkler_controller")
        .finalize();

    let client = mqtt::AsyncClient::new(create_opts)?;

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    client.connect(conn_opts).await?;

    client.subscribe("sprinklers/#", 1).await?;

    Ok(client)
}

async fn start_plan(config: &Configuration, pump_config: Arc<PumpConfig>, plan: &SprinklerPlan) {
    for zone_duration in &plan.zone_durations {
        let zone_config = config
            .zones
            .iter()
            .find(|z| z.zone == zone_duration.zone)
            .unwrap();
        println!(
            "Activating zone {} for {} minutes",
            zone_config.zone, zone_duration.duration
        );
        activate_zone(
            pump_config.clone(),
            zone_config.clone(),
            zone_duration.duration.into(),
        )
        .await;
    }
}

#[tokio::main]
async fn main() {
    let config = read_config("config.yaml").unwrap_or_else(|e| {
        println!("Error reading config: {:?}", e);
        process::exit(1);
    });

    set_cleanup_on_exit(&config);

    let pump_config = Arc::new(config.pump);

    let mut mqtt_client = start_mqtt_client(&config).await.unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });
    let mut stream = mqtt_client.get_stream(25);

    let mut current_task_handle = None;

    while let Some(msg_opt) = stream.next().await {
        if let Some(message) = msg_opt {
            let topic = message.topic();
            let payload_str = message.payload_str();

            println!("Received message: {} -> {}", topic, payload_str);

            if topic.starts_with("sprinklers/start_plan/") {
                if current_task_handle.is_some() {
                    println!("Already have an ongoing sprinklers task. Ignoring.");
                }

                let plan_name = &topic["sprinklers/start_plan/".len()..];
                let plan = config.plans.iter().find(|p| p.name == plan_name);

                if let Some(plan) = plan {
                    start_plan(&config, pump_config.clone(), plan).await;
                    let (task, handle) = abortable(async {
                        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
                    });
                    tokio::spawn(task);
                    current_task_handle = Some(handle);
                } else {
                    println!("Unknown plan: {}", plan_name);
                }
            } else if topic.starts_with("sprinklers/water_zone/") {
                let zone_number: u8 = payload_str.parse().unwrap();
                let zone_config = config.zones.iter().find(|z| z.zone == zone_number).unwrap();
                let payload: WaterZonePayload = serde_json::from_str(&payload_str).unwrap();
                activate_zone(
                    pump_config.clone(),
                    zone_config.clone(),
                    payload.duration.into(),
                )
                .await;
            } else if topic == "sprinklers/stop" {
                if let Some(handle) = current_task_handle {
                    println!("Force-stopping sprinklers");
                    handle.abort();
                    cleanup(&config);
                    current_task_handle = None;
                } else {
                    println!("No ongoing sprinklers task to stop.");
                }
            }
        }
    }
}
