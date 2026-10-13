// SPDX-License-Identifier: GPL-2.0-only
use super::{Failure, Point2, SketchHandleOptionsV1, SketchSplineDefinition, Vector2};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
type CheckedResiduals = ([f64; 2], Option<[f64; 2]>);
pub(super) fn measure(
    definition: &SketchSplineDefinition,
    points: &[Point2],
    stations: &[f64],
    derivatives: &[Vector2],
    closed: bool,
    options: SketchHandleOptionsV1,
) -> Result<CheckedResiduals, Failure> {
    let curve = definition.curve()?;
    let maximum = station_errors(&curve, points, stations, derivatives, options)?;
    let seam = if closed {
        Some(closure(&curve, stations.len(), options)?)
    } else {
        None
    };
    Ok((maximum, seam))
}
fn station_errors(
    curve: &Geom2dBSplineCurve,
    points: &[Point2],
    stations: &[f64],
    derivatives: &[Vector2],
    options: SketchHandleOptionsV1,
) -> Result<[f64; 2], Failure> {
    let mut maximum: [f64; 2] = [0.0; 2];
    for (span, interval) in stations.windows(2).enumerate() {
        for (side, &parameter) in interval.iter().enumerate() {
            let source = (span + side) % points.len();
            let actual = curve.local_d1(parameter, span as i32 + 1, span as i32 + 2)?;
            let residuals = difference(actual, (points[source], derivatives[source]));
            require(residuals, options, &format!("span {span} side {side}"))?;
            maximum = std::array::from_fn(|axis| maximum[axis].max(residuals[axis]));
        }
    }
    Ok(maximum)
}
fn closure(
    curve: &Geom2dBSplineCurve,
    station_count: usize,
    options: SketchHandleOptionsV1,
) -> Result<[f64; 2], Failure> {
    let first = curve.local_d1(0.0, 1, 2)?;
    let last = curve.local_d1(1.0, station_count as i32 - 1, station_count as i32)?;
    let residuals = difference(first, last);
    require(residuals, options, "handled closure")?;
    Ok(residuals)
}
fn difference(actual: (Point2, Vector2), expected: (Point2, Vector2)) -> [f64; 2] {
    [
        (actual.0.x() - expected.0.x()).hypot(actual.0.y() - expected.0.y()),
        (actual.1.x() - expected.1.x()).hypot(actual.1.y() - expected.1.y()),
    ]
}
fn require(
    residuals: [f64; 2],
    options: SketchHandleOptionsV1,
    context: &str,
) -> Result<(), Failure> {
    let limits = [
        options.maximum_position_residual,
        options.maximum_derivative_residual,
    ];
    for (order, (residual, limit)) in residuals.into_iter().zip(limits).enumerate() {
        super::super::measure::require_residual(
            residual,
            limit,
            &format!("{context} derivative order {order}"),
        )?;
    }
    Ok(())
}
