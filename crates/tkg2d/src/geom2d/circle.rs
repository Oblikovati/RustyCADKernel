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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Circle.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Circle`: a circle parameterized by angle from its X axis.
use super::cell::GeometryCell;
use super::conic::{Geom2dConic, dump_conic, impl_conic_position, reverse_conic_position};
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Axis22d, Circle2, Point2, Transform2, Vector2};
use std::f64::consts::PI;
use std::io;

#[derive(Clone, Copy)]
struct CircleState {
    pos: Axis22d,
    radius: f64,
}

/// A circle; `Handle::new(Geom2dCircle::from_axis(axis, 2.0, true)?)`.
pub struct Geom2dCircle {
    parent: StandardTransient,
    state: GeometryCell<CircleState>,
}
impl_transient!(
    Geom2dCircle,
    "Geom2d_Circle",
    dyn Geom2dConic,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dConic]
);
impl Clone for Geom2dCircle {
    fn clone(&self) -> Self {
        Self::stored(self.state.snapshot())
    }
}

fn checked_radius(radius: f64) -> Result<f64, Failure> {
    if radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom2d_Circle: Radius = {radius}; expected Radius >= 0"
        )));
    }
    Ok(radius)
}

impl Geom2dCircle {
    fn stored(state: CircleState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(state),
        }
    }
    /// Circle from a gp circle (`Geom2d_Circle(gp_Circ2d)`); `Geom2dCircle::from_circ2d(c)`.
    pub fn from_circ2d(circle: Circle2) -> Self {
        Self::stored(CircleState {
            pos: circle.axis(),
            radius: circle.radius(),
        })
    }
    /// Circle on an X axis with a sense (`Geom2d_Circle(gp_Ax2d, Radius, Sense)`);
    /// Standard_ConstructionError when `radius < 0`; `Geom2dCircle::from_axis(axis, 2.0, true)?`.
    pub fn from_axis(axis: Axis2d, radius: f64, sense: bool) -> Result<Self, Failure> {
        let radius: f64 = checked_radius(radius)?;
        let pos = Axis22d::from_axis(axis, sense)?;
        Ok(Self::stored(CircleState { pos, radius }))
    }
    /// Circle in a coordinate system (`Geom2d_Circle(gp_Ax22d, Radius)`);
    /// `Geom2dCircle::new(frame, 2.0)?`.
    pub fn new(axis: Axis22d, radius: f64) -> Result<Self, Failure> {
        let radius: f64 = checked_radius(radius)?;
        Ok(Self::stored(CircleState { pos: axis, radius }))
    }
    /// Replace by a gp circle (`SetCirc2d`); `circle.set_circ2d(c)`.
    pub fn set_circ2d(&self, circle: Circle2) {
        let mut state = self.state.write();
        state.radius = circle.radius();
        state.pos = circle.axis();
    }
    /// Set the radius; Standard_ConstructionError when negative (`SetRadius`);
    /// `circle.set_radius(5.0)?`.
    pub fn set_radius(&self, radius: f64) -> Result<(), Failure> {
        self.state.write().radius = checked_radius(radius)?;
        Ok(())
    }
    /// The gp circle (`Circ2d`); `circle.circ2d()?`.
    pub fn circ2d(&self) -> Result<Circle2, Failure> {
        let state: CircleState = *self.state.read();
        Circle2::new(state.pos, state.radius)
    }
    /// The radius (`Radius`); `circle.radius()`.
    pub fn radius(&self) -> f64 {
        self.state.read().radius
    }
}
impl Geom2dGeometry for Geom2dCircle {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.radius *= transform.scale_factor().abs();
        state.pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: CircleState = *self.state.read();
        dump_class_name(stream, "Geom2d_Circle")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "radius", state.radius)
    }
}
impl Geom2dCurve for Geom2dCircle {
    fn reverse(&self) -> Result<(), Failure> {
        reverse_conic_position(&mut self.state.write().pos)
    }
    fn reversed_parameter(&self, u: f64) -> f64 {
        2. * PI - u
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
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        let state: CircleState = *self.state.read();
        Ok(elclib::circle_value_2d(u, state.pos, state.radius))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let state: CircleState = *self.state.read();
        let (point, d1) = elclib::circle_d1_2d(u, state.pos, state.radius);
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let state: CircleState = *self.state.read();
        let (point, d1, d2) = elclib::circle_d2_2d(u, state.pos, state.radius);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let state: CircleState = *self.state.read();
        let (point, d1, d2, d3) = elclib::circle_d3_2d(u, state.pos, state.radius);
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom2d_Circle::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let state: CircleState = *self.state.read();
        Ok(elclib::circle_dn_2d(u, state.pos, state.radius, n))
    }
}
impl Geom2dConic for Geom2dCircle {
    impl_conic_position!();
    fn eccentricity(&self) -> Result<f64, Failure> {
        Ok(0.0)
    }
}
