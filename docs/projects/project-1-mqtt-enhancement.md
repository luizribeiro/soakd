# Project 1: Core MQTT Enhancement

**Priority**: URGENT  
**Status**: Ready to Start  
**Dependencies**: None  
**Parallelizable**: Yes (independent of other projects)

## Overview

Enhance soakd's MQTT capabilities to provide comprehensive status reporting and Home Assistant device discovery. This project establishes the foundation for smart home integration while maintaining soakd's core reliability principles.

## Goals

1. **MQTT Device Discovery**: Automatic Home Assistant integration without manual configuration
2. **Status Reporting**: Real-time visibility into system state and operations
3. **Pump Configuration**: Optional pump enable/disable for high-pressure systems
4. **Foundation**: Establish MQTT infrastructure for future enhancements

## Implementation Phases

### Phase 1A: Configuration Enhancement

**Files to Modify:**
- `src/config.rs`
- `config.yaml` (example)

**New Configuration Options:**
```yaml
# Enhanced MQTT configuration
mqtt:
  broker: "krypton"
  port: 1883
  topic_prefix: "sprinklers"
  client_id: "soakd"
  # NEW: Home Assistant Discovery
  homeassistant:
    enabled: true
    discovery_prefix: "homeassistant"
    device:
      name: "Soakd Controller"
      model: "soakd"
      manufacturer: "luizribeiro"
      identifiers: ["soakd_curium"]

# Enhanced pump configuration  
pump:
  pin: 7
  delay: 5
  enabled: true  # NEW: Allow disabling pump for high-pressure systems
```

**Implementation Tasks:**
- [ ] Add `HomeAssistantConfig` struct
- [ ] Add `DeviceInfo` struct for HA discovery
- [ ] Add `enabled` field to `PumpConfig`
- [ ] Update configuration parsing and validation
- [ ] Add configuration defaults and feature flags

### Phase 1B: Status Topics Infrastructure

**New Files:**
- `src/status.rs` - Status management and publishing
- `src/state.rs` - Application state tracking

**Status Topics to Implement:**
```
sprinklers/status              # "idle", "running", "error"
sprinklers/zone/active         # Current active zone name or "none"  
sprinklers/zone/remaining      # Minutes remaining for current zone
sprinklers/pump/status         # "on", "off", "disabled"
sprinklers/plan/active         # Currently running plan name or "none"
sprinklers/availability        # "online", "offline" (LWT)
```

**State Management:**
```rust
#[derive(Debug, Clone)]
pub struct SystemState {
    pub status: SystemStatus,
    pub active_zone: Option<String>,
    pub zone_remaining: Option<u16>,
    pub pump_status: PumpStatus,
    pub active_plan: Option<String>,
    pub plan_start_time: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub enum SystemStatus {
    Idle,
    Running,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum PumpStatus {
    On,
    Off,
    Disabled,
}
```

**Implementation Tasks:**
- [ ] Create status publishing module
- [ ] Implement state management
- [ ] Add status update methods
- [ ] Integrate with existing handlers
- [ ] Add Last Will and Testament (LWT) for availability

### Phase 1C: Home Assistant Discovery Module

**New Files:**
- `src/hass/mod.rs` - Home Assistant integration module
- `src/hass/discovery.rs` - Discovery message generation
- `src/hass/device.rs` - Device and entity definitions

**Discovery Messages to Generate:**
```json
// System status sensor
{
  "name": "Sprinkler System Status",
  "state_topic": "sprinklers/status",
  "unique_id": "soakd_system_status",
  "device": {...},
  "icon": "mdi:sprinkler"
}

// Active zone sensor
{
  "name": "Active Sprinkler Zone", 
  "state_topic": "sprinklers/zone/active",
  "unique_id": "soakd_active_zone",
  "device": {...},
  "icon": "mdi:sprinkler-variant"
}

// Zone remaining sensor
{
  "name": "Zone Time Remaining",
  "state_topic": "sprinklers/zone/remaining", 
  "unique_id": "soakd_zone_remaining",
  "device": {...},
  "unit_of_measurement": "min",
  "icon": "mdi:timer"
}

// Pump status sensor
{
  "name": "Sprinkler Pump Status",
  "state_topic": "sprinklers/pump/status",
  "unique_id": "soakd_pump_status", 
  "device": {...},
  "icon": "mdi:pump"
}

// Active plan sensor
{
  "name": "Active Sprinkler Plan",
  "state_topic": "sprinklers/plan/active",
  "unique_id": "soakd_active_plan",
  "device": {...},
  "icon": "mdi:calendar-check"
}

// Individual zone switches (dynamic based on config)
{
  "name": "Front Yard Sprinkler",
  "command_topic": "sprinklers/water_zone/",
  "state_topic": "sprinklers/zone/front_yard/status",
  "unique_id": "soakd_zone_front_yard",
  "device": {...},
  "payload_on": "{\"zone\": \"Front Yard\", \"duration\": 15}",
  "payload_off": "{\"zone\": \"Front Yard\", \"duration\": 0}",
  "icon": "mdi:sprinkler"
}
```

