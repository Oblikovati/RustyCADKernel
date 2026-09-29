// Created on: 1997-09-11
// Created by: Roman BORISOV
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
// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_ApproxCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert_ApproxCurve`: the polynomial B-spline approximation of a 2D curve
//! by AdvApprox, cut preferably at its C2, then C3, breaks.
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use crate::native_text::stream_real;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adv_approx::{
    AdvApproxApproxAFunction, AdvApproxEvaluatorFunction, AdvApproxPrefAndRec,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Vector2};
use std::fmt;

/// The result of one approximation: the B-spline, whether it meets the tolerance,
/// whether there is one at all, and the greatest error.
///
/// `let approx = ApproxCurve::from_curve(&offset, 1e-4, Shape::C2, 16, 14)?;
/// if approx.has_result() { let bspline = approx.curve(); }`
#[derive(Clone)]
pub struct ApproxCurve {
    my_is_done: bool,
    my_has_result: bool,
    my_bspl_curve: Handle<Geom2dBSplineCurve>,
    my_max_error: f64,
}

impl ApproxCurve {
    /// Approximates `curve` through a Geom2dAdaptor_Curve within `tol2d`, with the
    /// continuity `order`, at most `max_segments` spans of degree `max_degree`
    /// (`Geom2dConvert_ApproxCurve(Curve, Tol2d, Order, MaxSegments, MaxDegree)`);
    /// `ApproxCurve::from_curve(&curve, 1e-4, Shape::C2, 16, 14)?`.
    pub fn from_curve(
        curve: &Handle<dyn Geom2dCurve>,
        tol2d: f64,
        order: Shape,
        max_segments: i32,
        max_degree: i32,
    ) -> Result<Self, Failure> {
        let adaptor: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_curve(curve)?).cast::<dyn Adaptor2dCurve2d>();
        Self::from_adaptor(&adaptor, tol2d, order, max_segments, max_degree)
    }

    /// Approximates the adapted `curve` (`Geom2dConvert_ApproxCurve(const
    /// occ::handle<Adaptor2d_Curve2d>&, Tol2d, Order, MaxSegments, MaxDegree)`); an
    /// exception of the adaptor propagates;
    /// `ApproxCurve::from_adaptor(&adaptor, 1e-4, Shape::C2, 16, 14)?`.
    pub fn from_adaptor(
        curve: &Handle<dyn Adaptor2dCurve2d>,
        tol2d: f64,
        order: Shape,
        max_segments: i32,
        max_degree: i32,
    ) -> Result<Self, Failure> {
        let mut result = Self {
            my_is_done: false,
            my_has_result: false,
            my_bspl_curve: Handle::default(),
            my_max_error: 0.0,
        };
        result.approximate(curve, tol2d, order, [max_segments, max_degree])?;
        Ok(result)
    }

    /// The approximating B-spline, null without a result (`Curve()`); `approx.curve()`.
    pub fn curve(&self) -> Handle<Geom2dBSplineCurve> {
        self.my_bspl_curve.clone()
    }

    /// Whether the approximation meets the tolerance (`IsDone()`); `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// Whether there is a result, within the tolerance or not (`HasResult()`);
    /// `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.my_has_result
    }

    /// The greatest distance between the curve and its approximation, 0 without a
    /// result (`MaxError()`); `approx.max_error()`.
    pub fn max_error(&self) -> f64 {
        self.my_max_error
    }

    /// Writes the native two-line text, the error printed as `%g` (`Dump(o)`);
    /// `approx.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        writeln!(stream, "******* Dump of ApproxCurve *******")?;
        writeln!(stream, "******* Error   {}", stream_real(self.max_error()))
    }

    // Approximate: one 2D subspace of tolerance tol2d on the curve range, cut with
    // AdvApprox_PrefAndRec at the C2 (recommended) and C3 (preferred) breaks.
    fn approximate(
        &mut self,
        curve: &Handle<dyn Adaptor2dCurve2d>,
        tol2d: f64,
        order: Shape,
        [max_segments, max_degree]: [i32; 2],
    ) -> Result<(), Failure> {
        let adaptor: &dyn Adaptor2dCurve2d = curve.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "Geom2dConvert_ApproxCurve: null adaptor; expected a curve",
            )
        })?;
        let mut two_d_tol: Array1<'static, f64> = one_based(1)?;
        two_d_tol.init(tol2d);
        let (first, last): (f64, f64) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let cut_tool = AdvApproxPrefAndRec::new(
            &cut_points(adaptor, Shape::C2)?,
            &cut_points(adaptor, Shape::C3)?,
            5.0,
        )?;
        self.my_max_error = 0.0;
        let mut evaluator = CurveEvaluator {
            fonct: curve.clone(),
            start_end_sav: [first, last],
            capture: FailureCapture::default(),
        };
        let approx = AdvApproxApproxAFunction::with_cutting(
            0,
            1,
            0,
            None,
            Some(&two_d_tol),
            None,
            first,
            last,
            order,
            max_degree,
            max_segments,
            &mut evaluator,
            &cut_tool,
        );
        evaluator.capture.take()?;
        self.store(&approx?)
    }

    // The B-spline of the approximation and its error.
    fn store(&mut self, approx: &AdvApproxApproxAFunction) -> Result<(), Failure> {
        self.my_is_done = approx.is_done();
        self.my_has_result = approx.has_result();
        if !self.my_has_result {
            return Ok(());
        }
        let mut poles: Array1<'static, Point2> = one_based(approx.nb_poles()?)?;
        approx.poles_2d_of(1, &mut poles)?;
        let missing = |name: &str| {
            failure(
                "Standard_NullObject",
                &format!("AdvApprox_ApproxAFunction::{name}: no table"),
            )
        };
        let knots: &Array1<'static, f64> = approx.knots().ok_or_else(|| missing("Knots"))?;
        let mults: &Array1<'static, i32> = approx
            .multiplicities()
            .ok_or_else(|| missing("Multiplicities"))?;
        self.my_bspl_curve = Handle::new(Geom2dBSplineCurve::new(
            &poles,
            knots,
            mults,
            approx.degree(),
            false,
        )?);
        self.my_max_error = approx.max_error(2, 1)?;
        Ok(())
    }
}

