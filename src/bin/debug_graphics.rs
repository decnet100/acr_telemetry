//! Debug tool to check GraphicsMap values from shared memory.

use acc_shared_memory_rs::ACCSharedMemory;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut acc = ACCSharedMemory::new()?;
    
    println!("Reading GraphicsMap from shared memory...");
    println!("Press Ctrl+C to stop.\n");
    
    let mut count = 0;
    loop {
        if let Some(data) = acc.read_shared_memory()? {
            count += 1;
            
            if count % 60 == 0 {
                let g = &data.graphics;
                println!("=== Graphics Sample {} ===", count);
                println!("packet_id: {}", g.packet_id);
                println!("status: {:?}", g.status);
                println!("session_type: {:?}", g.session_type);
                println!("completed_lap: {}", g.completed_lap);
                println!("position: {}", g.position);
                println!("current_time: {} ms", g.current_time);
                println!("speed (from physics): {:.1} km/h", data.physics.speed_kmh);
                println!("normalized_car_position: {:.6}", g.normalized_car_position);
                println!("distance_traveled: {:.2} m", g.distance_traveled);
                println!("car_coordinates len: {}", g.car_coordinates.len());
                println!("player_car_id: {}", g.player_car_id);
                println!("active_cars: {}", g.active_cars);
                
                if !g.car_coordinates.is_empty() {
                    println!("First car coords: x={:.2}, y={:.2}, z={:.2}", 
                        g.car_coordinates[0].x,
                        g.car_coordinates[0].y,
                        g.car_coordinates[0].z);
                }
                
                println!("fuel_per_lap: {:.3}", g.fuel_per_lap);
                println!("is_valid_lap: {}", g.is_valid_lap);
                println!();
            }
        }
        
        std::thread::sleep(Duration::from_millis(16));
    }
}
