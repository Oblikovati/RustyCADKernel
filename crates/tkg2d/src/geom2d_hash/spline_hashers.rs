// Rust translation of ModelingData/TKG2d/Geom2dHash/Geom2dHash_BezierCurveHasher.pxx, Geom2dHash_BSplineCurveHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Bezier and B-spline hashers: the hash covers only metadata (degree, pole count,
//! knot count, rationality), while the comparison walks the full geometric data.
//!
//! The native comparisons are `noexcept` and read `Pole(i)`/`Weight(i)` of the second
//! curve with the first curve's count; a B-spline pair with equal knots but different
//! pole counts (periodic against non-periodic) would raise inside them and terminate
//! the process. The port reports such a pair as different instead.
use super::hash_key::{KnotAccess, combined_hash, int_hash, same_knots, same_poles_and_weights};
use super::placement_hashers::Geom2dHashPointHasher;
use crate::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

hasher_tolerances!(
    /// `Geom2dHash_BezierCurveHasher`: metadata hash, full comparison;
    /// `Geom2dHashBezierCurveHasher::default().equal(&a, &b)`.
    Geom2dHashBezierCurveHasher
);
hasher_tolerances!(
    /// `Geom2dHash_BSplineCurveHasher`: metadata hash, full comparison;
    /// `Geom2dHashBSplineCurveHasher::default().equal(&a, &b)`.
    Geom2dHashBSplineCurveHasher
);

// The pole and weight walk both comparisons end with: poles 1..=count within
// CompTolerance as points, then, for rational curves, weights within CompTolerance.
fn same_curve_poles(
    points: Geom2dHashPointHasher,
    count: i32,
    rational: bool,
    pole: [&dyn Fn(i32) -> Result<Point2, Failure>; 2],
    weight: [&dyn Fn(i32) -> Result<f64, Failure>; 2],
) -> Result<bool, Failure> {
    let indices: Vec<i32> = (1..=count).collect();
    let equal_poles = |a: Point2, b: Point2| points.equal(a, b);
    same_poles_and_weights(
        &indices,
        rational,
        points.comp_tolerance,
        equal_poles,
        pole,
        weight,
    )
}

impl Geom2dHashBezierCurveHasher {
    /// Degree, pole count and rationality hashes combined (`operator()(handle)`);
    /// `hasher.hash(&curve)`.
    pub fn hash(&self, curve: &Geom2dBezierCurve) -> usize {
        combined_hash(&[
            int_hash(curve.degree()),
            int_hash(curve.nb_poles()),
            int_hash(i32::from(curve.is_rational())),
        ])
    }
    /// Same degree and rationality, poles and weights within tolerance
    /// (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dBezierCurve, second: &Geom2dBezierCurve) -> bool {
        if first.degree() != second.degree() || first.is_rational() != second.is_rational() {
            return false;
        }
        let points = Geom2dHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let poles: [&dyn Fn(i32) -> Result<Point2, Failure>; 2] =
            [&|i| first.pole(i), &|i| second.pole(i)];
        let weights: [&dyn Fn(i32) -> Result<f64, Failure>; 2] =
            [&|i| first.weight(i), &|i| second.weight(i)];
        let rational: bool = first.is_rational();
        same_curve_poles(points, first.nb_poles(), rational, poles, weights).unwrap_or(false)
    }
}

impl Geom2dHashBSplineCurveHasher {
    /// Degree, pole count, knot count and rationality hashes combined
    /// (`operator()(handle)`); `hasher.hash(&curve)`.
    pub fn hash(&self, curve: &Geom2dBSplineCurve) -> usize {
        combined_hash(&[
            int_hash(curve.degree()),
            int_hash(curve.nb_poles()),
            int_hash(curve.nb_knots()),
            int_hash(i32::from(curve.is_rational())),
        ])
    }
    /// Same degree, knots and multiplicities, rationality, then poles and weights within
    /// tolerance (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dBSplineCurve, second: &Geom2dBSplineCurve) -> bool {
        if first.degree() != second.degree() || first.nb_knots() != second.nb_knots() {
            return false;
        }
        let knots: [KnotAccess<'_>; 2] = [
            (&|i| first.knot(i), &|i| first.multiplicity(i)),
            (&|i| second.knot(i), &|i| second.multiplicity(i)),
        ];
        if !same_knots(self.comp_tolerance, first.nb_knots(), knots).unwrap_or(false) {
            return false;
        }
        if first.is_rational() != second.is_rational() {
            return false;
        }
        let points = Geom2dHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let poles: [&dyn Fn(i32) -> Result<Point2, Failure>; 2] =
            [&|i| first.pole(i), &|i| second.pole(i)];
        let weights: [&dyn Fn(i32) -> Result<f64, Failure>; 2] =
            [&|i| first.weight(i), &|i| second.weight(i)];
        let rational: bool = first.is_rational();
        same_curve_poles(points, first.nb_poles(), rational, poles, weights).unwrap_or(false)
    }
}
