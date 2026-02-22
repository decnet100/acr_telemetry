//! Read rkyv telemetry files written by acr_recorder.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::record::{GraphicsRecord, PhysicsRecord};

const FILE_MAGIC: [u8; 4] = *b"ACCR";
const GRAPHICS_FILE_MAGIC: [u8; 4] = *b"ACCG";

/// Read all physics records from an acr_recorder rkyv file.
pub fn read_rkyv(path: impl AsRef<Path>) -> std::io::Result<(u32, Vec<PhysicsRecord>)> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != FILE_MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid magic: expected ACCR, got {:?}", magic),
        ));
    }

    let mut version = [0u8; 2];
    reader.read_exact(&mut version)?;
    let _version = u16::from_le_bytes(version);

    let mut sample_rate = [0u8; 4];
    reader.read_exact(&mut sample_rate)?;
    let sample_rate = u32::from_le_bytes(sample_rate);

    reader.read_exact(&mut [0u8; 6])?; // reserved

    let mut records = Vec::new();
    let mut len_buf = [0u8; 4];

    while reader.read_exact(&mut len_buf).is_ok() {
        let chunk_len = u32::from_le_bytes(len_buf) as usize;
        if chunk_len == 0 {
            break;
        }

        let mut chunk = vec![0u8; chunk_len];
        reader.read_exact(&mut chunk)?;

        let chunk_records: Vec<PhysicsRecord> = unsafe {
            rkyv::from_bytes_unchecked(&chunk)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
        };
        records.extend(chunk_records);
    }

    Ok((sample_rate, records))
}

/// Read all graphics records from an acr_recorder graphics rkyv file.
pub fn read_graphics_rkyv(path: impl AsRef<Path>) -> std::io::Result<(u32, Vec<GraphicsRecord>)> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != GRAPHICS_FILE_MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid magic: expected ACCG, got {:?}", magic),
        ));
    }

    let mut version = [0u8; 2];
    reader.read_exact(&mut version)?;
    let _version = u16::from_le_bytes(version);

    let mut sample_rate = [0u8; 4];
    reader.read_exact(&mut sample_rate)?;
    let sample_rate = u32::from_le_bytes(sample_rate);

    reader.read_exact(&mut [0u8; 6])?; // reserved

    let mut records = Vec::new();
    let mut len_buf = [0u8; 4];

    while reader.read_exact(&mut len_buf).is_ok() {
        let chunk_len = u32::from_le_bytes(len_buf) as usize;
        if chunk_len == 0 {
            break;
        }

        let mut chunk = vec![0u8; chunk_len];
        reader.read_exact(&mut chunk)?;

        let chunk_records: Vec<GraphicsRecord> = unsafe {
            rkyv::from_bytes_unchecked(&chunk)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
        };
        records.extend(chunk_records);
    }

    Ok((sample_rate, records))
}
