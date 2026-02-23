# Dashboard Setup on a Fresh Grafana Installation

Quick guide to get the AC Rally Full Telemetry Dashboard running in Grafana.

## 1. Install Grafana (Windows)

1. Download from [grafana.com/grafana/download](https://grafana.com/grafana/download?platform=windows)
2. Run the installer
3. Open http://localhost:3000
4. Log in with **admin / admin** (this is grafanas initial password for fresh installation, you will get asked to change it right away)

## 2. Prepare the Database

Record some data from the game using acr_recorder.exe, drive for a few seconds at least. This will create an rkyv-file for raw storage. Please note that these files (recording 100+ channels at 333Hz) will become quite large and hard to use if you keep the recorder running for hour-long sessions. For testing, I recommend to do small recordings of not even a full stage.

Export rkyv-telemetry with `acr_export`:

```cmd
cd [project root]
acr_export telemetry_raw\acc_physics_1771667046.rkyv --sqlite c:\telemetry\telemetry.db
# Or all .rkyv files, if your raw-Directory and the path to the telemetry data is already set:
acr_export --rawDir --sqlite
```
As you run this tool in the command line, whenever it successfully enters recordings to your db-file, it tells you the resulting recording id (starting at 0). You will use this number to open this recording later.

## 3. Add SQLite Datasource

1. **Connections** → **Add new connection** → **SQLite**
2. Enter path to `telemetry.db` (absolute path)
3. **Save & test**
4. Note the datasource **UID** (in URL or settings); this will be shown as an alphanumerical code in the URL when you look at this datasource you created in the browser, like localhost:3000//grafana/connections/datasources/edit/ceikogwighs00f -> ceikogwighs00f is your UID

## 4. Match Dashboard to Datasource

1. Open `grafanimate/dashboard.json` with a text editor.
2. Any grafana server will assign their own UID to the SQLite-Datasource we just created. So find and replace the datasource UID from mine `e8e7df64-1fd3-4995-acb0-f66db5fda3ab` with your UID.
3. Save

## 5. Import Dashboard

1. **Dashboards** → **Import** → **Upload JSON file**
2. Select the edited `dashboard.json`

## 6. Select Recording

Use the `recording_id` dropdown at the top; enter the recording you want to open (numbering starting with 0, 1...). If empty, run `acr_export --sqlite` first. If you cannot see any data, set the timeframe accordingly: 2001-09-09 00:00 to 2001-09-10 00:00 is a good starting point (Every recording is starting at unix timestamp 100000000 - that is so that similar recordings can be compared more easily instead of having to change the time around).

## 7. Watch this space

In this folder I'll publish my own dashbords, which I'm playing around with almost daily. Next step will be A-to-B comparisons between different recordings, so you can analyze effects of changes. As the game is nearing v0.3, I'm sure this will bring some changes in the data handling as well.
