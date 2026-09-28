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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Ellipse.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Ellipse`: an ellipse parameterized by angle from its major axis.
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
use rustycad_tkmath::gp::{Axis1, Axis2, Ellipse3, Point3, Transform3, Vector3};
use std::f64::consts::PI;
use std::io;

#[derive(Clone, Copy)]
struct EllipseState {
    pos: Axis2,
    major_radius: f64,
    minor_radius: f64,
}

/// An ellipse; `Handle::new(GeomEllipse::new(frame, 3.0, 2.0)?)`.
pub struct GeomEllipse {
    parent: StandardTransient,
    state: GeometryCell<EllipseState>,
}
impl_transient!(
    GeomEllipse,
    "Geom_Ellipse",
    dyn GeomConic,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomConic]
);
impl Clone for GeomEllipse {
    fn clone(&self) -> Self {
        Self::stored(self.state.snapshot())
    }
}

impl GeomEllipse {
    fn stored(state: EllipseState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(state),
        }
    }
    /// Ellipse from a gp ellipse (`Geom_Ellipse(gp_Elips)`); `GeomEllipse::from_elips(e)`.
    pub fn from_elips(ellipse: Ellipse3) -> Self {
        Self::stored(EllipseState {
            pos: ellipse.position(),
            major_radius: ellipse.major_radius(),
            minor_radius: ellipse.minor_radius(),
        })
    }
    /// Ellipse in a coordinate system (`Geom_Ellipse(gp_Ax2, MajorRadius, MinorRadius)`);
    /// Standard_ConstructionError unless `major >= minor >= 0`;
    /// `GeomEllipse::new(frame, 3.0, 2.0)?`.
    pub fn new(frame: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if major_radius < minor_radius || minor_radius < 0.0 {
            return Err(errors::construction(&format!(
                "Geom_Ellipse: MajorRadius = {major_radius}, MinorRadius = {minor_radius}; \
                 expected MajorRadius >= MinorRadius >= 0"
            )));
        }
        Ok(Self::stored(EllipseState {
            pos: frame,
            major_radius,
            minor_radius,
        }))
    }
    /// Replace by a gp ellipse (`SetElips`); `ellipse.set_elips(e)`.
    pub fn set_elips(&self, ellipse: Ellipse3) {
        *self.state.write() = EllipseState {
            pos: ellipse.position(),
            major_radius: ellipse.major_radius(),
            minor_radius: ellipse.minor_radius(),
        };
    }
    /// Set the major radius; must stay >= the minor radius (`SetMajorRadius`);
    /// `ellipse.set_major_radius(4.0)?`.
    pub fn set_major_radius(&self, major_radius: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        if major_radius < state.minor_radius {
            return Err(errors::construction(&format!(
                "Geom_Ellipse::SetMajorRadius: MajorRadius = {major_radius}; \
                 expected >= MinorRadius = {}",
                state.minor_radius
            )));
        }
        state.major_radius = major_radius;
        Ok(())
    }
    /// Set the minor radius; must stay in [0, major] (`SetMinorRadius`);
    /// `ellipse.set_minor_radius(1.0)?`.
    pub fn set_minor_radius(&self, minor_radius: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        if minor_radius < 0.0 || state.major_radius < minor_radius {
            return Err(errors::construction(&format!(
                "Geom_Ellipse::SetMinorRadius: MinorRadius = {minor_radius}; \
                 expected in [0, MajorRadius = {}]",
                state.major_radius
            )));
        }
        state.minor_radius = minor_radius;
        Ok(())
    }
    /// The gp ellipse (`Elips`); `ellipse.elips()?`.
    pub fn elips(&self) -> Result<Ellipse3, Failure> {
        let state: EllipseState = *self.state.read();
        Ellipse3::new(state.pos, state.major_radius, state.minor_radius)
    }
    /// First directrix, through the gp ellipse (`Directrix1`); `ellipse.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis1, Failure> {
        self.elips()?.directrix1()
    }
    /// Second directrix, through the gp ellipse (`Directrix2`); `ellipse.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis1, Failure> {
        self.elips()?.directrix2()
    }
    /// Distance between the foci (`Focal`); `ellipse.focal()`.
    pub fn focal(&self) -> f64 {
        let state: EllipseState = *self.state.read();
        let (a, b) = (state.major_radius, state.minor_radius);
        2.0 * (a * a - b * b).sqrt()
    }
    /// Focus on the positive X side (`Focus1`); `ellipse.focus1()`.
    pub fn focus1(&self) -> Point3 {
        self.focus(true)
    }
    /// Focus on the negative X side (`Focus2`); `ellipse.focus2()`.
    pub fn focus2(&self) -> Point3 {
        self.focus(false)
    }
    // Focus1 adds and Focus2 subtracts C * XDirection, coordinate by coordinate.
    fn focus(&self, positive: bool) -> Point3 {
        let state: EllipseState = *self.state.read();
        let (a, b) = (state.major_radius, state.minor_radius);
        let c: f64 = (a * a - b * b).sqrt();
        focus_point(state.pos, c, positive)
    }
    /// The major radius (`MajorRadius`); `ellipse.major_radius()`.
    pub fn major_radius(&self) -> f64 {
        self.state.read().major_radius
    }
    /// The minor radius (`MinorRadius`); `ellipse.minor_radius()`.
    pub fn minor_radius(&self) -> f64 {
        self.state.read().minor_radius
    }
    /// Semi-latus rectum `b²/a`, 0 for a null major radius (`Parameter`); `ellipse.parameter()`.
    pub fn parameter(&self) -> f64 {
        let state: EllipseState = *self.state.read();
        if state.major_radius == 0.0 {
            return 0.0;
        }
        (state.minor_radius * state.minor_radius) / state.major_radius
    }
}

