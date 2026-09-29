// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.hxx/.cxx, Geom_BSplineCurve_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BSplineCurve`: rational or non-rational, periodic or not, B-spline curves.
mod access;
mod periodic;
mod traits;
use super::bounded_curve::GeomBoundedCurve;
use super::curve::GeomCurve;
use super::errors;
use super::geometry::GeomGeometry;
use super::spline_kind::BSplineState;
use crate::geom_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, StandardTransient, real};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::bspline_curve_core::GivenWeights;
use rustycad_tkg2d::geom2d::spline_data::check_curve_data;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, Vector3, resolution};

/// A B-spline curve; `Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 3, false)?)`.
pub struct GeomBSplineCurve {
    parent: StandardTransient,
    state: SharedState<BSplineState>,
}
impl_transient!(
    GeomBSplineCurve,
    "Geom_BSplineCurve",
    dyn GeomBoundedCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomBoundedCurve]
);

// The weight checks of the rational constructor.
fn check_weights(weights: &Array1<'_, f64>, nbpoles: usize) -> Result<(), Failure> {
    if weights.len() != nbpoles {
        return Err(errors::construction(&format!(
            "Geom_BSplineCurve: {} weights for {nbpoles} poles; Weights and Poles array size mismatch",
            weights.len()
        )));
    }
    for i in weights.lower()..=weights.upper() {
        let weight: f64 = at(weights, i)?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "Geom_BSplineCurve: Weights({i}) = {weight}; Weights values too small"
            )));
        }
    }
    Ok(())
}

impl GeomBSplineCurve {
    rustycad_tkg2d::bspline_curve_edits!(Point3, Vector3);
    fn from_state(state: BSplineState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational curve (`Geom_BSplineCurve(Poles, Knots, Multiplicities, Degree, Periodic)`);
    /// Standard_ConstructionError for inconsistent data;
    /// `GeomBSplineCurve::new(&poles, &knots, &mults, 3, false)?`.
    pub fn new(
        poles: &Array1<'_, Point3>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        check_curve_data(poles, knots, mults, degree, periodic)?;
        let state = BSplineState::from_arrays(poles, None, knots, mults, degree, periodic)?;
        Ok(Self::from_state(state))
    }
    /// Rational curve (`Geom_BSplineCurve(Poles, Weights, Knots, Multiplicities, Degree,
    /// Periodic, CheckRational = true)`); equal weights give a non-rational curve;
    /// `GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, 2, false)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        Self::with_weights_checked(poles, weights, knots, mults, degree, periodic, true)
    }
    /// The rational constructor with its `CheckRational` flag: when false the curve
    /// stays rational even with equal weights;
    /// `GeomBSplineCurve::with_weights_checked(&poles, &weights, &knots, &mults, 2, false, false)?`.
    #[allow(clippy::too_many_arguments)] // The native constructor signature.
    pub fn with_weights_checked(
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
        check_rational: bool,
    ) -> Result<Self, Failure> {
        check_curve_data(poles, knots, mults, degree, periodic)?;
        check_weights(weights, poles.len())?;
        let given = GivenWeights {
            weights,
            check_rational,
        };
        let state = BSplineState::from_arrays(poles, Some(given), knots, mults, degree, periodic)?;
        Ok(Self::from_state(state))
    }
    /// The native copy constructor: the evaluation representation is deep-copied, the
    /// cached resolution value is kept but marked stale
    /// (`Geom_BSplineCurve(const Geom_BSplineCurve&)`); `GeomBSplineCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &GeomBSplineCurve) -> Result<Self, Failure> {
        let mut copy: BSplineState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_curve_desc(&copy.eval_rep)?;
        if !copy.rational {
            copy.reset_unit_weights()?;
        }
        copy.max_deriv_inv_ok = false;
        Ok(Self::from_state(copy))
    }
    rustycad_tkg2d::eval_representation_members!(
        dyn RepCurveDesc,
        rep_utils::validate_curve_desc,
        "curve"
    );
    /// Insert knot `u` with multiplicity `mult` (`InsertKnot(U, M, ParametricTolerance, Add)`,
    /// native defaults 1, 0 and true); `curve.insert_knot(0.5, 1, 0.0, true)?`.
    pub fn insert_knot(
        &self,
        u: f64,
        mult: i32,
        parametric_tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let mut knots: Array1<'static, f64> = one_based(1)?;
        put(&mut knots, 1, u)?;
        let mut mults: Array1<'static, i32> = one_based(1)?;
        put(&mut mults, 1, mult)?;
        self.insert_knots(&knots, &mults, parametric_tolerance, add)
    }
    /// Make parameter `u` the origin of a periodic curve, inserting a knot when none is
    /// within `tol` (`SetOrigin(U, Tol)`); `curve.set_origin_parameter(0.3, 1e-9)?`.
    pub fn set_origin_parameter(&self, u: f64, tol: f64) -> Result<(), Failure> {
        periodic::set_origin_parameter(&mut self.state.write(), u, tol)
    }
    /// Same degree, rationality, knot and pole counts, poles within `preci` per
    /// coordinate, knots within `Precision::Parametric(preci)`, equal multiplicities and,
    /// for rational curves, weights within `Epsilon(weight)` (`IsEqual`);
    /// `curve.is_equal(&other, 1e-7)?`.
    pub fn is_equal(&self, other: &GeomBSplineCurve, preci: f64) -> Result<bool, Failure> {
        // Read the other curve first: it can be this one, and the lock is not reentrant.
        let theirs: BSplineState = other.state.snapshot();
        let own = self.state.read();
        if !same_shape(&own, &theirs) {
            return Ok(false);
        }
        if !poles_close(&own, &theirs, preci)? || !knots_equal(&own, &theirs, preci)? {
            return Ok(false);
        }
        if !own.rational {
            return Ok(true);
        }
        weights_equal(&own, &theirs)
    }
}

