use std::{panic, process};

#[macro_use]
extern crate lazy_static;

mod config;
mod driver;
mod err;
mod handlers;
mod mqtt;

fn await_synchronously<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn synchronous_shutdown() {
    await_synchronously(async {
        driver::get_driver().lock().await.shutoff_all_valves();
    });
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

    let mut mqtt_client = mqtt::MQTTClient::new(&config).await.unwrap();

    let mut current_task_handle = None;

    loop {
        let message = mqtt_client.next().await.unwrap();
        let topic = message.topic();
        let payload_str = message.payload_str();

        println!("Received message: {} -> {}", topic, payload_str);

        handlers::handle_message(&mut current_task_handle, &config, &topic, &payload_str).await;
    }
}
