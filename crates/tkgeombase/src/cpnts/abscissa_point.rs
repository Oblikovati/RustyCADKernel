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
// Rust translation of ModelingData/TKGeomBase/CPnts/CPnts_AbscissaPoint.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CPnts_AbscissaPoint`: the curve length by Gauss integration, and the parameter
//! at a given curvilinear distance by a bounded Newton search.
use super::{MyGaussFunction, MyRootFunction, PntsCurve, RealFunction};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::math::{FunctionRoot, GaussSingleIntegration};

/// `CPnts_AbscissaPoint`: the point at the curvilinear distance `Abscissa` from the
/// point of parameter `U0`; `AbscissaPoint::with_resolution(&adaptor, 1.0, 0.0, 1e-9)?`.
///
/// The object borrows the curve given to `Init` for its lifetime `'a`, as the native
/// object keeps a pointer to it. Every member that raises natively returns `Err`.
#[derive(Default)]
pub struct AbscissaPoint<'a> {
    my_done: bool,
    my_l: f64,
    my_param: f64,
    my_u_min: f64,
    my_u_max: f64,
    my_f: MyRootFunction<'a>,
}

// The integrand of the length: |C'(u)| (`f3d`, `f2d`).
fn speed_function(curve: PntsCurve<'_>) -> RealFunction<'_> {
    Box::new(move |u: f64| curve.speed(u))
}

// |Integral(U1, U2, |C'|)| with the Gauss order of the curve, with or without the
// convergence tolerance; Standard_ConstructionError when the integration fails.
fn integrate_length(
    curve: PntsCurve<'_>,
    u1: f64,
    u2: f64,
    tol: Option<f64>,
) -> Result<f64, Failure> {
    let mut fg: MyGaussFunction = MyGaussFunction::new();
    fg.init(speed_function(curve));
    let order: i32 = curve.gauss_order()?;
    let the_length: GaussSingleIntegration = match tol {
        Some(tol) => GaussSingleIntegration::with_tolerance(&mut fg, u1, u2, order, tol),
        None => GaussSingleIntegration::new(&mut fg, u1, u2, order),
    };
    if let Some(failure) = fg.take_failure() {
        return Err(failure);
    }
    match the_length.value() {
        Ok(value) => Ok(value.abs()),
        Err(_) => Err(Failure::with_type(
            "Standard_ConstructionError",
            &format!("CPnts_AbscissaPoint::Length - Gauss integration failed on [{u1}, {u2}]"),
            "",
        )),
    }
}

impl AbscissaPoint<'_> {
    /// The length of the curve (`Length(C)`); `AbscissaPoint::length(&adaptor)?`.
    pub fn length<'c>(curve: impl Into<PntsCurve<'c>>) -> Result<f64, Failure> {
        let curve: PntsCurve = curve.into();
        integrate_length(curve, curve.first_parameter()?, curve.last_parameter()?, None)
    }
    /// The length of the curve with the integration tolerance `tol`
    /// (`Length(C, Tol)`); `AbscissaPoint::length_with_tolerance(&adaptor, 1e-9)?`.
    pub fn length_with_tolerance<'c>(
        curve: impl Into<PntsCurve<'c>>,
        tol: f64,
    ) -> Result<f64, Failure> {
        let curve: PntsCurve = curve.into();
        let (u1, u2) = (curve.first_parameter()?, curve.last_parameter()?);
        integrate_length(curve, u1, u2, Some(tol))
    }
    /// The length between `u1` and `u2` (`Length(C, U1, U2)`);
    /// `AbscissaPoint::length_between(&adaptor, 0.0, 1.0)?`.
    pub fn length_between<'c>(
        curve: impl Into<PntsCurve<'c>>,
        u1: f64,
        u2: f64,
    ) -> Result<f64, Failure> {
        integrate_length(curve.into(), u1, u2, None)
    }
    /// The length between `u1` and `u2` with the integration tolerance `tol`
    /// (`Length(C, U1, U2, Tol)`);
    /// `AbscissaPoint::length_between_with_tolerance(&adaptor, 0.0, 1.0, 1e-9)?`.
    pub fn length_between_with_tolerance<'c>(
        curve: impl Into<PntsCurve<'c>>,
        u1: f64,
        u2: f64,
        tol: f64,
    ) -> Result<f64, Failure> {
        integrate_length(curve.into(), u1, u2, Some(tol))
    }
}

