use super::{GraphicsMap, PhysicsMap, StaticsMap};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Top-level container that aggregates all three shared memory segments from ACC.
/// This represents a complete snapshot of the simulation state at any given moment.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ACCMap {
    /// High-frequency physics data (~333Hz)
    pub physics: PhysicsMap,
    /// Medium-frequency graphics data (~60Hz)
    pub graphics: GraphicsMap,
    /// Low-frequency static data (session constants)
    pub statics: StaticsMap,
    /// Timestamp when this data was captured
    pub timestamp: f64,
}

impl ACCMap {
    /// Create a new ACCMap instance
    pub fn new(physics: PhysicsMap, graphics: GraphicsMap, statics: StaticsMap) -> Self {
        Self {
            physics,
            graphics,
            statics,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
        }
    }

    /// Check if the data represents an active session
    pub fn is_active_session(&self) -> bool {
        self.graphics.is_session_active() && self.physics.is_engine_running
    }

    /// Check if the car is currently racing (moving and on track)
    pub fn is_racing(&self) -> bool {
        self.physics.is_moving() && !self.graphics.is_in_pit
    }

    /// Get the current session information as a formatted string
    pub fn session_info(&self) -> String {
        format!(
            "{} - {} (Lap {}/{})",
            self.statics.track,
            self.graphics.session_type,
            self.graphics.completed_lap,
            self.graphics.number_of_laps
        )
    }

    /// Get a summary of current performance
    pub fn performance_summary(&self) -> String {
        format!(
            "Speed: {:.1} km/h, RPM: {}, Gear: {}, Position: {}",
            self.physics.speed_kmh, self.physics.rpm, self.physics.gear, self.graphics.position
        )
    }

    /// Calculate estimated fuel needed for remaining laps
    pub fn fuel_needed_for_race(&self) -> f32 {
        let remaining_laps = (self.graphics.number_of_laps - self.graphics.completed_lap) as f32;
        remaining_laps * self.graphics.fuel_per_lap
    }

    /// Check if a pit stop is required (low fuel or mandatory pit not done)
    pub fn pit_stop_required(&self) -> bool {
        let fuel_critical = self.physics.fuel < self.graphics.fuel_per_lap * 2.0;
        let mandatory_pit_required =
            !self.graphics.mandatory_pit_done && self.graphics.missing_mandatory_pits > 0;

        fuel_critical || mandatory_pit_required
    }
}