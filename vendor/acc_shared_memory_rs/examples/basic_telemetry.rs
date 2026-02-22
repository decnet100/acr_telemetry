use acc_shared_memory_rs::{ACCSharedMemory, ACCError};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), ACCError> {
    println!("ACC Telemetry Reader - Basic Example");
    println!("====================================");

    // Initialize the shared memory reader
    let mut acc = match ACCSharedMemory::new() {
        Ok(reader) => {
            println!("✓ Connected to ACC shared memory");
            println!("  {}", reader.memory_info());
            reader
        }
        Err(ACCError::SharedMemoryNotAvailable) => {
            eprintln!("✗ ACC is not running or shared memory is not available");
            eprintln!("  Please start Assetto Corsa Competizione and try again.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize ACC shared memory: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nWaiting for telemetry data...");
    println!("Press Ctrl+C to exit\n");

    // Main telemetry loop
    loop {
        match acc.read_shared_memory() {
            Ok(Some(data)) => {
                // Clear screen and move cursor to top
                print!("\x1B[2J\x1B[H");
                
                println!("ACC Telemetry Data");
                println!("==================");
                
                // Session info
                println!("Session: {} | Track: {}", 
                         data.graphics.session_type, 
                         data.statics.track);
                
                println!("Player: {} | Car: {}", 
                         data.statics.full_player_name(),
                         data.statics.car_model);
                
                // Current status
                println!("\n--- Current Status ---");
                println!("Position: {} / {}", 
                         data.graphics.position, 
                         data.graphics.active_cars);
                
                println!("Lap: {} / {}", 
                         data.graphics.completed_lap, 
                         data.graphics.number_of_laps);
                
                println!("Status: {}", data.graphics.status);
                
                // Performance data
                println!("\n--- Performance ---");
                println!("Speed: {:.1} km/h", data.physics.speed_kmh);
                println!("RPM: {} / {}", data.physics.rpm, data.statics.max_rpm);
                println!("Gear: {}", data.physics.gear);
                println!("Fuel: {:.1} L", data.physics.fuel);
                
                // Timing
                if !data.graphics.last_time_str.is_empty() {
                    println!("\n--- Timing ---");
                    println!("Current: {}", data.graphics.current_time_str);
                    println!("Last: {}", data.graphics.last_time_str);
                    println!("Best: {}", data.graphics.best_time_str);
                    
                    if data.graphics.is_valid_lap {
                        println!("Delta: {} {}", 
                                if data.graphics.is_delta_positive { "+" } else { "-" },
                                data.graphics.delta_lap_time_str);
                    }
                }
                
                // Inputs
                println!("\n--- Inputs ---");
                println!("Throttle: {:.1}%", data.physics.gas * 100.0);
                println!("Brake: {:.1}%", data.physics.brake * 100.0);
                println!("Steering: {:.1}°", data.physics.steer_angle.to_degrees());
                
                // Car status
                println!("\n--- Car Status ---");
                if data.physics.tc > 0.0 {
                    println!("TC Active: {:.1}", data.physics.tc);
                }
                if data.physics.abs > 0.0 {
                    println!("ABS Active: {:.1}", data.physics.abs);
                }
                if data.physics.pit_limiter_on {
                    println!("Pit Limiter: ON");
                }
                
                // Tyre info
                println!("\n--- Tyres ---");
                println!("Compound: {}", data.graphics.tyre_compound);
                println!("Pressures: FL:{:.1} FR:{:.1} RL:{:.1} RR:{:.1}", 
                         data.physics.wheel_pressure.front_left,
                         data.physics.wheel_pressure.front_right,
                         data.physics.wheel_pressure.rear_left,
                         data.physics.wheel_pressure.rear_right);
                
                println!("Temps: FL:{:.0}° FR:{:.0}° RL:{:.0}° RR:{:.0}°", 
                         data.physics.tyre_core_temp.front_left,
                         data.physics.tyre_core_temp.front_right,
                         data.physics.tyre_core_temp.rear_left,
                         data.physics.tyre_core_temp.rear_right);
                
                // Flags and penalties
                if data.graphics.penalty != acc_shared_memory_rs::enums::AccPenaltyType::None {
                    println!("\n--- PENALTY ---");
                    println!("Type: {}", data.graphics.penalty);
                    if data.graphics.penalty_time > 0.0 {
                        println!("Time: {:.1}s", data.graphics.penalty_time);
                    }
                }
                
                if data.graphics.flag != acc_shared_memory_rs::enums::AccFlagType::NoFlag {
                    println!("\n--- FLAG ---");
                    println!("Flag: {}", data.graphics.flag);
                }
                
                // Weather
                if data.graphics.rain_intensity.is_wet() {
                    println!("\n--- Weather ---");
                    println!("Rain: {} (Grip: {})", 
                             data.graphics.rain_intensity,
                             data.graphics.track_grip_status);
                }
                
                println!("\n--- Debug ---");
                println!("Physics ID: {}", data.physics.packet_id);
                println!("Timestamp: {:.3}", data.timestamp);
            }
            Ok(None) => {
                // No new data, continue polling
            }
            Err(ACCError::SharedMemoryNotAvailable) => {
                println!("ACC connection lost. Exiting...");
                break;
            }
            Err(e) => {
                eprintln!("Error reading telemetry: {}", e);
                thread::sleep(Duration::from_millis(1000));
            }
        }
        
        // Small delay to prevent excessive CPU usage
        thread::sleep(Duration::from_millis(50));
    }
    
    Ok(())
}