// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_BSplComputeLine.gxx (fields, constructors, accessors)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_BSplComputeLine`: the approximation of a MultiLine by one B-spline
//! MultiCurve, adding knots until the tolerances are reached (or interpolating
//! when the points run out).
mod fit;
mod interpol;
mod perform;

use super::compute_line::{ComputeLineOptions, constraint_handle, owned_array};
use super::status::ParametrizationType;
use crate::app_par_curves::{Constraint, ConstraintCouple, Constraints, MultiBSpCurve};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkmath::math::Vector;

/// `Approx_BSplComputeLine` over any [`crate::app_par_curves::LineTool`]
/// (AppDef_BSplineCompute, GeomInt_TheComputeLineOfWLApprox...).
///
/// `realfirstC`/`reallastC` and the lambdas are uninitialized natively until
/// Perform; the port starts them at `NoConstraint` and 0.
///
/// ```ignore
/// let compute = BSplComputeLine::with_line(&line, ComputeLineOptions::default())?;
/// let spline = compute.value();
/// ```
#[derive(Clone)]
pub struct BSplComputeLine {
    the_multi_bsp_curve: MultiBSpCurve,
    alldone: bool,
    tolreached: bool,
    par: ParametrizationType,
    my_parameters: Option<Array1<'static, f64>>,
    first_param: Option<Array1<'static, f64>>,
    knots: Option<Array1<'static, f64>>,
    mults: Option<Array1<'static, i32>>,
    has_knots: bool,
    has_mults: bool,
    couples: [ConstraintCouple; 2],
    options: ComputeLineOptions,
    currenttol3d: f64,
    currenttol2d: f64,
    first_c: Constraint,
    last_c: Constraint,
    real_first_c: Constraint,
    real_last_c: Constraint,
    continuity: i32,
    lambda1: f64,
    lambda2: f64,
    periodic: bool,
}

impl BSplComputeLine {
    /// The constructor without a line (`degreemin`... `Squares`): initializes the
    /// fields only; `BSplComputeLine::new(ComputeLineOptions::default())`.
    pub fn new(options: ComputeLineOptions) -> Self {
        Self {
            the_multi_bsp_curve: MultiBSpCurve::new(),
            alldone: false,
            tolreached: false,
            par: options.parametrization,
            my_parameters: None,
            first_param: None,
            knots: None,
            mults: None,
            has_knots: false,
            has_mults: false,
            couples: [ConstraintCouple::default(); 2],
            options,
            currenttol3d: real_last(),
            currenttol2d: real_last(),
            first_c: Constraint::TangencyPoint,
            last_c: Constraint::TangencyPoint,
            real_first_c: Constraint::NoConstraint,
            real_last_c: Constraint::NoConstraint,
            continuity: -1,
            lambda1: 0.0,
            lambda2: 0.0,
            periodic: false,
        }
    }

    /// The constructor with a line: initializes and performs;
    /// `BSplComputeLine::with_line(&line, ComputeLineOptions::default())?`.
    pub fn with_line<L: crate::app_par_curves::LineTool + Clone>(
        line: &L,
        options: ComputeLineOptions,
    ) -> Result<Self, Failure> {
        let mut compute: Self = Self::new(options);
        compute.perform(line)?;
        Ok(compute)
    }

    /// The constructor with initial parameters and no line (iso-parametric);
    /// `BSplComputeLine::from_parameters(&u, ComputeLineOptions::default())?`.
    pub fn from_parameters(
        parameters: &Vector<'_>,
        options: ComputeLineOptions,
    ) -> Result<Self, Failure> {
        let mut compute: Self = Self::new(options);
        compute.first_param = Some(owned_array(parameters)?);
        compute.par = ParametrizationType::IsoParametric;
        Ok(compute)
    }

    /// The constructor with a line and initial parameters: initializes and performs;
    /// `BSplComputeLine::with_line_and_parameters(&line, &u, ComputeLineOptions::default())?`.
    pub fn with_line_and_parameters<L: crate::app_par_curves::LineTool + Clone>(
        line: &L,
        parameters: &Vector<'_>,
        options: ComputeLineOptions,
    ) -> Result<Self, Failure> {
        let mut compute: Self = Self::from_parameters(parameters, options)?;
        compute.perform(line)?;
        Ok(compute)
    }

