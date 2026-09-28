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
// Rust translation of ModelingData/TKG3d/Geom/Geom_ToroidalSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_ToroidalSurface`: a torus, U the angle around the main axis and V the
//! angle around the generating circle, both periodic.
use super::circle::GeomCircle;
use super::curve::GeomCurve;
use super::elementary_results::{surface_d2, surface_d3};
use super::elementary_surface::{
    GeomElementarySurface, dump_elementary_surface, elementary_u_reverse, elementary_v_reverse,
    impl_elementary_position,
};
use super::errors;
use super::geometry::GeomGeometry;
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis3, Point3, Torus3, Transform3, Vector3, resolution};
use std::f64::consts::PI;
use std::io;

#[derive(Clone, Copy)]
struct TorusState {
    pos: Axis3,
    major_radius: f64,
    minor_radius: f64,
}

/// A torus; `Handle::new(GeomToroidalSurface::new(frame, 5.0, 1.0)?)`.
pub struct GeomToroidalSurface {
    parent: StandardTransient,
    state: GeometryCell<TorusState>,
}
impl_transient!(
    GeomToroidalSurface,
    "Geom_ToroidalSurface",
    dyn GeomElementarySurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomElementarySurface]
);
impl Clone for GeomToroidalSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

impl GeomToroidalSurface {
    fn stored(state: TorusState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(state),
        }
    }
    /// Torus around the main axis of `frame`; Standard_ConstructionError when a radius
    /// is negative. Like the native constructor, a minor radius at least equal to the
    /// major one is accepted, although the setters refuse it
    /// (`Geom_ToroidalSurface(gp_Ax3, MajorRadius, MinorRadius)`);
    /// `GeomToroidalSurface::new(frame, 5.0, 1.0)?`.
    pub fn new(frame: Axis3, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if minor_radius < 0.0 || major_radius < 0.0 {
            return Err(errors::construction(&format!(
                "Geom_ToroidalSurface: MajorRadius = {major_radius}, MinorRadius = {minor_radius}; expected both >= 0"
            )));
        }
        Ok(Self::stored(TorusState {
            pos: frame,
            major_radius,
            minor_radius,
        }))
    }
    /// Torus from a gp torus (`Geom_ToroidalSurface(gp_Torus)`);
    /// `GeomToroidalSurface::from_torus(t)`.
    pub fn from_torus(torus: Torus3) -> Self {
        Self::stored(TorusState {
            pos: torus.position(),
            major_radius: torus.major_radius(),
            minor_radius: torus.minor_radius(),
        })
    }
    /// Set the major radius; Standard_ConstructionError unless it exceeds the minor
    /// radius by more than gp::Resolution() (`SetMajorRadius`); `surface.set_major_radius(6.0)?`.
    pub fn set_major_radius(&self, major_radius: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        if major_radius - state.minor_radius <= resolution() {
            return Err(errors::construction(&format!(
                "Geom_ToroidalSurface::SetMajorRadius: MajorRadius = {major_radius}, MinorRadius = {}; expected MajorRadius - MinorRadius > gp::Resolution()",
                state.minor_radius
            )));
        }
        state.major_radius = major_radius;
        Ok(())
    }
    /// Set the minor radius; Standard_ConstructionError when negative or not smaller
    /// than the major radius by more than gp::Resolution() (`SetMinorRadius`);
    /// `surface.set_minor_radius(0.5)?`.
    pub fn set_minor_radius(&self, minor_radius: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        if minor_radius < 0.0 || state.major_radius - minor_radius <= resolution() {
            return Err(errors::construction(&format!(
                "Geom_ToroidalSurface::SetMinorRadius: MinorRadius = {minor_radius}, MajorRadius = {}; expected 0 <= MinorRadius < MajorRadius - gp::Resolution()",
                state.major_radius
            )));
        }
        state.minor_radius = minor_radius;
        Ok(())
    }
    /// Replace by a gp torus (`SetTorus`); `surface.set_torus(t)`.
    pub fn set_torus(&self, torus: Torus3) {
        let mut state = self.state.write();
        state.minor_radius = torus.minor_radius();
        state.major_radius = torus.major_radius();
        state.pos = torus.position();
    }
    /// The gp torus (`Torus`); `surface.torus()?`.
    pub fn torus(&self) -> Result<Torus3, Failure> {
        let s: TorusState = self.snapshot();
        Torus3::new(s.pos, s.major_radius, s.minor_radius)
    }
    /// The area `4 * PI^2 * r * R` (`Area`); `surface.area()`.
    pub fn area(&self) -> f64 {
        let s: TorusState = self.snapshot();
        4.0 * PI * PI * s.minor_radius * s.major_radius
    }
    /// The 35 quartic equation coefficients through gp_Torus; Standard_DimensionError
    /// for an array shorter than 35 (`Coefficients`); `surface.coefficients(&mut coef)?`.
    pub fn coefficients(&self, coef: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.torus()?.coefficients(coef)
    }
    /// The major radius (`MajorRadius`); `surface.major_radius()`.
    pub fn major_radius(&self) -> f64 {
        self.state.read().major_radius
    }
    /// The minor radius (`MinorRadius`); `surface.minor_radius()`.
    pub fn minor_radius(&self) -> f64 {
        self.state.read().minor_radius
    }
    /// The volume `(PI * r^2) * (2 * PI * R)` (`Volume`); `surface.volume()`.
    pub fn volume(&self) -> f64 {
        let s: TorusState = self.snapshot();
        (PI * s.minor_radius * s.minor_radius) * (2.0 * PI * s.major_radius)
    }
    fn snapshot(&self) -> TorusState {
        *self.state.read()
    }
}
impl GeomGeometry for GeomToroidalSurface {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.major_radius *= transform.scale_factor().abs();
        state.minor_radius *= transform.scale_factor().abs();
        state.pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let s: TorusState = self.snapshot();
        dump_class_name(stream, "Geom_ToroidalSurface")?;
        dump_base(stream, depth, |st, d| dump_elementary_surface(st, d, s.pos))?;
        dump::dump_real(stream, "majorRadius", s.major_radius)?;
        dump::dump_real(stream, "minorRadius", s.minor_radius)
    }
}
impl GeomSurface for GeomToroidalSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        elementary_u_reverse(&mut self.state.write().pos);
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> f64 {
        2. * PI - u
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        elementary_v_reverse(&mut self.state.write().pos);
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> f64 {
        2. * PI - v
    }
    fn bounds(&self) -> SurfaceBounds {
        (0.0, 2.0 * PI, 0.0, 2.0 * PI)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_u_periodic(&self) -> bool {
        true
    }
    fn is_v_periodic(&self) -> bool {
        true
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let s: TorusState = self.snapshot();
        let circle = elslib::torus_u_iso(s.pos, s.major_radius, s.minor_radius, u);
        Ok(Handle::new(GeomCircle::from_circ(circle)).cast())
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let s: TorusState = self.snapshot();
        let circle = elslib::torus_v_iso(s.pos, s.major_radius, s.minor_radius, v);
        Ok(Handle::new(GeomCircle::from_circ(circle)).cast())
    }
    fn continuity(&self) -> Shape {
        Shape::CN
    }
    fn is_cn_u(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_cn_v(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let s: TorusState = self.snapshot();
        Ok(elslib::torus_d0(
            u,
            v,
            s.pos,
            s.major_radius,
            s.minor_radius,
        ))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let s: TorusState = self.snapshot();
        let (point, d1u, d1v) = elslib::torus_d1(u, v, s.pos, s.major_radius, s.minor_radius);
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let s: TorusState = self.snapshot();
        let values = elslib::torus_d2(u, v, s.pos, s.major_radius, s.minor_radius);
        Ok(surface_d2(values))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let s: TorusState = self.snapshot();
        let values = elslib::torus_d3(u, v, s.pos, s.major_radius, s.minor_radius);
        Ok(surface_d3(values))
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_ToroidalSurface", nu, nv)?;
        let s: TorusState = self.snapshot();
        Ok(elslib::torus_dn(
            u,
            v,
            s.pos,
            s.major_radius,
            s.minor_radius,
            nu,
            nv,
        ))
    }
}
impl GeomElementarySurface for GeomToroidalSurface {
    impl_elementary_position!();
}
