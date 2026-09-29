// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_ApproxCurve.cxx, Geom2dConvert/Geom2dConvert_ApproxCurve.cxx, Approx/Approx_Curve3d.cxx, Approx_Curve2d.cxx, Approx_CurveOnSurface.cxx (the local AdvApprox evaluators of an adapted curve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The AdvApprox evaluators of one adapted curve. The native files repeat the same
//! class (`GeomConvert_ApproxCurve_Eval`, `Approx_Curve3d_Eval`,
//! `Approx_CurveOnSurface_Eval3d`; `Geom2dConvert_ApproxCurve_Eval`,
//! `Approx_Curve2d_Eval`, and `Approx_CurveOnSurface_Eval2d` without the parameter
//! check); the port keeps one of each, with the NbIntervals/Intervals break helpers.
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adv_approx::AdvApproxEvaluatorFunction;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// `NbIntervals(shape) + 1` break parameters of a 3D adaptor, from `Intervals(shape)`.
pub(crate) fn cut_points_3d(
    adaptor: &dyn Adaptor3dCurve,
    shape: Shape,
) -> Result<Array1<'static, f64>, Failure> {
    let count: i32 = adaptor.nb_intervals(shape)?;
    let mut points: Array1<'static, f64> = one_based(count + 1)?;
    adaptor.intervals(&mut points, shape)?;
    Ok(points)
}

/// `NbIntervals(shape) + 1` break parameters of a 2D adaptor.
pub(crate) fn cut_points_2d(
    adaptor: &dyn Adaptor2dCurve2d,
    shape: Shape,
) -> Result<Array1<'static, f64>, Failure> {
    let count: i32 = adaptor.nb_intervals(shape)?;
    let mut points: Array1<'static, f64> = one_based(count + 1)?;
    adaptor.intervals(&mut points, shape)?;
    Ok(points)
}

// The Standard_NullObject of a null adaptor handle (a native null dereference).
fn null_adaptor(context: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("{context}: null adaptor; expected a curve"),
    )
}

/// The 3D evaluator: the value, first or second derivative of the adapted curve,
/// trimmed again whenever AdvApprox moves to another interval. An adaptor exception
/// unwinds natively; it is carried to the end of AdvApprox ([`Self::take_failure`]).
pub(crate) struct Curve3dEvaluator {
    fonct: Handle<dyn Adaptor3dCurve>,
    start_end_sav: [f64; 2],
    capture: FailureCapture,
    context: &'static str,
}

impl Curve3dEvaluator {
    /// The evaluator of `fonct` over `[first, last]`; `context` names the native class.
    pub(crate) fn new(
        fonct: Handle<dyn Adaptor3dCurve>,
        range: [f64; 2],
        context: &'static str,
    ) -> Self {
        Self {
            fonct,
            start_end_sav: range,
            capture: FailureCapture::default(),
            context,
        }
    }

    /// The first adaptor failure of the approximation, if any.
    pub(crate) fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    // fonct = fonct->Trim(StartEnd[0], StartEnd[1], Precision::PConfusion()).
    fn retrim(&mut self, start_end: [f64; 2]) -> bool {
        let trimmed = match self.fonct.get() {
            Some(fonct) => fonct.trim(start_end[0], start_end[1], precision::p_confusion()),
            None => Err(null_adaptor(self.context)),
        };
        let Some(trimmed) = self.capture.keep(trimmed) else {
            return false;
        };
        self.fonct = trimmed;
        self.start_end_sav = start_end;
        true
    }

    // Writes the derivative of `order` into `result[0..3]` and returns its error
    // code (3 for an order beyond 2, with zeros); None after an adaptor failure.
    fn derivative(&mut self, par: f64, order: i32, result: &mut [f64]) -> Option<i32> {
        let fonct: &dyn Adaptor3dCurve = self.fonct.get()?;
        let value: Result<[f64; 3], Failure> = match order {
            0 => fonct.value(par).map(|p: Point3| [p.x(), p.y(), p.z()]),
            1 => fonct
                .d1(par)
                .map(|(_, v1): (Point3, Vector3)| [v1.x(), v1.y(), v1.z()]),
            2 => fonct
                .d2(par)
                .map(|(_, _, v2): (Point3, Vector3, Vector3)| [v2.x(), v2.y(), v2.z()]),
            _ => Ok([0.0, 0.0, 0.0]),
        };
        let [x, y, z] = self.capture.keep(value)?;
        (result[0], result[1], result[2]) = (x, y, z);
        Some(if (0..=2).contains(&order) { 0 } else { 3 })
    }
}

