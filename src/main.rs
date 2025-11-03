#![allow(unexpected_cfgs)]

use std::{panic, process};

#[macro_use]
extern crate lazy_static;

mod config;
mod driver;
mod err;
mod handlers;
mod mqtt;

/// Attempts emergency shutoff using try_lock to avoid deadlock.
/// Returns Ok if shutoff succeeded, Err if lock could not be acquired.
pub fn try_emergency_shutoff() -> Result<(), String> {
    match driver::get_driver().try_lock() {
        Ok(mut driver) => {
            driver.shutoff_all_valves();
            Ok(())
        }
        Err(_) => {
            Err("Could not acquire driver lock".to_string())
        }
    }
}

fn synchronous_shutdown() {
    if let Err(e) = try_emergency_shutoff() {
        eprintln!("Warning: {}. Mutex may be poisoned or held by panicking thread.", e);
    }
}

fn set_cleanup_on_exit() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        synchronous_shutdown();
        process::exit(1);
    }));

    let _ = ctrlc::set_handler(move || {
        synchronous_shutdown();
        process::exit(0);
    });
}

#[tokio::main]
async fn main() {
    // TODO: better error handling on this entire method
    // read config either from the first parameter of the program or config.yaml by default
    let config_file = std::env::args()
        .nth(1)
        .unwrap_or(String::from("config.yaml"));
    let config = config::read_config(config_file).unwrap_or_else(|e| {
        println!("Error reading config: {:?}", e);
        process::exit(1);
    });

    set_cleanup_on_exit();
    driver::init_driver(&config).await;

    let mut mqtt_client = mqtt::MQTTClient::new(&config).await.unwrap();

    let mut current_task_handle = None;

    loop {
        let message = mqtt_client.next().await.unwrap();
        let topic = message.topic();
        let payload_str = message.payload_str();

        println!("Received message: {} -> {}", topic, payload_str);

        handlers::handle_message(&mut current_task_handle, &config, topic, &payload_str).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_emergency_shutoff_succeeds_when_lock_available() {
        // Initialize driver with noop driver (safe for testing)
        let config = config::Configuration {
            mqtt: config::MQTTConfig {
                broker: "localhost".to_string(),
                port: 1883,
                client_id: "test".to_string(),
                topic_prefix: "test".to_string(),
            },
            driver: config::Driver::Noop,
            pump: config::PumpConfig { pin: 0, delay: 1 },
            zones: vec![],
            plans: vec![],
        };

        // This is a bit hacky but needed since init_driver is async
        // We rely on lazy_static initializing with the configured driver
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                driver::init_driver(&config).await;
            });

        // Should succeed when no one holds the lock
        let result = try_emergency_shutoff();
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_emergency_shutoff_fails_when_lock_held() {
        // Initialize driver
        let config = config::Configuration {
            mqtt: config::MQTTConfig {
                broker: "localhost".to_string(),
                port: 1883,
                client_id: "test".to_string(),
                topic_prefix: "test".to_string(),
            },
            driver: config::Driver::Noop,
            pump: config::PumpConfig { pin: 0, delay: 1 },
            zones: vec![],
            plans: vec![],
        };

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                driver::init_driver(&config).await;
            });

        // Hold the lock
        let _guard = driver::get_driver().try_lock().unwrap();

        // Should fail when lock is held
        let result = try_emergency_shutoff();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Could not acquire driver lock");
    }
}
