use acc_shared_memory_rs::{ACCSharedMemory, ACCError};

fn main() -> Result<(), ACCError> {
    println!("ACC Shared Memory - Simple Test");
    println!("===============================");

    // Try to connect to ACC
    let mut acc = ACCSharedMemory::new()?;
    println!("✓ Successfully connected to ACC shared memory");
    
    // Check if ACC is running
    if !acc.is_acc_running() {
        println!("⚠ ACC doesn't appear to be running");
        return Ok(());
    }
    
    println!("✓ ACC is running and responsive");
    
    // Try to get some data
    match acc.read_shared_memory()? {
        Some(data) => {
            println!("\n--- Session Information ---");
            println!("Track: {}", data.statics.track);
            println!("Car: {}", data.statics.car_model);
            println!("Player: {}", data.statics.full_player_name());
            println!("Session: {}", data.graphics.session_type);
            println!("Status: {}", data.graphics.status);
            
            println!("\n--- Current Data ---");
            println!("Speed: {:.1} km/h", data.physics.speed_kmh);
            println!("RPM: {}", data.physics.rpm);
            println!("Gear: {}", data.physics.gear);
            println!("Fuel: {:.1} L", data.physics.fuel);
            
            if data.physics.is_moving() {
                println!("✓ Car is moving");
            } else {
                println!("⚬ Car is stationary");
            }
            
            if data.is_racing() {
                println!("✓ Currently racing");
            } else {
                println!("⚬ Not currently racing");
            }
            
            println!("\n--- Memory Info ---");
            println!("{}", acc.memory_info());
        }
        None => {
            println!("⚬ No fresh telemetry data available");
            println!("  This is normal if the car is not moving or game is paused");
        }
    }
    
    println!("\n✓ Test completed successfully");
    Ok(())
}