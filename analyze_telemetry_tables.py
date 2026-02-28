#!/usr/bin/env python3
"""
Analyze graphics and statics tables in telemetry database.
Generates documentation showing field variability and useful info.
"""

import sqlite3
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Any

def analyze_field_variability(cursor, table: str, field: str, field_type: str) -> Tuple[str, str]:
    """
    Analyze a field for variability and range.
    Returns (variability_status, range_description)
    """
    # Check if field has any non-null values
    cursor.execute(f"SELECT COUNT(*) FROM {table} WHERE {field} IS NOT NULL")
    non_null_count = cursor.fetchone()[0]
    
    if non_null_count == 0:
        return ("no", "no data")
    
    if field_type in ['INTEGER', 'REAL']:
        # For numeric fields, check min/max (excluding zeros as per physics convention)
        cursor.execute(f"""
            SELECT 
                MIN(CASE WHEN {field} != 0 THEN {field} END) as min_val,
                MAX(CASE WHEN {field} != 0 THEN {field} END) as max_val,
                COUNT(DISTINCT {field}) as distinct_count,
                COUNT(*) as total_count
            FROM {table}
            WHERE {field} IS NOT NULL
        """)
        min_val, max_val, distinct_count, total_count = cursor.fetchone()
        
        if min_val is None and max_val is None:
            # All values are 0 or NULL
            return ("no", "constant 0 (no data)")
        
        if distinct_count == 1:
            return ("no", f"constant {min_val}")
        
        # Format range nicely
        if field_type == 'INTEGER':
            range_str = f"{int(min_val) if min_val else 0} … {int(max_val) if max_val else 0}"
        else:
            # For floats, use appropriate precision
            if abs(max_val - min_val) < 0.01:
                range_str = f"~{min_val:.6f} … {max_val:.6f}"
            elif abs(max_val - min_val) < 1:
                range_str = f"~{min_val:.4f} … {max_val:.4f}"
            else:
                range_str = f"~{min_val:.2f} … {max_val:.2f}"
        
        return ("yes", range_str)
    
    else:  # TEXT fields
        cursor.execute(f"""
            SELECT 
                COUNT(DISTINCT {field}) as distinct_count,
                COUNT(*) as total_count
            FROM {table}
            WHERE {field} IS NOT NULL AND {field} != ''
        """)
        distinct_count, total_count = cursor.fetchone()
        
        if distinct_count == 0:
            return ("no", "empty/null")
        
        if distinct_count == 1:
            cursor.execute(f"SELECT {field} FROM {table} WHERE {field} IS NOT NULL AND {field} != '' LIMIT 1")
            value = cursor.fetchone()[0]
            return ("no", f'constant "{value}"')
        
        # Get sample values
        cursor.execute(f"SELECT DISTINCT {field} FROM {table} WHERE {field} IS NOT NULL AND {field} != '' LIMIT 5")
        samples = [row[0] for row in cursor.fetchall()]
        if len(samples) <= 3:
            return ("yes", f"values: {', '.join(repr(s) for s in samples)}")
        else:
            return ("yes", f"{distinct_count} distinct values (e.g., {', '.join(repr(s) for s in samples[:3])}, ...)")

def get_table_schema(cursor, table: str) -> List[Tuple[str, str]]:
    """Get list of (column_name, type) for a table."""
    cursor.execute(f"PRAGMA table_info({table})")
    return [(row[1], row[2]) for row in cursor.fetchall()]

def analyze_table(db_path: Path, table: str) -> Dict[str, Dict[str, Any]]:
    """Analyze all fields in a table."""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Check if table exists and has data
    cursor.execute(f"SELECT COUNT(*) FROM {table}")
    row_count = cursor.fetchone()[0]
    
    if row_count == 0:
        print(f"Warning: {table} table is empty", file=sys.stderr)
        return {}
    
    schema = get_table_schema(cursor, table)
    results = {}
    
    # Skip metadata fields
    skip_fields = {'recording_id', 'time_offset', 'packet_id', 'id'}
    
    for field_name, field_type in schema:
        if field_name in skip_fields:
            continue
        
        variability, range_info = analyze_field_variability(cursor, table, field_name, field_type)
        results[field_name] = {
            'type': field_type,
            'variable': variability,
            'range': range_info
        }
    
    conn.close()
    return results

