//! High-rate physics recorder using rkyv.

use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::record::{GraphicsRecord, PhysicsRecord, StaticsRecord};

/// Target physics update rate (ACC/AC Rally).
pub const TARGET_HZ: u32 = 333;
/// Target graphics update rate (ACC/AC Rally).
pub const GRAPHICS_TARGET_HZ: u32 = 60;
/// Minimal sleep when no new data – keeps poll rate high to not miss samples.
const POLL_INTERVAL: Duration = Duration::from_micros(500);
const BUFFER_FLUSH_SAMPLES: usize = 333; // flush every ~1 second
const GRAPHICS_BUFFER_FLUSH_SAMPLES: usize = 60; // flush every ~1 second
const FILE_MAGIC: [u8; 4] = *b"ACCR";
const GRAPHICS_FILE_MAGIC: [u8; 4] = *b"ACCG";
const FILE_VERSION: u16 = 1;

/// Records physics data to an rkyv file at maximum rate.
pub struct Recorder {
    buffer: Vec<PhysicsRecord>,
    writer: BufWriter<std::fs::File>,
    graphics_buffer: Option<Vec<GraphicsRecord>>,
    graphics_writer: Option<BufWriter<std::fs::File>>,
    start_time: Instant,
    sample_count: u64,
    graphics_sample_count: u64,
}

impl Recorder {
    /// Create a new recorder, writing to the given path.
    /// Overwrites existing file. Writes a companion .json with format metadata.
    pub fn new(path: impl AsRef<Path>, statics: Option<&StaticsRecord>, enable_graphics: bool) -> std::io::Result<Self> {
        let path = path.as_ref();
        crate::format_meta::write_format_metadata(path, statics)?;
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::with_capacity(2 * 1024 * 1024, file);

        // Write file header
        writer.write_all(&FILE_MAGIC)?;
        writer.write_all(&FILE_VERSION.to_le_bytes())?;
        writer.write_all(&(TARGET_HZ as u32).to_le_bytes())?; // sample rate
        writer.write_all(&[0u8; 6])?; // reserved, total header 16 bytes

        // Create graphics file only if enabled
        let (graphics_buffer, graphics_writer) = if enable_graphics {
            let graphics_path = path.with_extension("graphics.rkyv");
            let graphics_file = std::fs::File::create(graphics_path)?;
            let mut graphics_writer = BufWriter::with_capacity(512 * 1024, graphics_file);
            
            graphics_writer.write_all(&GRAPHICS_FILE_MAGIC)?;
            graphics_writer.write_all(&FILE_VERSION.to_le_bytes())?;
            graphics_writer.write_all(&(GRAPHICS_TARGET_HZ as u32).to_le_bytes())?;
            graphics_writer.write_all(&[0u8; 6])?;
            
            (Some(Vec::with_capacity(GRAPHICS_BUFFER_FLUSH_SAMPLES * 2)), Some(graphics_writer))
        } else {
            (None, None)
        };

        Ok(Self {
            buffer: Vec::with_capacity(BUFFER_FLUSH_SAMPLES * 2),
            writer,
            graphics_buffer,
            graphics_writer,
            start_time: Instant::now(),
            sample_count: 0,
            graphics_sample_count: 0,
        })
    }

    /// Record a physics snapshot. Flushes to disk when buffer is full.
    pub fn record(&mut self, record: PhysicsRecord) -> std::io::Result<()> {
        self.buffer.push(record);
        self.sample_count += 1;

        if self.buffer.len() >= BUFFER_FLUSH_SAMPLES {
            self.flush()?;
        }

        Ok(())
    }

    /// Record a graphics snapshot. Flushes to disk when buffer is full.
    pub fn record_graphics(&mut self, record: GraphicsRecord) -> std::io::Result<()> {
        if let Some(buffer) = &mut self.graphics_buffer {
            buffer.push(record);
            self.graphics_sample_count += 1;

            if buffer.len() >= GRAPHICS_BUFFER_FLUSH_SAMPLES {
                self.flush_graphics()?;
            }
        }

        Ok(())
    }

    /// Flush buffered records to disk.
    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let records = std::mem::take(&mut self.buffer);
        let bytes = rkyv::to_bytes::<_, 1024>(&records).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        self.writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;

        Ok(())
    }

    /// Flush buffered graphics records to disk.
    pub fn flush_graphics(&mut self) -> std::io::Result<()> {
        if let (Some(buffer), Some(writer)) = (&mut self.graphics_buffer, &mut self.graphics_writer) {
            if buffer.is_empty() {
                return Ok(());
            }

            let records = std::mem::take(buffer);
            let bytes = rkyv::to_bytes::<_, 1024>(&records).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;

            writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
            writer.write_all(&bytes)?;
            writer.flush()?;
        }

        Ok(())
    }

    /// Total samples recorded.
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    /// Elapsed recording time.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.flush();
        let _ = self.flush_graphics();
    }
}

/// Sleep duration when waiting for new data (short to not miss 333 Hz updates).
pub fn poll_interval() -> Duration {
    POLL_INTERVAL
}
