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
// Rust translation of ModelingData/TKG3d/Geom/Geom_ConicalSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_ConicalSurface`: an infinite cone, U the angle around the main axis and V
//! the length along a generating line from the reference circle.
use super::circle::GeomCircle;
use super::curve::GeomCurve;
use super::elementary_results::{surface_d2, surface_d3};
use super::elementary_surface::{
    GeomElementarySurface, dump_elementary_surface, elementary_u_reverse, impl_elementary_position,
};
use super::errors;
use super::geometry::GeomGeometry;
use super::line::GeomLine;
use super::parametric::{scaled_parameter, v_affinity};
use super::quadric::cone_coefficients;
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{
    Axis3, Cone3, GeneralTransform2, Point3, Transform3, Vector3, Xyz, resolution,
};
use std::f64::consts::PI;
use std::io;

#[derive(Clone, Copy)]
struct ConeState {
    pos: Axis3,
    radius: f64,
    semi_angle: f64,
}

/// A cone; `Handle::new(GeomConicalSurface::new(frame, 0.3, 2.0)?)`.
pub struct GeomConicalSurface {
    parent: StandardTransient,
    state: GeometryCell<ConeState>,
}
impl_transient!(
    GeomConicalSurface,
    "Geom_ConicalSurface",
    dyn GeomElementarySurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomElementarySurface]
);
impl Clone for GeomConicalSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

// The native angle check: gp::Resolution() < |Ang| < PI/2 - gp::Resolution().
fn angle_out_of_range(angle: f64) -> bool {
    angle.abs() <= resolution() || angle.abs() >= PI / 2.0 - resolution()
}

fn checked_radius(radius: f64) -> Result<f64, Failure> {
    if radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom_ConicalSurface::SetRadius: R = {radius}; expected R >= 0"
        )));
    }
    Ok(radius)
}

