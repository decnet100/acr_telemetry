# ACC Shared Memory (Rust)

A Rust library for reading Assetto Corsa Competizione (ACC) shared memory telemetry data. This is a port of the Python `acc_shared_memory` library with full feature parity and additional type safety.

## Features

- **Real-time telemetry**: Read physics data at ~333Hz update rate
- **Session information**: Access graphics and timing data at ~60Hz
- **Static data**: Car and session configuration data
- **Type-safe enums**: All ACC status codes, flags, and types
- **Zero-copy parsing**: Efficient memory-mapped file access
- **Windows support**: Native Windows shared memory API integration
- **Optional Serde**: Serialize/deserialize support with feature flag

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
acc_shared_memory_rs = "0.1.0"

# Enable serde support (optional)
acc_shared_memory_rs = { version = "0.1.0", features = ["serde"] }
```

## Quick Start

```rust
use acc_shared_memory_rs::{ACCSharedMemory, ACCError};

fn main() -> Result<(), ACCError> {
    let mut acc = ACCSharedMemory::new()?;
    
    loop {
        if let Some(data) = acc.read_shared_memory()? {
            println!("Speed: {:.1} km/h, RPM: {}", 
                     data.physics.speed_kmh, 
                     data.physics.rpm);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
    }
}
```

## Data Structure

The library provides three main data structures:

- [PhysicsMap (~333Hz)](PHYSICS_MAP.md): High-frequency telemetry data (car dynamics, driver inputs, tyres, engine, suspension, brakes, etc.)
- [GraphicsMap (~60Hz)](GRAPHICS_MAP.md): Session and timing information (lap times, session status, car positions, flags, weather, etc.)
- [StaticsMap (Session constants)](STATICS_MAP.md): Static configuration data (car/track info, player details, session rules, pit window, etc.)

See the linked documentation files above for a full list of fields and their descriptions.

## Examples

### Basic Telemetry Reader
```rust
use acc_shared_memory_rs::ACCSharedMemory;

let mut acc = ACCSharedMemory::new()?;

loop {
    if let Some(data) = acc.read_shared_memory()? {
        // Physics data
        println!("Speed: {:.1} km/h", data.physics.speed_kmh);
        println!("RPM: {} / {}", data.physics.rpm, data.statics.max_rpm);
        
        // Session info
        println!("Lap: {} / {}", 
                 data.graphics.completed_lap, 
                 data.graphics.number_of_laps);
        
        // Tyre temperatures
        let tyres = &data.physics.tyre_core_temp;
        println!("Tyre temps: FL:{:.0}° FR:{:.0}° RL:{:.0}° RR:{:.0}°",
                 tyres.front_left, tyres.front_right,
                 tyres.rear_left, tyres.rear_right);
    }
}
```

### Fuel Strategy Calculator
```rust
if let Some(data) = acc.read_shared_memory()? {
    let remaining_laps = data.graphics.number_of_laps - data.graphics.completed_lap;
    let fuel_needed = remaining_laps as f32 * data.graphics.fuel_per_lap;
    
    if data.physics.fuel < fuel_needed {
        println!("Pit stop required! Need {:.1}L more fuel", 
                 fuel_needed - data.physics.fuel);
    }
}
```

### Weather Monitoring
```rust
if let Some(data) = acc.read_shared_memory()? {
    match data.graphics.rain_intensity {
        AccRainIntensity::NoRain => println!("Dry conditions"),
        AccRainIntensity::LightRain => println!("Light rain - consider wet tyres"),
        AccRainIntensity::HeavyRain => println!("Heavy rain - wet tyres required"),
        _ => println!("Rain: {}", data.graphics.rain_intensity),
    }
}
```

## Error Handling

The library provides comprehensive error handling:

```rust
match acc.read_shared_memory() {
    Ok(Some(data)) => {
        // Process telemetry data
    }
    Ok(None) => {
        // No new data (normal when car is stationary)
    }
    Err(ACCError::SharedMemoryNotAvailable) => {
        println!("ACC is not running");
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Building

Requires Windows and the Windows SDK for shared memory access.

```bash
# Build the library
cargo build --release

# Run examples
cargo run --example basic_telemetry
cargo run --example simple_test

# Run tests
cargo test
```

## Compatibility

- **Windows**: Full support (primary platform)
- **Linux/macOS**: Not supported (ACC uses Windows-specific shared memory)

## Performance

- Zero-copy memory access using memory-mapped files
- Efficient enum parsing with fallback handling
- Minimal allocations for string data
- ~1-2ms parsing time for complete data set

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Based on the Python `acc_shared_memory` library
- ACC shared memory documentation by Kunos Simulazioni
- Thanks to the ACC modding community for reverse engineering efforts