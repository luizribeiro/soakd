use crate::config;
use std::time::Duration;

pub fn shutoff_all_valves(config: &config::Configuration) {
    set_pin(config.pump.pin, false);
    for zone in &config.zones {
        set_pin(zone.pin, false);
    }
}

pub async fn activate_zone(
    pump_config: &config::PumpConfig,
    zone: &config::ZoneConfig,
    duration: u64,
) {
    set_pin(zone.pin, true);
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    set_pin(pump_config.pin, true);
    tokio::time::sleep(Duration::from_secs(duration * 60 - 2 * pump_config.delay)).await;
    set_pin(pump_config.pin, false);
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    set_pin(zone.pin, false);
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
