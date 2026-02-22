# Dashboard Setup on a Fresh Grafana Installation

Quick guide to get the AC Rally Full Telemetry Dashboard running.

## 1. Install Grafana (Windows)

1. Download from [grafana.com/grafana/download](https://grafana.com/grafana/download?platform=windows)
2. Run the installer
3. Open http://localhost:3000
4. Log in with **admin / admin**

## 2. Prepare the Database

Export telemetry with `acr_export`:

```cmd
cd [project root]
acr_export telemetry_raw\acc_physics_1771667046.rkyv --sqlite telemetry.db
# Or all .rkyv files:
acr_export --rawDir --sqlite
```

## 3. Add SQLite Datasource

1. **Connections** → **Add new connection** → **SQLite**
2. Enter path to `telemetry.db` (absolute path)
3. **Save & test**
4. Note the datasource **UID** (in URL or settings)

## 4. Match Dashboard to Datasource

1. Open `grafanimate/dashboard.json`
2. Find and replace datasource UID `e8e7df64-1fd3-4995-acb0-f66db5fda3ab` with your UID
3. Save

## 5. Import Dashboard

1. **Dashboards** → **Import** → **Upload JSON file**
2. Select the edited `dashboard.json`

## 6. Select Recording

Use the `recording_id` dropdown at the top. If empty, run `acr_export --sqlite` first.
