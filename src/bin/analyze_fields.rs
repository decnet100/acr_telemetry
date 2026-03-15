//! Analyze telemetry.db to check field variability and completeness

use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "telemetry.db".to_string());
    
    let conn = Connection::open(&db_path)?;
    
    // Get all column names from physics table
    let mut stmt = conn.prepare("PRAGMA table_info(physics)")?;
    let columns: Vec<String> = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        Ok(name)
    })?.collect::<Result<Vec<_>>>()?;
    
    println!("=== PHYSICS TABLE ANALYSIS ===\n");
    println!("Database: {}", db_path);
    println!("Total columns: {}\n", columns.len());
    
    // Count total rows and recordings
    let total_rows: i64 = conn.query_row("SELECT COUNT(*) FROM physics", [], |row| row.get(0))?;
    let recordings: i64 = conn.query_row("SELECT COUNT(DISTINCT recording_id) FROM physics", [], |row| row.get(0))?;
    println!("Total rows: {}", total_rows);
    println!("Recordings: {}\n", recordings);
    
    // Analyze each column for variability
    let mut variable_fields = Vec::new();
    let mut constant_fields = Vec::new();
    
    for col in &columns {
        if col == "recording_id" || col == "time_offset" {
            continue; // Skip metadata
        }
        
        // Try numeric analysis (sample for speed)
        let query = format!(
            "SELECT COUNT(DISTINCT {0}) as unique_vals, MIN({0}) as min_val, MAX({0}) as max_val FROM (SELECT {0} FROM physics LIMIT 50000) WHERE {0} IS NOT NULL AND {0} != 0", 
            col
        );
        
        let result: Result<(i64, Option<f64>, Option<f64>)> = conn.query_row(&query, [], |row| {
            Ok((row.get(0)?, row.get(1).ok(), row.get(2).ok()))
        });
        
        match result {
            Ok((unique, min, max)) => {
                if unique > 1 {
                    variable_fields.push((col.clone(), unique, min, max));
                } else if unique == 1 {
                    constant_fields.push((col.clone(), unique, min, max));
                } else {
                    constant_fields.push((col.clone(), 0, None, None));
                }
            }
            Err(_) => {
                // String column, check differently
                let query = format!(
                    "SELECT COUNT(DISTINCT {}) as unique_vals FROM physics WHERE {} IS NOT NULL AND {} != ''", 
                    col, col, col
                );
                if let Ok(unique) = conn.query_row::<i64, _, _>(&query, [], |row| row.get(0)) {
                    if unique > 1 {
                        variable_fields.push((col.clone(), unique, None, None));
                    } else {
                        constant_fields.push((col.clone(), unique, None, None));
                    }
                }
            }
        }
    }
    
    println!("=== VARIABLE FIELDS ({}) ===", variable_fields.len());
    variable_fields.sort_by(|a, b| a.0.cmp(&b.0));
    for (field, unique, min, max) in &variable_fields {
        match (min, max) {
            (Some(min), Some(max)) => {
                if min.abs() < 0.001 && max.abs() < 0.001 {
                    println!("  {}: {} unique (near-zero)", field, unique);
                } else {
                    println!("  {}: {} unique, range [{:.6} to {:.6}]", field, unique, min, max);
                }
            }
            _ => println!("  {}: {} unique values", field, unique),
        }
    }
    
    println!("\n=== CONSTANT/ZERO FIELDS ({}) ===", constant_fields.len());
    constant_fields.sort_by(|a, b| a.0.cmp(&b.0));
    for (field, unique, min, max) in &constant_fields {
        if *unique == 0 {
            println!("  {}: all NULL or zero", field);
        } else {
            match (min, max) {
                (Some(v), _) => println!("  {}: constant value {:.6}", field, v),
                _ => println!("  {}: {} unique value(s)", field, unique),
            }
        }
    }
    
    // Check for specific missing fields in FIELDS.md
    println!("\n=== CHECKING DOCUMENTED FIELDS ===");
    let documented_fields = vec![
        "tyre_contact_point_fl_x", "tyre_contact_point_fl_y", "tyre_contact_point_fl_z",
        "tyre_contact_normal_fl_x", "tyre_contact_heading_fl_x",
        "local_angular_vel_x", "local_angular_vel_y", "local_angular_vel_z",
    ];
    
    for field in documented_fields {
        if columns.contains(&field.to_string()) {
            println!("  ✓ {} present", field);
        } else {
            println!("  ✗ {} MISSING FROM TABLE", field);
        }
    }
    
    Ok(())
}
