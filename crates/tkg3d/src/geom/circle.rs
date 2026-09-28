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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Circle.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Circle`: a circle parameterized by angle from its X axis.
use super::conic::{GeomConic, dump_conic, impl_conic_position, reverse_conic_position};
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use super::errors;
use super::geometry::GeomGeometry;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Circle3, Point3, Transform3, Vector3};
use std::f64::consts::PI;
use std::io;

#[derive(Clone, Copy)]
struct CircleState {
    pos: Axis2,
    radius: f64,
}

/// A circle; `Handle::new(GeomCircle::new(frame, 2.0)?)`.
pub struct GeomCircle {
    parent: StandardTransient,
    state: GeometryCell<CircleState>,
}
impl_transient!(
    GeomCircle,
    "Geom_Circle",
    dyn GeomConic,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomConic]
);
impl Clone for GeomCircle {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

fn checked_radius(radius: f64) -> Result<f64, Failure> {
    if radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom_Circle: Radius = {radius}; expected Radius >= 0"
        )));
    }
    Ok(radius)
}

impl GeomCircle {
    fn stored(state: CircleState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(state),
        }
    }
    /// Circle from a gp circle (`Geom_Circle(gp_Circ)`); `GeomCircle::from_circ(c)`.
    pub fn from_circ(circle: Circle3) -> Self {
        Self::stored(CircleState {
            pos: circle.position(),
            radius: circle.radius(),
        })
    }
    /// Circle in a coordinate system (`Geom_Circle(gp_Ax2, Radius)`);
    /// Standard_ConstructionError when `radius < 0`; `GeomCircle::new(frame, 2.0)?`.
    pub fn new(frame: Axis2, radius: f64) -> Result<Self, Failure> {
        let radius: f64 = checked_radius(radius)?;
        Ok(Self::stored(CircleState { pos: frame, radius }))
    }
    /// Replace by a gp circle (`SetCirc`); `circle.set_circ(c)`.
    pub fn set_circ(&self, circle: Circle3) {
        let mut state = self.state.write();
        state.radius = circle.radius();
        state.pos = circle.position();
    }
    /// Set the radius; Standard_ConstructionError when negative (`SetRadius`);
    /// `circle.set_radius(5.0)?`.
    pub fn set_radius(&self, radius: f64) -> Result<(), Failure> {
        self.state.write().radius = checked_radius(radius)?;
        Ok(())
    }
    /// The gp circle (`Circ`); `circle.circ()?`.
    pub fn circ(&self) -> Result<Circle3, Failure> {
        let state: CircleState = *self.state.read();
        Circle3::new(state.pos, state.radius)
    }
    /// The radius (`Radius`); `circle.radius()`.
    pub fn radius(&self) -> f64 {
        self.state.read().radius
    }
}
impl GeomGeometry for GeomCircle {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.radius *= transform.scale_factor().abs();
        state.pos.transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: CircleState = *self.state.read();
        dump_class_name(stream, "Geom_Circle")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "radius", state.radius)
    }
}
impl GeomCurve for GeomCircle {
    fn reverse(&self) -> Result<(), Failure> {
        reverse_conic_position(&mut self.state.write().pos)
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(2. * PI - u)
    }
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        2.0 * PI
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_periodic(&self) -> bool {
        true
    }
    fn continuity(&self) -> Shape {
        Shape::CN
    }
    fn is_cn(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let state: CircleState = *self.state.read();
        Ok(elclib::circle_value(u, state.pos, state.radius))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let state: CircleState = *self.state.read();
        let (point, d1) = elclib::circle_d1(u, state.pos, state.radius);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let state: CircleState = *self.state.read();
        let (point, d1, d2) = elclib::circle_d2(u, state.pos, state.radius);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let state: CircleState = *self.state.read();
        let (point, d1, d2, d3) = elclib::circle_d3(u, state.pos, state.radius);
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom_Circle::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let state: CircleState = *self.state.read();
        Ok(elclib::circle_dn(u, state.pos, state.radius, n))
    }
}
impl GeomConic for GeomCircle {
    impl_conic_position!();
    fn eccentricity(&self) -> Result<f64, Failure> {
        Ok(0.0)
    }
}
