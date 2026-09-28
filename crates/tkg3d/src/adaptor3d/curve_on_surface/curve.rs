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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_CurveOnSurface.cxx (the Adaptor3d_Curve members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_CurveOnSurface` as an `Adaptor3d_Curve`. Every member reads a snapshot
//! of the fields and calls the curve and surfaces without holding this lock.
use super::intervals::copy_intervals;
use super::kpart::{to3d_ellipse, to3d_hyperbola, to3d_parabola, to3d_point};
use super::{Adaptor3dCurveOnSurface, CosState, eval};
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::errors::{array_bounds, no_such_object, not_implemented};
use crate::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Plane3, Point2, Point3, Vector3,
};
use std::f64::consts::PI;

impl Adaptor3dCurve for Adaptor3dCurveOnSurface {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let mut copy: CosState = self.snapshot();
        if let Some(surface) = copy.surface.get() {
            copy.surface = surface.shallow_copy()?;
        }
        if let Some(curve) = copy.curve.get() {
            copy.curve = curve.shallow_copy()?;
        }
        if let Some(first) = copy.first_surface.get() {
            copy.first_surface = first.shallow_copy()?;
        }
        if let Some(last) = copy.last_surface.get() {
            copy.last_surface = last.shallow_copy()?;
        }
        Ok(Handle::new(Self::from_state(copy)).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        self.snapshot().curve()?.first_parameter()
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        self.snapshot().curve()?.last_parameter()
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        let state = self.snapshot();
        let mut result: Shape = state.curve()?.continuity()?;
        let surface_u: Shape = state.surface()?.u_continuity()?;
        if surface_u < result {
            result = surface_u;
        }
        let surface_v: Shape = state.surface()?.v_continuity()?;
        if surface_v < result {
            result = surface_v;
        }
        Ok(result)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        Ok(self.interval_bounds(shape)?.len() as i32 - 1)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        copy_intervals(&self.interval_bounds(shape)?, bounds)
    }
    fn trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let state = self.snapshot();
        let trimmed = Self::default();
        trimmed.load_surface(&state.surface)?;
        trimmed.load_curve(&state.curve()?.trim(first, last, tolerance)?)?;
        Ok(Handle::new(trimmed).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        self.snapshot().curve()?.is_closed()
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        let state = self.snapshot();
        if matches!(state.kind, CurveType::Circle | CurveType::Ellipse) {
            return Ok(true);
        }
        state.curve()?.is_periodic()
    }
    fn period(&self) -> Result<f64, Failure> {
        let state = self.snapshot();
        if matches!(state.kind, CurveType::Circle | CurveType::Ellipse) {
            return Ok(2. * PI);
        }
        state.curve()?.period()
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        let state = self.snapshot();
        let ru: f64 = state.surface()?.u_resolution(r3d)?;
        let rv: f64 = state.surface()?.v_resolution(r3d)?;
        state.curve()?.resolution(std_min(ru, rv))
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(self.snapshot().kind)
    }
    fn line(&self) -> Result<Line3, Failure> {
        let state = self.snapshot();
        if state.kind != CurveType::Line {
            return Err(no_such_object(
                "Adaptor3d_CurveOnSurface::Line(): curve is not a line",
            ));
        }
        Ok(state.line)
    }
    fn circle(&self) -> Result<Circle3, Failure> {
        let state = self.snapshot();
        if state.kind != CurveType::Circle {
            // The native message names Line().
            return Err(no_such_object(
                "Adaptor3d_CurveOnSurface::Line(): curve is not a circle",
            ));
        }
        Ok(state.circle)
    }
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        let state = self.snapshot();
        to3d_ellipse(&state.surface()?.plane()?, state.curve()?.ellipse()?)
    }
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        let state = self.snapshot();
        to3d_hyperbola(&state.surface()?.plane()?, state.curve()?.hyperbola()?)
    }
    fn parabola(&self) -> Result<Parabola3, Failure> {
        let state = self.snapshot();
        to3d_parabola(&state.surface()?.plane()?, state.curve()?.parabola()?)
    }
    fn degree(&self) -> Result<i32, Failure> {
        // On a parametric surface it should multiply (native comment).
        self.snapshot().curve()?.degree()
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        let state = self.snapshot();
        Ok(state.curve()?.is_rational()?
            || state.surface()?.is_u_rational()?
            || state.surface()?.is_v_rational()?)
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        self.snapshot().curve()?.nb_poles()
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        let state = self.snapshot();
        if state.surface()?.get_type()? != SurfaceType::Plane {
            return Err(no_such_object(
                "Adaptor3d_CurveOnSurface::NbKnots: the surface is not a plane",
            ));
        }
        state.curve()?.nb_knots()
    }
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        let state = self.snapshot();
        let plane: Plane3 = plane_of(state.surface()?, "Bezier")?;
        let bezier: Handle<Geom2dBezierCurve> = state.curve()?.bezier()?;
        let Some(source) = bezier.get() else {
            return Err(not_implemented(
                "Adaptor3d_CurveOnSurface::Bezier: null 2D Bezier curve",
            ));
        };
        let poles = mapped_poles(&plane, source.nb_poles(), |i| source.pole(i))?;
        let curve = match source.is_rational() {
            true => {
                source.weights_array(|weights| GeomBezierCurve::with_weights(&poles, weights))?
            }
            false => GeomBezierCurve::new(&poles)?,
        };
        Ok(Handle::new(curve))
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        let state = self.snapshot();
        let plane: Plane3 = plane_of(state.surface()?, "BSpline")?;
        let bspline: Handle<Geom2dBSplineCurve> = state.curve()?.bspline()?;
        let Some(source) = bspline.get() else {
            return Err(not_implemented(
                "Adaptor3d_CurveOnSurface::BSpline: null 2D B-spline curve",
            ));
        };
        let poles = mapped_poles(&plane, source.nb_poles(), |i| source.pole(i))?;
        let (knots, mults) = (source.knots(), source.multiplicities());
        let (degree, periodic) = (source.degree(), source.is_periodic());
        let curve = match source.is_rational() {
            true => source.weights_array(|weights| {
                GeomBSplineCurve::with_weights(&poles, weights, &knots, &mults, degree, periodic)
            })?,
            false => GeomBSplineCurve::new(&poles, &knots, &mults, degree, periodic)?,
        };
        Ok(Handle::new(curve))
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        eval::d0(&self.snapshot(), u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        eval::d1(&self.snapshot(), u)
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        eval::d2(&self.snapshot(), u)
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        eval::d3(&self.snapshot(), u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        match n {
            1 => Ok(self.eval_d1(u)?.d1),
            2 => Ok(self.eval_d2(u)?.d2),
            3 => Ok(self.eval_d3(u)?.d3),
            _ => Err(not_implemented("Adaptor3d_CurveOnSurface:EvalDN")),
        }
    }
}

// The plane of Bezier/BSpline: Standard_NoSuchObject on another surface kind.
fn plane_of(surface: &dyn Adaptor3dSurface, member: &str) -> Result<Plane3, Failure> {
    if surface.get_type()? != SurfaceType::Plane {
        return Err(no_such_object(&format!(
            "Adaptor3d_CurveOnSurface : {member}"
        )));
    }
    surface.plane()
}

// The 2D poles mapped onto the plane, into 1..=NbPoles.
fn mapped_poles(
    plane: &Plane3,
    count: i32,
    pole: impl Fn(i32) -> Result<Point2, Failure>,
) -> Result<Array1<'static, Point3>, Failure> {
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, count).map_err(array_bounds)?;
    for i in 1..=count {
        poles
            .set_value(i, to3d_point(plane, pole(i)?))
            .map_err(array_bounds)?;
    }
    Ok(poles)
}
