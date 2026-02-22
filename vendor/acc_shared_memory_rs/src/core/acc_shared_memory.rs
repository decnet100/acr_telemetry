use std::time::{Duration, Instant};

use crate::core::SharedMemoryReader;
use crate::maps::{ACCMap, PhysicsMap};
use crate::parsers::{parse_graphics_map, parse_physics_map, parse_statics_map};
use crate::{ACCError, Result};

/// Main interface for reading ACC shared memory data.
pub struct ACCSharedMemory {
    physics_reader: SharedMemoryReader,
    graphics_reader: SharedMemoryReader,
    statics_reader: SharedMemoryReader,
    last_physics_id: i32,
    last_physics: Option<PhysicsMap>,
}

impl ACCSharedMemory {
    /// Create a new ACC shared memory reader.
    /// 
    /// This will attempt to connect to all three ACC shared memory segments:
    /// - Physics: High-frequency telemetry data (~333Hz)
    /// - Graphics: Medium-frequency session data (~60Hz)  
    /// - Statics: Low-frequency static data (session constants)
    pub fn new() -> Result<Self> {
        let physics_reader = SharedMemoryReader::new("Local\\acpmf_physics", 800)?;
        let graphics_reader = SharedMemoryReader::new("Local\\acpmf_graphics", 1588)?;
        let statics_reader = SharedMemoryReader::new("Local\\acpmf_static", 784)?;

        Ok(Self {
            physics_reader,
            graphics_reader,
            statics_reader,
            last_physics_id: 0,
            last_physics: None,
        })
    }

    /// Read shared memory data, returning None if no new data is available.
    /// 
    /// This method only returns data when the physics packet ID has changed,
    /// avoiding duplicate processing of the same telemetry frame.
    pub fn read_shared_memory(&mut self) -> Result<Option<ACCMap>> {
        let physics = parse_physics_map(&self.physics_reader)?;
        
        // Check if we have new data
        if physics.packet_id == self.last_physics_id {
            return Ok(None);
        }

        // Additional check: compare with last physics data to detect stale updates
        if let Some(ref last_physics) = self.last_physics {
            if physics.is_equal(last_physics) {
                return Ok(None);
            }
        }

        let graphics = parse_graphics_map(&self.graphics_reader)?;
        let statics = parse_statics_map(&self.statics_reader)?;

        self.last_physics_id = physics.packet_id;
        self.last_physics = Some(physics.clone());

        Ok(Some(ACCMap::new(physics, graphics, statics)))
    }

    /// Wait for fresh shared memory data with a timeout.
    /// 
    /// This method will poll `read_shared_memory()` until fresh data is available
    /// or the timeout is reached.
    pub fn wait_for_data(&mut self, timeout: Duration) -> Result<ACCMap> {
        let start = Instant::now();
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 10000; // Prevent infinite loops

        while start.elapsed() < timeout && attempts < MAX_ATTEMPTS {
            if let Some(data) = self.read_shared_memory()? {
                return Ok(data);
            }
            attempts += 1;
            
            // Small sleep to prevent excessive CPU usage
            std::thread::sleep(Duration::from_micros(100));
        }

        Err(ACCError::Timeout)
    }

    /// Get shared memory data immediately, blocking until data is available.
    /// 
    /// This is equivalent to `wait_for_data()` with a 10-second timeout.
    pub fn get_shared_memory_data(&mut self) -> Result<ACCMap> {
        self.wait_for_data(Duration::from_secs(10))
    }

    /// Check if ACC is currently running and shared memory is available.
    pub fn is_acc_running(&self) -> bool {
        // Try to read a single byte from physics memory to test availability
        self.physics_reader.read_at::<u8>(0).is_ok()
    }

    /// Get information about the connected shared memory segments.
    pub fn memory_info(&self) -> String {
        format!(
            "Physics: {} ({} bytes), Graphics: {} ({} bytes), Statics: {} ({} bytes)",
            self.physics_reader.name(),
            self.physics_reader.size(),
            self.graphics_reader.name(),
            self.graphics_reader.size(),
            self.statics_reader.name(),
            self.statics_reader.size()
        )
    }

    /// Reset the internal state, forcing the next read to return data.
    pub fn reset(&mut self) {
        self.last_physics_id = 0;
        self.last_physics = None;
    }
}

impl Default for ACCSharedMemory {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ACC shared memory")
    }
}