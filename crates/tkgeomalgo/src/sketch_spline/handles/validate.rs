// SPDX-License-Identifier: GPL-2.0-only
use super::{Failure, Point2, SketchHandleOptionsV1, SketchSplineHandle, decline};
use std::collections::BTreeSet;
pub(super) fn request(
    points: &[Point2],
    handles: &[SketchSplineHandle],
    options: SketchHandleOptionsV1,
) -> Result<(), Failure> {
    if handles.is_empty() || points.len() > (i32::MAX as usize - 5) / 3 {
        return Err(decline(format!(
            "handle count {}, point count {}: expected active handles and <= (i32::MAX-5)/3 points",
            handles.len(),
            points.len()
        )));
    }
    require_indices(points.len(), handles)?;
    require_limits(options)
}
fn require_indices(count: usize, handles: &[SketchSplineHandle]) -> Result<(), Failure> {
    let mut seen: BTreeSet<usize> = BTreeSet::new();
    for handle in handles {
        if handle.fit_index >= count || !seen.insert(handle.fit_index) {
            return Err(decline(format!(
                "handle index {}: expected unique source index < {}",
                handle.fit_index, count
            )));
        }
        super::super::validate::coordinates(&[handle.end])?;
    }
    Ok(())
}
fn require_limits(options: SketchHandleOptionsV1) -> Result<(), Failure> {
    for limit in [
        options.maximum_position_residual,
        options.maximum_derivative_residual,
    ] {
        if !limit.is_finite() || limit <= 0.0 {
            return Err(decline(format!(
                "handled residual bound {limit}: expected finite positive distance"
            )));
        }
    }
    Ok(())
}
