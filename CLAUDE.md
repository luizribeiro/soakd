# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

soakd is a minimalistic MQTT-based irrigation controller for OpenSprinkler Pi hardware, written in Rust. It's a system daemon that controls irrigation valves and pumps through GPIO pins, responding to MQTT commands for automated lawn and garden watering.

## Development Commands

### Environment Setup
- Enter development environment: `devenv shell` (Nix-based environment with Rust, cmake, git, openssl, paho-mqtt-c)

### Build Commands
- Build: `cargo build`
- Release build: `cargo build --release`
- Check without building: `cargo check` (alias: `cargo c`)

### Code Quality
- Format code: `cargo fmt`
- Lint: `cargo clippy`
- Generate docs: `cargo doc` (alias: `cargo d`)

### Running
- Run with default config: `cargo run` (alias: `cargo r`)
- Run with custom config: `cargo run -- path/to/config.yaml`

### Testing
- Run tests: `cargo test` (alias: `cargo t`)
- Note: Currently no tests implemented; planned to use `async-time-mock-tokio`

## Architecture Overview

### Core Components

1. **Main Event Loop** (`src/main.rs`)
   - Tokio async runtime
   - Reads configuration → initializes driver → connects to MQTT → processes messages
   - Panic/SIGINT handlers ensure valves shut off on exit

2. **Configuration** (`src/config.rs`)
   - YAML-based configuration
   - Key structures: MQTTConfig, PumpConfig, ZoneConfig, SprinklerPlan
   - Driver selection (GPIO or Noop)

3. **MQTT Client** (`src/mqtt.rs`)
   - Wrapper around paho_mqtt::AsyncClient
   - Subscribes to `{topic_prefix}/#` pattern
   - Async stream interface for messages

4. **Hardware Abstraction** (`src/driver/`)
   - Driver trait with `shutoff_all_valves()` and `activate_zone()`
   - Global singleton pattern using `lazy_static!`
   - Implementations: GpioDriver (74HC595 shift register), NoopDriver (testing)

5. **Handler System** (`src/handlers/`)
   - Dynamic registration via `#[mqtt_handler(topic = "pattern")]` macro
   - Topic pattern matching with MQTT wildcards
   - Abortable tasks for long-running operations
   - Current handlers: `water_zone`, `start_plan`, `stop_plan`

### Key Patterns

- **Macro-based Registration**: The `#[mqtt_handler]` macro (in `macros/` workspace) auto-registers handlers at startup
- **Global State**: Uses `lazy_static!` with `Mutex` for thread-safe driver and handler registry
- **Safety-First**: Panic handlers, SIGINT handling, and abortable tasks ensure reliable valve control
- **Async Throughout**: Tokio-based async/await for concurrent operations

### MQTT Commands

- `{topic_prefix}/start_plan/{PlanName}`: Execute predefined watering plan
- `{topic_prefix}/stop`: Stop all watering immediately  
- `{topic_prefix}/water_zone/`: Water individual zone (JSON payload with duration)

## Current Development Priorities

### Urgent (Project 1)
- Home Assistant MQTT Discovery implementation
- Real-time status reporting via MQTT
- Pump enable/disable configuration

### High Priority
- Individual zone control enhancements (Project 2)
- System reliability improvements (Project 3)
- Better error handling and recovery

### Medium Priority
- Hardware enhancements: flow sensors, multiple pumps (Project 4)
- Advanced monitoring and analytics (Project 5)

See `docs/projects/` for detailed enhancement plans.

## Important Notes

- The project follows a minimalistic philosophy: soakd handles reliable irrigation execution while Home Assistant provides intelligent scheduling
- When adding features, maintain clear separation between hardware control and smart logic
- All handlers use the new macro system - see existing handlers for patterns
- The driver system is designed to be pluggable - new hardware support should implement the Driver trait
- Configuration changes require application restart (hot-reload planned in Project 3)