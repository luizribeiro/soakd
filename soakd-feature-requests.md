# Soakd Feature Requests

*Based on building smart Home Assistant integration*

## Architecture Vision

**soakd**: Rock-solid, reliable irrigation controller
- Executes irrigation plans and individual zone commands
- Provides status and progress reporting
- Handles hardware control (GPIO, pumps, valves)
- Simple, robust, always-available

**Home Assistant**: Smart scheduling and decision-making
- Weather intelligence and forecasting
- Complex scheduling logic with multiple conditions
- User interface and notifications
- Integration with other smart home systems
- Sensor data analysis and optimization

**Clear Separation**: soakd does irrigation, HA does intelligence

## Current State Analysis

**What Works Well:**
- Simple YAML configuration
- Reliable GPIO control
- Basic MQTT plan execution via `sprinklers/start_plan/{PlanName}`
- Stable operation on Raspberry Pi

**Integration Challenges:**
- Limited MQTT status reporting
- No dynamic duration control
- No individual zone control without predefined plans
- No progress or error reporting

## Feature Requests

### URGENT (Before Config Deployment)

#### 0. ✅ CONFIRMED: Stop Command Works
**Solution**: soakd supports `{topic_prefix}/stop` command
**Implementation**: `sprinklers/stop` immediately aborts current watering and shuts off all valves
**Status**: ✅ IMPLEMENTED in Home Assistant integration

#### 1. Home Assistant MQTT Device Discovery
**Problem**: Manual entity creation required for all soakd devices/sensors
**Solution**: Implement Home Assistant MQTT Discovery protocol
**Reference**: https://www.home-assistant.io/integrations/mqtt/#mqtt-discovery

```bash
# Example discovery message for sprinkler system status
curl -X POST -H "Content-Type: application/json" \
  -d '{"name": "Sprinkler System", "state_topic": "sprinklers/status", "unique_id": "soakd_system_status", "device": {"identifiers": ["soakd_curium"], "name": "Soakd Controller", "model": "soakd", "manufacturer": "luizribeiro"}}' \
  http://krypton:1883/homeassistant/sensor/soakd_system_status/config
```

**Discovery Topics to Implement:**
```
homeassistant/sensor/soakd_system_status/config
homeassistant/sensor/soakd_active_zone/config  
homeassistant/sensor/soakd_plan_progress/config
homeassistant/switch/soakd_zone_front/config
homeassistant/switch/soakd_zone_side/config
homeassistant/switch/soakd_zone_backyard/config
```

**Benefits**: 
- Automatic Home Assistant integration
- No manual entity configuration required
- Professional device presentation in HA
- Easier deployment and maintenance

### High Priority (Essential for Smart Home Integration)

#### 0. Pump Enable/Disable Configuration
**Problem**: soakd always activates pump during watering, no way to disable
**Use Case**: High-pressure municipal water (80+ PSI) doesn't need pump
**Current Workaround**: Set pump pin to unused GPIO pin (pin 31)
**Solution**: Add optional `enabled: false` field to pump configuration:
```yaml
pump:
  pin: 7
  delay: 5
  enabled: false  # Don't activate pump for high-pressure municipal water
```

#### 1. Status MQTT Topics
**Problem**: No visibility into system state from Home Assistant
**Solution**: Publish status information to MQTT topics:
```
sprinklers/status              # "idle", "running", "error"
sprinklers/zone/active         # Current active zone name or "none"
sprinklers/zone/remaining      # Minutes remaining for current zone
sprinklers/pump/status         # "on", "off", "disabled"
sprinklers/plan/active         # Currently running plan name
```

#### 2. ~~Dynamic Duration Override~~ **ARCHITECTURE CHANGE**
**Old Approach**: Override plan durations via MQTT
**New Approach**: Home Assistant uses individual zone control with calculated durations
**Reason**: Keep soakd simple - HA calculates optimal durations and commands individual zones
**Implementation**: Use existing `sprinklers/water_zone/` with HA-calculated durations

#### 3. ✅ CONFIRMED: Individual Zone Control
**Solution**: soakd supports `{topic_prefix}/water_zone/` with JSON payload:
```json
# Topic: sprinklers/water_zone/
{"zone": "Front Yard", "duration": 15}
{"zone": "Side", "duration": 20}
{"zone": "Backyard", "duration": 12}
```
**Status**: ✅ IMPLEMENTED in Home Assistant integration

#### 4. Progress Reporting
**Problem**: No way to track watering progress
**Solution**: Regular status updates:
```
# Published every 30 seconds during watering
sprinklers/status/progress {
  "plan": "Nightly",
  "zone": "front_yard", 
  "elapsed": 5,
  "remaining": 10,
  "total_elapsed": 25,
  "total_remaining": 30
}
```

#### 5. Error Reporting
**Problem**: Silent failures are hard to detect
**Solution**: Error notifications:
```
sprinklers/error {
  "type": "gpio_failure",
  "zone": "front_yard",
  "message": "Unable to control GPIO pin 0",
  "timestamp": "2025-01-27T15:30:00Z"
}
```

### Medium Priority (Enhanced Operations)

