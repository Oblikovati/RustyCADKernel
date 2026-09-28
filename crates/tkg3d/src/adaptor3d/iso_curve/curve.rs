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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_IsoCurve.cxx (the Adaptor3d_Curve members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_IsoCurve` as an `Adaptor3d_Curve`. Every member reads a snapshot of
//! the fields and calls the surface without holding this curve's lock.
use super::{Adaptor3dIsoCurve, IsoState, kinds, shape};
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

impl Adaptor3dCurve for Adaptor3dIsoCurve {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let mut copy: IsoState = self.snapshot();
        if let Some(surface) = copy.surface.get() {
            copy.surface = surface.shallow_copy()?;
        }
        Ok(Handle::new(Self::from_state(copy)).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.snapshot().first)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.snapshot().last)
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        shape::continuity(&self.snapshot())
    }
    fn nb_intervals(&self, continuity: Shape) -> Result<i32, Failure> {
        shape::nb_intervals(&self.snapshot(), continuity)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, continuity: Shape) -> Result<(), Failure> {
        shape::intervals(&self.snapshot(), bounds, continuity)
    }
    fn trim(
        &self,
        first: f64,
        last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        // A copy sharing the surface, reloaded on the new range.
        let state: IsoState = self.snapshot();
        let copy = Self::from_state(state.clone());
        copy.load_iso_range(state.iso, state.parameter, (first, last))?;
        Ok(Handle::new(copy).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        shape::is_closed(&self.snapshot())
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        shape::is_periodic(&self.snapshot())
    }
    fn period(&self) -> Result<f64, Failure> {
        shape::period(&self.snapshot())
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        // Peut-on faire mieux ?? (native comment): the parametric default.
        Ok(precision::parametric(r3d))
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        kinds::get_type(&self.snapshot())
    }
    fn line(&self) -> Result<Line3, Failure> {
        kinds::line(self.d1(0.0)?)
    }
    fn circle(&self) -> Result<Circle3, Failure> {
        kinds::circle(&self.snapshot(), || self.value(0.0))
    }
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        kinds::ellipse(&self.snapshot())
    }
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        Err(kinds::hyperbola())
    }
    fn parabola(&self) -> Result<Parabola3, Failure> {
        Err(kinds::parabola())
    }
    fn degree(&self) -> Result<i32, Failure> {
        let splines = [SurfaceType::BezierSurface, SurfaceType::BSplineSurface];
        kinds::swept_or_spline(&self.snapshot(), &splines, spline_degree, |c| c.degree())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        let splines = [SurfaceType::BezierSurface, SurfaceType::BSplineSurface];
        kinds::swept_or_spline(&self.snapshot(), &splines, spline_rational, |c| {
            c.is_rational()
        })
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        let splines = [SurfaceType::BezierSurface, SurfaceType::BSplineSurface];
        kinds::swept_or_spline(&self.snapshot(), &splines, spline_poles, |c| c.nb_poles())
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        let splines = [SurfaceType::BSplineSurface];
        kinds::swept_or_spline(&self.snapshot(), &splines, spline_knots, |c| c.nb_knots())
    }
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        kinds::bezier(&self.snapshot())
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        kinds::bspline(&self.snapshot())
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        shape::d0(&self.snapshot(), u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        shape::d1(&self.snapshot(), u)
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        shape::d2(&self.snapshot(), u)
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        shape::d3(&self.snapshot(), u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        shape::dn(&self.snapshot(), u, n)
    }
}

// The spline members of the curve direction: V for an iso-U curve, U otherwise.
fn spline_degree(surface: &dyn Adaptor3dSurface, u_iso: bool) -> Result<i32, Failure> {
    if u_iso {
        surface.v_degree()
    } else {
        surface.u_degree()
    }
}
fn spline_rational(surface: &dyn Adaptor3dSurface, u_iso: bool) -> Result<bool, Failure> {
    if u_iso {
        surface.is_v_rational()
    } else {
        surface.is_u_rational()
    }
}
fn spline_poles(surface: &dyn Adaptor3dSurface, u_iso: bool) -> Result<i32, Failure> {
    if u_iso {
        surface.nb_v_poles()
    } else {
        surface.nb_u_poles()
    }
}
fn spline_knots(surface: &dyn Adaptor3dSurface, u_iso: bool) -> Result<i32, Failure> {
    if u_iso {
        surface.nb_v_knots()
    } else {
        surface.nb_u_knots()
    }
}
