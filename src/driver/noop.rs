use crate::config;
use crate::driver::Driver;
use async_trait::async_trait;

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
