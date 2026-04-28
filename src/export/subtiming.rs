//! Sub-timing markers from downsampled trajectory (SHP-style samples).
//!
//! Pipeline (user-specified): classify consecutive samples as "straight" when
//! speed > threshold km/h and |steer| < threshold; keep runs that are long enough
//! in space or time; place one marker at the middle sample of each run; drop later
//! markers that lie within `merge_close_m` of the last kept marker (keep first).
//!
//! **Chain order (default on):** markers are merged in ACC lap order: sort by
//! `lap` then `distance_traveled` (always 0→… within a lap). That is **not** the
//! car’s physical travel sense; forward vs. reverse on track comes only from
//! runtime sector transitions — see [`SectorPassTracker`].
//! For live snapping, see [`snap_to_chain_neighbor`] (ring distance, `max_step = 1`).

use std::convert::TryInto;
use std::path::Path;

use shapefile::dbase::{FieldValue, Record, TableWriterBuilder};
use shapefile::{Point, Writer};

/// One input sample (same semantics as `points.shp` rows).
#[derive(Clone, Debug)]
pub struct ShpSample {
    pub idx: u32,
    pub t_sec: f64,
    pub lap: i32,
    pub dist_m: f64,
    pub x: f64,
    pub z: f64,
    pub speed_kmh: f64,
    pub steer_angle: f64,
}

#[derive(Clone, Debug)]
pub struct SubtimingParams {
    pub speed_min_kmh: f64,
    pub steer_max_abs: f64,
    pub min_run_m: f64,
    pub min_run_sec: f64,
    pub merge_close_m: f64,
    /// If true (default), sort markers by `lap` then `dist_m` (ACC lap progression)
    /// before spatial merge, instead of raw discovery time order.
    pub use_chain_order_merge: bool,
}