impl GeomConicalSurface {
    fn stored(state: ConeState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(state),
        }
    }
    /// Cone with reference radius `radius` in the XY plane of `frame` and half-angle
    /// `angle`; Standard_ConstructionError when `radius < 0` or `|angle|` is not in
    /// `(gp::Resolution(), PI/2 - gp::Resolution())`
    /// (`Geom_ConicalSurface(gp_Ax3, Ang, Radius)`); `GeomConicalSurface::new(frame, 0.3, 2.0)?`.
    pub fn new(frame: Axis3, angle: f64, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 || angle_out_of_range(angle) {
            return Err(errors::construction(&format!(
                "Geom_ConicalSurface: Ang = {angle}, Radius = {radius}; expected Radius >= 0 and gp::Resolution() < |Ang| < PI/2 - gp::Resolution()"
            )));
        }
        let state = ConeState {
            pos: frame,
            radius,
            semi_angle: angle,
        };
        Ok(Self::stored(state))
    }
    /// Cone from a gp cone (`Geom_ConicalSurface(gp_Cone)`); `GeomConicalSurface::from_cone(c)`.
    pub fn from_cone(cone: Cone3) -> Self {
        Self::stored(ConeState {
            pos: cone.position(),
            radius: cone.ref_radius(),
            semi_angle: cone.semi_angle(),
        })
    }
    /// Replace by a gp cone (`SetCone`); `surface.set_cone(c)`.
    pub fn set_cone(&self, cone: Cone3) {
        let mut state = self.state.write();
        state.radius = cone.ref_radius();
        state.semi_angle = cone.semi_angle();
        state.pos = cone.position();
    }
    /// Set the reference radius; Standard_ConstructionError when negative (`SetRadius`);
    /// `surface.set_radius(3.0)?`.
    pub fn set_radius(&self, radius: f64) -> Result<(), Failure> {
        self.state.write().radius = checked_radius(radius)?;
        Ok(())
    }
    /// Set the half-angle; Standard_ConstructionError out of
    /// `(gp::Resolution(), PI/2 - gp::Resolution())` in absolute value (`SetSemiAngle`);
    /// `surface.set_semi_angle(0.5)?`.
    pub fn set_semi_angle(&self, angle: f64) -> Result<(), Failure> {
        if angle_out_of_range(angle) {
            return Err(errors::construction(&format!(
                "Geom_ConicalSurface::SetSemiAngle: Ang = {angle}; expected gp::Resolution() < |Ang| < PI/2 - gp::Resolution()"
            )));
        }
        self.state.write().semi_angle = angle;
        Ok(())
    }
    /// The gp cone (`Cone`); `surface.cone()?`.
    pub fn cone(&self) -> Result<Cone3, Failure> {
        let state: ConeState = self.snapshot();
        Cone3::new(state.pos, state.semi_angle, state.radius)
    }
    /// The apex, `Location - Direction * RefRadius / tan(SemiAngle)` (`Apex`);
    /// `surface.apex()`.
    pub fn apex(&self) -> Point3 {
        let state: ConeState = self.snapshot();
        let mut coord: Xyz = state.pos.direction().xyz();
        coord *= -state.radius / state.semi_angle.tan();
        coord += state.pos.location().xyz();
        Point3::from(coord)
    }
    /// The implicit equation coefficients `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]`
    /// (`Coefficients`); `surface.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let state: ConeState = self.snapshot();
        cone_coefficients(state.pos, state.radius, state.semi_angle)
    }
    /// The reference radius (`RefRadius`); `surface.ref_radius()`.
    pub fn ref_radius(&self) -> f64 {
        self.state.read().radius
    }
    /// The half-angle (`SemiAngle`); `surface.semi_angle()`.
    pub fn semi_angle(&self) -> f64 {
        self.state.read().semi_angle
    }
    fn snapshot(&self) -> ConeState {
        *self.state.read()
    }
}
impl GeomGeometry for GeomConicalSurface {
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
        let state: ConeState = self.snapshot();
        dump_class_name(stream, "Geom_ConicalSurface")?;
        dump_base(stream, depth, |s, d| {
            dump_elementary_surface(s, d, state.pos)
        })?;
        dump::dump_real(stream, "radius", state.radius)?;
        dump::dump_real(stream, "semiAngle", state.semi_angle)
    }
}
impl GeomSurface for GeomConicalSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        elementary_u_reverse(&mut self.state.write().pos);
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> f64 {
        2. * PI - u
    }
    /// Negates the half-angle and reverses the main direction.
    fn v_reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.semi_angle = -state.semi_angle;
        state.pos.z_reverse();
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> f64 {
        -v
    }
    fn transform_parameters(&self, u: f64, v: f64, transform: Transform3) -> (f64, f64) {
        (u, scaled_parameter(v, transform))
    }
    fn parametric_transformation(
        &self,
        transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        Ok(v_affinity(transform.scale_factor().abs()))
    }
    fn bounds(&self) -> SurfaceBounds {
        (0.0, 2.0 * PI, -precision::infinite(), precision::infinite())
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
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let s: ConeState = self.snapshot();
        let line = elslib::cone_u_iso(s.pos, s.radius, s.semi_angle, u);
        Ok(Handle::new(GeomLine::from_lin(line)).cast())
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let s: ConeState = self.snapshot();
        let circle = elslib::cone_v_iso(s.pos, s.radius, s.semi_angle, v);
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
        let s: ConeState = self.snapshot();
        Ok(elslib::cone_value(u, v, s.pos, s.radius, s.semi_angle))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let s: ConeState = self.snapshot();
        let (point, d1u, d1v) = elslib::cone_d1(u, v, s.pos, s.radius, s.semi_angle);
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let s: ConeState = self.snapshot();
        Ok(surface_d2(elslib::cone_d2(
            u,
            v,
            s.pos,
            s.radius,
            s.semi_angle,
        )))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let s: ConeState = self.snapshot();
        Ok(surface_d3(elslib::cone_d3(
            u,
            v,
            s.pos,
            s.radius,
            s.semi_angle,
        )))
    }
    /// Orders above 1 in V are zero; the others come from ElSLib::ConeDN.
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_ConicalSurface", nu, nv)?;
        if nv > 1 {
            return Ok(Vector3::new(0.0, 0.0, 0.0));
        }
        let s: ConeState = self.snapshot();
        Ok(elslib::cone_dn(u, v, s.pos, s.radius, s.semi_angle, nu, nv))
    }
}
impl GeomElementarySurface for GeomConicalSurface {
    impl_elementary_position!();
}
