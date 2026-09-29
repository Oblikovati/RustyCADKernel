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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Ellipse.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Ellipse`: an ellipse parameterized by angle from its major axis.
use super::conic::{Geom2dConic, dump_conic, impl_conic_position, reverse_conic_position};
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2d, Axis22d, Ellipse2, Point2, Transform2, Vector2};
use std::io;

#[derive(Clone, Copy)]
struct EllipseState {
    pos: Axis22d,
    major_radius: f64,
    minor_radius: f64,
}

/// An ellipse; `Handle::new(Geom2dEllipse::from_axis(axis, 3.0, 2.0, true)?)`.
pub struct Geom2dEllipse {
    parent: StandardTransient,
    state: SharedState<EllipseState>,
}
impl_transient!(
    Geom2dEllipse,
    "Geom2d_Ellipse",
    dyn Geom2dConic,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dConic]
);
impl Clone for Geom2dEllipse {
    fn clone(&self) -> Self {
        Self::stored(self.state.snapshot())
    }
}

fn check_radii(major_radius: f64, minor_radius: f64) -> Result<(), Failure> {
    if major_radius < minor_radius || minor_radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom2d_Ellipse: MajorRadius = {major_radius}, MinorRadius = {minor_radius}; \
             expected MajorRadius >= MinorRadius >= 0"
        )));
    }
    Ok(())
}

impl Geom2dEllipse {
    fn stored(state: EllipseState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Ellipse from a gp ellipse (`Geom2d_Ellipse(gp_Elips2d)`); `Geom2dEllipse::from_elips2d(e)`.
    pub fn from_elips2d(ellipse: Ellipse2) -> Self {
        Self::stored(EllipseState {
            pos: ellipse.axis(),
            major_radius: ellipse.major_radius(),
            minor_radius: ellipse.minor_radius(),
        })
    }
    /// Ellipse on its major axis with a sense (`Geom2d_Ellipse(gp_Ax2d, Major, Minor, Sense)`);
    /// `Geom2dEllipse::from_axis(axis, 3.0, 2.0, true)?`.
    pub fn from_axis(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        check_radii(major_radius, minor_radius)?;
        let pos = Axis22d::from_axis(major_axis, sense)?;
        Ok(Self::stored(EllipseState {
            pos,
            major_radius,
            minor_radius,
        }))
    }
    /// Ellipse in a coordinate system (`Geom2d_Ellipse(gp_Ax22d, Major, Minor)`);
    /// `Geom2dEllipse::new(frame, 3.0, 2.0)?`.
    pub fn new(axis: Axis22d, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        check_radii(major_radius, minor_radius)?;
        Ok(Self::stored(EllipseState {
            pos: axis,
            major_radius,
            minor_radius,
        }))
    }
    /// Replace by a gp ellipse (`SetElips2d`); `ellipse.set_elips2d(e)`.
    pub fn set_elips2d(&self, ellipse: Ellipse2) {
        *self.state.write() = EllipseState {
            pos: ellipse.axis(),
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
                "Geom2d_Ellipse::SetMajorRadius: MajorRadius = {major_radius}; \
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
                "Geom2d_Ellipse::SetMinorRadius: MinorRadius = {minor_radius}; \
                 expected in [0, MajorRadius = {}]",
                state.major_radius
            )));
        }
        state.minor_radius = minor_radius;
        Ok(())
    }
    /// The gp ellipse (`Elips2d`); `ellipse.elips2d()?`.
    pub fn elips2d(&self) -> Result<Ellipse2, Failure> {
        let state: EllipseState = *self.state.read();
        Ellipse2::new(state.pos, state.major_radius, state.minor_radius)
    }
    /// First directrix (`Directrix1`); `ellipse.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis2d, Failure> {
        self.elips2d()?.directrix1()
    }
    /// Second directrix (`Directrix2`); `ellipse.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis2d, Failure> {
        self.elips2d()?.directrix2()
    }
    /// Distance between the foci (`Focal`); `ellipse.focal()`.
    pub fn focal(&self) -> f64 {
        let state: EllipseState = *self.state.read();
        let (a, b) = (state.major_radius, state.minor_radius);
        2.0 * (a * a - b * b).sqrt()
    }
    /// Focus on the positive X side (`Focus1`); `ellipse.focus1()`.
    pub fn focus1(&self) -> Point2 {
        self.focus(1.0)
    }
    /// Focus on the negative X side (`Focus2`); `ellipse.focus2()`.
    pub fn focus2(&self) -> Point2 {
        self.focus(-1.0)
    }
    // Focus1 adds and Focus2 subtracts C * XDirection, with the native operand order.
    fn focus(&self, side: f64) -> Point2 {
        let state: EllipseState = *self.state.read();
        let (a, b) = (state.major_radius, state.minor_radius);
        let c: f64 = (a * a - b * b).sqrt();
        let (location, x) = (state.pos.location(), state.pos.x_direction());
        if side > 0.0 {
            return Point2::new(location.x() + c * x.x(), location.y() + c * x.y());
        }
        Point2::new(location.x() - c * x.x(), location.y() - c * x.y())
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
impl Geom2dGeometry for Geom2dEllipse {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.major_radius *= transform.scale_factor().abs();
        state.minor_radius *= transform.scale_factor().abs();
        state.pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: EllipseState = *self.state.read();
        dump_class_name(stream, "Geom2d_Ellipse")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "majorRadius", state.major_radius)?;
        dump::dump_real(stream, "minorRadius", state.minor_radius)
    }
}
impl Geom2dCurve for Geom2dEllipse {
    fn reverse(&self) -> Result<(), Failure> {
        reverse_conic_position(&mut self.state.write().pos)
    }
    crate::full_turn_curve_members!();
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        let s: EllipseState = *self.state.read();
        Ok(elclib::ellipse_value_2d(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
        ))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let s: EllipseState = *self.state.read();
        let (point, d1) = elclib::ellipse_d1_2d(u, s.pos, s.major_radius, s.minor_radius);
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let s: EllipseState = *self.state.read();
        let (point, d1, d2) = elclib::ellipse_d2_2d(u, s.pos, s.major_radius, s.minor_radius);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let s: EllipseState = *self.state.read();
        let (point, d1, d2, d3) = elclib::ellipse_d3_2d(u, s.pos, s.major_radius, s.minor_radius);
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom2d_Ellipse::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let s: EllipseState = *self.state.read();
        Ok(elclib::ellipse_dn_2d(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
            n,
        ))
    }
}
impl Geom2dConic for Geom2dEllipse {
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
