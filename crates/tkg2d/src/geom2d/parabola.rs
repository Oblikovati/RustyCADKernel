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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Parabola.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Parabola`: a parabola parameterized by its ordinate along the Y axis.
use super::conic::{Geom2dConic, dump_conic, impl_conic_position, reverse_conic_position};
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2d, Axis22d, Parabola2, Point2, Transform2, Vector2, Xy};
use std::io;

#[derive(Clone, Copy)]
struct ParabolaState {
    pos: Axis22d,
    focal_length: f64,
}

/// A parabola; `Handle::new(Geom2dParabola::from_axis(axis, 1.0, true)?)`.
pub struct Geom2dParabola {
    parent: StandardTransient,
    state: SharedState<ParabolaState>,
}
impl_transient!(
    Geom2dParabola,
    "Geom2d_Parabola",
    dyn Geom2dConic,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dConic]
);
impl Clone for Geom2dParabola {
    fn clone(&self) -> Self {
        Self::stored(self.state.snapshot())
    }
}

fn checked_focal(focal: f64) -> Result<f64, Failure> {
    if focal < 0.0 {
        return Err(errors::construction(&format!(
            "Geom2d_Parabola: Focal = {focal}; expected Focal >= 0"
        )));
    }
    Ok(focal)
}

impl Geom2dParabola {
    fn stored(state: ParabolaState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Parabola from a gp one (`Geom2d_Parabola(gp_Parab2d)`); `Geom2dParabola::from_parab2d(p)`.
    pub fn from_parab2d(parabola: Parabola2) -> Self {
        Self::stored(ParabolaState {
            pos: parabola.axis(),
            focal_length: parabola.focal(),
        })
    }
    /// Parabola on its symmetry axis with a sense (`Geom2d_Parabola(gp_Ax2d, Focal, Sense)`);
    /// `Geom2dParabola::from_axis(axis, 1.0, true)?`.
    pub fn from_axis(mirror_axis: Axis2d, focal: f64, sense: bool) -> Result<Self, Failure> {
        let focal_length: f64 = checked_focal(focal)?;
        let pos = Axis22d::from_axis(mirror_axis, sense)?;
        Ok(Self::stored(ParabolaState { pos, focal_length }))
    }
    /// Parabola in a coordinate system (`Geom2d_Parabola(gp_Ax22d, Focal)`);
    /// `Geom2dParabola::new(frame, 1.0)?`.
    pub fn new(axis: Axis22d, focal: f64) -> Result<Self, Failure> {
        let focal_length: f64 = checked_focal(focal)?;
        Ok(Self::stored(ParabolaState {
            pos: axis,
            focal_length,
        }))
    }
    /// Parabola from its directrix and focus (`Geom2d_Parabola(gp_Ax2d D, gp_Pnt2d F)`, the
    /// gp_Parab2d default sense); `Geom2dParabola::from_directrix(d, f)?`.
    pub fn from_directrix(directrix: Axis2d, focus: Point2) -> Result<Self, Failure> {
        let parabola: Parabola2 = Parabola2::from_directrix(directrix, focus, true)?;
        Ok(Self::from_parab2d(parabola))
    }
    /// Set the focal length; Standard_ConstructionError when negative (`SetFocal`);
    /// `parabola.set_focal(2.0)?`.
    pub fn set_focal(&self, focal: f64) -> Result<(), Failure> {
        self.state.write().focal_length = checked_focal(focal)?;
        Ok(())
    }
    /// Replace by a gp parabola (`SetParab2d`); `parabola.set_parab2d(p)`.
    pub fn set_parab2d(&self, parabola: Parabola2) {
        *self.state.write() = ParabolaState {
            pos: parabola.axis(),
            focal_length: parabola.focal(),
        };
    }
    /// The gp parabola (`Parab2d`); `parabola.parab2d()?`.
    pub fn parab2d(&self) -> Result<Parabola2, Failure> {
        let s: ParabolaState = *self.state.read();
        Parabola2::new(s.pos, s.focal_length)
    }
    /// The directrix (`Directrix`); `parabola.directrix()?`.
    pub fn directrix(&self) -> Result<Axis2d, Failure> {
        Ok(self.parab2d()?.directrix())
    }
    /// The focus: `XDirection * Focal + Location` (`Focus`); `parabola.focus()`.
    pub fn focus(&self) -> Point2 {
        let s: ParabolaState = *self.state.read();
        let mut pxy: Xy = s.pos.x_direction().xy();
        pxy *= s.focal_length;
        pxy += s.pos.location().xy();
        Point2::from(pxy)
    }
    /// The focal length (`Focal`); `parabola.focal()`.
    pub fn focal(&self) -> f64 {
        self.state.read().focal_length
    }
    /// The parameter `2 * Focal` (`Parameter`); `parabola.parameter()`.
    pub fn parameter(&self) -> f64 {
        2.0 * self.state.read().focal_length
    }
}
impl Geom2dGeometry for Geom2dParabola {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.focal_length *= transform.scale_factor().abs();
        state.pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: ParabolaState = *self.state.read();
        dump_class_name(stream, "Geom2d_Parabola")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "focalLength", state.focal_length)
    }
}
impl Geom2dCurve for Geom2dParabola {
    fn reverse(&self) -> Result<(), Failure> {
        reverse_conic_position(&mut self.state.write().pos)
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(-u)
    }
    crate::scaled_parameter_members!(Transform2);
    crate::open_curve_range!(-precision::infinite(), precision::infinite());
    crate::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        let s: ParabolaState = *self.state.read();
        Ok(elclib::parabola_value_2d(u, s.pos, s.focal_length))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let s: ParabolaState = *self.state.read();
        let (point, d1) = elclib::parabola_d1_2d(u, s.pos, s.focal_length);
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let s: ParabolaState = *self.state.read();
        let (point, d1, d2) = elclib::parabola_d2_2d(u, s.pos, s.focal_length);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        // The native D3 reuses ParabolaD2 and sets a zero third derivative.
        let s: ParabolaState = *self.state.read();
        let (point, d1, d2) = elclib::parabola_d2_2d(u, s.pos, s.focal_length);
        let d3 = Vector2::new(0.0, 0.0);
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom2d_Parabola::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let s: ParabolaState = *self.state.read();
        Ok(elclib::parabola_dn_2d(u, s.pos, s.focal_length, n))
    }
}
impl Geom2dConic for Geom2dParabola {
    impl_conic_position!();
    fn eccentricity(&self) -> Result<f64, Failure> {
        Ok(1.0)
    }
}
