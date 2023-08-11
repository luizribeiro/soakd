use std::{panic, process};

mod config;
mod driver;
mod err;
mod handlers;
mod mqtt;

fn set_cleanup_on_exit(config: &config::Configuration) {
    let cfg = config.clone();
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        driver::shutoff_all_valves(&cfg);
        process::exit(1);
    }));

    let cfg = config.clone();
    let _ = ctrlc::set_handler(move || {
        driver::shutoff_all_valves(&cfg);
        process::exit(0);
    });
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
                handlers::start_plan::handle_message(
                    &mut current_task_handle,
                    &config,
                    &topic,
                    &payload_str,
                )
                .await
            }
            t if t.starts_with("sprinklers/water_zone/") => {
                handlers::water_zone::handle_message(
                    &mut current_task_handle,
                    &config,
                    topic,
                    &payload_str,
                )
                .await
            }
            "sprinklers/stop" => {
                handlers::stop_plan::handle_message(
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
