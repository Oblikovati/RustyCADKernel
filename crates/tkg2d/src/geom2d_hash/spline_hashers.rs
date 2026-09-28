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
// Rust translation of ModelingData/TKG2d/Geom2dHash/Geom2dHash_BezierCurveHasher.pxx, Geom2dHash_BSplineCurveHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Bezier and B-spline hashers: the hash covers only metadata (degree, pole count,
//! knot count, rationality), while the comparison walks the full geometric data.
//!
//! The native comparisons are `noexcept` and read `Pole(i)`/`Weight(i)` of the second
//! curve with the first curve's count; a B-spline pair with equal knots but different
//! pole counts (periodic against non-periodic) would raise inside them and terminate
//! the process. The port reports such a pair as different instead.
use super::hash_key::{combined_hash, int_hash};
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
fn same_poles_and_weights(
    points: Geom2dHashPointHasher,
    count: i32,
    rational: bool,
    pole: [&dyn Fn(i32) -> Result<Point2, Failure>; 2],
    weight: [&dyn Fn(i32) -> Result<f64, Failure>; 2],
) -> Result<bool, Failure> {
    for i in 1..=count {
        if !points.equal(pole[0](i)?, pole[1](i)?) {
            return Ok(false);
        }
    }
    if !rational {
        return Ok(true);
    }
    for i in 1..=count {
        if (weight[0](i)? - weight[1](i)?).abs() > points.comp_tolerance {
            return Ok(false);
        }
    }
    Ok(true)
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
        same_poles_and_weights(points, first.nb_poles(), rational, poles, weights).unwrap_or(false)
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
        if !self.same_knots(first, second).unwrap_or(false) {
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
        same_poles_and_weights(points, first.nb_poles(), rational, poles, weights).unwrap_or(false)
    }
    // Knot values within CompTolerance and equal multiplicities, knot by knot.
    fn same_knots(
        &self,
        first: &Geom2dBSplineCurve,
        second: &Geom2dBSplineCurve,
    ) -> Result<bool, Failure> {
        for i in 1..=first.nb_knots() {
            let knot_gap: f64 = (first.knot(i)? - second.knot(i)?).abs();
            if knot_gap > self.comp_tolerance || first.multiplicity(i)? != second.multiplicity(i)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
