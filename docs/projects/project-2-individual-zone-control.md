# Project 2: Individual Zone Control & Progress Tracking

**Priority**: HIGH  
**Status**: Ready to Start  
**Dependencies**: Project 1 (MQTT Enhancement) - for status infrastructure  
**Parallelizable**: Partially (can start design, implementation waits for Project 1)

## Overview

Enable Home Assistant to directly control individual zones with real-time progress tracking. This allows HA to implement intelligent watering schedules with dynamic durations while soakd maintains reliable execution.

## Goals

1. **Direct Zone Control**: HA can water any zone for any duration without predefined plans
2. **Progress Tracking**: Real-time updates on watering progress and time remaining
3. **Queue Management**: Handle multiple zone requests intelligently
4. **Status Integration**: Leverage Project 1's status infrastructure

## Current State Analysis

**Already Implemented** (confirmed working):
- ✅ `sprinklers/water_zone/` command with JSON payload
- ✅ Basic individual zone watering functionality

**Needs Enhancement**:
- ❌ No progress reporting during watering
- ❌ No queue management for multiple requests
- ❌ Limited status visibility
- ❌ No cancellation of individual zones

## Implementation Phases

### Phase 2A: Enhanced Zone Control

**Files to Modify:**
- `src/handlers/water_zone.rs`
- `src/handlers/mod.rs`

**New Command Features:**
```json
// Enhanced water_zone command
{
  "zone": "Front Yard",
  "duration": 15,
  "priority": "normal",     // "high", "normal", "low"
  "queue": true,           // Queue if system busy vs reject
  "id": "ha_morning_1"     // Optional request ID for tracking
}

// New zone control commands
{
  "action": "cancel",
  "zone": "Front Yard"
}

{
  "action": "pause",
  "zone": "Front Yard"  
}

{
  "action": "resume", 
  "zone": "Front Yard"
}
```

**Implementation Tasks:**
- [ ] Enhance water_zone command parsing
- [ ] Add zone-specific control commands
- [ ] Implement priority handling
- [ ] Add request ID tracking
- [ ] Validate zone names against configuration

### Phase 2B: Progress Tracking System

**New Files:**
- `src/progress.rs` - Progress tracking and reporting
- `src/queue.rs` - Zone request queue management

**Progress Reporting Structure:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ZoneProgress {
    pub zone: String,
    pub request_id: Option<String>,
    pub start_time: SystemTime,
    pub duration_minutes: u16,
    pub elapsed_minutes: u16,
    pub remaining_minutes: u16,
    pub status: ZoneStatus,
}

