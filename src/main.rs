use std::{panic, process, time::Duration};

mod config;
mod err;
mod handlers;
mod mqtt;

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
    let cfg = config.clone();
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        shutoff_all_valves(&cfg);
        process::exit(1);
    }));

    let cfg = config.clone();
    let _ = ctrlc::set_handler(move || {
        shutoff_all_valves(&cfg);
        process::exit(0);
    });
}

fn shutoff_all_valves(config: &config::Configuration) {
    set_pin(config.pump.pin, false);
    for zone in &config.zones {
        set_pin(zone.pin, false);
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

        match topic {
            t if t.starts_with("sprinklers/start_plan/") => {
                handlers::start_plan::handle_start_plan(
                    &mut current_task_handle,
                    &config,
                    &topic,
                    &payload_str,
                )
                .await
            }
            t if t.starts_with("sprinklers/water_zone/") => {
                handlers::water_zone::handle_water_zone(
                    &mut current_task_handle,
                    &config,
                    topic,
                    &payload_str,
                )
                .await
            }
            "sprinklers/stop" => {
                handlers::stop_plan::handle_stop_plan(
                    &mut current_task_handle,
                    &config,
                    &topic,
                    &payload_str,
                )
                .await
            }
            _ => {}
        }
    }
}