def format_markdown_table(table_name: str, results: Dict[str, Dict[str, Any]], descriptions: Dict[str, str]) -> str:
    """Format results as markdown table."""
    output = [f"## {table_name.title()} Fields\n"]
    output.append("| Field | Description | Variable | Range |")
    output.append("|-------|-------------|----------|-------|")
    
    for field, info in sorted(results.items()):
        desc = descriptions.get(field, "")
        var = info['variable']
        range_info = info['range']
        output.append(f"| `{field}` | {desc} | {var} | {range_info} |")
    
    return "\n".join(output)

# Field descriptions based on schema and AC documentation
GRAPHICS_DESCRIPTIONS = {
    'status': 'Session status (0=off, 1=replay, 2=live, 3=pause)',
    'session_type': 'Session type (0=unknown, 1=practice, 2=qualify, 3=race, etc.)',
    'session_index': 'Current session index',
    'current_time_str': 'Current lap time (formatted string)',
    'last_time_str': 'Last lap time (formatted string)',
    'best_time_str': 'Best lap time (formatted string)',
    'last_sector_time_str': 'Last sector time (formatted string)',
    'completed_lap': 'Number of completed laps',
    'position': 'Current position in race',
    'current_time': 'Current lap time (ms)',
    'last_time': 'Last lap time (ms)',
    'best_time': 'Best lap time (ms)',
    'last_sector_time': 'Last sector time (ms)',
    'number_of_laps': 'Total number of laps in session',
    'delta_lap_time_str': 'Delta to best lap (formatted string)',
    'estimated_lap_time_str': 'Estimated lap time (formatted string)',
    'delta_lap_time': 'Delta to best lap (ms)',
    'estimated_lap_time': 'Estimated lap time (ms)',
    'is_delta_positive': 'Delta is positive (slower than best)',
    'is_valid_lap': 'Current lap is valid',
    'fuel_estimated_laps': 'Estimated laps remaining with current fuel',
    'distance_traveled': 'Distance traveled (m)',
    'normalized_car_position': 'Position on track (0-1)',
    'session_time_left': 'Session time remaining (s)',
    'current_sector_index': 'Current sector (0-based)',
    'is_in_pit': 'Car is in pit box',
    'is_in_pit_lane': 'Car is in pit lane',
    'ideal_line_on': 'Ideal racing line enabled',
    'mandatory_pit_done': 'Mandatory pit stop completed',
    'missing_mandatory_pits': 'Number of mandatory pits remaining',
    'penalty_time': 'Penalty time (s)',
    'penalty': 'Penalty type',
    'flag': 'Flag status (0=none, 1=blue, 2=yellow, 3=black, 4=white, 5=checkered, 6=penalty)',
    'player_car_id': 'Player car ID',
    'active_cars': 'Number of active cars',
    'car_coordinates_x': 'Car world position X',
    'car_coordinates_y': 'Car world position Y',
    'car_coordinates_z': 'Car world position Z',
    'wind_speed': 'Wind speed (m/s)',
    'wind_direction': 'Wind direction (rad)',
    'rain_intensity': 'Current rain intensity',
    'rain_intensity_in_10min': 'Rain intensity forecast +10min',
    'rain_intensity_in_30min': 'Rain intensity forecast +30min',
    'track_grip_status': 'Track grip status',
    'track_status': 'Track status string',
    'clock': 'Session time (s)',
    'tc_level': 'Traction control level',
    'tc_cut_level': 'TC cut level',
    'engine_map': 'Engine map setting',
    'abs_level': 'ABS level',
    'wiper_stage': 'Wiper setting',
    'driver_stint_total_time_left': 'Driver stint total time left (s)',
    'driver_stint_time_left': 'Driver stint time left (s)',
    'rain_tyres': 'Rain tyres equipped',
    'rain_light': 'Rain light on',
    'flashing_light': 'Flashing light on',
    'light_stage': 'Light stage',
    'direction_light_left': 'Left indicator on',
    'direction_light_right': 'Right indicator on',
    'tyre_compound': 'Tyre compound name',
    'is_setup_menu_visible': 'Setup menu is visible',
    'main_display_index': 'Main display page index',
    'secondary_display_index': 'Secondary display page index',
    'fuel_per_lap': 'Fuel consumption per lap (L)',
    'used_fuel': 'Fuel used (L)',
    'exhaust_temp': 'Exhaust temperature (K)',
    'gap_ahead': 'Gap to car ahead (ms)',
    'gap_behind': 'Gap to car behind (ms)',
    'global_yellow': 'Global yellow flag',
    'global_yellow_s1': 'Yellow flag sector 1',
    'global_yellow_s2': 'Yellow flag sector 2',
    'global_yellow_s3': 'Yellow flag sector 3',
    'global_white': 'Global white flag',
    'global_green': 'Global green flag',
    'global_chequered': 'Global checkered flag',
    'global_red': 'Global red flag',
    'mfd_tyre_set': 'MFD tyre set selection',
    'mfd_fuel_to_add': 'MFD fuel to add (L)',
    'mfd_tyre_pressure_fl': 'MFD target tyre pressure FL (psi)',
    'mfd_tyre_pressure_fr': 'MFD target tyre pressure FR (psi)',
    'mfd_tyre_pressure_rl': 'MFD target tyre pressure RL (psi)',
    'mfd_tyre_pressure_rr': 'MFD target tyre pressure RR (psi)',
    'current_tyre_set': 'Current tyre set',
    'strategy_tyre_set': 'Strategy tyre set',
}

