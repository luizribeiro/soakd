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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_driver_shutoff() {
        let mut driver = NoopDriver {};
        // Should not panic
        driver.shutoff_all_valves();
    }

    #[tokio::test]
    async fn test_noop_driver_activate_zone() {
        let mut driver = NoopDriver {};
        let pump_config = config::PumpConfig { pin: 0, delay: 1 };
        let zone_config = config::ZoneConfig {
            zone: "test".to_string(),
            pin: 1
        };
        // Should not panic
        driver.activate_zone(&pump_config, &zone_config, 1).await;
    }

    #[tokio::test]
    async fn test_noop_driver_multiple_zones() {
        let mut driver = NoopDriver {};
        let pump_config = config::PumpConfig { pin: 0, delay: 1 };

        // Test activating multiple zones in sequence
        for i in 0..8 {
            let zone_config = config::ZoneConfig {
                zone: format!("zone_{}", i),
                pin: i
            };
            driver.activate_zone(&pump_config, &zone_config, 1).await;
        }

        // All zones should shut off without panic
        driver.shutoff_all_valves();
    }
}
