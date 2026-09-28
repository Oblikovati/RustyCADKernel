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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Line.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Line`: an infinite line parameterized by arc length from its origin.
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, dump_curve};
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name, dump_field};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Point2, Transform2, Vector2};
use std::io;

/// A line through a point along a direction; `Handle::new(Geom2dLine::new(axis))`.
pub struct Geom2dLine {
    parent: StandardTransient,
    state: SharedState<Axis2d>,
}
impl_transient!(
    Geom2dLine,
    "Geom2d_Line",
    dyn Geom2dCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve]
);
impl Clone for Geom2dLine {
    fn clone(&self) -> Self {
        Self::new(self.position())
    }
}
impl Geom2dLine {
    /// Line on an axis (`Geom2d_Line(gp_Ax2d)`); `Geom2dLine::new(axis)`.
    pub fn new(axis: Axis2d) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(axis),
        }
    }
    /// Line from a gp line (`Geom2d_Line(gp_Lin2d)`); `Geom2dLine::from_lin2d(line)`.
    pub fn from_lin2d(line: Line2) -> Self {
        Self::new(line.position())
    }
    /// Line through a point along a direction (`Geom2d_Line(P, V)`);
    /// `Geom2dLine::from_point(p, d)`.
    pub fn from_point(point: Point2, direction: Direction2) -> Self {
        Self::new(Axis2d::new(point, direction))
    }
    /// Replace by a gp line (`SetLin2d`); `line.set_lin2d(l)`.
    pub fn set_lin2d(&self, line: Line2) {
        *self.state.write() = line.position();
    }
    /// Replace the direction (`SetDirection`); `line.set_direction(d)`.
    pub fn set_direction(&self, direction: Direction2) {
        self.state.write().set_direction(direction);
    }
    /// The direction (`Direction`); `line.direction()`.
    pub fn direction(&self) -> Direction2 {
        self.state.read().direction()
    }
    /// Replace the origin (`SetLocation`); `line.set_location(p)`.
    pub fn set_location(&self, point: Point2) {
        self.state.write().set_location(point);
    }
    /// The origin (`Location`); `line.location()`.
    pub fn location(&self) -> Point2 {
        self.state.read().location()
    }
    /// Replace the axis (`SetPosition`); `line.set_position(axis)`.
    pub fn set_position(&self, axis: Axis2d) {
        *self.state.write() = axis;
    }
    /// The axis (`Position`); `line.position()`.
    pub fn position(&self) -> Axis2d {
        *self.state.read()
    }
    /// The gp line (`Lin2d`); `line.lin2d()`.
    pub fn lin2d(&self) -> Line2 {
        Line2::from(self.position())
    }
    /// Distance from `point` to the line (`Distance`); `line.distance(p)`.
    pub fn distance(&self, point: Point2) -> f64 {
        self.lin2d().distance(point)
    }
}
impl Geom2dGeometry for Geom2dLine {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let position: Axis2d = self.position();
        dump_class_name(stream, "Geom2d_Line")?;
        dump_base(stream, depth, dump_curve)?;
        dump_field(stream, depth, "pos", |nested, level| {
            position.dump_json(nested, level)
        })
    }
}
impl Geom2dCurve for Geom2dLine {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse();
        Ok(())
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(-u)
    }
    fn transformed_parameter(&self, u: f64, transform: Transform2) -> f64 {
        if precision::is_infinite(u) {
            return u;
        }
        u * transform.scale_factor().abs()
    }
    fn parametric_transformation(&self, transform: Transform2) -> f64 {
        transform.scale_factor().abs()
    }
    fn first_parameter(&self) -> f64 {
        -precision::infinite()
    }
    fn last_parameter(&self) -> f64 {
        precision::infinite()
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_periodic(&self) -> bool {
        false
    }
    fn continuity(&self) -> Shape {
        Shape::CN
    }
    fn is_cn(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        Ok(elclib::line_value_2d(u, self.position()))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let (point, d1) = elclib::line_d1_2d(u, self.position());
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let (point, d1) = elclib::line_d1_2d(u, self.position());
        let d2 = Vector2::new(0.0, 0.0);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let (point, d1) = elclib::line_d1_2d(u, self.position());
        let zero = Vector2::new(0.0, 0.0);
        Ok(Geom2dCurveD3 {
            point,
            d1,
            d2: zero,
            d3: zero,
        })
    }
    fn eval_dn(&self, _u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom2d_Line::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        if n == 1 {
            return Ok(Vector2::from(self.direction()));
        }
        Ok(Vector2::new(0.0, 0.0))
    }
}
