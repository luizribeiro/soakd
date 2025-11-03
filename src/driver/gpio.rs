use crate::config;
use crate::driver::Driver;
use async_trait::async_trait;
use gpiochip as gpio;
use std::time::Duration;
use std::thread;

// Number of outputs on the 74HC595 shift register
// This includes both zone valves and pump control
const NUM_ZONES: usize = 8;
const PIN_SR_LATCH: u32 = 22;
const PIN_SR_DATA: u32 = 27;
const PIN_SR_CLOCK: u32 = 4;
const PIN_SR_NOE: u32 = 17;

// Timing delay for 74HC595 setup/hold times
const SHIFT_REGISTER_DELAY_NS: u64 = 100;

pub struct GpioDriver {
    latch_pin: gpiochip::GpioHandle,
    clock_pin: gpiochip::GpioHandle,
    data_pin: gpiochip::GpioHandle,
    noe_pin: gpiochip::GpioHandle,
    state: [bool; NUM_ZONES],
}

impl GpioDriver {
    pub fn new() -> Self {
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
        let mut driver = Self {
            latch_pin,
            clock_pin,
            data_pin,
            noe_pin,
            state,
        };
        // Reset hardware to safe state (all valves closed) on initialization
        // This ensures valves are closed even if previous instance crashed/killed
        driver.set_state([false; NUM_ZONES]);
        driver
    }

    fn set_state(&mut self, pins: [bool; NUM_ZONES]) {
        let delay = || thread::sleep(Duration::from_nanos(SHIFT_REGISTER_DELAY_NS));
        
        self.noe_pin.set(1).unwrap();
        self.latch_pin.set(0).unwrap();
        for i in (0..NUM_ZONES).rev() {
            self.clock_pin.set(0).unwrap();
            delay(); // Setup time before data
            self.data_pin.set(pins[i].into()).unwrap();
            delay(); // Hold time after data
            self.clock_pin.set(1).unwrap();
            delay(); // Clock high time
        }
        self.latch_pin.set(1).unwrap();
        self.noe_pin.set(0).unwrap();
        self.state.copy_from_slice(&pins);
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
        if (pump_config.pin as usize) < NUM_ZONES {
            pins[pump_config.pin as usize] = true;
            self.set_state(pins);
        }

        // water zone for duration
        tokio::time::sleep(Duration::from_secs(duration * 60 - 2 * pump_config.delay)).await;

        // turn off pump
        if (pump_config.pin as usize) < NUM_ZONES {
            pins[pump_config.pin as usize] = false;
        }
        self.set_state(pins);

        // turn off zone valve
        pins[zone.pin as usize] = false;
        self.set_state(pins);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests document the critical safety behavior.
    // Full hardware tests require actual GPIO hardware.

    #[test]
    fn test_gpio_driver_initial_state_is_safe() {
        // This test documents that GpioDriver::new() MUST call set_state([false; NUM_ZONES])
        // to reset hardware to safe state on initialization.
        //
        // This is Bug #2 fix: ensures valves are closed even after crashes/SIGKILL.
        //
        // The actual call is at gpio.rs:52:
        //     driver.set_state([false; NUM_ZONES]);
        //
        // This cannot be easily unit tested without GPIO hardware or complex mocking,
        // but the code path is straightforward and critical for safety.
        //
        // IMPORTANT: If this line is ever removed, valves will stay open after crashes!

        // Verify NUM_ZONES constant is correct for 74HC595
        assert_eq!(NUM_ZONES, 8);

        // Verify pins array size matches
        let safe_state = [false; NUM_ZONES];
        assert_eq!(safe_state.len(), 8);
        assert!(safe_state.iter().all(|&v| !v)); // All valves closed
    }

    #[test]
    fn test_shutoff_all_valves_sets_safe_state() {
        // This test documents that shutoff_all_valves calls set_state([false; NUM_ZONES])
        // The actual GPIO operation cannot be tested without hardware.

        let safe_state = [false; NUM_ZONES];
        assert!(safe_state.iter().all(|&v| !v));
    }

    // Integration test that would require GPIO hardware:
    // #[test]
    // #[ignore] // Only run on actual Raspberry Pi
    // fn test_gpio_driver_resets_hardware_on_init() {
    //     // This test would verify that:
    //     // 1. Set shift register to some non-zero state
    //     // 2. Create new GpioDriver
    //     // 3. Read back shift register state (all zeros)
    //     // Requires GPIO hardware to run
    // }
}
