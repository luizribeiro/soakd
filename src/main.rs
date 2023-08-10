use futures::future::abortable;
use serde::{Deserialize, Serialize};
use std::{panic, process, time::Duration};

mod config;
mod err;
mod mqtt;

#[derive(Serialize, Deserialize, Clone)]
pub struct WaterZonePayload {
    duration: u16,
}

fn set_pin(pin: u8, state: bool) {
    println!("Setting pin {} to {}", pin, state);
    /*
    let mut device = OutputDevice::new(pin.into());
    if state {
        device.on();
    } else {
        device.off();
    }
    */
}

async fn activate_zone(pump_config: &config::PumpConfig, zone: &config::ZoneConfig, duration: u64) {
    set_pin(zone.pin, true);
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    set_pin(pump_config.pin, true);
    tokio::time::sleep(Duration::from_secs(duration * 60 - 2 * pump_config.delay)).await;
    set_pin(pump_config.pin, false);
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    set_pin(zone.pin, false);
}

fn set_cleanup_on_exit(config: &config::Configuration) {
    let config = config.clone();
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        cleanup(&config);
        process::exit(1);
    }));
}

fn cleanup(config: &config::Configuration) {
    set_pin(config.pump.pin, false);
    for zone in &config.zones {
        set_pin(zone.pin, false);
    }
}

async fn start_plan(config: &config::Configuration, plan: &config::SprinklerPlan) {
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
        activate_zone(&config.pump, &zone_config, zone_duration.duration.into()).await;
        println!("Done watering zone {}", zone_config.zone);
    }
}

#[tokio::main]
async fn main() {
    // TODO: better error handling on this entire method
    let config = config::read_config("config.yaml").unwrap_or_else(|e| {
        println!("Error reading config: {:?}", e);
        process::exit(1);
    });

    set_cleanup_on_exit(&config);

    let mut mqtt_client = mqtt::MQTTClient::new(&config).await.unwrap();

    let mut current_task_handle = None;

    loop {
        let message = mqtt_client.next().await.unwrap();
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
                let config = config.clone();
                let plan = plan.clone();
                let (task, handle) = abortable(async move {
                    start_plan(&config, &plan).await;
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
            activate_zone(&config.pump, &zone_config, payload.duration.into()).await;
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
