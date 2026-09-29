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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Line.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Line`: an infinite line parameterized by arc length from its origin.
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, dump_curve};
use super::errors;
use super::geometry::GeomGeometry;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Point3, Transform3, Vector3};
use std::io;

/// A line through a point along a direction; `Handle::new(GeomLine::new(axis))`.
pub struct GeomLine {
    parent: StandardTransient,
    state: SharedState<Axis1>,
}
impl_transient!(
    GeomLine,
    "Geom_Line",
    dyn GeomCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve]
);
impl Clone for GeomLine {
    fn clone(&self) -> Self {
        Self::new(self.position())
    }
}
impl GeomLine {
    /// Line on an axis (`Geom_Line(gp_Ax1)`); `GeomLine::new(axis)`.
    pub fn new(axis: Axis1) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(axis),
        }
    }
    /// Line from a gp line (`Geom_Line(gp_Lin)`); `GeomLine::from_lin(line)`.
    pub fn from_lin(line: Line3) -> Self {
        Self::new(line.position())
    }
    /// Line through a point along a direction (`Geom_Line(P, V)`); `GeomLine::from_point(p, d)`.
    pub fn from_point(point: Point3, direction: Direction3) -> Self {
        Self::new(Axis1::new(point, direction))
    }
    /// Replace by a gp line (`SetLin`); `line.set_lin(l)`.
    pub fn set_lin(&self, line: Line3) {
        *self.state.write() = line.position();
    }
    /// Replace the direction (`SetDirection`); `line.set_direction(d)`.
    pub fn set_direction(&self, direction: Direction3) {
        self.state.write().set_direction(direction);
    }
    /// Replace the origin (`SetLocation`); `line.set_location(p)`.
    pub fn set_location(&self, point: Point3) {
        self.state.write().set_location(point);
    }
    /// Replace the axis (`SetPosition`); `line.set_position(axis)`.
    pub fn set_position(&self, axis: Axis1) {
        *self.state.write() = axis;
    }
    /// The gp line (`Lin`); `line.lin()`.
    pub fn lin(&self) -> Line3 {
        Line3::from_axis(self.position())
    }
    /// The axis (`Position`); `line.position()`.
    pub fn position(&self) -> Axis1 {
        *self.state.read()
    }
}
impl GeomGeometry for GeomLine {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let position: Axis1 = self.position();
        dump_class_name(stream, "Geom_Line")?;
        dump_base(stream, depth, dump_curve)?;
        dump_field(stream, depth, "pos", |nested, _| position.dump_json(nested))
    }
}
impl GeomCurve for GeomLine {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse();
        Ok(())
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(-u)
    }
    rustycad_tkg2d::scaled_parameter_members!(Transform3);
    rustycad_tkg2d::open_curve_range!(-precision::infinite(), precision::infinite());
    rustycad_tkg2d::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        Ok(elclib::line_value(u, self.position()))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let (point, d1) = elclib::line_d1(u, self.position());
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let (point, d1) = elclib::line_d1(u, self.position());
        let d2 = Vector3::new(0.0, 0.0, 0.0);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let (point, d1) = elclib::line_d1(u, self.position());
        let zero = Vector3::new(0.0, 0.0, 0.0);
        Ok(GeomCurveD3 {
            point,
            d1,
            d2: zero,
            d3: zero,
        })
    }
    fn eval_dn(&self, _u: f64, n: i32) -> Result<Vector3, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom_Line::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        if n == 1 {
            return Ok(Vector3::from(self.position().direction()));
        }
        Ok(Vector3::new(0.0, 0.0, 0.0))
    }
}