#### 6. Configuration Hot-Reload
**Problem**: Must restart service to change configuration
**Solution**: MQTT command to reload config:
```
sprinklers/config/reload        # Reload configuration file
sprinklers/config/status        # Report config load status
```

#### 7. Water Flow Monitoring
**Problem**: No leak detection or flow verification
**Solution**: Flow sensor integration:
```yaml
# In soakd.yaml
flow_sensor:
  pin: 8
  pulses_per_liter: 450
  
# MQTT topics
sprinklers/flow/rate           # Current flow rate (L/min)
sprinklers/flow/total          # Total flow for current session
sprinklers/flow/alert          # Flow anomaly detection
```

#### 8. Zone Health Monitoring
**Problem**: Can't detect clogged or broken sprinkler heads
**Solution**: Track zone performance:
```
sprinklers/zone/front_yard/health {
  "last_run": "2025-01-27T06:00:00Z",
  "total_runtime_today": 15,
  "avg_flow_rate": 12.5,
  "anomalies": []
}
```

#### 9. ~~Scheduling Integration~~ **REMOVED**
**Reason**: Scheduling should remain in Home Assistant for smart decision-making
**Architecture**: soakd = reliable irrigation executor, HA = smart scheduler

#### 10. ~~Weather API Integration~~ **REMOVED** 
**Reason**: Weather logic should remain in Home Assistant for complex decision-making
**Architecture**: soakd = irrigation controller, HA = weather intelligence

### Nice to Have (Future Enhancements)

#### 11. Web Interface
Simple web UI for manual control and system monitoring

#### 12. Historical Logging  
Built-in SQLite database for watering history and analytics

#### 13. Multiple Pump Support
Different pumps for different zones or pressure requirements

#### 14. Pressure Monitoring
Integration with pressure sensors for system health

#### 15. Fertilizer Injection
Control fertilizer injection pumps with watering cycles

## Implementation Priority for Home Assistant Integration

**URGENT (Before Config Push)**: MQTT Device Discovery (#0), Status topics (#1)
**Phase 1 (Critical)**: Individual zone control (#3), Progress reporting (#4)
**Phase 2 (Important)**: Error reporting (#5), Configuration hot-reload (#6)
**Phase 3 (Nice)**: Flow monitoring (#7), Zone health monitoring (#8)

## Benefits for Users

**soakd Benefits** (Reliability & Control):
- **Rock-solid irrigation execution**: Never fails to water when commanded
- **Hardware abstraction**: Clean interface to GPIO, pumps, and valves  
- **Status visibility**: Always know what the system is doing
- **Error detection**: Immediate notification of hardware problems
- **Simple configuration**: Easy to understand and maintain

**Home Assistant Benefits** (Intelligence & Integration):
- **Smart scheduling**: Weather-aware, condition-based automation
- **Dynamic optimization**: Adjust based on sensors, seasons, forecasts
- **User interface**: Easy control and monitoring dashboards
- **Notifications**: Integrated alerts and status reports
- **Data analysis**: Historical tracking and optimization insights

**Combined Benefits**:
- **Separation of concerns**: Hardware control vs intelligence
- **Reliability**: soakd handles irrigation, HA handles decisions
- **Flexibility**: Complex logic in HA, simple execution in soakd
- **Maintainability**: Each system does what it's best at

## Current Workarounds for Immediate Deployment

Since we need to deploy configs to krypton before soakd enhancements:

### 1. Manual Entity Creation
**Issue**: No MQTT device discovery
**Workaround**: Comment out non-existent MQTT sensors in `configuration.yaml`
```yaml
# These sensors don't exist yet - uncomment when soakd supports them
# - name: "Sprinkler System Status"
#   state_topic: "sprinklers/status"
```

### 2. No Status Monitoring  
**Issue**: Can't see what soakd is doing
**Workaround**: Rely on notifications and time-based assumptions
- Assume watering completes based on plan durations
- Monitor for stuck irrigation by checking time since last command

### 3. Basic Plan Execution Only
**Issue**: No individual zone control or dynamic durations
**Workaround**: Use existing predefined plans
- Create multiple static plans for different scenarios
- Accept fixed durations until dynamic control available

### 4. No Progress Reporting
**Issue**: Can't track watering progress in real-time  
**Workaround**: Calculate estimated completion based on plan start time
```yaml
# Template sensor to estimate completion
- sensor:
    name: "Estimated Irrigation Completion"
    state: >
      {% set start = state_attr('automation.lawn_morning_watering', 'last_triggered') %}
      {% set duration = states('input_number.front_yard_duration')|int + states('input_number.side_yard_duration')|int + states('input_number.backyard_duration')|int %}
      {% if start %}
        {{ (start + timedelta(minutes=duration)).strftime('%H:%M') }}
      {% else %}
        Unknown
      {% endif %}
```

## Deployment Strategy

**Phase 1**: Deploy current configs with workarounds
**Phase 2**: Add soakd enhancements incrementally  
**Phase 3**: Remove workarounds as features become available

The system will work with basic functionality immediately, then become progressively smarter as soakd features are added.