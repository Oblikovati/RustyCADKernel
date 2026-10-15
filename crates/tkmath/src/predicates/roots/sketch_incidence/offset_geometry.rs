// SPDX-License-Identifier: GPL-2.0-only
//! Exact geometry shared by offsets: travel sense, circle radii, axis lengths,
//! tangents and squared speeds of homogeneous pieces (ADR 0080).
//!
//! `Geom2d_OffsetCurve` moves each basis point by `d (C'_y, -C'_x) / |C'|`, to the
//! right of the travel direction. A counterclockwise circle therefore grows to
//! radius `r + d`; a clockwise arc shrinks to `r - d`.
use super::super::certificate_arithmetic::ExactRatio;
use super::input::{SketchConicBounds, SketchIncidenceCurve, SketchIncidenceOffset};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::spans::HomogeneousPoint;
use super::surd_square::ratio_sqrt;

/// True when a conic with these bounds travels counterclockwise (a full conic does).
pub(super) fn counterclockwise(bounds: Option<&SketchConicBounds>) -> bool {
    match bounds {
        None => true,
        Some(SketchConicBounds::Rays(sector)) => sector.counterclockwise,
        Some(SketchConicBounds::Angles(angles)) => angles.counterclockwise,
    }
}
/// The distance along the outward normal of a conic basis traveled in its sense.
pub(super) fn outward_distance(
    offset: &SketchIncidenceOffset,
    bounds: Option<&SketchConicBounds>,
) -> ExactRatio {
    let distance = ratio(offset.distance);
    if counterclockwise(bounds) {
        return distance;
    }
    distance.negated()
}
/// Exact radius `r +- d` of a circle offset; None for other bases.
pub(super) fn circle_radius(offset: &SketchIncidenceOffset) -> Option<ExactRatio> {
    let SketchIncidenceCurve::Circle { radius, sector, .. } = offset.basis.as_ref() else {
        return None;
    };
    Some(ratio(*radius).add(&outward_distance(offset, sector.as_ref())))
}
/// Exact `|axis|` when it is rational; example: `rational_length([3., 4.])` is 5.
pub(super) fn rational_length(axis: [f64; 2]) -> Option<ExactRatio> {
    let [x, y] = axis.map(ratio);
    ratio_sqrt(&x.multiply(&x).add(&y.multiply(&y)))
}
/// `V = (x' w - x w', y' w - y w')`: the tangent `C' = V / w^2`, same direction.
pub(super) fn tangent(point: &HomogeneousPoint) -> [RationalPolynomial; 2] {
    let slope = point.w.derivative();
    let along = |coordinate: &RationalPolynomial| {
        let moving = coordinate.derivative().multiply(&point.w);
        moving.add(&coordinate.multiply(&slope).scale(&ratio(-1.)))
    };
    [along(&point.x), along(&point.y)]
}
/// `|V|^2`, positive wherever the piece is regular.
pub(super) fn speed_squared(tangent: &[RationalPolynomial; 2]) -> RationalPolynomial {
    let [x, y] = tangent;
    x.multiply(x).add(&y.multiply(y))
}
