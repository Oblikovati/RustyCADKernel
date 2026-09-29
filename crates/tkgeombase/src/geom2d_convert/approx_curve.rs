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
use crate::curve_approx_eval::{Curve2dEvaluator, cut_points_2d};
use crate::native_failure::failure;
use crate::native_text::stream_real;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adv_approx::{AdvApproxApproxAFunction, AdvApproxPrefAndRec};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point2;
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
            &cut_points_2d(adaptor, Shape::C2)?,
            &cut_points_2d(adaptor, Shape::C3)?,
            5.0,
        )?;
        self.my_max_error = 0.0;
        let mut evaluator = Curve2dEvaluator::new(
            curve.clone(),
            [first, last],
            true,
            "Geom2dConvert_ApproxCurve_Eval",
        );
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
        evaluator.take_failure()?;
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
