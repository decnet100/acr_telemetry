# ACC Shared Memory Rust - Usage Guide

## Quick Start

### 1. Build the Project
```bash
cd acc_shared_memory_rs
cargo build --release --examples
```

### 2. Run Examples

#### Basic Telemetry Monitor
```bash
cargo run --example basic_telemetry
```
Real-time telemetry display with speed, RPM, timing, and more.

#### Simple Connectivity Test
```bash
cargo run --example simple_test
```
Quick test to verify ACC connection and basic data reading.

#### Fuel Strategy Logger (Simple)
```bash
# Use default settings (fuel_strategy_log.csv, 20 second interval)
cargo run --example fuel_strategy_logger_simple

# Custom file and interval
cargo run --example fuel_strategy_logger_simple my_fuel_data.csv 15.0
```
Direct equivalent of your Python fuel_strategy_logger.py.

#### Fuel Strategy Logger (Enhanced)
```bash
# Enhanced version with additional features
cargo run --example fuel_strategy_logger enhanced_fuel_data.csv 10.0
```
Extended version with pit strategy, session info, and detailed logging.

## Library Usage in Your Code

### Add to Cargo.toml
```toml
[dependencies]
acc_shared_memory_rs = { path = "../acc_shared_memory_rs" }

# Or if published to crates.io:
# acc_shared_memory_rs = "0.1.0"
```

### Basic Usage
```rust
use acc_shared_memory_rs::{ACCSharedMemory, ACCError};
use std::time::Duration;

fn main() -> Result<(), ACCError> {
    let mut acc = ACCSharedMemory::new()?;
    
    loop {
        match acc.read_shared_memory() {
            Ok(Some(data)) => {
                println!("Speed: {:.1} km/h, RPM: {}", 
                         data.physics.speed_kmh, 
                         data.physics.rpm);
                
                // Access any telemetry data
                println!("Fuel: {:.1}L, Gear: {}", 
                         data.physics.fuel, 
                         data.physics.gear);
            }
            Ok(None) => {
                // No new data, continue polling
            }
            Err(ACCError::SharedMemoryNotAvailable) => {
                eprintln!("ACC is not running");
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
        
        std::thread::sleep(Duration::from_millis(16)); // ~60fps
    }
    
    Ok(())
}
```

### Advanced Usage
```rust
use acc_shared_memory_rs::{ACCSharedMemory, enums::*};

fn analyze_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    let mut acc = ACCSharedMemory::new()?;
    
    if let Some(data) = acc.read_shared_memory()? {
        // Session analysis
        match data.graphics.session_type {
            AccSessionType::Race => println!("Racing!"),
            AccSessionType::Qualifying => println!("Qualifying session"),
            AccSessionType::Practice => println!("Practice session"),
            _ => {}
        }
        
        // Weather analysis
        if data.graphics.rain_intensity.requires_wet_tyres() {
            println!("Need wet tyres!");
        }
        
        // Performance analysis
        let max_tyre_temp = data.physics.max_tyre_temp();
        if max_tyre_temp > 110.0 {
            println!("Tyres overheating: {:.0}°C", max_tyre_temp);
        }
        
        // Strategy analysis
        if data.pit_stop_required() {
            println!("Pit stop recommended");
        }
        
        // Flag monitoring
        if data.graphics.has_yellow_flags() {
            println!("Yellow flag - caution required");
        }
    }
    
    Ok(())
}
```

## Available Data

### Physics Data (333Hz)
```rust
data.physics.speed_kmh          // Current speed
data.physics.rpm                // Engine RPM
data.physics.gear               // Current gear
data.physics.fuel               // Fuel level
data.physics.wheel_pressure     // Tyre pressures (all 4 wheels)
data.physics.tyre_core_temp     // Tyre temperatures
data.physics.brake_temp         // Brake temperatures
data.physics.g_force            // G-forces (Vector3f)
data.physics.velocity           // 3D velocity vector
data.physics.car_damage         // Damage state
data.physics.tc                 // Traction control activity
data.physics.abs                // ABS activity
// ... and many more fields
```

### Graphics Data (60Hz)
```rust
data.graphics.session_type      // Practice/Qualifying/Race
data.graphics.completed_lap     // Current lap number
data.graphics.position          // Race position
data.graphics.best_time_str     // Best lap time (formatted)
data.graphics.fuel_per_lap      // Fuel consumption per lap
data.graphics.rain_intensity    // Weather conditions
data.graphics.flag              // Current flag status
data.graphics.penalty           // Active penalties
data.graphics.is_in_pit         // In pit lane
// ... and many more fields
```

### Static Data (Session constants)
```rust
data.statics.track              // Track name
data.statics.car_model          // Car model
data.statics.player_name        // Player name
data.statics.max_rpm            // Maximum RPM
data.statics.max_fuel           // Fuel tank capacity
data.statics.pit_window_start   // Pit window timing
// ... and more configuration data
```

## Utility Methods

### Data Analysis
```rust
// Performance checks
data.is_racing()                    // Currently racing
data.physics.is_moving()            // Car is moving
data.physics.max_tyre_temp()        // Highest tyre temperature
data.graphics.is_wet_conditions()   // Wet track conditions

// Strategy helpers
data.fuel_needed_for_race()         // Estimated fuel needed
data.pit_stop_required()            // Pit stop recommended
data.graphics.current_lap_time_seconds() // Current lap in seconds

// Session info
data.session_info()                 // Formatted session description
data.performance_summary()          // Current performance summary
```

### Enum Methods
```rust
// Weather analysis
data.graphics.rain_intensity.is_wet()              // Any rain
data.graphics.rain_intensity.requires_wet_tyres()  // Wet tyres needed
data.graphics.rain_intensity.grip_level()          // Grip coefficient

// Track conditions
data.graphics.track_grip_status.is_slippery()      // Low grip
data.graphics.track_grip_status.grip_level()       // Relative grip

// Penalty analysis
data.graphics.penalty.is_disqualification()        // DQ penalty
data.graphics.penalty.is_cutting_penalty()         // Track cutting
data.graphics.penalty.is_pit_speeding_penalty()    // Pit lane speeding
```

## Error Handling

### Error Types
```rust
match acc.read_shared_memory() {
    Ok(Some(data)) => { /* Process data */ }
    Ok(None) => { /* No new data */ }
    Err(ACCError::SharedMemoryNotAvailable) => {
        // ACC not running
    }
    Err(ACCError::SharedMemoryOpen(msg)) => {
        // Failed to open shared memory
    }
    Err(ACCError::InvalidData(msg)) => {
        // Data corruption or parsing error
    }
    Err(e) => {
        // Other errors
    }
}
```

## Platform Support

- **Windows**: Full support with native shared memory access
- **Linux/macOS**: Compiles but will return `SharedMemoryNotAvailable` (ACC is Windows-only)

## Performance Tips

1. **Polling Frequency**: Use 16ms intervals (~60fps) for responsive updates
2. **Data Filtering**: Only process when `read_shared_memory()` returns `Some(data)`
3. **Memory Efficiency**: The library uses zero-copy parsing for optimal performance
4. **Threading**: The library is thread-safe for concurrent access

## Troubleshooting

### ACC Not Detected
- Ensure ACC is running and in a session
- Check that shared memory is enabled in ACC settings
- Verify Windows permissions

### Performance Issues
- Reduce polling frequency if CPU usage is high
- Use release builds for production (`cargo build --release`)
- Consider processing data in a separate thread

### Data Inconsistency
- Always check for `None` returns from `read_shared_memory()`
- Validate critical data before making decisions
- Use the packet ID to detect fresh data