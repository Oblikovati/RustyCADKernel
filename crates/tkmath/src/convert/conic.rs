// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/Convert/Convert_ConicToBSplineCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Storage and accessors of the B-spline data computed for a 2D conic.
use super::ParameterisationType;
use super::cos_sin::{CosAndSin, build_trimmed};
use super::cos_sin_periodic::build_periodic;
use crate::gp::Point2;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Degree, periodicity, poles, weights, knots and multiplicities of the B-spline
/// equivalent to a 2D conic (`Convert_ConicToBSplineCurve` and its subclasses);
/// `ConicToBSplineCurve::from_circle(&circle, ParameterisationType::TgtThetaOver2)?.degree() == 2`.
#[derive(Clone, Debug)]
pub struct ConicToBSplineCurve {
    pub(super) poles: Array1<'static, Point2>,
    pub(super) weights: Array1<'static, f64>,
    pub(super) knots: Array1<'static, f64>,
    pub(super) mults: Array1<'static, i32>,
    pub(super) degree: i32,
    pub(super) periodic: bool,
}

impl ConicToBSplineCurve {
    /// The protected base constructor: arrays of the given lengths when at least 2,
    /// empty arrays otherwise; `ConicToBSplineCurve::with_sizes(3, 2, 2)`.
    pub(super) fn with_sizes(nb_poles: i32, nb_knots: i32, degree: i32) -> Self {
        Self {
            poles: base_array(nb_poles),
            weights: base_array(nb_poles),
            knots: base_array(nb_knots),
            mults: base_array(nb_knots),
            degree,
            periodic: false,
        }
    }

    /// Stores the knot vector, weights and degree of a cos/sin representation;
    /// the poles are placed by the caller; `ConicToBSplineCurve::from_cos_and_sin(&cs, true)`.
    pub(super) fn from_cos_and_sin(cos_and_sin: &CosAndSin, periodic: bool) -> Self {
        let mut result = Self::with_sizes(0, 0, cos_and_sin.degree);
        result.weights = cos_and_sin.denominator.clone();
        result.knots = cos_and_sin.knots.clone();
        result.mults = cos_and_sin.mults.clone();
        result.periodic = periodic;
        result
    }

    /// Degree of the B-spline; `conversion.degree()`.
    pub fn degree(&self) -> i32 {
        self.degree
    }

    /// Number of poles; `conversion.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        self.poles.len() as i32
    }

    /// Number of knots; `conversion.nb_knots()`.
    pub fn nb_knots(&self) -> i32 {
        self.knots.len() as i32
    }

    /// True for the periodic full-conic parameterisations; `conversion.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        self.periodic
    }

    /// Pole of one-based index (deprecated `Pole`); `conversion.pole(1)?`.
    pub fn pole(&self, index: i32) -> Result<Point2, Failure> {
        checked_index("Pole", index, self.poles.len())?;
        Ok(self.poles[index])
    }

    /// Weight of one-based index, bounded by the pole count (deprecated `Weight`);
    /// `conversion.weight(1)?`.
    pub fn weight(&self, index: i32) -> Result<f64, Failure> {
        checked_index("Weight", index, self.poles.len())?;
        Ok(self.weights[index])
    }

    /// Knot of one-based index (deprecated `Knot`); `conversion.knot(1)?`.
    pub fn knot(&self, index: i32) -> Result<f64, Failure> {
        checked_index("Knot", index, self.knots.len())?;
        Ok(self.knots[index])
    }

    /// Multiplicity of one-based index (deprecated `Multiplicity`); `conversion.multiplicity(1)?`.
    pub fn multiplicity(&self, index: i32) -> Result<i32, Failure> {
        checked_index("Multiplicity", index, self.knots.len())?;
        Ok(self.mults[index])
    }

    /// All poles; `conversion.poles()[1]`.
    pub fn poles(&self) -> &Array1<'static, Point2> {
        &self.poles
    }

    /// All weights; `conversion.weights()[1]`.
    pub fn weights(&self) -> &Array1<'static, f64> {
        &self.weights
    }

    /// All knots; `conversion.knots()[1]`.
    pub fn knots(&self) -> &Array1<'static, f64> {
        &self.knots
    }

    /// All multiplicities; `conversion.multiplicities()[1]`.
    pub fn multiplicities(&self) -> &Array1<'static, i32> {
        &self.mults
    }

    /// Periodic cos/sin representation of the full circle (the public legacy
    /// `BuildCosAndSin` overload); only TgtThetaOver2 and RationalC1 are accepted;
    /// `ConicToBSplineCurve::build_cos_and_sin(ParameterisationType::RationalC1)?`.
    pub fn build_cos_and_sin(parameterisation: ParameterisationType) -> Result<CosAndSin, Failure> {
        build_periodic(parameterisation)
    }

    /// Cos/sin representation of the arc `[u_first, u_last]` (the public legacy
    /// trimmed `BuildCosAndSin` overload);
    /// `ConicToBSplineCurve::build_cos_and_sin_trimmed(ParameterisationType::Polynomial, 0.0, 1.0)?`.
    pub fn build_cos_and_sin_trimmed(
        parameterisation: ParameterisationType,
        u_first: f64,
        u_last: f64,
    ) -> Result<CosAndSin, Failure> {
        build_trimmed(parameterisation, u_first, u_last)
    }
}

// A 1..count array when count is at least 2; the default (empty, lower bound 1) otherwise.
fn base_array<T: Default>(count: i32) -> Array1<'static, T> {
    let mut array: Array1<'static, T> =
        Array1::with_size(if count >= 2 { count as usize } else { 0 });
    array.update_lower_bound(1);
    array
}

// The deprecated accessors raise Standard_OutOfRange outside 1..=length.
fn checked_index(accessor: &str, index: i32, length: usize) -> Result<(), Failure> {
    if index >= 1 && index as usize <= length {
        return Ok(());
    }
    Err(Failure::out_of_range(&format!(
        "Convert_ConicToBSplineCurve::{accessor}: Index out of range: {index} not in 1..={length}"
    )))
}
