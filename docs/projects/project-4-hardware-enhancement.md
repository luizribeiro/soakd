# Project 4: Hardware Enhancement & Flow Monitoring

**Priority**: MEDIUM  
**Status**: Ready to Start  
**Dependencies**: None (independent hardware features)  
**Parallelizable**: Yes (completely independent of other projects)

## Overview

Enhance soakd's hardware capabilities with advanced pump configuration options and water flow monitoring. This project adds professional-grade features for system optimization, leak detection, and water usage tracking.

## Goals

1. **Advanced Pump Control**: Multiple pump support and flexible configuration
2. **Flow Monitoring**: Real-time water flow measurement and leak detection
3. **Water Usage Tracking**: Accurate measurement and logging of water consumption
4. **System Optimization**: Flow-based irrigation adjustments and efficiency monitoring

## Implementation Phases

### Phase 4A: Enhanced Pump Configuration

**Current Pump Limitations:**
- Single pump only
- Always enabled (except workaround with unused GPIO)
- Fixed activation delay
- No pump feedback or monitoring

**Enhanced Pump Configuration:**
```yaml
# Single pump (backward compatible)
pump:
  pin: 7
  delay: 5
  enabled: true
  name: "Main Pump"
  
# Multiple pumps (new feature)
pumps:
  - name: "Main Pump"
    pin: 7
    delay: 5
    enabled: true
    zones: ["Front Yard", "Side"]        # Zones served by this pump
    pressure_range: [30, 50]             # PSI range for this pump
    
  - name: "Boost Pump" 
    pin: 8
    delay: 3
    enabled: true
    zones: ["Backyard"]                  # High-pressure zone
    pressure_range: [50, 80]
    
  - name: "Drip System"
    pin: 9
    delay: 10
    enabled: true
    zones: ["Garden Bed", "Planters"]
    pressure_range: [10, 25]

# Pump control strategies
pump_control:
  strategy: "zone_based"                 # "zone_based", "pressure_based", "manual"
  auto_select: true                      # Automatically select pump based on zone
  simultaneous_pumps: false              # Allow multiple pumps at once
```

**Implementation Tasks:**
- [ ] Extend pump configuration to support multiple pumps
- [ ] Add zone-to-pump mapping logic
- [ ] Implement pump selection strategies
- [ ] Add pump status monitoring and reporting
- [ ] Maintain backward compatibility with single pump config

### Phase 4B: Flow Sensor Integration

**Flow Sensor Configuration:**
```yaml
flow_sensors:
  - name: "Main Line"
    pin: 11                              # GPIO pin for pulse input
    sensor_type: "hall_effect"           # "hall_effect", "turbine", "ultrasonic"
    pulses_per_liter: 450               # Calibration factor
    debounce_ms: 10                     # Pulse debouncing
    zones: ["Front Yard", "Side"]       # Zones monitored by this sensor
    
  - name: "Drip Line"
    pin: 12
    sensor_type: "hall_effect"
    pulses_per_liter: 2250              # Higher resolution for low flow
    debounce_ms: 5
    zones: ["Garden Bed", "Planters"]

# Flow monitoring settings
flow_monitoring:
  enabled: true
  sample_interval: 5                    # Seconds between readings
  anomaly_detection: true
  leak_threshold: 0.5                   # L/min minimum flow to detect leak
  max_flow_threshold: 50                # L/min maximum expected flow
```

**Flow Sensor Hardware Support:**
```rust
#[derive(Debug, Clone)]
pub struct FlowSensor {
    pub name: String,
    pub pin: u8,
    pub sensor_type: FlowSensorType,
    pub pulses_per_liter: u32,
    pub debounce_ms: u32,
    pub zones: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum FlowSensorType {
    HallEffect,
    Turbine,
    Ultrasonic,
}

#[derive(Debug, Clone)]
pub struct FlowReading {
    pub sensor: String,
    pub timestamp: SystemTime,
    pub flow_rate_lpm: f32,              // Liters per minute
    pub total_volume_l: f32,             // Total liters this session
    pub pulse_count: u32,
    pub anomaly: Option<FlowAnomaly>,
}

#[derive(Debug, Clone)]
pub enum FlowAnomaly {
    NoFlow,                              // Expected flow but none detected
    ExcessiveFlow,                       // Flow above maximum threshold
    LeakDetected,                        // Flow when no zones active
    SensorFailure,                       // Sensor not responding
}
```

