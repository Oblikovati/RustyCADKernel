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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Hyperbola.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Hyperbola`: the main branch of a hyperbola, parameterized by
//! `P(u) = O + Major * cosh(u) * XDir + Minor * sinh(u) * YDir`.
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
use rustycad_tkmath::gp::{Axis1, Axis2, Hyperbola3, Point3, Transform3, Vector3};
use std::io;

#[derive(Clone, Copy)]
struct HyperbolaState {
    pos: Axis2,
    major_radius: f64,
    minor_radius: f64,
}

/// A hyperbola; `Handle::new(GeomHyperbola::new(frame, 3.0, 2.0)?)`.
pub struct GeomHyperbola {
    parent: StandardTransient,
    state: SharedState<HyperbolaState>,
}
impl_transient!(
    GeomHyperbola,
    "Geom_Hyperbola",
    dyn GeomConic,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomConic]
);
impl Clone for GeomHyperbola {
    fn clone(&self) -> Self {
        Self::stored(self.state.snapshot())
    }
}

fn check_radius(member: &str, name: &str, radius: f64) -> Result<(), Failure> {
    if radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom_Hyperbola{member}: {name} = {radius}; expected {name} >= 0"
        )));
    }
    Ok(())
}

impl GeomHyperbola {
    fn stored(state: HyperbolaState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Hyperbola from a gp hyperbola (`Geom_Hyperbola(gp_Hypr)`); `GeomHyperbola::from_hypr(h)`.
    pub fn from_hypr(hyperbola: Hyperbola3) -> Self {
        Self::stored(HyperbolaState {
            pos: hyperbola.position(),
            major_radius: hyperbola.major_radius(),
            minor_radius: hyperbola.minor_radius(),
        })
    }
    /// Hyperbola in a coordinate system (`Geom_Hyperbola(gp_Ax2, MajorRadius, MinorRadius)`);
    /// Standard_ConstructionError for a negative radius; `GeomHyperbola::new(frame, 3.0, 2.0)?`.
    pub fn new(frame: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if major_radius < 0.0 || minor_radius < 0.0 {
            return Err(errors::construction(&format!(
                "Geom_Hyperbola: MajorRadius = {major_radius}, MinorRadius = {minor_radius}; \
                 expected both >= 0"
            )));
        }
        Ok(Self::stored(HyperbolaState {
            pos: frame,
            major_radius,
            minor_radius,
        }))
    }
    /// Replace by a gp hyperbola (`SetHypr`); `hyperbola.set_hypr(h)`.
    pub fn set_hypr(&self, hyperbola: Hyperbola3) {
        *self.state.write() = HyperbolaState {
            pos: hyperbola.position(),
            major_radius: hyperbola.major_radius(),
            minor_radius: hyperbola.minor_radius(),
        };
    }
    /// Set the major radius; Standard_ConstructionError when negative (`SetMajorRadius`);
    /// `hyperbola.set_major_radius(4.0)?`.
    pub fn set_major_radius(&self, major_radius: f64) -> Result<(), Failure> {
        check_radius("::SetMajorRadius", "MajorRadius", major_radius)?;
        self.state.write().major_radius = major_radius;
        Ok(())
    }
    /// Set the minor radius; Standard_ConstructionError when negative (`SetMinorRadius`);
    /// `hyperbola.set_minor_radius(1.0)?`.
    pub fn set_minor_radius(&self, minor_radius: f64) -> Result<(), Failure> {
        check_radius("::SetMinorRadius", "MinorRadius", minor_radius)?;
        self.state.write().minor_radius = minor_radius;
        Ok(())
    }
    /// The gp hyperbola (`Hypr`); `hyperbola.hypr()?`.
    pub fn hypr(&self) -> Result<Hyperbola3, Failure> {
        let state: HyperbolaState = *self.state.read();
        Hyperbola3::new(state.pos, state.major_radius, state.minor_radius)
    }
    /// First asymptote, through the gp hyperbola (`Asymptote1`); `hyperbola.asymptote1()?`.
    pub fn asymptote1(&self) -> Result<Axis1, Failure> {
        self.hypr()?.asymptote1()
    }
    /// Second asymptote (`Asymptote2`); `hyperbola.asymptote2()?`.
    pub fn asymptote2(&self) -> Result<Axis1, Failure> {
        self.hypr()?.asymptote2()
    }
    /// The conjugate branch on the positive Y side (`ConjugateBranch1`); `hyperbola.conjugate_branch1()?`.
    pub fn conjugate_branch1(&self) -> Result<Hyperbola3, Failure> {
        self.hypr()?.conjugate_branch1()
    }
    /// The conjugate branch on the negative Y side (`ConjugateBranch2`); `hyperbola.conjugate_branch2()?`.
    pub fn conjugate_branch2(&self) -> Result<Hyperbola3, Failure> {
        self.hypr()?.conjugate_branch2()
    }
    /// First directrix (`Directrix1`); `hyperbola.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis1, Failure> {
        self.hypr()?.directrix1()
    }
    /// Second directrix (`Directrix2`); `hyperbola.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis1, Failure> {
        self.hypr()?.directrix2()
    }
    /// Distance between the foci (`Focal`); `hyperbola.focal()`.
    pub fn focal(&self) -> f64 {
        2.0 * self.focus_distance()
    }
    /// Focus on the positive X side (`Focus1`); `hyperbola.focus1()`.
    pub fn focus1(&self) -> Point3 {
        focus_point(self.state.read().pos, self.focus_distance(), true)
    }
    /// Focus on the negative X side (`Focus2`); `hyperbola.focus2()`.
    pub fn focus2(&self) -> Point3 {
        focus_point(self.state.read().pos, self.focus_distance(), false)
    }
    // sqrt(Major² + Minor²), the native C.
    fn focus_distance(&self) -> f64 {
        let state: HyperbolaState = *self.state.read();
        let (a, b) = (state.major_radius, state.minor_radius);
        (a * a + b * b).sqrt()
    }
    /// The major radius (`MajorRadius`); `hyperbola.major_radius()`.
    pub fn major_radius(&self) -> f64 {
        self.state.read().major_radius
    }
    /// The minor radius (`MinorRadius`); `hyperbola.minor_radius()`.
    pub fn minor_radius(&self) -> f64 {
        self.state.read().minor_radius
    }
    /// The other branch, symmetric about the Y axis (`OtherBranch`); `hyperbola.other_branch()?`.
    pub fn other_branch(&self) -> Result<Hyperbola3, Failure> {
        self.hypr()?.other_branch()
    }
    /// Semi-latus rectum `b²/a`; Standard_ConstructionError for a null major radius
    /// (`Parameter`); `hyperbola.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        let state: HyperbolaState = *self.state.read();
        if state.major_radius == 0.0 {
            return Err(errors::construction(
                "Geom_Hyperbola::Parameter: MajorRadius = 0; expected a non-null major radius",
            ));
        }
        Ok((state.minor_radius * state.minor_radius) / state.major_radius)
    }
}
impl GeomGeometry for GeomHyperbola {
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
        let state: HyperbolaState = *self.state.read();
        dump_class_name(stream, "Geom_Hyperbola")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "majorRadius", state.major_radius)?;
        dump::dump_real(stream, "minorRadius", state.minor_radius)
    }
}
impl GeomCurve for GeomHyperbola {
    fn reverse(&self) -> Result<(), Failure> {
        reverse_conic_position(&mut self.state.write().pos)
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(-u)
    }
    rustycad_tkg2d::open_curve_range!(-precision::infinite(), precision::infinite());
    rustycad_tkg2d::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let s: HyperbolaState = *self.state.read();
        Ok(elclib::hyperbola_value(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
        ))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let s: HyperbolaState = *self.state.read();
        let (point, d1) = elclib::hyperbola_d1(u, s.pos, s.major_radius, s.minor_radius);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let s: HyperbolaState = *self.state.read();
        let (point, d1, d2) = elclib::hyperbola_d2(u, s.pos, s.major_radius, s.minor_radius);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let s: HyperbolaState = *self.state.read();
        let (point, d1, d2, d3) = elclib::hyperbola_d3(u, s.pos, s.major_radius, s.minor_radius);
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom_Hyperbola::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let s: HyperbolaState = *self.state.read();
        Ok(elclib::hyperbola_dn(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
            n,
        ))
    }
}
impl GeomConic for GeomHyperbola {
    impl_conic_position!();
    fn eccentricity(&self) -> Result<f64, Failure> {
        let state: HyperbolaState = *self.state.read();
        let (a, b) = (state.major_radius, state.minor_radius);
        if a == 0.0 {
            return Err(errors::construction(
                "Geom_Hyperbola::Eccentricity: MajorRadius = 0; expected a non-null major radius",
            ));
        }
        Ok((a * a + b * b).sqrt() / a)
    }
}
