use crate::config;
use async_trait::async_trait;
use gpiochip as gpio;
use std::time::Duration;
use tokio::sync::Mutex;

const NUM_ZONES: usize = 8;
const PIN_SR_LATCH: u32 = 22;
const PIN_SR_DATA: u32 = 27;
const PIN_SR_CLOCK: u32 = 4;
const PIN_SR_NOE: u32 = 17;

#[async_trait]
pub trait Driver {
    fn shutoff_all_valves(&mut self);
    async fn activate_zone(
        &mut self,
        pump_config: &config::PumpConfig,
        zone: &config::ZoneConfig,
        duration: u64,
    );
}

pub struct NoopDriver {}

#[async_trait]
impl Driver for NoopDriver {
    fn shutoff_all_valves(&mut self) {}
    async fn activate_zone(
        &mut self,
        _pump_config: &config::PumpConfig,
        _zone: &config::ZoneConfig,
        _duration: u64,
    ) {
    }
}

lazy_static! {
    //static ref DRIVER: Mutex<Box<dyn Driver + Send>> = Mutex::new(Box::new(NoopDriver {}));
    static ref DRIVER: Mutex<Box<dyn Driver + Send>> = Mutex::new(Box::new(GpioDriver::new()));
}

pub fn get_driver() -> &'static Mutex<Box<dyn Driver + Send>> {
    &DRIVER
}

pub struct GpioDriver {
    latch_pin: gpiochip::GpioHandle,
    clock_pin: gpiochip::GpioHandle,
    data_pin: gpiochip::GpioHandle,
    noe_pin: gpiochip::GpioHandle,
    state: [bool; NUM_ZONES],
}

impl GpioDriver {
    fn new() -> Self {
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
        let state = [false; NUM_ZONES];
        Self {
            latch_pin,
            clock_pin,
            data_pin,
            noe_pin,
            state,
        }
    }

    fn set_state(&mut self, pins: [bool; NUM_ZONES]) {
        self.noe_pin.set(1).unwrap();
        self.latch_pin.set(0).unwrap();
        for i in (0..NUM_ZONES).rev() {
            self.clock_pin.set(0).unwrap();
            self.data_pin.set(pins[i].into()).unwrap();
            self.clock_pin.set(1).unwrap();
        }
        self.latch_pin.set(1).unwrap();
        self.noe_pin.set(0).unwrap();
        for i in 0..NUM_ZONES {
            self.state[i] = pins[i];
        }
    }
}

#[async_trait]
impl Driver for GpioDriver {
    fn shutoff_all_valves(&mut self) {
        self.set_state([false; NUM_ZONES]);
    }

    async fn activate_zone(
        &mut self,
        pump_config: &config::PumpConfig,
        zone: &config::ZoneConfig,
        duration: u64,
    ) {
        let mut pins = [false; NUM_ZONES];

        // turn on zone valve
        pins[zone.pin as usize] = true;
        self.set_state(pins);

        // turn on pump after a bit
        tokio::time::sleep(Duration::from_secs(pump_config.delay)).await;
        pins[pump_config.pin as usize] = true;
        self.set_state(pins);

        // water zone for duration
        tokio::time::sleep(Duration::from_secs(duration * 60 - 2 * pump_config.delay)).await;

        // turn off pump
        pins[pump_config.pin as usize] = false;
        self.set_state(pins);

        // turn off zone valve
        pins[zone.pin as usize] = false;
        self.set_state(pins);
    }
}
