// SPDX-License-Identifier: GPL-2.0-only
use super::{Failure, Point2, SketchSplineDefinition, Vector2};
pub(super) fn construct(
    points: &[Point2],
    stations: &[f64],
    derivatives: &[Vector2],
) -> Result<SketchSplineDefinition, Failure> {
    let mut poles: Vec<Point2> = vec![points[0]];
    let mut knots: Vec<f64> = vec![0.0];
    for (span, interval) in stations.windows(2).enumerate() {
        append_span(
            &mut poles,
            points,
            derivatives,
            span,
            interval[1] - interval[0],
        );
        knots.extend([interval[0]; 3]);
    }
    knots.extend([1.0; 4]);
    super::super::validate::coordinates(&poles)?;
    Ok(SketchSplineDefinition::new(3, poles, knots))
}
fn append_span(
    poles: &mut Vec<Point2>,
    points: &[Point2],
    derivatives: &[Vector2],
    span: usize,
    width: f64,
) {
    let first = span % points.len();
    let last = (span + 1) % points.len();
    // Hermite-to-Bezier identity; the final shared endpoints stay authored, never perturbed.
    poles.push(displaced(points[first], derivatives[first], width / 3.0));
    poles.push(displaced(points[last], derivatives[last], -width / 3.0));
    poles.push(points[last]);
}
fn displaced(point: Point2, derivative: Vector2, factor: f64) -> Point2 {
    Point2::new(
        point.x() + derivative.x() * factor,
        point.y() + derivative.y() * factor,
    )
}
