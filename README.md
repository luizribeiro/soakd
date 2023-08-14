# 💧 soakd

A minimalistic MQTT-based [OpenSprinkler
Pi](https://opensprinkler.com/product/opensprinkler-pi/) controller
written in Rust.

## TODOs

* Proper stop procedure *(turn pump off first)
* Command for fetching current state and integration with Home Assistant
* Better logs
* Allow for plug and play drivers
* Implement a dummy driver for tests
* Implement a driver for 74HC595
* Write tests with [async-time-mock-tokio](https://crates.io/crates/async-time-mock-tokio)
