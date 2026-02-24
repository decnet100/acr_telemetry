# Grafana annotations from recording markers

When you use the batch scripts `acr_note_good.bat` / `acr_note_aborted.bat` (or append lines with `[elapsed Ns]` and `#marker TAG#` to `acr_notes`), the recorder writes them into `<stem>.notes.json`. On `acr_export ... --sqlite`, these are inserted into the **`annotations`** table so Grafana can show them on the time axis.

## Annotation table

| Column           | Type   | Description |
|------------------|--------|-------------|
| recording_id     | int    | Links to `recordings.id` |
| time_offset_sec  | real   | Seconds since recording start (same scale as `physics.time_offset`) |
| time_end_sec     | real   | Optional; if set, annotation is a **range** (region), else a **point** |
| text             | text   | Label shown in Grafana |
| tag              | text   | e.g. `good`, `aborted` for filtering |

Grafana supports **point** annotations (single time) and **region** annotations (time + timeEnd). Our schema supports both via `time_end_sec`.

## Adding an annotation layer in Grafana

1. Edit a panel or the dashboard → **Add visualization** or **Edit**.
2. In the right sidebar, open **Annotations** (or **Dashboard settings** → **Annotations**).
3. **New annotation** → choose your **SQLite** datasource.
4. Use a query that returns columns: **time** (epoch milliseconds), **timeEnd** (optional, ms), **text**, **tags**.

The dashboard time axis uses `(1000000000 + time_offset) * 1000` milliseconds (so recordings align around 2001-09-09). Use the same base for annotations:

```sql
SELECT
  (1000000000 + time_offset_sec) * 1000 AS time,
  (1000000000 + COALESCE(time_end_sec, time_offset_sec)) * 1000 AS timeEnd,
  text,
  tag AS tags
FROM annotations
WHERE recording_id = $recording_id
  AND (1000000000 + time_offset_sec) * 1000 BETWEEN $__from AND $__to
```

Replace `$recording_id` with your dashboard variable (e.g. `$id_a` or the recording dropdown). Grafana will then show markers (e.g. “good”, “aborted”) at the correct positions on the charts.
