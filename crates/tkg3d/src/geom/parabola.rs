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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Parabola.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Parabola`: a parabola parameterized by `P(u) = O + u²/(4F) * XDir + u * YDir`.
use super::conic::{GeomConic, dump_conic, impl_conic_position, reverse_conic_position};
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use super::ellipse::focus_point;
use super::errors;
use super::geometry::GeomGeometry;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis1, Axis2, Parabola3, Point3, Transform3, Vector3};
use std::io;

#[derive(Clone, Copy)]
struct ParabolaState {
    pos: Axis2,
    focal_length: f64,
}

/// A parabola; `Handle::new(GeomParabola::new(frame, 2.0)?)`.
pub struct GeomParabola {
    parent: StandardTransient,
    state: SharedState<ParabolaState>,
}
impl_transient!(
    GeomParabola,
    "Geom_Parabola",
    dyn GeomConic,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomConic]
);
impl Clone for GeomParabola {
    fn clone(&self) -> Self {
        Self::stored(self.state.snapshot())
    }
}

fn checked_focal(member: &str, focal: f64) -> Result<f64, Failure> {
    if focal < 0.0 {
        return Err(errors::construction(&format!(
            "Geom_Parabola{member}: Focal = {focal}; expected Focal >= 0"
        )));
    }
    Ok(focal)
}

impl GeomParabola {
    fn stored(state: ParabolaState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Parabola from a gp parabola (`Geom_Parabola(gp_Parab)`); `GeomParabola::from_parab(p)`.
    pub fn from_parab(parabola: Parabola3) -> Self {
        Self::stored(ParabolaState {
            pos: parabola.position(),
            focal_length: parabola.focal(),
        })
    }
    /// Parabola in a coordinate system (`Geom_Parabola(gp_Ax2, Focal)`);
    /// Standard_ConstructionError for a negative focal; `GeomParabola::new(frame, 2.0)?`.
    pub fn new(frame: Axis2, focal: f64) -> Result<Self, Failure> {
        let focal_length: f64 = checked_focal("", focal)?;
        Ok(Self::stored(ParabolaState {
            pos: frame,
            focal_length,
        }))
    }
    /// Parabola of directrix `directrix` and focus `focus`, through gp_Parab
    /// (`Geom_Parabola(gp_Ax1, gp_Pnt)`); `GeomParabola::from_directrix(d, f)?`.
    pub fn from_directrix(directrix: Axis1, focus: Point3) -> Result<Self, Failure> {
        Ok(Self::from_parab(Parabola3::from_directrix(
            directrix, focus,
        )?))
    }
    /// Set the focal distance; Standard_ConstructionError when negative (`SetFocal`);
    /// `parabola.set_focal(3.0)?`.
    pub fn set_focal(&self, focal: f64) -> Result<(), Failure> {
        self.state.write().focal_length = checked_focal("::SetFocal", focal)?;
        Ok(())
    }
    /// Replace by a gp parabola (`SetParab`); `parabola.set_parab(p)`.
    pub fn set_parab(&self, parabola: Parabola3) {
        *self.state.write() = ParabolaState {
            pos: parabola.position(),
            focal_length: parabola.focal(),
        };
    }
    /// The gp parabola (`Parab`); `parabola.parab()?`.
    pub fn parab(&self) -> Result<Parabola3, Failure> {
        let state: ParabolaState = *self.state.read();
        Parabola3::new(state.pos, state.focal_length)
    }
    /// The directrix, through the gp parabola (`Directrix`); `parabola.directrix()?`.
    pub fn directrix(&self) -> Result<Axis1, Failure> {
        Ok(self.parab()?.directrix())
    }
    /// The focal distance (`Focal`); `parabola.focal()`.
    pub fn focal(&self) -> f64 {
        self.state.read().focal_length
    }
    /// The focus `O + Focal * XDir` (`Focus`); `parabola.focus()`.
    pub fn focus(&self) -> Point3 {
        let state: ParabolaState = *self.state.read();
        focus_point(state.pos, state.focal_length, true)
    }
    /// The parameter `2 * Focal` (`Parameter`); `parabola.parameter()`.
    pub fn parameter(&self) -> f64 {
        2.0 * self.state.read().focal_length
    }
}
impl GeomGeometry for GeomParabola {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.focal_length *= transform.scale_factor().abs();
        state.pos.transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: ParabolaState = *self.state.read();
        dump_class_name(stream, "Geom_Parabola")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "focalLength", state.focal_length)
    }
}
impl GeomCurve for GeomParabola {
    fn reverse(&self) -> Result<(), Failure> {
        reverse_conic_position(&mut self.state.write().pos)
    }
    fn reversed_parameter(&self, u: f64) -> f64 {
        -u
    }
    fn transformed_parameter(&self, u: f64, transform: Transform3) -> f64 {
        if precision::is_infinite(u) {
            return u;
        }
        u * transform.scale_factor().abs()
    }
    fn parametric_transformation(&self, transform: Transform3) -> f64 {
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
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let s: ParabolaState = *self.state.read();
        Ok(elclib::parabola_value(u, s.pos, s.focal_length))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let s: ParabolaState = *self.state.read();
        let (point, d1) = elclib::parabola_d1(u, s.pos, s.focal_length);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let s: ParabolaState = *self.state.read();
        let (point, d1, d2) = elclib::parabola_d2(u, s.pos, s.focal_length);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        // The native EvalD3 evaluates D2 and sets a null third derivative.
        let s: ParabolaState = *self.state.read();
        let (point, d1, d2) = elclib::parabola_d2(u, s.pos, s.focal_length);
        let d3 = Vector3::new(0.0, 0.0, 0.0);
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom_Parabola::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let s: ParabolaState = *self.state.read();
        Ok(elclib::parabola_dn(u, s.pos, s.focal_length, n))
    }
}
impl GeomConic for GeomParabola {
    impl_conic_position!();
    fn eccentricity(&self) -> Result<f64, Failure> {
        Ok(1.0)
    }
}
