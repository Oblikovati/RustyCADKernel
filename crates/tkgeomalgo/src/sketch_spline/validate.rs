// SPDX-License-Identifier: GPL-2.0-only
use super::{SketchSplineOptionsV1, decline};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;
pub(super) fn request(
    points: &[Point2],
    closed: bool,
    options: SketchSplineOptionsV1,
) -> Result<(), Failure> {
    let minimum = if closed { 3 } else { 2 };
    if points.len() < minimum || points.len() > (i32::MAX as usize - 7) {
        return Err(decline(format!(
            "point count {}, closed {closed}: expected {minimum}..=i32::MAX-7 points",
            points.len()
        )));
    }
    coordinates(points)?;
    let limits = [options.minimum_pivot, options.maximum_fit_residual];
    if limits
        .into_iter()
        .chain(options.maximum_seam_residuals)
        .any(|value| !value.is_finite() || value <= 0.0)
    {
        return Err(decline(format!(
            "options {options:?}: expected finite positive pivot and spatial/derivative bounds"
        )));
    }
    Ok(())
}

pub(super) fn coordinates(points: &[Point2]) -> Result<(), Failure> {
    if let Some((index, point)) = points
        .iter()
        .enumerate()
        .find(|(_, point)| !point.x().is_finite() || !point.y().is_finite())
    {
        return Err(decline(format!(
            "point {index} = {point:?}: expected finite coordinates"
        )));
    }
    Ok(())
}
