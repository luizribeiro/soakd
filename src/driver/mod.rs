use crate::config;
use async_trait::async_trait;
use tokio::sync::Mutex;

mod gpio;
mod noop;

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

lazy_static! {
    //static ref DRIVER: Mutex<Box<dyn Driver + Send>> = Mutex::new(Box::new(noop::NoopDriver {}));
    static ref DRIVER: Mutex<Box<dyn Driver + Send>> = Mutex::new(Box::new(gpio::GpioDriver::new()));
}

pub fn get_driver() -> &'static Mutex<Box<dyn Driver + Send>> {
    &DRIVER
}
