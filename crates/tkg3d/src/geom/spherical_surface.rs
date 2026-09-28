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
// Rust translation of ModelingData/TKG3d/Geom/Geom_SphericalSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_SphericalSurface`: a sphere, U the longitude in [0, 2*PI] and V the
//! latitude in [-PI/2, PI/2].
use super::circle::GeomCircle;
use super::curve::GeomCurve;
use super::elementary_results::{surface_d2, surface_d3};
use super::elementary_surface::{
    GeomElementarySurface, dump_elementary_surface, elementary_u_reverse, elementary_v_reverse,
    impl_elementary_position,
};
use super::errors;
use super::geometry::GeomGeometry;
use super::quadric::sphere_coefficients;
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
};
use super::trimmed_curve::GeomTrimmedCurve;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis3, Point3, Sphere3, Transform3, Vector3};
use std::f64::consts::PI;
use std::io;

#[derive(Clone, Copy)]
struct SphereState {
    pos: Axis3,
    radius: f64,
}

/// A sphere; `Handle::new(GeomSphericalSurface::new(frame, 2.0)?)`.
pub struct GeomSphericalSurface {
    parent: StandardTransient,
    state: SharedState<SphereState>,
}
impl_transient!(
    GeomSphericalSurface,
    "Geom_SphericalSurface",
    dyn GeomElementarySurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomElementarySurface]
);
impl Clone for GeomSphericalSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

fn checked_radius(radius: f64, member: &str) -> Result<f64, Failure> {
    if radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom_SphericalSurface::{member}: R = {radius}; expected R >= 0"
        )));
    }
    Ok(radius)
}

impl GeomSphericalSurface {
    fn stored(state: SphereState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Sphere of `radius` centered on the origin of `frame`; Standard_ConstructionError
    /// when `radius < 0` (`Geom_SphericalSurface(gp_Ax3, Radius)`);
    /// `GeomSphericalSurface::new(frame, 2.0)?`.
    pub fn new(frame: Axis3, radius: f64) -> Result<Self, Failure> {
        let radius: f64 = checked_radius(radius, "Geom_SphericalSurface")?;
        Ok(Self::stored(SphereState { pos: frame, radius }))
    }
    /// Sphere from a gp sphere (`Geom_SphericalSurface(gp_Sphere)`);
    /// `GeomSphericalSurface::from_sphere(s)`.
    pub fn from_sphere(sphere: Sphere3) -> Self {
        Self::stored(SphereState {
            pos: sphere.position(),
            radius: sphere.radius(),
        })
    }
    /// Set the radius; Standard_ConstructionError when negative (`SetRadius`);
    /// `surface.set_radius(3.0)?`.
    pub fn set_radius(&self, radius: f64) -> Result<(), Failure> {
        self.state.write().radius = checked_radius(radius, "SetRadius")?;
        Ok(())
    }
    /// Replace by a gp sphere (`SetSphere`); `surface.set_sphere(s)`.
    pub fn set_sphere(&self, sphere: Sphere3) {
        let mut state = self.state.write();
        state.radius = sphere.radius();
        state.pos = sphere.position();
    }
    /// The gp sphere (`Sphere`); `surface.sphere()?`.
    pub fn sphere(&self) -> Result<Sphere3, Failure> {
        let state: SphereState = self.snapshot();
        Sphere3::new(state.pos, state.radius)
    }
    /// The area `4 * PI * R * R` (`Area`); `surface.area()`.
    pub fn area(&self) -> f64 {
        let radius: f64 = self.radius();
        4.0 * PI * radius * radius
    }
    /// The implicit equation coefficients `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]`
    /// (`Coefficients`); `surface.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let state: SphereState = self.snapshot();
        sphere_coefficients(state.pos, state.radius)
    }
    /// The radius (`Radius`); `surface.radius()`.
    pub fn radius(&self) -> f64 {
        self.state.read().radius
    }
    /// The volume `4 * PI * R^3 / 3` (`Volume`); `surface.volume()`.
    pub fn volume(&self) -> f64 {
        let radius: f64 = self.radius();
        (4.0 * PI * radius * radius * radius) / 3.0
    }
    fn snapshot(&self) -> SphereState {
        *self.state.read()
    }
}
impl GeomGeometry for GeomSphericalSurface {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.radius *= transform.scale_factor().abs();
        state.pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: SphereState = self.snapshot();
        dump_class_name(stream, "Geom_SphericalSurface")?;
        dump_base(stream, depth, |s, d| {
            dump_elementary_surface(s, d, state.pos)
        })?;
        dump::dump_real(stream, "radius", state.radius)
    }
}
impl GeomSurface for GeomSphericalSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        elementary_u_reverse(&mut self.state.write().pos);
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(2. * PI - u)
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        elementary_v_reverse(&mut self.state.write().pos);
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        Ok(-v)
    }
    fn bounds(&self) -> SurfaceBounds {
        (0.0, PI * 2.0, -PI / 2.0, PI / 2.0)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_u_periodic(&self) -> bool {
        true
    }
    fn is_v_periodic(&self) -> bool {
        false
    }
    /// The meridian circle trimmed to the latitude range `[-PI/2, PI/2]`.
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let state: SphereState = self.snapshot();
        let circle = elslib::sphere_u_iso(state.pos, state.radius, u);
        let meridian: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circle)).cast();
        let iso = GeomTrimmedCurve::new(&meridian, -PI / 2., PI / 2., true, true)?;
        Ok(Handle::new(iso).cast())
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let state: SphereState = self.snapshot();
        let circle = elslib::sphere_v_iso(state.pos, state.radius, v);
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
        let state: SphereState = self.snapshot();
        Ok(elslib::sphere_d0(u, v, state.pos, state.radius))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let state: SphereState = self.snapshot();
        let (point, d1u, d1v) = elslib::sphere_d1(u, v, state.pos, state.radius);
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let state: SphereState = self.snapshot();
        Ok(surface_d2(elslib::sphere_d2(u, v, state.pos, state.radius)))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let state: SphereState = self.snapshot();
        Ok(surface_d3(elslib::sphere_d3(u, v, state.pos, state.radius)))
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_SphericalSurface", nu, nv)?;
        let state: SphereState = self.snapshot();
        Ok(elslib::sphere_dn(u, v, state.pos, state.radius, nu, nv))
    }
}
impl GeomElementarySurface for GeomSphericalSurface {
    impl_elementary_position!();
}