#[derive(Debug, Clone, Serialize)]
pub enum ZoneStatus {
    Queued,
    Starting,
    Running, 
    Paused,
    Completed,
    Cancelled,
    Error(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemProgress {
    pub current_zone: Option<ZoneProgress>,
    pub queue: Vec<QueuedZone>,
    pub total_remaining_minutes: u16,
    pub estimated_completion: Option<SystemTime>,
}
```

**MQTT Progress Topics:**
```
sprinklers/progress                      # Overall system progress
sprinklers/zone/{zone_name}/progress     # Individual zone progress
sprinklers/queue/status                  # Queue status
sprinklers/queue/length                  # Number of queued requests
```

**Implementation Tasks:**
- [ ] Create progress tracking structures
- [ ] Implement progress calculation logic
- [ ] Add periodic progress publishing (every 30 seconds)
- [ ] Create queue management system
- [ ] Add estimated completion time calculation

### Phase 2C: Queue Management

**Queue Features:**
```rust
#[derive(Debug, Clone)]
pub struct QueuedZone {
    pub zone: String,
    pub duration: u16,
    pub priority: Priority,
    pub request_id: Option<String>,
    pub queued_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    High = 3,
    Normal = 2, 
    Low = 1,
}

pub struct ZoneQueue {
    queue: BinaryHeap<QueuedZone>,
    max_size: usize,
}
```

**Queue Management Logic:**
- Priority-based ordering (High > Normal > Low)
- FIFO within same priority level
- Configurable maximum queue size
- Queue overflow handling (reject vs replace oldest)
- Queue persistence across restarts (optional)

**Implementation Tasks:**
- [ ] Implement priority queue system
- [ ] Add queue size limits and overflow handling
- [ ] Create queue manipulation methods (add, remove, clear)
- [ ] Add queue status reporting
- [ ] Implement queue persistence (optional)

### Phase 2D: Advanced Zone Control

**New MQTT Commands:**
```
sprinklers/zone/cancel                   # Cancel current zone
sprinklers/zone/pause                    # Pause current zone
sprinklers/zone/resume                   # Resume paused zone
sprinklers/queue/clear                   # Clear all queued requests
sprinklers/queue/remove                  # Remove specific queued request
```

**Command Payloads:**
```json
// Cancel current zone
{"action": "cancel"}

// Pause/resume current zone  
{"action": "pause"}
{"action": "resume"}

// Remove from queue
{"action": "remove", "request_id": "ha_morning_1"}
{"action": "remove", "zone": "Front Yard"}
```

**Implementation Tasks:**
- [ ] Add zone control command handlers
- [ ] Implement pause/resume functionality
- [ ] Add queue manipulation commands
- [ ] Create zone state persistence for pause/resume
- [ ] Add validation and error handling

## Technical Specifications

### Enhanced MQTT API

**Command Topics:**
```
sprinklers/water_zone/                   # Enhanced zone watering
sprinklers/zone/control                  # Zone control commands
sprinklers/queue/control                 # Queue management
```

**Status Topics (extends Project 1):**
```
sprinklers/progress                      # System-wide progress
sprinklers/zone/{zone_name}/progress     # Per-zone progress
sprinklers/zone/{zone_name}/status       # Per-zone status
sprinklers/queue/status                  # Queue contents
sprinklers/queue/length                  # Queue size
```

### State Management

**Zone State Tracking:**
```rust
#[derive(Debug)]
pub struct ZoneState {
    pub zone: String,
    pub status: ZoneStatus,
    pub start_time: Option<SystemTime>,
    pub duration: Option<u16>,
    pub pause_time: Option<SystemTime>,
    pub paused_duration: u16,
}
```

**System State Integration:**
- Extends Project 1's SystemState
- Adds zone-specific state tracking
- Maintains queue state
- Tracks pause/resume state

### Error Handling

**Zone Control Errors:**
- Invalid zone name
- Zone already running
- Queue full
- Hardware failure during operation
- Invalid command format

**Error Reporting:**
```json
{
  "error": "zone_not_found",
  "message": "Zone 'Invalid Zone' not found in configuration",
  "request_id": "ha_morning_1",
  "timestamp": "2025-01-27T15:30:00Z"
}
```

## Home Assistant Integration

### Discovery Enhancements

**Additional Entities:**
```json
// Zone progress sensor
{
  "name": "Front Yard Progress",
  "state_topic": "sprinklers/zone/front_yard/progress",
  "unique_id": "soakd_zone_front_yard_progress",
  "value_template": "{{ value_json.remaining_minutes }}",
  "unit_of_measurement": "min"
}

// Queue length sensor
{
  "name": "Sprinkler Queue Length", 
  "state_topic": "sprinklers/queue/length",
  "unique_id": "soakd_queue_length",
  "icon": "mdi:format-list-numbered"
}

// Zone control switches with progress
{
  "name": "Front Yard Sprinkler",
  "command_topic": "sprinklers/water_zone/",
  "state_topic": "sprinklers/zone/front_yard/status",
  "unique_id": "soakd_zone_front_yard",
  "payload_on": "{\"zone\": \"Front Yard\", \"duration\": 15, \"id\": \"ha_switch\"}",
  "payload_off": "{\"action\": \"cancel\", \"zone\": \"Front Yard\"}"
}
```

### HA Automation Examples

**Smart Duration Calculation:**
```yaml
# HA calculates optimal duration based on weather/sensors
- service: mqtt.publish
  data:
    topic: sprinklers/water_zone/
    payload: >
      {
        "zone": "{{ zone_name }}",
        "duration": {{ calculated_duration }},
        "priority": "normal",
        "id": "weather_based_{{ now().timestamp() }}"
      }
```

**Queue-based Scheduling:**
```yaml
# Queue multiple zones for sequential watering
- repeat:
    for_each: ["Front Yard", "Side", "Backyard"]
    sequence:
      - service: mqtt.publish
        data:
          topic: sprinklers/water_zone/
          payload: >
            {
              "zone": "{{ repeat.item }}",
              "duration": {{ states('input_number.' + repeat.item.lower().replace(' ', '_') + '_duration') | int }},
              "queue": true,
              "priority": "normal"
            }
```

## Testing Strategy

### Unit Tests
- [ ] Queue management logic
- [ ] Progress calculation accuracy
- [ ] Priority ordering
- [ ] Command parsing and validation
- [ ] State transitions

### Integration Tests
- [ ] Zone control with status updates
- [ ] Queue processing order
- [ ] Pause/resume functionality
- [ ] Error handling and recovery
- [ ] MQTT message flow

### Manual Testing
- [ ] Home Assistant zone control switches
- [ ] Progress updates in HA dashboard
- [ ] Queue visualization in HA
- [ ] Emergency stop functionality
- [ ] System behavior under high load

## Success Criteria

1. **Direct Control**: HA can start any zone for any duration immediately
2. **Real-time Progress**: HA shows accurate progress and time remaining
3. **Queue Management**: Multiple zone requests are handled intelligently
4. **Responsive Control**: Pause, resume, and cancel work instantly
5. **Reliable Operation**: No impact on irrigation reliability or timing accuracy

## Configuration

### New Configuration Options
```yaml
# Zone control settings
zone_control:
  queue_size: 10                    # Maximum queued requests
  progress_interval: 30             # Progress update interval (seconds)
  allow_queue_overflow: false       # Reject vs replace when queue full
  persist_queue: false              # Save queue across restarts

# Home Assistant discovery for zone control
homeassistant:
  zone_controls:
    enabled: true
    default_duration: 15            # Default duration for HA switches
    create_progress_sensors: true   # Create progress sensors for each zone
```

## Future Extensions

**Phase 2E (Future)**:
- Zone scheduling conflicts detection
- Water pressure optimization between zones
- Zone dependency management (e.g., don't run adjacent zones simultaneously)
- Advanced queue algorithms (shortest job first, etc.)
- Zone group control (water multiple zones simultaneously)

## Dependencies & Coordination

**Depends on Project 1**:
- Status infrastructure (SystemState, status publishing)
- MQTT client enhancements
- Home Assistant discovery framework

**Enables**:
- Home Assistant advanced automation
- Dynamic watering schedules
- Weather-responsive irrigation
- User-friendly manual control

**Rust Crates to Add**:
- `tokio::time` (for progress intervals)
- `std::collections::BinaryHeap` (for priority queue)
- `chrono` (for time calculations)