**Implementation Tasks:**
- [ ] Create discovery message builders
- [ ] Implement device info generation
- [ ] Add entity creation for sensors and switches
- [ ] Add discovery publishing on startup
- [ ] Add discovery removal on shutdown

### Phase 1D: Integration & Status Updates

**Files to Modify:**
- `src/handlers/start_plan.rs`
- `src/handlers/water_zone.rs` 
- `src/handlers/stop_plan.rs`
- `src/main.rs`

**Status Update Integration:**
```rust
// Example integration in water_zone handler
pub async fn handle_water_zone(/* ... */) -> Result<(), SprinklerError> {
    // Update status to running
    status::publish_system_status(SystemStatus::Running).await?;
    status::publish_active_zone(Some(zone_name.clone())).await?;
    status::publish_zone_remaining(Some(duration)).await?;
    
    // Start watering
    // ... existing logic ...
    
    // Update pump status
    if pump_enabled {
        status::publish_pump_status(PumpStatus::On).await?;
    }
    
    // ... watering logic with periodic updates ...
    
    // Update status to idle when complete
    status::publish_system_status(SystemStatus::Idle).await?;
    status::publish_active_zone(None).await?;
    status::publish_zone_remaining(None).await?;
    
    Ok(())
}
```

**Implementation Tasks:**
- [ ] Add status updates to all handlers
- [ ] Implement periodic status publishing during operations
- [ ] Add error status reporting
- [ ] Integrate pump enable/disable logic
- [ ] Add graceful shutdown status updates

## Technical Specifications

### MQTT Topic Structure
```
{topic_prefix}/status                    # System status
{topic_prefix}/zone/active               # Active zone
{topic_prefix}/zone/remaining            # Time remaining  
{topic_prefix}/zone/{zone_name}/status   # Individual zone status
{topic_prefix}/pump/status               # Pump status
{topic_prefix}/plan/active               # Active plan
{topic_prefix}/availability              # LWT topic
```

### Home Assistant Discovery Structure
```
{discovery_prefix}/sensor/{device_id}_{entity_id}/config
{discovery_prefix}/switch/{device_id}_{entity_id}/config
```

### Error Handling
- Graceful degradation if MQTT publishing fails
- Continue irrigation operations even if status updates fail
- Log but don't halt on discovery publishing errors
- Maintain existing reliability guarantees

## Testing Strategy

### Unit Tests
- [ ] Configuration parsing with new fields
- [ ] Status message generation
- [ ] Discovery message generation
- [ ] State management logic

### Integration Tests  
- [ ] MQTT publishing functionality
- [ ] Home Assistant discovery registration
- [ ] Status updates during watering operations
- [ ] Error condition handling

### Manual Testing
- [ ] Home Assistant automatic device discovery
- [ ] Status sensor updates in real-time
- [ ] Pump enable/disable functionality
- [ ] System behavior with MQTT broker disconnection

## Success Criteria

1. **Automatic Discovery**: soakd devices appear in Home Assistant without manual configuration
2. **Real-time Status**: Home Assistant shows current system state and active operations
3. **Pump Control**: High-pressure systems can disable pump via configuration
4. **Reliability**: All existing functionality continues to work without degradation
5. **Clean Integration**: Status updates don't interfere with irrigation timing

## Future Extension Points

This project establishes patterns for:
- Additional sensor integration (flow, pressure, etc.)
- More complex device discovery (multiple devices, sub-devices)
- Enhanced error reporting and diagnostics
- Configuration hot-reload capabilities

## Dependencies & Coordination

**Rust Crates to Add:**
- `serde_json` (for HA discovery messages)
- `chrono` (for timestamps)
- `uuid` (for unique device IDs)

**No External Dependencies**: This project is self-contained and doesn't require coordination with other projects.

**Enables Future Projects**:
- Project 2 (Individual Zone Control) - builds on status infrastructure
- Project 3 (System Reliability) - uses status and discovery patterns
- Project 5 (Advanced Monitoring) - extends status reporting