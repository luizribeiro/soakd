use crate::config;
use gpiochip as gpio;
use std::time::Duration;

const PIN_SR_LATCH: u32 = 3;
const PIN_SR_DATA: u32 = 21;
const PIN_SR_CLOCK: u32 = 22;
const PIN_SR_NOE: u32 = 1;

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
    let chip = gpio::GpioChip::new("/dev/gpiochip0").unwrap();
    let latch_pin = chip
        .request("sr_latch", gpio::RequestFlags::OUTPUT, PIN_SR_LATCH, 0)
        .unwrap();
    let clock_pin = chip
        .request("sr_clock", gpio::RequestFlags::OUTPUT, PIN_SR_CLOCK, 0)
        .unwrap();
    let data_pin = chip
        .request("sr_data", gpio::RequestFlags::OUTPUT, PIN_SR_DATA, 0)
        .unwrap();
    let noe_pin = chip
        .request("sr_noe", gpio::RequestFlags::OUTPUT, PIN_SR_NOE, 0)
        .unwrap();

    // turn off the latch pin
    latch_pin.set(0).unwrap();
    clock_pin.set(0).unwrap();

    for i in 0..16 {
        clock_pin.set(0).unwrap();
        data_pin.set(pins[i].into()).unwrap();
        clock_pin.set(1).unwrap();
    }

    // latch the outputs
    latch_pin.set(1).unwrap();

    // turn off the NOT enable pin (turns on outputs)
    noe_pin.set(0).unwrap();
}
