use crate::config;
use crate::driver;
use crate::handlers::mqtt_handler;
use futures::{future::abortable, stream::AbortHandle};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct WaterZonePayload {
    zone: String,
    duration: u16,
}

#[mqtt_handler(topic = "+/water_zone/#")]
pub async fn handle_message(
    current_task_handle: &mut Option<AbortHandle>,
    config: &config::Configuration,
    _topic: &str,
    payload: &str,
) {
    if current_task_handle.is_some() {
        println!("Already have an ongoing sprinklers task. Ignoring.");
        return;
    }

    let payload: WaterZonePayload = serde_json::from_str(&payload).unwrap();
    let zone_config = config
        .zones
        .iter()
        .find(|z| z.zone == payload.zone)
        .cloned();

    if let Some(zone_config) = zone_config {
        let config = config.clone();
        let (task, handle) = abortable(async move {
            println!("Activating zone {} for {} minutes", zone_config.zone, payload.duration);
            driver::get_driver()
                .lock()
                .await
                .activate_zone(&config.pump, &zone_config, payload.duration.into())
                .await;
            println!("Done watering zone {}", zone_config.zone);
        });
        tokio::spawn(task);
        *current_task_handle = Some(handle);
    } else {
        println!("Unknown zone: {}", payload.zone);
    }
}
