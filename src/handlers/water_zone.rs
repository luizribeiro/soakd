use crate::config;
use crate::driver;
use futures::stream::AbortHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct WaterZonePayload {
    zone: String,
    duration: u16,
}

pub async fn handle_message(
    current_task_handle: &mut Option<AbortHandle>,
    config: &config::Configuration,
    _topic: &str,
    payload: &str,
) {
    if current_task_handle.is_some() {
        println!("Already have an ongoing sprinklers task. Ignoring.");
    }

    let payload: WaterZonePayload = serde_json::from_str(&payload).unwrap();
    let zone_config = config
        .zones
        .iter()
        .find(|z| z.zone == payload.zone)
        .unwrap();
    driver::activate_zone(&config.pump, &zone_config, payload.duration.into()).await;
}
