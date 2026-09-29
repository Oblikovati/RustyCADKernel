// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of the Adaptor3d_Curve members of ProjLib_ProjectOnPlane
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Adaptor3d_Curve` interface of `ProjLib_ProjectOnPlane`: the bounds,
//! continuity and intervals of the original curve, the evaluation and the kind of
//! the projected result.
use super::{OnPlaneState, ProjectOnPlane};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3,
};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

impl ProjectOnPlane {
    fn snapshot(&self) -> OnPlaneState {
        self.state.read().clone()
    }
    // Standard_NoSuchObject unless the projected kind is `expected`.
    fn require_type(&self, expected: &[CurveType], member: &str) -> Result<OnPlaneState, Failure> {
        let state: OnPlaneState = self.snapshot();
        if expected.contains(&state.curve_type) {
            return Ok(state);
        }
        Err(failure(
            "Standard_NoSuchObject",
            &format!(
                "ProjLib_ProjectOnPlane:{member}: the projection is {:?}; expected {expected:?}",
                state.curve_type
            ),
        ))
    }
    // The spline kinds answer from the approximation or from the original curve.
    fn spline_source(&self, expected: &[CurveType], member: &str) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let state: OnPlaneState = self.require_type(expected, member)?;
        if state.is_approx {
            return Ok(state.result.cast());
        }
        Ok(state.curve)
    }
}

const SPLINES: [CurveType; 2] = [CurveType::BSplineCurve, CurveType::BezierCurve];

fn live(curve: &Handle<dyn Adaptor3dCurve>) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure("Standard_NullObject", "ProjLib_ProjectOnPlane: null curve; expected a loaded curve")
    })
}

impl Adaptor3dCurve for ProjectOnPlane {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let copy: OnPlaneState = self.snapshot().shallow_copy()?;
        Ok(Handle::new(ProjectOnPlane::from_state(copy)).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.keep_param || state.is_approx {
            return state.curve()?.first_parameter();
        }
        Ok(state.first_par)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.keep_param || state.is_approx {
            return state.curve()?.last_parameter();
        }
        Ok(state.last_par)
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        self.snapshot().curve()?.continuity()
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.snapshot().curve()?.nb_intervals(shape)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.snapshot().curve()?.intervals(bounds, shape)
    }
    fn trim(&self, first: f64, last: f64, tolerance: f64) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.curve_type == CurveType::OtherCurve {
            return Err(failure(
                "Standard_NotImplemented",
                "ProjLib_ProjectOnPlane::Trim() - curve of unsupported type",
            ));
        }
        state.result()?.trim(first, last, tolerance)
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        self.snapshot().curve()?.is_closed()
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.is_approx {
            return Ok(false);
        }
        state.curve()?.is_periodic()
    }
    // The native `return false` of an approximation (0.0) is unreachable, since
    // IsPeriodic is then false.
    fn period(&self) -> Result<f64, Failure> {
        if !self.is_periodic()? {
            return Err(failure("Standard_NoSuchObject", "ProjLib_ProjectOnPlane::Period"));
        }
        let state: OnPlaneState = self.snapshot();
        if state.is_approx {
            return Ok(0.0);
        }
        state.curve()?.period()
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.curve_type != CurveType::OtherCurve {
            return state.result()?.eval_d0(u);
        }
        state.oblique().value(state.curve()?, u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.curve_type != CurveType::OtherCurve {
            return state.result()?.eval_d1(u);
        }
        let (point, d1) = state.oblique().d1(state.curve()?, u)?;
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.curve_type != CurveType::OtherCurve {
            return state.result()?.eval_d2(u);
        }
        let (point, d1, d2) = state.oblique().d2(state.curve()?, u)?;
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.curve_type != CurveType::OtherCurve {
            return state.result()?.eval_d3(u);
        }
        let (point, d1, d2, d3) = state.oblique().d3(state.curve()?, u)?;
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.curve_type != CurveType::OtherCurve {
            return state.result()?.eval_dn(u, n);
        }
        state.oblique().dn(state.curve()?, u, n)
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        let state: OnPlaneState = self.snapshot();
        if state.curve_type != CurveType::OtherCurve {
            return state.result()?.resolution(r3d);
        }
        Ok(0.)
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(self.snapshot().curve_type)
    }
    fn line(&self) -> Result<Line3, Failure> {
        self.require_type(&[CurveType::Line], "Line")?.result()?.line()
    }
    fn circle(&self) -> Result<Circle3, Failure> {
        self.require_type(&[CurveType::Circle], "Circle")?.result()?.circle()
    }
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        self.require_type(&[CurveType::Ellipse], "Ellipse")?.result()?.ellipse()
    }
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        self.require_type(&[CurveType::Hyperbola], "Hyperbola")?.result()?.hyperbola()
    }
    fn parabola(&self) -> Result<Parabola3, Failure> {
        self.require_type(&[CurveType::Parabola], "Parabola")?.result()?.parabola()
    }
    fn degree(&self) -> Result<i32, Failure> {
        live(&self.spline_source(&SPLINES, "Degree")?)?.degree()
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        live(&self.spline_source(&SPLINES, "IsRational")?)?.is_rational()
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        live(&self.spline_source(&SPLINES, "NbPoles")?)?.nb_poles()
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        live(&self.spline_source(&[CurveType::BSplineCurve], "NbKnots")?)?.nb_knots()
    }
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        self.require_type(&[CurveType::BezierCurve], "Bezier")?.result()?.bezier()
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        self.require_type(&[CurveType::BSplineCurve], "BSpline")?.result()?.bspline()
    }
}