impl AdvApproxEvaluatorFunction for Curve3dEvaluator {
    fn evaluate(
        &mut self,
        dimension: i32,
        start_end: [f64; 2],
        par: f64,
        order: i32,
        result: &mut [f64],
    ) -> i32 {
        // The dimension check sets an error code but does not stop the evaluation,
        // as natively; the 3D evaluators have no parameter check.
        let error_code: i32 = if dimension != 3 { 1 } else { 0 };
        if (start_end[0] != self.start_end_sav[0] || start_end[1] != self.start_end_sav[1])
            && !self.retrim(start_end)
        {
            return 1;
        }
        // A failed adaptor stops AdvApprox; the captured failure is returned after it.
        let Some(order_code) = self.derivative(par, order, result) else {
            return 1;
        };
        // An order beyond 2 overwrites the dimension code with 3.
        if order_code != 0 {
            return order_code;
        }
        error_code
    }
}

/// The 2D evaluator; `check_parameter` adds the error code 2 for a parameter out of
/// the interval (Geom2dConvert_ApproxCurve_Eval, Approx_Curve2d_Eval; the curve on
/// surface evaluator has no such check).
pub(crate) struct Curve2dEvaluator {
    fonct: Handle<dyn Adaptor2dCurve2d>,
    start_end_sav: [f64; 2],
    capture: FailureCapture,
    check_parameter: bool,
    context: &'static str,
}

impl Curve2dEvaluator {
    /// The evaluator of `fonct` over `[first, last]`.
    pub(crate) fn new(
        fonct: Handle<dyn Adaptor2dCurve2d>,
        range: [f64; 2],
        check_parameter: bool,
        context: &'static str,
    ) -> Self {
        Self {
            fonct,
            start_end_sav: range,
            capture: FailureCapture::default(),
            check_parameter,
            context,
        }
    }

    /// The first adaptor failure of the approximation, if any.
    pub(crate) fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    fn retrim(&mut self, start_end: [f64; 2]) -> bool {
        let trimmed = match self.fonct.get() {
            Some(fonct) => fonct.trim(start_end[0], start_end[1], precision::p_confusion()),
            None => Err(null_adaptor(self.context)),
        };
        let Some(trimmed) = self.capture.keep(trimmed) else {
            return false;
        };
        self.fonct = trimmed;
        self.start_end_sav = start_end;
        true
    }

    // Writes the derivative of `order` into `result[0..2]` and returns its error
    // code (3 for an order beyond 2, with zeros); None after an adaptor failure.
    fn derivative(&mut self, par: f64, order: i32, result: &mut [f64]) -> Option<i32> {
        let fonct: &dyn Adaptor2dCurve2d = self.fonct.get()?;
        let value: Result<[f64; 2], Failure> = match order {
            0 => fonct.value(par).map(|p: Point2| [p.x(), p.y()]),
            1 => fonct
                .d1(par)
                .map(|(_, v1): (Point2, Vector2)| [v1.x(), v1.y()]),
            2 => fonct
                .d2(par)
                .map(|(_, _, v2): (Point2, Vector2, Vector2)| [v2.x(), v2.y()]),
            _ => Ok([0.0, 0.0]),
        };
        let [x, y] = self.capture.keep(value)?;
        (result[0], result[1]) = (x, y);
        Some(if (0..=2).contains(&order) { 0 } else { 3 })
    }
}

impl AdvApproxEvaluatorFunction for Curve2dEvaluator {
    fn evaluate(
        &mut self,
        dimension: i32,
        start_end: [f64; 2],
        par: f64,
        order: i32,
        result: &mut [f64],
    ) -> i32 {
        // The dimension and parameter checks set an error code but do not stop the
        // evaluation, as natively.
        let mut error_code: i32 = 0;
        if dimension != 2 {
            error_code = 1;
        }
        if self.check_parameter && (par < start_end[0] || par > start_end[1]) {
            error_code = 2;
        }
        if (start_end[0] != self.start_end_sav[0] || start_end[1] != self.start_end_sav[1])
            && !self.retrim(start_end)
        {
            return 1;
        }
        let Some(order_code) = self.derivative(par, order, result) else {
            return 1;
        };
        // An order beyond 2 overwrites the earlier codes with 3.
        if order_code != 0 {
            return order_code;
        }
        error_code
    }
}