// The early exit of IsEqual on empty arrays and different counts.
fn same_shape(own: &BSplineState, theirs: &BSplineState) -> bool {
    !(own.knots.is_empty()
        || own.poles.is_empty()
        || own.mults.is_empty()
        || own.deg != theirs.deg
        || own.rational != theirs.rational
        || own.knots.len() != theirs.knots.len()
        || own.poles.len() != theirs.poles.len())
}

fn poles_close(own: &BSplineState, theirs: &BSplineState, preci: f64) -> Result<bool, Failure> {
    for i in 1..=own.poles.len() as i32 {
        let (a, b) = (at(&own.poles, i)?, at(&theirs.poles, i)?);
        if (a.x() - b.x()).abs() > preci
            || (a.y() - b.y()).abs() > preci
            || (a.z() - b.z()).abs() > preci
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn knots_equal(own: &BSplineState, theirs: &BSplineState, preci: f64) -> Result<bool, Failure> {
    for i in 1..=own.knots.len() as i32 {
        if (at(&own.knots, i)? - at(&theirs.knots, i)?).abs() > precision::parametric(preci) {
            return Ok(false);
        }
    }
    for i in 1..=own.mults.len() as i32 {
        if at(&own.mults, i)? != at(&theirs.mults, i)? {
            return Ok(false);
        }
    }
    Ok(true)
}

// The other curve's `Weight(i)`, which reads 1 for a non-rational curve; both are
// rational here, as IsEqual compares weights only then.
fn weights_equal(own: &BSplineState, theirs: &BSplineState) -> Result<bool, Failure> {
    for i in 1..=own.weights.len() as i32 {
        let weight: f64 = at(&own.weights, i)?;
        if (weight - at(&theirs.weights, i)?).abs() > real::epsilon(weight) {
            return Ok(false);
        }
    }
    Ok(true)
}
