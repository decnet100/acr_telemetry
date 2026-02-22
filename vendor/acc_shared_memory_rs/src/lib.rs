//! # ACC Shared Memory (Rust)
//! 
//! A Rust library for reading Assetto Corsa Competizione (ACC) shared memory telemetry data.
//! This is a port of the Python acc_shared_memory library.
//! 
//! ## Features
//! 
//! - Read real-time telemetry data from ACC
//! - Type-safe enums for all ACC status codes
//! - Structured data types for physics, graphics, and static information
//! - Cross-platform shared memory access (Windows)
//! 
//! ## Usage
//! 
//! ```no_run
//! use acc_shared_memory_rs::{ACCSharedMemory, ACCError};
//! 
//! fn main() -> Result<(), ACCError> {
//!     let mut acc = ACCSharedMemory::new()?;
//!     
//!     loop {
//!         if let Some(data) = acc.read_shared_memory()? {
//!             println!("Speed: {:.1} km/h, RPM: {}", 
//!                      data.physics.speed_kmh, 
//!                      data.physics.rpm);
//!         }
//!         std::thread::sleep(std::time::Duration::from_millis(16));
//!     }
//! }
//! ```

pub mod core;
pub mod datatypes;
pub mod enums;
pub mod maps;
pub mod parsers;

pub use core::ACCSharedMemory;
pub use maps::ACCMap;

/// Error types for ACC shared memory operations
#[derive(thiserror::Error, Debug)]
pub enum ACCError {
    #[error("Shared memory not available or game not running")]
    SharedMemoryNotAvailable,
    
    #[error("Failed to open shared memory: {0}")]
    SharedMemoryOpen(String),
    
    #[error("Failed to map shared memory: {0}")]
    SharedMemoryMap(String),
    
    #[error("Timeout waiting for fresh data")]
    Timeout,
    
    #[error("Invalid data format: {0}")]
    InvalidData(String),
    
    #[error("Windows API error: {0}")]
    WindowsApi(String),
}

pub type Result<T> = std::result::Result<T, ACCError>;