STATICS_DESCRIPTIONS = {
    'sm_version': 'Shared memory version',
    'ac_version': 'Assetto Corsa version',
    'number_of_sessions': 'Number of sessions',
    'num_cars': 'Number of cars',
    'track': 'Track name',
    'sector_count': 'Number of sectors',
    'player_name': 'Player first name',
    'player_surname': 'Player surname',
    'player_nick': 'Player nickname',
    'car_model': 'Car model name',
    'max_rpm': 'Maximum RPM',
    'max_fuel': 'Maximum fuel capacity (L)',
    'penalty_enabled': 'Penalties enabled',
    'aid_fuel_rate': 'Fuel consumption aid multiplier',
    'aid_tyre_rate': 'Tyre wear aid multiplier',
    'aid_mechanical_damage': 'Mechanical damage aid multiplier',
    'aid_stability': 'Stability aid level',
    'aid_auto_clutch': 'Auto clutch enabled',
    'pit_window_start': 'Pit window start lap',
    'pit_window_end': 'Pit window end lap',
    'is_online': 'Online session',
    'dry_tyres_name': 'Dry tyres name',
    'wet_tyres_name': 'Wet tyres name',
}

def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze_telemetry_tables.py <telemetry.db>")
        sys.exit(1)
    
    db_path = Path(sys.argv[1])
    if not db_path.exists():
        print(f"Error: Database not found: {db_path}", file=sys.stderr)
        sys.exit(1)
    
    print(f"Analyzing {db_path}...\n", file=sys.stderr)
    
    # Analyze graphics table
    print("Analyzing graphics table...", file=sys.stderr)
    graphics_results = analyze_table(db_path, 'graphics')
    
    # Analyze statics table
    print("Analyzing statics table...", file=sys.stderr)
    statics_results = analyze_table(db_path, 'statics')
    
    # Generate markdown output
    print("\n# Graphics and Statics Fields Analysis\n")
    print("Analysis of graphics and statics tables showing field variability and ranges.")
    print("Data source:", db_path.name)
    print("\n**Variability:** Fields marked 'yes' contain varying data. Fields marked 'no' are constant or contain no useful data.")
    print("**Range:** Shows the range of values (for numeric fields) or sample values (for text fields).")
    print("**Note:** Zeros are excluded from min/max calculations (assumption: 0 = no data).\n")
    print("---\n")
    
    if graphics_results:
        print(format_markdown_table("Graphics", graphics_results, GRAPHICS_DESCRIPTIONS))
        print("\n---\n")
    
    if statics_results:
        print(format_markdown_table("Statics", statics_results, STATICS_DESCRIPTIONS))
        print("\n---\n")
    
    print("\n## Summary\n")
    
    if graphics_results:
        variable_count = sum(1 for info in graphics_results.values() if info['variable'] == 'yes')
        total_count = len(graphics_results)
        print(f"**Graphics:** {variable_count}/{total_count} fields have variable data")
    
    if statics_results:
        variable_count = sum(1 for info in statics_results.values() if info['variable'] == 'yes')
        total_count = len(statics_results)
        print(f"**Statics:** {variable_count}/{total_count} fields have variable data")

if __name__ == '__main__':
    main()
