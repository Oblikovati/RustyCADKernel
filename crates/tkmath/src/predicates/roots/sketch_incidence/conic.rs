// SPDX-License-Identifier: GPL-2.0-only
//! Circles and ellipses: substitute the source line into the exact implicit equation.
//!
//! Every binary64 input is an integer multiple of 2^-1074. Each coefficient below is
//! homogeneous in those integers (degree 2 for circles, 6 for ellipses), so the
//! common scale cancels and the equation in `t` is exact. The parameter `t` is
//! unscaled. Both leading coefficients are positive, so no conic overlaps a line.
use super::super::polynomial::IntegerPolynomial;
use crate::predicates::{exact_determinant, exact_dot, exact_point};
use num_bigint::BigInt;

/// Exact integer data of the source line relative to a conic center.
pub(super) struct CenteredLine {
    /// `A - C`.
    pub offset: [BigInt; 2],
    /// `B - A`.
    pub direction: [BigInt; 2],
}
impl CenteredLine {
    pub fn new(points: [[f64; 2]; 2], center: [f64; 2]) -> Self {
        let start = lift(points[0]);
        let end = lift(points[1]);
        let center = lift(center);
        Self {
            offset: [&start[0] - &center[0], &start[1] - &center[1]],
            direction: [&end[0] - &start[0], &end[1] - &start[1]],
        }
    }
    /// Coefficients `[c0, c1]` of the linear function `t -> det(v, A - C + t d)`.
    pub fn cross_with(&self, vector: [f64; 2]) -> [BigInt; 2] {
        let vector = lift(vector);
        [
            exact_determinant(&vector, &self.offset),
            exact_determinant(&vector, &self.direction),
        ]
    }
    /// Coefficients of `t -> dot(v, A - C + t d)`.
    pub fn dot_with(&self, vector: [f64; 2]) -> [BigInt; 2] {
        let vector = lift(vector);
        [
            exact_dot(&vector, &self.offset),
            exact_dot(&vector, &self.direction),
        ]
    }
}
pub(super) fn lift(point: [f64; 2]) -> [BigInt; 2] {
    exact_point(point, 0).expect("admitted finite coordinates")
}
fn lift_scalar(value: f64) -> BigInt {
    lift([value, 0.])[0].clone()
}
/// `|A - C + t d|^2 - r^2`, ascending in `t`.
pub(super) fn circle_equation(line: &CenteredLine, radius: f64) -> Vec<BigInt> {
    let radius = lift_scalar(radius);
    vec![
        exact_dot(&line.offset, &line.offset) - &radius * &radius,
        exact_dot(&line.offset, &line.direction) * 2,
        exact_dot(&line.direction, &line.direction),
    ]
}
/// `rb^2 (q.a)^2 + ra^2 det(a,q)^2 - ra^2 rb^2 |a|^2` with `q = A - C + t d`.
pub(super) fn ellipse_equation(
    line: &CenteredLine,
    axis: [f64; 2],
    radii: [f64; 2],
) -> Vec<BigInt> {
    let [along_0, along_1] = line.dot_with(axis);
    // Squared, so the orientation of the normal coordinate is irrelevant.
    let [across_0, across_1] = line.cross_with(axis);
    let lifted_axis = lift(axis);
    let major = lift_scalar(radii[0]);
    let minor = lift_scalar(radii[1]);
    let (major, minor) = (&major * &major, &minor * &minor);
    vec![
        &minor * &along_0 * &along_0 + &major * &across_0 * &across_0
            - &major * &minor * exact_dot(&lifted_axis, &lifted_axis),
        (&minor * &along_0 * &along_1 + &major * &across_0 * &across_1) * 2,
        &minor * &along_1 * &along_1 + &major * &across_1 * &across_1,
    ]
}
/// Reduce exact coefficients; a conic equation is never identically zero.
pub(super) fn reduce(
    coefficients: Vec<BigInt>,
    limit: u16,
) -> Result<IntegerPolynomial, super::super::RootIsolationError> {
    IntegerPolynomial::from_integers(coefficients, limit)
}
