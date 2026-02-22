// fuel_strategy_logger.rs
// Rust equivalent of fuel_strategy_logger.py

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
    
    // Write CSV header
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
                
                // Write CSV row
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
                
                // Console output
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
                // No new data available, short sleep and retry
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
    
    // Handle Ctrl+C gracefully
    ctrlc::set_handler(move || {
        println!("\n[INFO] Logging stopped by user.");
        std::process::exit(0);
    })?;
    
    record_fuel_and_strategy(output_file, interval_secs)
}

// Optional: Enhanced version with additional features
#[allow(dead_code)]
fn record_fuel_and_strategy_enhanced(
    output_file: &str, 
    interval_secs: f64
) -> Result<(), Box<dyn std::error::Error>> {
    let mut acc = ACCSharedMemory::new()?;
    let start_time = Instant::now();
    
    let file = File::create(output_file)?;
    let mut writer = BufWriter::new(file);
    
    // Enhanced CSV header with additional data
    writeln!(
        writer,
        "timestamp,session_type,lap,total_laps,lap_completion_percent,fuel_liters,\
         fuel_per_lap,fuel_estimated_laps,can_complete_next_lap,pit_window_open,\
         mandatory_pit_done,speed_kmh,in_pit,track,car_model"
    )?;
    writer.flush()?;
    
    println!("[INFO] Enhanced fuel monitoring every {:.1} seconds. Press Ctrl+C to stop.", interval_secs);
    
    loop {
        match acc.read_shared_memory() {
            Ok(Some(data)) => {
                let timestamp = start_time.elapsed().as_secs_f64();
                
                let lap = data.graphics.completed_lap;
                let total_laps = data.graphics.number_of_laps;
                let lap_completion = data.graphics.normalized_car_position * 100.0;
                let fuel = data.physics.fuel;
                let fuel_per_lap = data.graphics.fuel_per_lap;
                let fuel_estimated_laps = data.graphics.fuel_estimated_laps;
                let speed = data.physics.speed_kmh;
                let in_pit = data.graphics.is_in_pit;
                
                let can_complete = if fuel_per_lap > 0.0 {
                    fuel >= fuel_per_lap
                } else {
                    true
                };
                
                // Check if pit window is open (simplified logic)
                let pit_window_open = if data.statics.has_pit_window() {
                    let session_time = data.graphics.session_time_left;
                    let total_time = data.statics.pit_window_end - data.statics.pit_window_start;
                    session_time <= total_time as f32
                } else {
                    false
                };
                
                // Enhanced CSV row
                writeln!(
                    writer,
                    "{:.2},{},{},{},{:.2},{:.2},{:.2},{:.1},{},{},{},{:.1},{},{},{}",
                    timestamp,
                    data.graphics.session_type,
                    lap,
                    total_laps,
                    lap_completion,
                    fuel,
                    fuel_per_lap,
                    fuel_estimated_laps,
                    if can_complete { "YES" } else { "NO" },
                    if pit_window_open { "YES" } else { "NO" },
                    if data.graphics.mandatory_pit_done { "YES" } else { "NO" },
                    speed,
                    if in_pit { "YES" } else { "NO" },
                    data.statics.track,
                    data.statics.car_model
                )?;
                writer.flush()?;
                
                // Enhanced console output with fuel strategy advice
                let mut status = format!(
                    "[{:.1}s] {} Lap {}/{}, {:.1}% complete, Fuel: {:.2}L ({:.1} laps), Speed: {:.0} km/h",
                    timestamp,
                    data.graphics.session_type,
                    lap,
                    total_laps,
                    lap_completion,
                    fuel,
                    fuel_estimated_laps,
                    speed
                );
                
                if in_pit {
                    status.push_str(" [IN PIT]");
                }
                
                if !can_complete && !in_pit {
                    status.push_str(" 🚨 FUEL CRITICAL - PIT NOW!");
                } else if fuel_estimated_laps < 2.0 && !data.graphics.mandatory_pit_done {
                    status.push_str(" ⚠️  Consider pitting soon");
                } else if can_complete {
                    status.push_str(" ✅ Fuel OK");
                }
                
                if pit_window_open && !data.graphics.mandatory_pit_done {
                    status.push_str(" [PIT WINDOW OPEN]");
                }
                
                println!("{}", status);
                
                thread::sleep(Duration::from_secs_f64(interval_secs));
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(ACCError::SharedMemoryNotAvailable) => {
                eprintln!("[ERROR] ACC is not running or shared memory is not available");
                break;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to read telemetry: {}", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    
    println!("\n[INFO] Enhanced logging stopped.");
    Ok(())
}