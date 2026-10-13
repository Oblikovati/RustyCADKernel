// SPDX-License-Identifier: GPL-2.0-only
use super::{Failure, Point2, SketchSplineHandle, SketchSplineReport, Vector2, decline};
use rustycad_tkg2d::geom2d::Geom2dCurve;
pub(super) fn effective(
    points: &[Point2],
    closed: bool,
    handles: &[SketchSplineHandle],
    natural: &SketchSplineReport,
) -> Result<(Vec<Vector2>, Vec<usize>), Failure> {
    let curve = natural.definition.curve()?;
    let mut derivatives: Vec<Vector2> = Vec::with_capacity(points.len());
    let mut ignored: Vec<usize> = Vec::new();
    for (index, &station) in natural.stations.iter().take(points.len()).enumerate() {
        let tangent = curve.eval_d1(station)?.d1;
        let handle = handles.iter().find(|handle| handle.fit_index == index);
        let effective = match handle {
            Some(handle) => override_tangent(points, closed, handle, tangent, &mut ignored)?,
            None => tangent,
        };
        require_finite(effective)?;
        derivatives.push(effective);
    }
    Ok((derivatives, ignored))
}
fn override_tangent(
    points: &[Point2],
    closed: bool,
    handle: &SketchSplineHandle,
    natural: Vector2,
    ignored: &mut Vec<usize>,
) -> Result<Vector2, Failure> {
    let anchor = points[handle.fit_index];
    let delta = Vector2::new(handle.end.x() - anchor.x(), handle.end.y() - anchor.y());
    if delta.x() == 0.0 && delta.y() == 0.0 {
        ignored.push(handle.fit_index);
        return Ok(natural);
    }
    let length = representable_length(delta)?;
    let chord = neighbor_length(points, closed, handle.fit_index)? / 3.0;
    let weight = if chord == 0.0 { 1.0 } else { length / chord };
    let magnitude = weight * representable_length(natural)?;
    Ok(Vector2::new(
        (delta.x() * (1.0 / length)) * magnitude,
        (delta.y() * (1.0 / length)) * magnitude,
    ))
}
fn neighbor_length(points: &[Point2], closed: bool, index: usize) -> Result<f64, Failure> {
    let count = points.len();
    let previous = if closed {
        (index + count - 1) % count
    } else {
        index.saturating_sub(1)
    };
    let next = if closed {
        (index + 1) % count
    } else {
        (index + 1).min(count - 1)
    };
    let delta = Vector2::new(
        points[next].x() - points[previous].x(),
        points[next].y() - points[previous].y(),
    );
    representable_length(delta)
}
fn representable_length(vector: Vector2) -> Result<f64, Failure> {
    // Go aa8297d1 uses ordinary squared length; refuse overflow/underflow rather than changing that arithmetic.
    let length = (vector.x() * vector.x() + vector.y() * vector.y()).sqrt();
    if !length.is_finite() || (length == 0.0 && (vector.x() != 0.0 || vector.y() != 0.0)) {
        return Err(decline(format!(
            "vector {vector:?}, length {length}: expected representable finite length"
        )));
    }
    Ok(length)
}
fn require_finite(vector: Vector2) -> Result<(), Failure> {
    if !vector.x().is_finite() || !vector.y().is_finite() {
        return Err(decline(format!(
            "effective derivative {vector:?}: expected finite components"
        )));
    }
    Ok(())
}
