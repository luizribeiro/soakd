# Project 3: System Reliability & Error Handling

**Priority**: HIGH  
**Status**: Ready to Start (Design Phase)  
**Dependencies**: Project 1 (MQTT Enhancement) - for status infrastructure  
**Parallelizable**: Partially (design now, implementation after Project 1)

## Overview

Enhance soakd's reliability through comprehensive error reporting, configuration hot-reload, and robust failure handling. This project ensures soakd can recover from failures, report issues clearly, and adapt to configuration changes without service interruption.

## Goals

1. **Comprehensive Error Reporting**: Detailed error information for troubleshooting
2. **Configuration Hot-Reload**: Update settings without restarting the service
3. **Failure Recovery**: Graceful handling of hardware and network failures
4. **Monitoring Integration**: Health checks and system diagnostics

## Implementation Phases

### Phase 3A: Error Reporting System

**New Files:**
- `src/error_reporter.rs` - Error reporting and categorization
- `src/health.rs` - System health monitoring

**Error Categories:**
```rust
#[derive(Debug, Clone, Serialize)]
pub enum ErrorCategory {
    Hardware,      // GPIO, pump, valve failures
    Network,       // MQTT connection issues
    Configuration, // Invalid config, missing zones
    System,        // File I/O, permissions, resources
    Validation,    // Invalid commands, bad parameters
}

#[derive(Debug, Clone, Serialize)]
pub struct SprinklerError {
    pub category: ErrorCategory,
    pub code: String,
    pub message: String,
    pub zone: Option<String>,
    pub timestamp: SystemTime,
    pub context: HashMap<String, String>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize)]
pub enum ErrorSeverity {
    Critical,  // System unusable
    High,      // Major functionality impaired
    Medium,    // Minor functionality affected
    Low,       // Warning, no functionality lost
}
```

**Error MQTT Topics:**
```
sprinklers/error                         # Current error state
sprinklers/error/history                 # Recent error history
sprinklers/health/status                 # Overall system health
sprinklers/health/components             # Component-level health
```

**Implementation Tasks:**
- [ ] Create error classification system
- [ ] Implement error reporting infrastructure
- [ ] Add error history tracking (last 50 errors)
- [ ] Create health status monitoring
- [ ] Add error recovery procedures

### Phase 3B: Configuration Hot-Reload

**New MQTT Commands:**
```
sprinklers/config/reload                 # Reload configuration
sprinklers/config/validate               # Validate config without applying
sprinklers/config/backup                 # Create config backup
sprinklers/config/restore                # Restore from backup
```

**Configuration Management:**
```rust
#[derive(Debug)]
pub struct ConfigManager {
    current_config: Arc<RwLock<Configuration>>,
    config_path: PathBuf,
    backup_dir: PathBuf,
    validation_errors: Vec<ConfigError>,
}

impl ConfigManager {
    pub async fn reload_config(&mut self) -> Result<(), ConfigError>;
    pub async fn validate_config(&self, path: &Path) -> Result<Configuration, Vec<ConfigError>>;
    pub async fn backup_config(&self) -> Result<PathBuf, ConfigError>;
    pub async fn apply_config(&mut self, config: Configuration) -> Result<(), ConfigError>;
}
```

**Configuration Validation:**
- Zone GPIO pin conflicts
- Pump GPIO pin availability
- MQTT broker connectivity
- Plan references to valid zones
- Hardware driver compatibility

**Implementation Tasks:**
- [ ] Create configuration management system
- [ ] Implement hot-reload mechanism
- [ ] Add configuration validation
- [ ] Create backup/restore functionality
- [ ] Add rollback on failed reload

### Phase 3C: Hardware Failure Handling

**Hardware Monitoring:**
```rust
#[derive(Debug, Clone)]
pub struct HardwareStatus {
    pub pump: ComponentStatus,
    pub zones: HashMap<String, ComponentStatus>,
    pub gpio_driver: ComponentStatus,
    pub last_check: SystemTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatus {
    pub name: String,
    pub status: HealthStatus,
    pub last_success: Option<SystemTime>,
    pub failure_count: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Failed,
    Unknown,
}
```

