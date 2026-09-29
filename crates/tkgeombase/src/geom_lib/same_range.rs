// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (SameRange)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::SameRange`: a 2D curve reparametrized onto a requested range.
use super::live;
use crate::geom2d_convert;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dConic, Geom2dCurve, Geom2dGeometry, Geom2dLine,
    Geom2dTrimmedCurve,
};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Direction2, Point2, Transform2, Vector2};

/// `GeomLib::SameRange(Tolerance, Curve2dPtr, First, Last, RequestedFirst,
/// RequestedLast, NewCurve2dPtr)`: the curve whose range `[first, last]` becomes
/// `[requested_first, requested_last]`, written into `new_curve_ptr`.
///
/// Within `tolerance` of the requested range the curve itself is returned. With the
/// same length, a line is translated, a circle rotated about its centre, a trimmed
/// curve handled through its basis and re-trimmed, and any other curve converted to
/// a reparametrized B-spline unless its range is null and `RequestedLast +
/// RequestedFirst` (a sum, as natively) is null too, in which case `new_curve_ptr`
/// is left unchanged. Otherwise the curve (a periodic basis on its range, any other
/// on the range clipped to its bounds, or its whole range when that clipping is
/// null) becomes a reparametrized B-spline. Standard_Failure for a null curve;
/// `geom_lib::same_range(1e-9, &c, 0.0, 1.0, 2.0, 3.0, &mut new_curve)?`.
pub fn same_range(
    tolerance: f64,
    curve_ptr: &Handle<dyn Geom2dCurve>,
    (first_on_curve, last_on_curve): (f64, f64),
    (requested_first, requested_last): (f64, f64),
    new_curve_ptr: &mut Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    if curve_ptr.is_null() {
        return Err(failure(
            "Standard_Failure",
            "GeomLib::SameRange: null curve; expected a Geom2d_Curve",
        ));
    }
    if (last_on_curve - requested_last).abs() <= tolerance
        && (first_on_curve - requested_first).abs() <= tolerance
    {
        *new_curve_ptr = curve_ptr.clone();
        return Ok(());
    }
    // The parametrisation length must at least be the same.
    let range = SameRangeRequest {
        tolerance,
        first_on_curve,
        last_on_curve,
        requested_first,
        requested_last,
    };
    if (last_on_curve - first_on_curve - requested_last + requested_first).abs() <= tolerance {
        return range.same_length(curve_ptr, new_curve_ptr);
    }
    // We segment the result.
    let tc: Handle<dyn Geom2dCurve> = range.segment(curve_ptr)?;
    *new_curve_ptr = range.reparametrized(&tc)?;
    Ok(())
}

// The ranges of one SameRange call.
#[derive(Clone, Copy)]
struct SameRangeRequest {
    tolerance: f64,
    first_on_curve: f64,
    last_on_curve: f64,
    requested_first: f64,
    requested_last: f64,
}

impl SameRangeRequest {
    fn same_length(
        &self,
        curve_ptr: &Handle<dyn Geom2dCurve>,
        new_curve_ptr: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        let source: &dyn Geom2dCurve = live(curve_ptr, "GeomLib::SameRange")?;
        if curve_ptr.cast::<Geom2dLine>().is_some() {
            let line: Handle<Geom2dLine> = source.copy()?.cast();
            let du: f64 = self.first_on_curve - self.requested_first;
            let d: Direction2 = live(&line, "GeomLib::SameRange")?.direction();
            live(&line, "GeomLib::SameRange")?.translate(du * Vector2::from(d))?;
            *new_curve_ptr = line.cast();
        } else if let Some(circle) = curve_ptr.cast::<Geom2dCircle>().get() {
            *new_curve_ptr = source.copy()?.cast();
            let p: Point2 = circle.location();
            let du: f64 = if circle.circ2d()?.is_direct() {
                self.first_on_curve - self.requested_first
            } else {
                self.requested_first - self.first_on_curve
            };
            let mut trsf = Transform2::default();
            trsf.set_rotation(p, du);
            live(new_curve_ptr, "GeomLib::SameRange")?.transform(trsf)?;
        } else if let Some(tc) = curve_ptr.cast::<Geom2dTrimmedCurve>().get() {
            same_range(
                self.tolerance,
                &tc.basis_curve(),
                (self.first_on_curve, self.last_on_curve),
                (self.requested_first, self.requested_last),
                new_curve_ptr,
            )?;
            let trimmed = Geom2dTrimmedCurve::new(
                new_curve_ptr,
                self.requested_first,
                self.requested_last,
                true,
                true,
            )?;
            *new_curve_ptr = Handle::new(trimmed).cast();
        } else if (self.last_on_curve - self.first_on_curve).abs() > precision::p_confusion()
            || (self.requested_last + self.requested_first).abs() > precision::p_confusion()
        {
            // Beware of limitation issues: the SAME test as Geom2d_TrimmedCurve::SetTrim,
            // since the result may be re-trimmed on the requested bounds.
            let tc = Geom2dTrimmedCurve::new(
                curve_ptr,
                self.first_on_curve,
                self.last_on_curve,
                true,
                true,
            )?;
            *new_curve_ptr = self.reparametrized(&Handle::new(tc).cast())?;
        }
        Ok(())
    }

    // The trimmed curve of the segmenting branch.
    fn segment(
        &self,
        curve_ptr: &Handle<dyn Geom2dCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let source: &dyn Geom2dCurve = live(curve_ptr, "GeomLib::SameRange")?;
        let a_c_check: Handle<dyn Geom2dCurve> = match curve_ptr.cast::<Geom2dTrimmedCurve>().get()
        {
            Some(trimmed) => trimmed.basis_curve(),
            None => curve_ptr.clone(),
        };
        let (u1, u2): (f64, f64) = if live(&a_c_check, "GeomLib::SameRange")?.is_periodic() {
            (self.first_on_curve, self.last_on_curve)
        } else {
            (
                std_max(source.first_parameter(), self.first_on_curve),
                std_min(source.last_parameter(), self.last_on_curve),
            )
        };
        let (u1, u2) = if (u2 - u1).abs() > precision::p_confusion() {
            (u1, u2)
        } else {
            (source.first_parameter(), source.last_parameter())
        };
        Ok(Handle::new(Geom2dTrimmedCurve::new(curve_ptr, u1, u2, true, true)?).cast())
    }

    // The B-spline of `tc` with its knots reparametrized onto the requested range.
    fn reparametrized(
        &self,
        tc: &Handle<dyn Geom2dCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let bs: Handle<Geom2dBSplineCurve> =
            geom2d_convert::curve_to_bspline_curve(tc, ParameterisationType::TgtThetaOver2)?;
        let bspline: &Geom2dBSplineCurve = live(&bs, "GeomLib::SameRange")?;
        let mut knots: Array1<'static, f64> = bspline.knots();
        bsplclib::reparametrize(self.requested_first, self.requested_last, &mut knots)?;
        bspline.set_knots(&knots)?;
        Ok(bs.cast())
    }
}
