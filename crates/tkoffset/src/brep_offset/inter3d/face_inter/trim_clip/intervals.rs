//! Sorting, tolerance-deduplicating, and classifying section parameter intervals.
use super::{Curve, FClass2d, inside_both};
use rustycad_tkernel::{precision, standard::Failure};

pub(super) fn normalize_events(events: &mut Vec<f64>) {
    events.sort_by(f64::total_cmp);
    events.dedup_by(|left, right| (*left - *right).abs() <= precision::p_confusion());
}

pub(super) fn inside_intervals(
    curve: &Curve,
    events: &[f64],
    first: &FClass2d,
    second: &FClass2d,
) -> Result<Vec<(f64, f64)>, Failure> {
    let mut intervals = Vec::new();
    for pair in events.windows(2) {
        append_if_inside(curve, (pair[0], pair[1]), first, second, &mut intervals)?;
    }
    Ok(intervals)
}

fn append_if_inside(
    curve: &Curve,
    interval: (f64, f64),
    first: &FClass2d,
    second: &FClass2d,
    intervals: &mut Vec<(f64, f64)>,
) -> Result<(), Failure> {
    if interval.1 - interval.0 <= precision::p_confusion() {
        return Ok(());
    }
    if !inside_both(curve, midpoint(interval), first, second)? {
        return Ok(());
    }
    merge_or_append(intervals, interval);
    Ok(())
}

fn midpoint((first, last): (f64, f64)) -> f64 {
    first + (last - first) / 2.0
}

fn merge_or_append(intervals: &mut Vec<(f64, f64)>, interval: (f64, f64)) {
    if let Some(previous) = intervals.last_mut()
        && (previous.1 - interval.0).abs() <= precision::p_confusion()
    {
        previous.1 = interval.1;
        return;
    }
    intervals.push(interval);
}
