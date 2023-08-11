use crate::config;
use rust_gpiozero::OutputDevice;
use std::time::Duration;

const PIN_SR_LATCH: u8 = 3;
const PIN_SR_DATA: u8 = 21;
const PIN_SR_CLOCK: u8 = 22;
const PIN_SR_NOE: u8 = 1;

pub fn shutoff_all_valves(_config: &config::Configuration) {
    set_state([false; 16]);
}

pub async fn activate_zone(
    pump_config: &config::PumpConfig,
    zone: &config::ZoneConfig,
    duration: u64,
) {
    let mut pins = [false; 16];

    // turn on zone valve
    pins[zone.pin as usize] = true;
    set_state(pins);

    // turn on pump after a bit
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    pins[pump_config.pin as usize] = true;
    set_state(pins);

    // water zone for duration
    tokio::time::sleep(Duration::from_secs(duration * 60 - 2 * pump_config.delay)).await;

    // turn off pump
    pins[pump_config.pin as usize] = false;
    set_state(pins);

    // turn off zone valve after a bit
    tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
    pins[zone.pin as usize] = false;
    set_state(pins);
}

fn set_state(pins: [bool; 16]) {
    let mut latch_pin = OutputDevice::new(PIN_SR_LATCH);
    let mut clock_pin = OutputDevice::new(PIN_SR_CLOCK);
    let mut data_pin = OutputDevice::new(PIN_SR_DATA);
    let mut noe_pin = OutputDevice::new(PIN_SR_NOE);

    // turn off the latch pin
    latch_pin.off();
    clock_pin.off();

    for i in 0..16 {
        clock_pin.off();
        if pins[i] {
            data_pin.on();
        } else {
            data_pin.off();
        }
        clock_pin.on();
    }

    // latch the outputs
    latch_pin.on();

    // turn off the NOT enable pin (turns on outputs)
    noe_pin.off();
}
