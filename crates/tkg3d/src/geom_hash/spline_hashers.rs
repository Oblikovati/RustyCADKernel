// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_BezierCurveHasher.pxx, GeomHash_BSplineCurveHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Bezier and B-spline curve hashers: the hash covers only metadata (degree, pole
//! count, knot count, rationality), while the comparison walks the full geometric data.
//!
//! The native comparisons are `noexcept` and read `Pole(i)`/`Weight(i)` of the second
//! curve with the first curve's count; a B-spline pair with equal knots but different
//! pole counts (periodic against non-periodic) would raise inside them and terminate
//! the process. The port reports such a pair as different instead.
use super::placement_hashers::GeomHashPointHasher;
use crate::geom::{GeomBSplineCurve, GeomBezierCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_hash::hash_key::{
    KnotAccess, combined_hash, int_hash, same_knots, same_poles_and_weights,
};
use rustycad_tkg2d::hasher_tolerances;
use rustycad_tkmath::gp::Point3;

hasher_tolerances!(
    /// `GeomHash_BezierCurveHasher`: metadata hash, full comparison;
    /// `GeomHashBezierCurveHasher::default().equal(&a, &b)`.
    GeomHashBezierCurveHasher
);
hasher_tolerances!(
    /// `GeomHash_BSplineCurveHasher`: metadata hash, full comparison;
    /// `GeomHashBSplineCurveHasher::default().equal(&a, &b)`.
    GeomHashBSplineCurveHasher
);

/// A pole accessor of one curve or surface (`Pole(i)` or `Pole(i, j)`).
pub(super) type PoleAccess<'a, I> = &'a dyn Fn(I) -> Result<Point3, Failure>;
/// A weight accessor of one curve or surface (`Weight(i)` or `Weight(i, j)`).
pub(super) type WeightAccess<'a, I> = &'a dyn Fn(I) -> Result<f64, Failure>;

/// The pole walk of every spline comparison: the poles at `indices` compared as
/// points, then the weights when `rational`; a raising accessor makes the pair
/// different; `same_net(points, &[1, 2], false, poles, weights)`.
pub(super) fn same_net<I: Copy>(
    points: GeomHashPointHasher,
    indices: &[I],
    rational: bool,
    pole: [PoleAccess<'_, I>; 2],
    weight: [WeightAccess<'_, I>; 2],
) -> bool {
    let equal_poles = |a: Point3, b: Point3| points.equal(a, b);
    let comp: f64 = points.comp_tolerance;
    same_poles_and_weights(indices, rational, comp, equal_poles, pole, weight).unwrap_or(false)
}

impl GeomHashBezierCurveHasher {
    /// Degree, pole count and rationality hashes combined (`operator()(handle)`);
    /// `hasher.hash(&curve)`.
    pub fn hash(&self, curve: &GeomBezierCurve) -> usize {
        combined_hash(&[
            int_hash(curve.degree()),
            int_hash(curve.nb_poles()),
            int_hash(i32::from(curve.is_rational())),
        ])
    }
    /// Same degree and rationality, poles and weights within tolerance
    /// (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomBezierCurve, second: &GeomBezierCurve) -> bool {
        if first.degree() != second.degree() || first.is_rational() != second.is_rational() {
            return false;
        }
        let points = GeomHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let indices: Vec<i32> = (1..=first.nb_poles()).collect();
        let poles: [PoleAccess<'_, i32>; 2] = [&|i| first.pole(i), &|i| second.pole(i)];
        let weights: [WeightAccess<'_, i32>; 2] = [&|i| first.weight(i), &|i| second.weight(i)];
        same_net(points, &indices, first.is_rational(), poles, weights)
    }
}

impl GeomHashBSplineCurveHasher {
    /// Degree, pole count, knot count and rationality hashes combined
    /// (`operator()(handle)`); `hasher.hash(&curve)`.
    pub fn hash(&self, curve: &GeomBSplineCurve) -> usize {
        combined_hash(&[
            int_hash(curve.degree()),
            int_hash(curve.nb_poles()),
            int_hash(curve.nb_knots()),
            int_hash(i32::from(curve.is_rational())),
        ])
    }
    /// Same degree, knots and multiplicities, rationality, then poles and weights within
    /// tolerance (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomBSplineCurve, second: &GeomBSplineCurve) -> bool {
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
        let points = GeomHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let indices: Vec<i32> = (1..=first.nb_poles()).collect();
        let poles: [PoleAccess<'_, i32>; 2] = [&|i| first.pole(i), &|i| second.pole(i)];
        let weights: [WeightAccess<'_, i32>; 2] = [&|i| first.weight(i), &|i| second.weight(i)];
        same_net(points, &indices, first.is_rational(), poles, weights)
    }
}