**Failure Recovery Strategies:**
- **GPIO Failures**: Retry with exponential backoff, fallback to noop driver
- **Pump Failures**: Continue zone operation without pump (if configured)
- **Zone Failures**: Skip failed zone, continue with remaining zones
- **MQTT Failures**: Queue status updates, reconnect automatically

**Implementation Tasks:**
- [ ] Implement hardware health monitoring
- [ ] Add component failure detection
- [ ] Create failure recovery procedures
- [ ] Add graceful degradation modes
- [ ] Implement retry mechanisms with backoff

### Phase 3D: Network Resilience

**MQTT Connection Management:**
```rust
#[derive(Debug)]
pub struct MqttManager {
    client: Arc<Mutex<Option<AsyncClient>>>,
    config: MQTTConfig,
    connection_state: ConnectionState,
    retry_count: u32,
    last_connected: Option<SystemTime>,
    message_queue: VecDeque<QueuedMessage>,
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Connected,
    Reconnecting,
    Disconnected,
    Failed,
}

#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub topic: String,
    pub payload: String,
    pub qos: QoS,
    pub retain: bool,
    pub timestamp: SystemTime,
}
```

**Network Resilience Features:**
- Automatic MQTT reconnection with exponential backoff
- Message queuing during disconnection
- Connection health monitoring
- Graceful degradation (continue irrigation without MQTT)

**Implementation Tasks:**
- [ ] Enhance MQTT connection management
- [ ] Implement message queuing for offline periods
- [ ] Add connection health monitoring
- [ ] Create network failure recovery
- [ ] Add connection status reporting

## Technical Specifications

### Error Reporting MQTT Messages

**Error Event:**
```json
{
  "category": "hardware",
  "code": "GPIO_WRITE_FAILED",
  "message": "Failed to write to GPIO pin 12 for zone 'Front Yard'",
  "zone": "Front Yard",
  "timestamp": "2025-01-27T15:30:00Z",
  "severity": "high",
  "context": {
    "pin": "12",
    "operation": "write_high",
    "retry_count": "3"
  }
}
```

**Health Status:**
```json
{
  "overall_status": "degraded",
  "components": {
    "pump": {
      "status": "healthy",
      "last_success": "2025-01-27T15:29:45Z",
      "failure_count": 0
    },
    "gpio_driver": {
      "status": "degraded", 
      "last_success": "2025-01-27T15:25:12Z",
      "failure_count": 3,
      "error_message": "Intermittent GPIO write failures"
    },
    "mqtt": {
      "status": "healthy",
      "last_success": "2025-01-27T15:30:00Z",
      "failure_count": 0
    }
  },
  "uptime_seconds": 86400,
  "last_irrigation": "2025-01-27T06:00:00Z"
}
```

### Configuration Hot-Reload

**Reload Command Response:**
```json
{
  "status": "success",
  "message": "Configuration reloaded successfully",
  "changes": [
    "Updated pump delay from 5s to 3s",
    "Added new zone 'Garden Bed'",
    "Modified MQTT topic prefix"
  ],
  "backup_created": "/etc/soakd/backups/config-20250127-153000.yaml",
  "timestamp": "2025-01-27T15:30:00Z"
}
```

**Validation Error Response:**
```json
{
  "status": "error",
  "message": "Configuration validation failed",
  "errors": [
    {
      "field": "zones[2].pin",
      "error": "GPIO pin 12 already in use by pump",
      "line": 25
    },
    {
      "field": "plans[0].zone_durations[1].zone", 
      "error": "Zone 'Invalid Zone' not found in configuration",
      "line": 45
    }
  ]
}
```

### Failure Recovery

**Recovery Procedures:**
1. **GPIO Failure**: Retry → Fallback to noop → Error report
2. **Pump Failure**: Continue without pump → Log warning → Report degraded status
3. **Zone Failure**: Skip zone → Continue plan → Report partial failure
4. **MQTT Failure**: Queue messages → Retry connection → Continue irrigation
5. **Config Failure**: Revert to backup → Report error → Continue with old config

## Home Assistant Integration

### Discovery for Error Monitoring

**Error Sensor:**
```json
{
  "name": "Sprinkler System Errors",
  "state_topic": "sprinklers/error",
  "unique_id": "soakd_error_status",
  "value_template": "{{ value_json.message if value_json else 'No errors' }}",
  "icon": "mdi:alert-circle"
}
```

