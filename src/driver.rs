use crate::config;
use gpiochip as gpio;
use std::sync::Mutex;
use std::time::Duration;

const NUM_ZONES: usize = 8;
const PIN_SR_LATCH: u32 = 22;
const PIN_SR_DATA: u32 = 27;
const PIN_SR_CLOCK: u32 = 4;
const PIN_SR_NOE: u32 = 17;

lazy_static! {
    static ref CHIP: gpio::GpioChip = gpio::GpioChip::new("/dev/gpiochip0").unwrap();
    static ref LATCH_PIN: gpiochip::GpioHandle = CHIP
        .request("sr_latch", gpio::RequestFlags::OUTPUT, PIN_SR_LATCH, 0)
        .unwrap();
    static ref CLOCK_PIN: gpiochip::GpioHandle = CHIP
        .request("sr_clock", gpio::RequestFlags::OUTPUT, PIN_SR_CLOCK, 0)
        .unwrap();
    static ref DATA_PIN: gpiochip::GpioHandle = CHIP
        .request("sr_data", gpio::RequestFlags::OUTPUT, PIN_SR_DATA, 0)
        .unwrap();
    static ref NOE_PIN: gpiochip::GpioHandle = CHIP
        .request("sr_noe", gpio::RequestFlags::OUTPUT, PIN_SR_NOE, 0)
        .unwrap();
    static ref STATE: Mutex<[bool; NUM_ZONES]> = Mutex::new([false; NUM_ZONES]);
}

pub fn shutoff_all_valves(_config: &config::Configuration) {
    set_state([false; NUM_ZONES]);
}

pub async fn activate_zone(
    pump_config: &config::PumpConfig,
    zone: &config::ZoneConfig,
    duration: u64,
) {
    let mut pins = [false; NUM_ZONES];

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

fn set_state(pins: [bool; NUM_ZONES]) {
    NOE_PIN.set(1).unwrap();
    LATCH_PIN.set(0).unwrap();
    for i in (0..NUM_ZONES).rev() {
        CLOCK_PIN.set(0).unwrap();
        DATA_PIN.set(pins[i].into()).unwrap();
        CLOCK_PIN.set(1).unwrap();
    }
    LATCH_PIN.set(1).unwrap();
    NOE_PIN.set(0).unwrap();
    let mut state = STATE.lock().unwrap();
    for i in 0..NUM_ZONES {
        state[i] = pins[i];
    }
}
