// fuel_strategy_logger_simple.rs
// Simple Rust equivalent of fuel_strategy_logger.py without external dependencies

use acc_shared_memory_rs::{ACCSharedMemory, ACCError};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};
use std::{env, thread};

/// Record fuel usage and strategy data to CSV for analysis
fn record_fuel_and_strategy(output_file: &str, interval_secs: f64) -> Result<(), Box<dyn std::error::Error>> {
    let mut acc = ACCSharedMemory::new()?;
    let start_time = Instant::now();
    
    // Create CSV file with headers
    let file = File::create(output_file)?;
    let mut writer = BufWriter::new(file);
    
    // Write CSV header (matching Python version exactly)
    writeln!(writer, "timestamp,lap,lap_completion_percent,fuel_liters,fuel_per_lap,can_complete_next_lap")?;
    writer.flush()?;
    
    println!("[INFO] Monitoring fuel every {:.1} seconds. Press Ctrl+C to stop.", interval_secs);
    
    loop {
        match acc.read_shared_memory() {
            Ok(Some(data)) => {
                let timestamp = start_time.elapsed().as_secs_f64();
                
                let lap = data.graphics.completed_lap;
                let lap_completion = data.graphics.normalized_car_position * 100.0;
                let fuel = data.physics.fuel;
                let fuel_per_lap = data.graphics.fuel_per_lap;
                
                let can_complete = if fuel_per_lap > 0.0 {
                    fuel >= fuel_per_lap
                } else {
                    true
                };
                
                // Write CSV row (matching Python format exactly)
                writeln!(
                    writer,
                    "{:.2},{},{:.2},{:.2},{:.2},{}",
                    timestamp,
                    lap,
                    lap_completion,
                    fuel,
                    fuel_per_lap,
                    if can_complete { "YES" } else { "NO" }
                )?;
                writer.flush()?;
                
                // Console output (matching Python format)
                let status = format!(
                    "[{:.1}s] Lap {}, Lap %: {:.1}%, Fuel: {:.2}L, Use/Lap: {:.2}L → ",
                    timestamp, lap, lap_completion, fuel, fuel_per_lap
                );
                
                if can_complete {
                    println!("{}✅ Enough fuel", status);
                } else {
                    println!("{}🚨 Fuel low — BOX this lap!", status);
                }
                
                // Wait for the specified interval
                thread::sleep(Duration::from_secs_f64(interval_secs));
            }
            Ok(None) => {
                // No new data available, short sleep and retry (matching Python behavior)
                thread::sleep(Duration::from_millis(10));
            }
            Err(ACCError::SharedMemoryNotAvailable) => {
                eprintln!("[ERROR] ACC is not running or shared memory is not available");
                eprintln!("        Please start Assetto Corsa Competizione and try again.");
                break;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to read telemetry: {}", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    
    println!("\n[INFO] Logging stopped.");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    let output_file = args.get(1)
        .map(|s| s.as_str())
        .unwrap_or("fuel_strategy_log.csv");
    
    let interval_secs = args.get(2)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(20.0);
    
    println!("ACC Fuel Strategy Logger (Rust)");
    println!("===============================");
    println!("Output file: {}", output_file);
    println!("Logging interval: {:.1} seconds", interval_secs);
    println!();
    
    record_fuel_and_strategy(output_file, interval_secs)
}