**Health Sensor:**
```json
{
  "name": "Sprinkler System Health",
  "state_topic": "sprinklers/health/status", 
  "unique_id": "soakd_health_status",
  "value_template": "{{ value_json.overall_status }}",
  "icon": "mdi:heart-pulse"
}
```

**Component Sensors (per component):**
```json
{
  "name": "Sprinkler Pump Health",
  "state_topic": "sprinklers/health/components",
  "unique_id": "soakd_pump_health",
  "value_template": "{{ value_json.pump.status }}",
  "icon": "mdi:pump"
}
```

### HA Automation Examples

**Error Notification:**
```yaml
- trigger:
    platform: mqtt
    topic: sprinklers/error
  action:
    - service: notify.mobile_app
      data:
        title: "Sprinkler System Error"
        message: "{{ trigger.payload_json.message }}"
        data:
          priority: high
```

**Health Monitoring:**
```yaml
- trigger:
    platform: state
    entity_id: sensor.sprinkler_system_health
    to: "failed"
  action:
    - service: automation.turn_off
      entity_id: automation.automatic_watering
    - service: notify.mobile_app
      data:
        title: "Sprinkler System Failed"
        message: "Automatic watering disabled until issue resolved"
```

## Testing Strategy

### Unit Tests
- [ ] Error classification and reporting
- [ ] Configuration validation logic
- [ ] Failure recovery procedures
- [ ] Health status calculation
- [ ] Message queuing during disconnection

### Integration Tests
- [ ] Configuration hot-reload without service interruption
- [ ] Error reporting through MQTT
- [ ] Hardware failure simulation and recovery
- [ ] Network disconnection handling
- [ ] Health monitoring accuracy

### Fault Injection Tests
- [ ] GPIO write failures
- [ ] MQTT broker disconnection
- [ ] Invalid configuration files
- [ ] Hardware device removal
- [ ] File system issues

### Manual Testing
- [ ] Home Assistant error notifications
- [ ] Configuration reload via HA dashboard
- [ ] System recovery after power loss
- [ ] Graceful degradation verification

## Success Criteria

1. **Error Visibility**: All errors are reported with sufficient detail for troubleshooting
2. **Hot Reload**: Configuration changes apply without service restart
3. **Failure Recovery**: System continues operating despite component failures
4. **Health Monitoring**: System health is continuously monitored and reported
5. **Network Resilience**: MQTT disconnection doesn't affect irrigation reliability

## Configuration

### New Configuration Options
```yaml
# Error reporting settings
error_reporting:
  enabled: true
  history_size: 50              # Number of errors to keep in history
  severity_threshold: "medium"  # Minimum severity to report via MQTT
  
# Health monitoring
health:
  check_interval: 60            # Health check interval (seconds)
  component_timeout: 30         # Component response timeout (seconds)
  failure_threshold: 3          # Failures before marking component as failed

# Configuration management
config:
  backup_dir: "/etc/soakd/backups"
  max_backups: 10               # Maximum backup files to keep
  auto_backup: true             # Create backup before reload

# Network resilience
mqtt:
  # ... existing mqtt config ...
  reconnect_interval: 30        # Reconnection attempt interval (seconds)
  max_reconnect_attempts: 0     # 0 = infinite attempts
  message_queue_size: 100       # Max queued messages during disconnection
  connection_timeout: 10        # Connection timeout (seconds)
```

## Future Extensions

**Phase 3E (Future)**:
- Performance monitoring and optimization
- Predictive failure analysis
- Remote diagnostics and support
- Integration with external monitoring systems (Grafana, Prometheus)
- Advanced recovery strategies (redundant hardware support)

## Dependencies & Coordination

**Depends on Project 1**:
- MQTT infrastructure
- Status publishing framework
- Home Assistant discovery

**Enables**:
- Production-ready deployment
- Proactive maintenance
- Reliable unattended operation
- Professional monitoring integration

**Rust Crates to Add**:
- `tokio::fs` (for async file operations)
- `notify` (for file system watching)
- `sysinfo` (for system health monitoring)
- `thiserror` (for structured error handling)