**Implementation Tasks:**
- [ ] Create flow sensor driver with GPIO interrupt handling
- [ ] Implement pulse counting and flow rate calculation
- [ ] Add flow anomaly detection algorithms
- [ ] Create flow data storage and history tracking
- [ ] Add calibration and sensor testing utilities

### Phase 4C: Water Usage Tracking

**Usage Tracking Features:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct WaterUsage {
    pub session_id: String,
    pub zone: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub planned_duration: u16,           // Minutes
    pub actual_duration: u16,            // Minutes
    pub total_volume_l: f32,             // Liters used
    pub average_flow_rate: f32,          // L/min
    pub efficiency_rating: f32,          // Planned vs actual flow
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyUsage {
    pub date: String,                    // YYYY-MM-DD
    pub total_volume_l: f32,
    pub total_runtime_minutes: u16,
    pub zones: HashMap<String, ZoneUsage>,
    pub efficiency: f32,
    pub cost_estimate: Option<f32>,      // Based on water rates
}

#[derive(Debug, Clone, Serialize)]
pub struct ZoneUsage {
    pub zone: String,
    pub sessions: u32,
    pub total_volume_l: f32,
    pub total_runtime_minutes: u16,
    pub average_flow_rate: f32,
}
```

**Usage Storage:**
- SQLite database for historical data
- JSON export capabilities
- Configurable retention period
- Data aggregation (daily, weekly, monthly)

**Implementation Tasks:**
- [ ] Create water usage tracking system
- [ ] Implement SQLite database for historical storage
- [ ] Add usage calculation and aggregation logic
- [ ] Create data export and reporting features
- [ ] Add water cost calculation (configurable rates)

### Phase 4D: Advanced Flow Features

**Leak Detection:**
```rust
#[derive(Debug, Clone)]
pub struct LeakDetector {
    baseline_flow: f32,                  // Normal system flow when off
    detection_threshold: f32,            // Minimum flow to trigger leak alert
    confirmation_time: Duration,         // Time to confirm leak (not transient)
    active_leak: Option<LeakEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeakEvent {
    pub detected_at: SystemTime,
    pub flow_rate: f32,
    pub estimated_volume_lost: f32,
    pub confidence: f32,                 // 0.0 - 1.0
    pub location_hint: Option<String>,   // Which sensor/zone area
}
```

**Flow-Based Irrigation Optimization:**
```rust
#[derive(Debug, Clone)]
pub struct FlowOptimizer {
    target_flow_rates: HashMap<String, f32>,    // Optimal flow per zone
    efficiency_history: VecDeque<EfficiencyReading>,
    optimization_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct EfficiencyReading {
    pub zone: String,
    pub timestamp: SystemTime,
    pub flow_rate: f32,
    pub pressure_estimate: f32,
    pub efficiency_score: f32,           // 0.0 - 1.0
}
```

**Implementation Tasks:**
- [ ] Implement leak detection algorithms
- [ ] Add flow-based irrigation optimization
- [ ] Create pressure estimation from flow data
- [ ] Add system efficiency monitoring
- [ ] Implement predictive maintenance alerts

## Technical Specifications

### MQTT Topics for Flow Monitoring

**Real-time Flow Data:**
```
sprinklers/flow/main_line/rate           # Current flow rate (L/min)
sprinklers/flow/main_line/total          # Session total volume (L)
sprinklers/flow/main_line/status         # Sensor status
sprinklers/flow/drip_line/rate
sprinklers/flow/drip_line/total
sprinklers/flow/drip_line/status
```

**Usage and Analytics:**
```
sprinklers/usage/current                 # Current session usage
sprinklers/usage/daily                   # Today's total usage
sprinklers/usage/zone/{zone_name}        # Per-zone usage
sprinklers/analytics/efficiency          # System efficiency metrics
```

**Alerts and Anomalies:**
```
sprinklers/alert/leak                    # Leak detection alerts
sprinklers/alert/flow_anomaly            # Flow anomaly alerts
sprinklers/alert/sensor_failure          # Sensor malfunction alerts
```

### Enhanced Pump Control

**Pump Status Topics:**
```
sprinklers/pump/{pump_name}/status       # Individual pump status
sprinklers/pump/{pump_name}/runtime      # Pump runtime statistics
sprinklers/pump/active                   # Currently active pumps
sprinklers/pump/selection                # Automatic pump selection info
```

**Pump Control Commands:**
```
sprinklers/pump/control                  # Manual pump control
sprinklers/pump/test                     # Pump testing commands
sprinklers/pump/calibrate                # Pump calibration mode
```

### Flow Sensor Data Format

**Flow Reading:**
```json
{
  "sensor": "main_line",
  "timestamp": "2025-01-27T15:30:00Z",
  "flow_rate_lpm": 12.5,
  "total_volume_l": 45.2,
  "pulse_count": 20340,
  "quality": "good",
  "anomaly": null
}
```

**Usage Summary:**
```json
{
  "session_id": "session_20250127_153000",
  "zone": "Front Yard",
  "start_time": "2025-01-27T15:30:00Z",
  "end_time": "2025-01-27T15:45:00Z",
  "planned_duration": 15,
  "actual_duration": 15.2,
  "total_volume_l": 187.5,
  "average_flow_rate": 12.3,
  "efficiency_rating": 0.95,
  "cost_estimate": 0.75
}
```

**Leak Alert:**
```json
{
  "alert_type": "leak_detected",
  "detected_at": "2025-01-27T02:15:30Z",
  "flow_rate": 2.1,
  "estimated_loss_rate": "3.0 L/hour",
  "confidence": 0.87,
  "sensor": "main_line",
  "location_hint": "Between pump and front yard zone"
}
```

## Home Assistant Integration

### Discovery for Flow Monitoring

**Flow Rate Sensors:**
```json
{
  "name": "Main Line Flow Rate",
  "state_topic": "sprinklers/flow/main_line/rate",
  "unique_id": "soakd_flow_main_line_rate",
  "unit_of_measurement": "L/min",
  "icon": "mdi:water-pump",
  "device_class": "volume_flow_rate"
}
```

**Usage Sensors:**
```json
{
  "name": "Daily Water Usage",
  "state_topic": "sprinklers/usage/daily",
  "unique_id": "soakd_daily_usage",
  "value_template": "{{ value_json.total_volume_l }}",
  "unit_of_measurement": "L",
  "icon": "mdi:water",
  "device_class": "water"
}
```

**Pump Control Switches:**
```json
{
  "name": "Main Pump",
  "command_topic": "sprinklers/pump/control",
  "state_topic": "sprinklers/pump/main_pump/status",
  "unique_id": "soakd_pump_main",
  "payload_on": "{\"pump\": \"main_pump\", \"action\": \"on\"}",
  "payload_off": "{\"pump\": \"main_pump\", \"action\": \"off\"}",
  "icon": "mdi:pump"
}
```

**Alert Sensors:**
```json
{
  "name": "Leak Detection Alert",
  "state_topic": "sprinklers/alert/leak",
  "unique_id": "soakd_leak_alert",
  "value_template": "{{ 'detected' if value_json else 'none' }}",
  "icon": "mdi:pipe-leak"
}
```

### HA Dashboard Examples

**Flow Monitoring Card:**
```yaml
type: entities
title: Water Flow Monitoring
entities:
  - entity: sensor.main_line_flow_rate
    name: Current Flow Rate
  - entity: sensor.daily_water_usage
    name: Today's Usage  
  - entity: sensor.sprinkler_efficiency
    name: System Efficiency
  - entity: binary_sensor.leak_detection_alert
    name: Leak Status
```

**Usage History Card:**
```yaml
type: history-graph
title: Water Usage History
entities:
  - entity: sensor.daily_water_usage
  - entity: sensor.front_yard_usage
  - entity: sensor.backyard_usage
hours_to_show: 168  # 1 week
```

## Testing Strategy

### Unit Tests
- [ ] Flow sensor pulse counting accuracy
- [ ] Usage calculation correctness
- [ ] Leak detection algorithm effectiveness
- [ ] Pump selection logic
- [ ] Data storage and retrieval

### Integration Tests
- [ ] Flow sensor integration with irrigation cycles
- [ ] Multi-pump coordination
- [ ] Usage tracking across multiple zones
- [ ] Alert generation and delivery
- [ ] Database operations and data integrity

### Hardware Tests
- [ ] Flow sensor calibration accuracy
- [ ] Pump control reliability
- [ ] GPIO interrupt handling under load
- [ ] Sensor failure detection
- [ ] System performance with multiple sensors

### Manual Testing
- [ ] Home Assistant flow monitoring dashboard
- [ ] Leak detection sensitivity
- [ ] Usage tracking accuracy vs manual measurement
- [ ] Pump automatic selection
- [ ] Alert notifications in HA

## Success Criteria

1. **Accurate Flow Measurement**: Flow readings within ±5% of manual measurement
2. **Reliable Leak Detection**: Detect leaks >0.5 L/min within 2 minutes
3. **Multi-Pump Support**: Seamless operation with multiple pumps and zones
4. **Usage Tracking**: Comprehensive water usage data with historical trends
5. **HA Integration**: Professional flow monitoring dashboard in Home Assistant

## Configuration

### New Configuration Sections
```yaml
# Enhanced pump configuration
pumps:
  - name: "Main Pump"
    pin: 7
    delay: 5
    enabled: true
    zones: ["Front Yard", "Side"]
    max_flow_rate: 25               # L/min
    
# Flow sensor configuration
flow_sensors:
  - name: "Main Line"
    pin: 11
    sensor_type: "hall_effect"
    pulses_per_liter: 450
    debounce_ms: 10
    zones: ["Front Yard", "Side"]
    calibration_factor: 1.0         # Adjustment multiplier

# Flow monitoring settings
flow_monitoring:
  enabled: true
  sample_interval: 5              # Seconds
  leak_detection:
    enabled: true
    threshold: 0.5                # L/min
    confirmation_time: 120        # Seconds
  usage_tracking:
    enabled: true
    database_path: "/var/lib/soakd/usage.db"
    retention_days: 365
    water_cost_per_liter: 0.004   # Local water rates
```

## Future Extensions

**Phase 4E (Future)**:
- Pressure sensor integration
- Advanced leak location algorithms
- Weather-based flow optimization
- Integration with smart water meters
- Predictive maintenance based on usage patterns
- Water quality monitoring (TDS, pH sensors)

## Dependencies & Coordination

**Independent Project**: No dependencies on other projects

**Rust Crates to Add**:
- `rusqlite` (for usage database)
- `tokio::time` (for flow sampling intervals)
- `rppal` or `gpio-rs` (for interrupt-based GPIO)
- `chrono` (for timestamp handling)
- `serde_json` (for data serialization)

**Hardware Requirements**:
- Flow sensors (hall effect or turbine)
- Additional GPIO pins for sensors
- Optional: Multiple pump control relays
- Optional: Pressure sensors for advanced features

**Enables**:
- Professional water management
- Leak detection and prevention
- Usage optimization and cost tracking
- Predictive maintenance capabilities