// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_Curve3d.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_Curve3d`: the polynomial B-spline approximation of an adapted 3D curve by
//! AdvApprox, cut preferably at its C2, then C3, breaks.
use crate::curve_approx_eval::{Curve3dEvaluator, cut_points_3d};
use crate::native_failure::failure;
use crate::native_text::stream_real;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adv_approx::{AdvApproxApproxAFunction, AdvApproxPrefAndRec};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use std::fmt;

/// `Approx_Curve3d`: the B-spline, whether it meets the tolerance, whether there is
/// one at all, and the greatest error.
///
/// ```ignore
/// let approx = Curve3d::new(&adaptor, 1e-4, Shape::C2, 16, 14)?;
/// if approx.has_result() { let bspline = approx.curve(); }
/// ```
#[derive(Clone)]
pub struct Curve3d {
    is_done: bool,
    has_result: bool,
    bspl_curve: Handle<GeomBSplineCurve>,
    max_error: f64,
}

/// The Standard_NullObject of a missing AdvApprox result table or a null handle.
pub(crate) fn null_table(what: &str) -> Failure {
    failure("Standard_NullObject", &format!("{what}: null handle"))
}

impl Curve3d {
    /// `Approx_Curve3d(Curve, Tol3d, Order, MaxSegments, MaxDegree)`: approximates the
    /// adapted curve on its range within `tol3d`; an adaptor exception propagates;
    /// `Curve3d::new(&adaptor, 1e-4, Shape::C2, 16, 14)?`.
    pub fn new(
        curve: &Handle<dyn Adaptor3dCurve>,
        tol3d: f64,
        order: Shape,
        max_segments: i32,
        max_degree: i32,
    ) -> Result<Self, Failure> {
        let adaptor: &dyn Adaptor3dCurve = curve
            .get()
            .ok_or_else(|| null_table("Approx_Curve3d: Curve"))?;
        let mut three_d_tol: Array1<'static, f64> = one_based(1)?;
        three_d_tol.init(tol3d);
        let (first, last): (f64, f64) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let cut_tool = AdvApproxPrefAndRec::new(
            &cut_points_3d(adaptor, Shape::C2)?,
            &cut_points_3d(adaptor, Shape::C3)?,
            5.0,
        )?;
        let mut evaluator =
            Curve3dEvaluator::new(curve.clone(), [first, last], "Approx_Curve3d_Eval");
        let approx = AdvApproxApproxAFunction::with_cutting(
            0,
            0,
            1,
            None,
            None,
            Some(&three_d_tol),
            first,
            last,
            order,
            max_degree,
            max_segments,
            &mut evaluator,
            &cut_tool,
        );
        evaluator.take_failure()?;
        Self::from_approximation(&approx?)
    }

    fn from_approximation(approx: &AdvApproxApproxAFunction) -> Result<Self, Failure> {
        let mut result = Self {
            is_done: approx.is_done(),
            has_result: approx.has_result(),
            bspl_curve: Handle::default(),
            max_error: 0.0,
        };
        if !result.has_result {
            return Ok(result);
        }
        let mut poles: Array1<'static, Point3> = one_based(approx.nb_poles()?)?;
        approx.poles_of(1, &mut poles)?;
        let knots = approx.knots().ok_or_else(|| null_table("Knots"))?;
        let mults = approx
            .multiplicities()
            .ok_or_else(|| null_table("Multiplicities"))?;
        result.bspl_curve = Handle::new(GeomBSplineCurve::new(
            &poles,
            knots,
            mults,
            approx.degree(),
            false,
        )?);
        result.max_error = approx.max_error(3, 1)?;
        Ok(result)
    }

    /// `Curve()`: the approximation, null without a result; `approx.curve()`.
    pub fn curve(&self) -> Handle<GeomBSplineCurve> {
        self.bspl_curve.clone()
    }

    /// `IsDone()`: the tolerance is reached; `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }

    /// `HasResult()`: there is a curve, within the tolerance or not;
    /// `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.has_result
    }

    /// `MaxError()`: the greatest error, 0 without a result; `approx.max_error()`.
    pub fn max_error(&self) -> f64 {
        self.max_error
    }

    /// `Dump(o)`: the degree, the number of spans and the error (`%g`); without a
    /// result the native member dereferences the null curve (Standard_NullObject);
    /// `approx.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut dyn fmt::Write) -> Result<(), Failure> {
        let curve: &GeomBSplineCurve = self
            .bspl_curve
            .get()
            .ok_or_else(|| null_table("Approx_Curve3d::Dump: Curve()"))?;
        let text = format!(
            "******* Dump of ApproxCurve *******\n*******Degree     {}\n*******NbSegments {}\n*******Error      {}\n",
            curve.degree(),
            curve.nb_knots() - 1,
            stream_real(self.max_error)
        );
        stream
            .write_str(&text)
            .map_err(|_| failure("Standard_Failure", "Approx_Curve3d::Dump: stream error"))
    }
}