/// `Location ± C * XDirection` computed per coordinate, as the Focus members of
/// Geom_Ellipse and Geom_Hyperbola do; `focus_point(frame, 2.0, true)`.
pub(crate) fn focus_point(frame: Axis2, c: f64, positive: bool) -> Point3 {
    let (location, x) = (frame.location(), frame.x_direction());
    if positive {
        return Point3::new(
            location.x() + c * x.x(),
            location.y() + c * x.y(),
            location.z() + c * x.z(),
        );
    }
    Point3::new(
        location.x() - c * x.x(),
        location.y() - c * x.y(),
        location.z() - c * x.z(),
    )
}

impl GeomGeometry for GeomEllipse {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.major_radius *= transform.scale_factor().abs();
        state.minor_radius *= transform.scale_factor().abs();
        state.pos.transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: EllipseState = *self.state.read();
        dump_class_name(stream, "Geom_Ellipse")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "majorRadius", state.major_radius)?;
        dump::dump_real(stream, "minorRadius", state.minor_radius)
    }
}
impl GeomCurve for GeomEllipse {
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
        let s: EllipseState = *self.state.read();
        Ok(elclib::ellipse_value(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
        ))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let s: EllipseState = *self.state.read();
        let (point, d1) = elclib::ellipse_d1(u, s.pos, s.major_radius, s.minor_radius);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let s: EllipseState = *self.state.read();
        let (point, d1, d2) = elclib::ellipse_d2(u, s.pos, s.major_radius, s.minor_radius);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let s: EllipseState = *self.state.read();
        let (point, d1, d2, d3) = elclib::ellipse_d3(u, s.pos, s.major_radius, s.minor_radius);
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom_Ellipse::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let s: EllipseState = *self.state.read();
        Ok(elclib::ellipse_dn(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
            n,
        ))
    }
}
impl GeomConic for GeomEllipse {
    impl_conic_position!();
    fn eccentricity(&self) -> Result<f64, Failure> {
        let state: EllipseState = *self.state.read();
        let (a, b) = (state.major_radius, state.minor_radius);
        if a == 0.0 {
            return Ok(0.0);
        }
        Ok((a * a - b * b).sqrt() / a)
    }
}