impl<'a> AbscissaPoint<'a> {
    /// The indefinite algorithm, not done (`CPnts_AbscissaPoint()`);
    /// `AbscissaPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Init then `Perform(Abscissa, U0, Resolution)`
    /// (`CPnts_AbscissaPoint(C, Abscissa, U0, Resolution)`); the 3D overload
    /// initializes with the tolerance `resolution` ("rbv's modification"), the 2D one
    /// without; `AbscissaPoint::with_resolution(&adaptor, 1.0, 0.0, 1e-9)?`.
    pub fn with_resolution(
        curve: impl Into<PntsCurve<'a>>,
        abscissa: f64,
        u0: f64,
        resolution: f64,
    ) -> Result<Self, Failure> {
        let curve: PntsCurve<'a> = curve.into();
        let mut result: Self = Self::new();
        match curve {
            PntsCurve::Curve3d(_) => result.init_with_tolerance(curve, resolution)?,
            PntsCurve::Curve2d(_) => result.init(curve)?,
        }
        result.perform(abscissa, u0, resolution)?;
        Ok(result)
    }
    /// `Init(C)` then `Perform(Abscissa, U0, Ui, Resolution)`
    /// (`CPnts_AbscissaPoint(C, Abscissa, U0, Ui, Resolution)`);
    /// `AbscissaPoint::with_start(&adaptor, 1.0, 0.0, 0.9, 1e-9)?`.
    pub fn with_start(
        curve: impl Into<PntsCurve<'a>>,
        abscissa: f64,
        u0: f64,
        ui: f64,
        resolution: f64,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.init(curve)?;
        result.perform_from(abscissa, u0, ui, resolution)?;
        Ok(result)
    }
}

impl<'a> AbscissaPoint<'a> {
    /// Initialize on the whole curve (`Init(C)`); `point.init(&adaptor)?`.
    pub fn init(&mut self, curve: impl Into<PntsCurve<'a>>) -> Result<(), Failure> {
        let curve: PntsCurve<'a> = curve.into();
        let (u1, u2) = (curve.first_parameter()?, curve.last_parameter()?);
        self.init_range(curve, u1, u2, None)
    }
    /// Initialize on the whole curve, its length computed with the integration
    /// tolerance `tol` (`Init(C, Tol)`); `point.init_with_tolerance(&adaptor, 1e-9)?`.
    pub fn init_with_tolerance(
        &mut self,
        curve: impl Into<PntsCurve<'a>>,
        tol: f64,
    ) -> Result<(), Failure> {
        let curve: PntsCurve<'a> = curve.into();
        let (u1, u2) = (curve.first_parameter()?, curve.last_parameter()?);
        self.init_range(curve, u1, u2, Some(tol))
    }
    /// Initialize between `u1` and `u2` (`Init(C, U1, U2)`);
    /// `point.init_between(&adaptor, 0.0, 1.0)?`.
    pub fn init_between(
        &mut self,
        curve: impl Into<PntsCurve<'a>>,
        u1: f64,
        u2: f64,
    ) -> Result<(), Failure> {
        self.init_range(curve.into(), u1, u2, None)
    }
    /// Initialize between `u1` and `u2` with the integration tolerance `tol`
    /// (`Init(C, U1, U2, Tol)`);
    /// `point.init_between_with_tolerance(&adaptor, 0.0, 1.0, 1e-9)?`.
    pub fn init_between_with_tolerance(
        &mut self,
        curve: impl Into<PntsCurve<'a>>,
        u1: f64,
        u2: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.init_range(curve.into(), u1, u2, Some(tol))
    }
    // The shared body of the Init overloads: the search domain is [U1, U2] widened by
    // its own width on both sides, so the solution may leave the curve bounds.
    fn init_range(
        &mut self,
        curve: PntsCurve<'a>,
        u1: f64,
        u2: f64,
        tol: Option<f64>,
    ) -> Result<(), Failure> {
        self.my_f
            .init_function(speed_function(curve), curve.gauss_order()?);
        self.my_l = integrate_length(curve, u1, u2, tol)?;
        self.my_u_min = std_min(u1, u2);
        self.my_u_max = std_max(u1, u2);
        let du: f64 = self.my_u_max - self.my_u_min;
        self.my_u_min -= du;
        self.my_u_max += du;
        Ok(())
    }
}