    /// `Init(degreemin, ..., Squares)`: replaces every option;
    /// `compute.init(ComputeLineOptions::default())`.
    pub fn init(&mut self, options: ComputeLineOptions) {
        self.options = options;
        self.par = options.parametrization;
    }

    /// `SetParameters(ThePar)`: the initial parameters of the next Perform;
    /// `compute.set_parameters(&u)?`.
    pub fn set_parameters(&mut self, parameters: &Vector<'_>) -> Result<(), Failure> {
        self.first_param = Some(owned_array(parameters)?);
        Ok(())
    }

    /// `SetKnots(Knots)`: the starting knots (the multiplicities follow the degree
    /// and the continuity); `compute.set_knots(&knots)`.
    pub fn set_knots(&mut self, knots: &Array1<'_, f64>) {
        self.has_knots = true;
        self.knots = Some(copy_array(knots));
    }

    /// `SetKnotsAndMultiplicities(Knots, Mults)`;
    /// `compute.set_knots_and_multiplicities(&knots, &mults)`.
    pub fn set_knots_and_multiplicities(
        &mut self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
    ) {
        self.has_knots = true;
        self.has_mults = true;
        self.knots = Some(copy_array(knots));
        self.mults = Some(copy_array(mults));
    }

    /// `SetDegrees(degreemin, degreemax)`; `compute.set_degrees(3, 6)`.
    pub fn set_degrees(&mut self, degree_min: i32, degree_max: i32) {
        self.options.degree_min = degree_min;
        self.options.degree_max = degree_max;
    }

    /// `SetTolerances(Tolerance3d, Tolerance2d)`; `compute.set_tolerances(1e-4, 1e-6)`.
    pub fn set_tolerances(&mut self, tolerance_3d: f64, tolerance_2d: f64) {
        self.options.tolerance_3d = tolerance_3d;
        self.options.tolerance_2d = tolerance_2d;
    }

    /// `SetContinuity(C)`: the continuity at the inner knots (-1, the default,
    /// gives multiplicity 1); `compute.set_continuity(2)`.
    pub fn set_continuity(&mut self, continuity: i32) {
        self.continuity = continuity;
    }

    /// `SetConstraints(firstC, lastC)`;
    /// `compute.set_constraints(Constraint::PassPoint, Constraint::PassPoint)`.
    pub fn set_constraints(&mut self, first: Constraint, last: Constraint) {
        self.first_c = first;
        self.last_c = last;
    }

    /// `SetPeriodic(thePeriodic)`: the interpolation averages the end tangents;
    /// `compute.set_periodic(true)`.
    pub fn set_periodic(&mut self, periodic: bool) {
        self.periodic = periodic;
    }

    /// `IsAllApproximated()`; `compute.is_all_approximated()`.
    pub fn is_all_approximated(&self) -> bool {
        self.alldone
    }

    /// `IsToleranceReached()`; `compute.is_tolerance_reached()`.
    pub fn is_tolerance_reached(&self) -> bool {
        self.tolreached
    }

    /// `Error(tol3d, tol2d)`: the errors of the result; `compute.error()`.
    pub fn error(&self) -> (f64, f64) {
        (self.currenttol3d, self.currenttol2d)
    }

    /// `Value()`: the B-spline MultiCurve; `compute.value()`.
    pub fn value(&self) -> &MultiBSpCurve {
        &self.the_multi_bsp_curve
    }

    /// `ChangeValue()`: the result, to modify; `compute.change_value()`.
    pub fn change_value(&mut self) -> &mut MultiBSpCurve {
        &mut self.the_multi_bsp_curve
    }

    /// `Parameters()`: the parameters of the points; Standard_NullObject before any
    /// Perform or Interpol (a null handle natively); `compute.parameters()?`.
    pub fn parameters(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.my_parameters.as_ref().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "Approx_BSplComputeLine::Parameters: no parameters before Perform",
            )
        })
    }

    fn constraints(&self) -> Constraints {
        constraint_handle(self.couples)
    }
}

/// A copy of `values` with its bounds.
fn copy_array<T: Copy + Default>(values: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy: Array1<'static, T> = Array1::with_size(values.len());
    copy.update_lower_bound(values.lower());
    copy.iter_mut()
        .zip(values.iter())
        .for_each(|(slot, value)| *slot = *value);
    copy
}