impl Default for SubtimingParams {
    fn default() -> Self {
        Self {
            speed_min_kmh: 50.0,
            steer_max_abs: 0.1,
            min_run_m: 80.0,
            min_run_sec: 2.0,
            merge_close_m: 40.0,
            use_chain_order_merge: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubtimingMarker {
    pub mark_id: u32,
    pub src_idx: u32,
    pub t_sec: f64,
    pub lap: i32,
    pub dist_m: f64,
    pub x: f64,
    pub z: f64,
    pub run_len_m: f64,
    pub run_n: usize,
}

#[inline]
fn dist_xz(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dz = a.1 - b.1;
    (dx * dx + dz * dz).sqrt()
}

fn run_polyline_length(samples: &[ShpSample], start: usize, end_inclusive: usize) -> f64 {
    if end_inclusive <= start {
        return 0.0;
    }
    let mut s = 0.0;
    for i in start..end_inclusive {
        s += dist_xz(
            (samples[i].x, samples[i].z),
            (samples[i + 1].x, samples[i + 1].z),
        );
    }
    s
}

fn is_straight(s: &ShpSample, p: &SubtimingParams) -> bool {
    s.speed_kmh > p.speed_min_kmh && s.steer_angle.abs() < p.steer_max_abs
}

/// Build sub-timing midpoints from ordered samples (e.g. one lap or full session).
pub fn compute_subtiming_markers(samples: &[ShpSample], p: &SubtimingParams) -> Vec<SubtimingMarker> {
    if samples.is_empty() {
        return Vec::new();
    }

    let mut raw: Vec<SubtimingMarker> = Vec::new();
    let mut i = 0usize;
    while i < samples.len() {
        if !is_straight(&samples[i], p) {
            i += 1;
            continue;
        }
        let start = i;
        while i < samples.len() && is_straight(&samples[i], p) {
            i += 1;
        }
        let end_exclusive = i;
        let end_inclusive = end_exclusive.saturating_sub(1);
        let n = end_exclusive - start;
        if n == 0 {
            continue;
        }
        let run_m = run_polyline_length(samples, start, end_inclusive);
        let run_sec = samples[end_inclusive].t_sec - samples[start].t_sec;
        if run_m < p.min_run_m && run_sec < p.min_run_sec {
            continue;
        }
        let mid = start + (n - 1) / 2;
        let m = &samples[mid];
        raw.push(SubtimingMarker {
            mark_id: 0,
            src_idx: m.idx,
            t_sec: m.t_sec,
            lap: m.lap,
            dist_m: m.dist_m,
            x: m.x,
            z: m.z,
            run_len_m: run_m,
            run_n: n,
        });
    }

    finalize_merged_markers(raw, p)
}

/// ACC lap chain key: `lap` then monotonically increasing `dist_m` within the lap.
fn chain_s(m: &SubtimingMarker) -> f64 {
    const LAP_SCALE: f64 = 10_000_000.0;
    m.lap as f64 * LAP_SCALE + m.dist_m
}

fn sort_markers_chain_order(markers: &mut [SubtimingMarker]) {
    markers.sort_by(|a, b| {
        chain_s(a)
            .partial_cmp(&chain_s(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Walk order given by `markers` (already chain-sorted if desired); drop a marker if
/// it lies within `merge_m` (XZ) of the **previous kept** point in that walk order.
fn merge_close_markers_ordered(markers: Vec<SubtimingMarker>, merge_m: f64) -> Vec<SubtimingMarker> {
    if markers.is_empty() {
        return markers;
    }
    let mut kept: Vec<SubtimingMarker> = Vec::with_capacity(markers.len());
    for m in markers {
        if let Some(prev) = kept.last() {
            let d = dist_xz((prev.x, prev.z), (m.x, m.z));
            if d < merge_m {
                continue;
            }
        }
        kept.push(m);
    }
    for (i, m) in kept.iter_mut().enumerate() {
        m.mark_id = i as u32;
    }
    kept
}

fn finalize_merged_markers(mut raw: Vec<SubtimingMarker>, p: &SubtimingParams) -> Vec<SubtimingMarker> {
    if raw.is_empty() {
        return raw;
    }
    if !p.use_chain_order_merge {
        return merge_close_markers_ordered(raw, p.merge_close_m);
    }
    sort_markers_chain_order(&mut raw);
    merge_close_markers_ordered(raw, p.merge_close_m)
}

/// Shortest step count between chain indices `a` and `b` on a closed ring of `n` markers.
pub fn cyclic_chain_index_dist(n: usize, a: usize, b: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let da = a.abs_diff(b);
    da.min(n - da)
}

/// Pick the nearest marker within `radius_m` of `(x,z)`, optionally restricted to at most
/// `max_step` away from `prev_index` on the **closed ring** (expected previous/next sector).
pub fn snap_to_chain_neighbor(
    markers: &[SubtimingMarker],
    x: f64,
    z: f64,
    radius_m: f64,
    prev_index: Option<usize>,
    max_step: usize,
) -> Option<usize> {
    let n = markers.len();
    if n == 0 {
        return None;
    }
    let mut best: Option<(usize, f64)> = None;
    for (i, m) in markers.iter().enumerate() {
        let d = dist_xz((x, z), (m.x, m.z));
        if d > radius_m {
            continue;
        }
        if let Some(pi) = prev_index {
            if cyclic_chain_index_dist(n, i, pi) > max_step {
                continue;
            }
        }
        if best.map_or(true, |(_, bd)| d < bd) {
            best = Some((i, d));
        }
    }
    best.map(|(i, _)| i)
}

/// Sense of travel through the numbered sub-sector ring `0..n-1` (export / `mark_id` order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorTravelDirection {
    /// Each crossing goes `(i + 1) mod n` — same sense as ACC `dist_m` along that marker order.
    Increasing,
    /// Each crossing goes `(i - 1 + n) mod n` (physical reverse through the same ring).
    Decreasing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SectorPassEvent {
    /// First sector; next different sector must be a ring neighbour to lock direction.
    Anchored { sector: usize },
    /// Valid neighbour step (`to` is `from±1` mod `n`).
    Step {
        from: usize,
        to: usize,
        direction: SectorTravelDirection,
    },
    /// Same sector as before.
    NoStep { sector: usize },
    /// `to` is not a neighbour of `from` on the ring.
    Unexpected { from: usize, to: usize },
    /// Step sense disagrees with an already locked direction (no state change).
    DirectionConflict {
        from: usize,
        to: usize,
        established: SectorTravelDirection,
        observed_step: SectorTravelDirection,
    },
}

/// Tracks which sub-sector the player is in; **forward vs. reverse** is inferred only from
/// whether consecutive sectors follow `i+1` or `i-1` (mod `n`), not from `distance_traveled`.
#[derive(Clone, Debug)]
pub struct SectorPassTracker {
    n: usize,
    current: Option<usize>,
    direction: Option<SectorTravelDirection>,
}

impl SectorPassTracker {
    pub fn new(n: usize) -> Self {
        Self {
            n: n.max(1),
            current: None,
            direction: None,
        }
    }

    pub fn n_sectors(&self) -> usize {
        self.n
    }

    pub fn current_sector(&self) -> Option<usize> {
        self.current
    }

    pub fn locked_direction(&self) -> Option<SectorTravelDirection> {
        self.direction
    }

    /// Register the player’s current sector index `sector` in `0..n`.
    pub fn observe(&mut self, sector: usize) -> SectorPassEvent {
        assert!(sector < self.n, "sector index out of range");
        let prev = match self.current {
            None => {
                self.current = Some(sector);
                return SectorPassEvent::Anchored { sector };
            }
            Some(p) => p,
        };
        if sector == prev {
            return SectorPassEvent::NoStep { sector };
        }
        let forward_next = (prev + 1) % self.n;
        let back_next = (prev + self.n - 1) % self.n;
        if sector != forward_next && sector != back_next {
            return SectorPassEvent::Unexpected { from: prev, to: sector };
        }
        let step_dir = if sector == forward_next {
            SectorTravelDirection::Increasing
        } else {
            SectorTravelDirection::Decreasing
        };
        if let Some(d) = self.direction {
            if d != step_dir {
                return SectorPassEvent::DirectionConflict {
                    from: prev,
                    to: sector,
                    established: d,
                    observed_step: step_dir,
                };
            }
        } else {
            self.direction = Some(step_dir);
        }
        self.current = Some(sector);
        SectorPassEvent::Step {
            from: prev,
            to: sector,
            direction: step_dir,
        }
    }

    /// Clear sector only (e.g. new lap) while keeping locked travel sense.
    pub fn reset_position(&mut self) {
        self.current = None;
    }

    pub fn reset_all(&mut self) {
        self.current = None;
        self.direction = None;
    }
}

/// Write point shapefile of sub-timing markers (same folder naming as `*.subtiming.shp`).
pub fn write_subtiming_shapefile(path: &Path, markers: &[SubtimingMarker]) -> Result<(), Box<dyn std::error::Error>> {
    let table_builder = TableWriterBuilder::new()
        .add_numeric_field("mark_id".try_into()?, 8, 0)
        .add_numeric_field("src_idx".try_into()?, 12, 0)
        .add_numeric_field("t_sec".try_into()?, 12, 3)
        .add_numeric_field("lap".try_into()?, 8, 0)
        .add_numeric_field("dist_m".try_into()?, 12, 3)
        .add_numeric_field("run_len_m".try_into()?, 12, 3)
        .add_numeric_field("run_n".try_into()?, 8, 0);

    let mut writer = Writer::from_path(path, table_builder)?;
    for m in markers {
        let pt = Point::new(m.x, m.z);
        let mut rec = Record::default();
        rec.insert("mark_id".to_string(), FieldValue::Numeric(Some(m.mark_id as f64)));
        rec.insert("src_idx".to_string(), FieldValue::Numeric(Some(m.src_idx as f64)));
        rec.insert("t_sec".to_string(), FieldValue::Numeric(Some(m.t_sec)));
        rec.insert("lap".to_string(), FieldValue::Numeric(Some(m.lap as f64)));
        rec.insert("dist_m".to_string(), FieldValue::Numeric(Some(m.dist_m)));
        rec.insert("run_len_m".to_string(), FieldValue::Numeric(Some(m.run_len_m)));
        rec.insert("run_n".to_string(), FieldValue::Numeric(Some(m.run_n as f64)));
        writer.write_shape_and_record(&pt, &rec)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(idx: u32, t: f64, x: f64, z: f64, speed: f64, steer: f64) -> ShpSample {
        ShpSample {
            idx,
            t_sec: t,
            lap: 0,
            dist_m: t * 50.0,
            x,
            z,
            speed_kmh: speed,
            steer_angle: steer,
        }
    }

    #[test]
    fn straight_run_midpoint_and_merge() {
        let p = SubtimingParams {
            speed_min_kmh: 50.0,
            steer_max_abs: 0.2,
            min_run_m: 50.0,
            min_run_sec: 0.0,
            merge_close_m: 100.0,
            ..Default::default()
        };
        // Run A: 0..12 straight (~110 m chord path), then one crooked sample, then run B far away
        let mut v = Vec::new();
        for i in 0..12 {
            v.push(sample(i, i as f64 * 0.1, i as f64 * 10.0, 0.0, 100.0, 0.0));
        }
        v.push(sample(12, 1.2, 120.0, 0.0, 100.0, 0.5));
        for i in 13..25 {
            v.push(sample(
                i,
                i as f64 * 0.1,
                500.0 + (i - 13) as f64 * 10.0,
                0.0,
                100.0,
                0.0,
            ));
        }
        let m = compute_subtiming_markers(&v, &p);
        assert_eq!(m.len(), 2);

        // Large merge radius: second midpoint within merge_m of first -> keep first only
        let p2 = SubtimingParams {
            merge_close_m: 1_000.0,
            ..p
        };
        let m2 = compute_subtiming_markers(&v, &p2);
        assert_eq!(m2.len(), 1);
    }

    #[test]
    fn sector_tracker_forward_then_no_step() {
        let mut t = SectorPassTracker::new(5);
        assert_eq!(t.observe(2), SectorPassEvent::Anchored { sector: 2 });
        assert_eq!(
            t.observe(3),
            SectorPassEvent::Step {
                from: 2,
                to: 3,
                direction: SectorTravelDirection::Increasing,
            }
        );
        assert_eq!(t.locked_direction(), Some(SectorTravelDirection::Increasing));
        assert_eq!(t.observe(3), SectorPassEvent::NoStep { sector: 3 });
    }

    #[test]
    fn sector_tracker_reverse_locks_decreasing() {
        let mut t = SectorPassTracker::new(5);
        assert!(matches!(t.observe(3), SectorPassEvent::Anchored { .. }));
        assert_eq!(
            t.observe(2),
            SectorPassEvent::Step {
                from: 3,
                to: 2,
                direction: SectorTravelDirection::Decreasing,
            }
        );
        assert_eq!(t.locked_direction(), Some(SectorTravelDirection::Decreasing));
    }

    #[test]
    fn sector_tracker_unexpected_skip() {
        let mut t = SectorPassTracker::new(6);
        t.observe(1);
        assert_eq!(
            t.observe(4),
            SectorPassEvent::Unexpected { from: 1, to: 4 }
        );
        assert_eq!(t.current_sector(), Some(1));
    }

    #[test]
    fn sector_tracker_direction_conflict_no_update() {
        let mut t = SectorPassTracker::new(5);
        t.observe(1);
        t.observe(2);
        assert!(matches!(
            t.observe(1),
            SectorPassEvent::DirectionConflict {
                established: SectorTravelDirection::Increasing,
                observed_step: SectorTravelDirection::Decreasing,
                ..
            }
        ));
        assert_eq!(t.current_sector(), Some(2));
    }

    #[test]
    fn snap_respects_ring_neighbor() {
        let markers = vec![
            SubtimingMarker {
                mark_id: 0,
                src_idx: 0,
                t_sec: 0.0,
                lap: 0,
                dist_m: 0.0,
                x: 0.0,
                z: 0.0,
                run_len_m: 10.0,
                run_n: 1,
            },
            SubtimingMarker {
                mark_id: 1,
                src_idx: 1,
                t_sec: 1.0,
                lap: 0,
                dist_m: 100.0,
                x: 100.0,
                z: 0.0,
                run_len_m: 10.0,
                run_n: 1,
            },
            SubtimingMarker {
                mark_id: 2,
                src_idx: 2,
                t_sec: 2.0,
                lap: 0,
                dist_m: 200.0,
                x: 50.0,
                z: 0.0,
                run_len_m: 10.0,
                run_n: 1,
            },
        ];
        // Near sector 2 but from prev=1 only ±1 allowed -> may pick 2 (dist 50) or 0 (dist 50)
        let s = snap_to_chain_neighbor(&markers, 52.0, 0.0, 80.0, Some(1), 1);
        assert_eq!(s, Some(2));
        let s2 = snap_to_chain_neighbor(&markers, 2.0, 0.0, 80.0, Some(1), 1);
        assert_eq!(s2, Some(0));
        // max_step 0 → only same sector; point not within radius of marker 1
        let s3 = snap_to_chain_neighbor(&markers, 2.0, 0.0, 10.0, Some(1), 0);
        assert_eq!(s3, None);
    }
}