impl AbscissaPoint<'_> {
    /// The point at the distance `abscissa` from `u0`, started at
    /// `U0 + (Abscissa / L) * (UMax - UMin) / 3` (`Perform(Abscissa, U0, Resolution)`);
    /// `point.perform(1.0, 0.0, 1e-9)?`.
    ///
    /// A curve shorter than Precision::Confusion() gives `U0`, done.
    pub fn perform(&mut self, abscissa: f64, u0: f64, resolution: f64) -> Result<(), Failure> {
        if self.my_l < precision::confusion() {
            self.set_parameter(u0);
            return Ok(());
        }
        // "exercise : why 3 ?" (native comment)
        let ui: f64 = u0 + (abscissa / self.my_l) * (self.my_u_max - self.my_u_min) / 3.0;
        self.perform_from(abscissa, u0, ui, resolution)
    }
    /// The point at the distance `abscissa` from `u0`, started at `ui`
    /// (`Perform(Abscissa, U0, Ui, Resolution)`); `point.perform_from(1.0, 0.0, 0.9, 1e-9)?`.
    pub fn perform_from(
        &mut self,
        abscissa: f64,
        u0: f64,
        ui: f64,
        resolution: f64,
    ) -> Result<(), Failure> {
        if self.my_l < precision::confusion() {
            self.set_parameter(u0);
            return Ok(());
        }
        self.my_f.init_target(u0, abscissa);
        self.solve(ui, resolution)
    }
    /// As [`Self::perform_from`], the lengths integrated with the tolerance
    /// `Resolution / 10` ("rbv's modification"); meant after an `Init` with the
    /// tolerance `Resolution` (`AdvPerform(Abscissa, U0, Ui, Resolution)`);
    /// `point.adv_perform(1.0, 0.0, 0.9, 1e-9)?`.
    pub fn adv_perform(
        &mut self,
        abscissa: f64,
        u0: f64,
        ui: f64,
        resolution: f64,
    ) -> Result<(), Failure> {
        if self.my_l < precision::confusion() {
            self.set_parameter(u0);
            return Ok(());
        }
        self.my_f
            .init_target_with_tolerance(u0, abscissa, resolution / 10.0);
        self.solve(ui, resolution)
    }
    // The bounded math_FunctionRoot search; the validity test of the root that the
    // native code comments out ("lbo 21/03/97") stays out.
    fn solve(&mut self, ui: f64, resolution: f64) -> Result<(), Failure> {
        self.my_done = false;
        let (lower, upper) = (self.my_u_min, self.my_u_max);
        let solution = FunctionRoot::bounded(&mut self.my_f, ui, resolution, lower, upper, 100);
        if let Some(failure) = self.my_f.take_failure() {
            return Err(failure);
        }
        let solution: FunctionRoot = solution?;
        if solution.is_done() {
            self.set_parameter(solution.root()?);
        }
        Ok(())
    }
    /// Whether the computation was successful (`IsDone`); `point.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }
    /// The parameter of the solution (`Parameter`); StdFail_NotDone when not done;
    /// `point.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        if !self.my_done {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "CPnts_AbscissaPoint::Parameter() - no result",
                "",
            ));
        }
        Ok(self.my_param)
    }
    /// Enforce the solution, done (`SetParameter`, used by GCPnts);
    /// `point.set_parameter(0.5)`.
    pub fn set_parameter(&mut self, p: f64) {
        self.my_param = p;
        self.my_done = true;
    }
}