// NbIntervals(shape) + 1 break parameters, from Intervals(shape).
fn cut_points(
    adaptor: &dyn Adaptor2dCurve2d,
    shape: Shape,
) -> Result<Array1<'static, f64>, Failure> {
    let count: i32 = adaptor.nb_intervals(shape)?;
    let mut points: Array1<'static, f64> = one_based(count + 1)?;
    adaptor.intervals(&mut points, shape)?;
    Ok(points)
}

/// `Geom2dConvert_ApproxCurve_Eval`: the value, first or second derivative of the
/// adapted curve, trimmed again whenever AdvApprox moves to another interval.
struct CurveEvaluator {
    fonct: Handle<dyn Adaptor2dCurve2d>,
    start_end_sav: [f64; 2],
    // An adaptor exception unwinds natively; it is carried to the end of AdvApprox.
    capture: FailureCapture,
}

impl AdvApproxEvaluatorFunction for CurveEvaluator {
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
        if par < start_end[0] || par > start_end[1] {
            error_code = 2;
        }
        if (start_end[0] != self.start_end_sav[0] || start_end[1] != self.start_end_sav[1])
            && !self.retrim(start_end)
        {
            return 1;
        }
        // A failed adaptor stops AdvApprox; the captured failure is returned after it.
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

impl CurveEvaluator {
    // fonct = fonct->Trim(StartEnd[0], StartEnd[1], Precision::PConfusion()).
    fn retrim(&mut self, start_end: [f64; 2]) -> bool {
        let trimmed = match self.fonct.get() {
            Some(fonct) => fonct.trim(start_end[0], start_end[1], precision::p_confusion()),
            None => Err(failure(
                "Standard_NullObject",
                "Geom2dConvert_ApproxCurve_Eval: null adaptor",
            )),
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
