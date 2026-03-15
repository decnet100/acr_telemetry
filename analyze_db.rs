use rusqlite::{Connection, Result};
use std::collections::HashMap;

fn main() -> Result<()> {
    let conn = Connection::open("c:/temp/acc/telemetry.db")?;
    
    // Get all column names from physics table
    let mut stmt = conn.prepare("PRAGMA table_info(physics)")?;
    let columns: Vec<String> = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        Ok(name)
    })?.collect::<Result<Vec<_>>>()?;
    
    println!("=== PHYSICS TABLE ANALYSIS ===\n");
    println!("Total columns: {}\n", columns.len());
    
    // Count total rows
    let total_rows: i64 = conn.query_row("SELECT COUNT(*) FROM physics", [], |row| row.get(0))?;
    println!("Total rows: {}\n", total_rows);
    
    // Analyze each column for variability
    let mut variable_fields = Vec::new();
    let mut constant_fields = Vec::new();
    
    for col in &columns {
        if col == "recording_id" || col == "time_offset" {
            continue; // Skip metadata
        }
        
        let query = format!("SELECT COUNT(DISTINCT {}) as unique_vals, MIN({}) as min_val, MAX({}) as max_val FROM physics WHERE {} IS NOT NULL AND {} != 0", 
            col, col, col, col, col);
        
        let result: Result<(i64, Option<f64>, Option<f64>)> = conn.query_row(&query, [], |row| {
            Ok((row.get(0)?, row.get(1).ok(), row.get(2).ok()))
        });
        
        match result {
            Ok((unique, min, max)) => {
                if unique > 1 {
                    variable_fields.push((col.clone(), unique, min, max));
                } else {
                    constant_fields.push((col.clone(), unique));
                }
            }
            Err(_) => {
                // String column, check differently
                let query = format!("SELECT COUNT(DISTINCT {}) as unique_vals FROM physics WHERE {} IS NOT NULL AND {} != ''", 
                    col, col, col);
                if let Ok(unique) = conn.query_row::<i64, _, _>(&query, [], |row| row.get(0)) {
                    if unique > 1 {
                        variable_fields.push((col.clone(), unique, None, None));
                    } else {
                        constant_fields.push((col.clone(), unique));
                    }
                }
            }
        }
    }
    
    println!("=== VARIABLE FIELDS ({}) ===", variable_fields.len());
    for (field, unique, min, max) in &variable_fields {
        match (min, max) {
            (Some(min), Some(max)) => println!("{}: {} unique values, range [{:.3} to {:.3}]", field, unique, min, max),
            _ => println!("{}: {} unique values", field, unique),
        }
    }
    
    println!("\n=== CONSTANT/ZERO FIELDS ({}) ===", constant_fields.len());
    for (field, unique) in &constant_fields {
        println!("{}: {} unique value(s)", field, unique);
    }
    
    // Check for missing tyre_contact fields
    println!("\n=== CHECKING TYRE_CONTACT FIELDS ===");
    let contact_fields = vec![
        "tyre_contact_point_fl_x", "tyre_contact_point_fl_y", "tyre_contact_point_fl_z",
        "tyre_contact_point_fr_x", "tyre_contact_point_fr_y", "tyre_contact_point_fr_z",
        "tyre_contact_point_rl_x", "tyre_contact_point_rl_y", "tyre_contact_point_rl_z",
        "tyre_contact_point_rr_x", "tyre_contact_point_rr_y", "tyre_contact_point_rr_z",
    ];
    
    for field in contact_fields {
        if columns.contains(&field.to_string()) {
            let query = format!("SELECT COUNT(DISTINCT {}) FROM physics WHERE {} != 0", field, field);
            let unique: i64 = conn.query_row(&query, [], |row| row.get(0))?;
            println!("{}: {} unique values", field, unique);
        } else {
            println!("{}: MISSING FROM TABLE", field);
        }
    }
    
    Ok(())
}
