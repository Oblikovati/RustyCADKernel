// Created on: 1995-01-27
// Created by: Jacques GOUSSARD
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_IntSS.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomInt_IntSS`: the intersection of two surfaces as 3d curves and, on demand, their
//! 2d curves on the surfaces (docs/geomalgo-design.md section 20). The front end
//! (`GeomInt_IntSS.cxx`) is here and in `internal_perform`; the curve making
//! (`GeomInt_IntSS_1.cxx`) is split by line kind (`conic_curves`, `walking_curves`,
//! `walking_approx`, `restriction_curves`) and by static member (`treat_rline`,
//! `pcurves`, `trim`, `bspline`).
mod bspline;
mod conic_curves;
mod internal_perform;
mod pcurves;
mod restriction_curves;
mod treat_rline;
mod trim;
mod walking_approx;
mod walking_curves;

pub use treat_rline::GeomIntRLineCurves;

use super::line_constructor::GeomIntLineConstructor;
use crate::int_patch::IntPatchIntersection;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Point2, Point3};

/// The approximation flags of `Perform` (`Approx`, `ApproxS1`, `ApproxS2`; native
/// defaults `true, false, false`): the walking lines are approximated, and the 2d
/// curves on the first and second surfaces are made.
///
/// ```
/// use rustycad_tkgeomalgo::geom_int::GeomIntIntSSApprox;
/// let flags = GeomIntIntSSApprox::default();
/// assert!(flags.approx && !flags.approx_s1 && !flags.approx_s2);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeomIntIntSSApprox {
    /// `Approx`.
    pub approx: bool,
    /// `ApproxS1`.
    pub approx_s1: bool,
    /// `ApproxS2`.
    pub approx_s2: bool,
}

impl Default for GeomIntIntSSApprox {
    fn default() -> Self {
        Self {
            approx: true,
            approx_s1: false,
            approx_s2: false,
        }
    }
}

/// `GeomInt_IntSS`: the intersection curves of two surfaces.
///
/// ```ignore
/// let mut inter = GeomIntIntSS::new();
/// inter.perform(&s1, &s2, 1e-7, GeomIntIntSSApprox::default())?;
/// for i in 1..=inter.nb_lines() { let c = inter.line(i)?; }
/// ```
pub struct GeomIntIntSS {
    my_intersector: IntPatchIntersection,
    my_lconstruct: GeomIntLineConstructor,
    my_hs1: Handle<GeomAdaptorSurface>,
    my_hs2: Handle<GeomAdaptorSurface>,
    my_nbrestr: i32,
    sline: Vec<Handle<dyn GeomCurve>>,
    sline_s1: Vec<Handle<dyn Geom2dCurve>>,
    sline_s2: Vec<Handle<dyn Geom2dCurve>>,
    my_tol_reached2d: f64,
    my_tol_reached3d: f64,
    // The tangent check tolerances: `SetTolFixTangents` and `TolFixTangents` are declared
    // without a definition natively (a call does not link), so they keep their values.
    my_tol_check: f64,
    my_tol_ang_check: f64,
}

impl Default for GeomIntIntSS {
    fn default() -> Self {
        Self::new()
    }
}

/// The 1-based `NCollection_Sequence::Value(Index)` (Standard_OutOfRange outside).
fn seq_at<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    seq: &[Handle<T>],
    index: i32,
    member: &str,
) -> Result<Handle<T>, Failure> {
    if index < 1 || index as usize > seq.len() {
        return Err(Failure::with_message(
            "Standard_OutOfRange",
            &format!("{member}: index {index} outside 1..={}", seq.len()),
        ));
    }
    Ok(seq[(index - 1) as usize].clone())
}

impl GeomIntIntSS {
    /// `GeomInt_IntSS()`: no result; the tangent check tolerances are 1e-7 and 1e-6;
    /// `GeomIntIntSS::new()`.
    pub fn new() -> Self {
        Self {
            my_intersector: IntPatchIntersection::new(),
            my_lconstruct: GeomIntLineConstructor::new(),
            my_hs1: Handle::default(),
            my_hs2: Handle::default(),
            my_nbrestr: 0,
            sline: Vec::new(),
            sline_s1: Vec::new(),
            sline_s2: Vec::new(),
            my_tol_reached2d: 0.0,
            my_tol_reached3d: 0.0,
            my_tol_check: 1.0e-7,
            my_tol_ang_check: 1.0e-6,
        }
    }

    /// `GeomInt_IntSS(S1, S2, Tol, Approx, ApproxS1, ApproxS2)`: the general
    /// intersection of two surfaces;
    /// `GeomIntIntSS::with_surfaces(&s1, &s2, 1e-7, GeomIntIntSSApprox::default())?`.
    pub fn with_surfaces(
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        tol: f64,
        approx: GeomIntIntSSApprox,
    ) -> Result<Self, Failure> {
        let mut inter = Self::new();
        inter.perform(s1, s2, tol, approx)?;
        Ok(inter)
    }

    /// The adaptors of the surfaces (the same one for one surface).
    fn load_surfaces(
        &mut self,
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        self.my_hs1 = Handle::new(GeomAdaptorSurface::with_surface(s1)?);
        self.my_hs2 = if s1 == s2 {
            self.my_hs1.clone()
        } else {
            Handle::new(GeomAdaptorSurface::with_surface(s2)?)
        };
        Ok(())
    }

    /// `Perform(S1, S2, Tol, Approx, ApproxS1, ApproxS2)`: the general intersection;
    /// `inter.perform(&s1, &s2, 1e-7, GeomIntIntSSApprox::default())?`.
    pub fn perform(
        &mut self,
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        tol: f64,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        self.load_surfaces(s1, s2)?;
        self.internal_perform(tol, approx, None)
    }

    /// `Perform(HS1, HS2, Tol, Approx, ApproxS1, ApproxS2)`: the intersection of adapted
    /// surfaces; `inter.perform_adaptors(&hs1, &hs2, 1e-7, GeomIntIntSSApprox::default())?`.
    pub fn perform_adaptors(
        &mut self,
        hs1: &Handle<GeomAdaptorSurface>,
        hs2: &Handle<GeomAdaptorSurface>,
        tol: f64,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        self.my_hs1 = hs1.clone();
        self.my_hs2 = hs2.clone();
        self.internal_perform(tol, approx, None)
    }

    /// `Perform(S1, S2, Tol, U1, V1, U2, V2, Approx, ApproxS1, ApproxS2)`: the
    /// intersection through a starting point;
    /// `inter.perform_from_start(&s1, &s2, 1e-7, [0.5; 4], GeomIntIntSSApprox::default())?`.
    pub fn perform_from_start(
        &mut self,
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        tol: f64,
        start: [f64; 4],
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        self.load_surfaces(s1, s2)?;
        self.internal_perform(tol, approx, Some(start))
    }

    /// `Perform(HS1, HS2, Tol, U1, V1, U2, V2, Approx, ApproxS1, ApproxS2)`: the
    /// intersection of adapted surfaces through a starting point;
    /// `inter.perform_adaptors_from_start(&hs1, &hs2, 1e-7, [0.5; 4], flags)?`.
    pub fn perform_adaptors_from_start(
        &mut self,
        hs1: &Handle<GeomAdaptorSurface>,
        hs2: &Handle<GeomAdaptorSurface>,
        tol: f64,
        start: [f64; 4],
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        self.my_hs1 = hs1.clone();
        self.my_hs2 = hs2.clone();
        self.internal_perform(tol, approx, Some(start))
    }

    /// StdFail_NotDone unless the intersector is done.
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.my_intersector.is_done() {
            return Ok(());
        }
        Err(Failure::with_message(
            "StdFail_NotDone",
            &format!("GeomInt_IntSS::{member} - no result; expected a successful Perform"),
        ))
    }

    /// `IsDone()`; `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_intersector.is_done()
    }

    /// `TolReached3d()`; `inter.tol_reached3d()`.
    pub fn tol_reached3d(&self) -> f64 {
        self.my_tol_reached3d
    }

    /// `TolReached2d()`; `inter.tol_reached2d()`.
    pub fn tol_reached2d(&self) -> f64 {
        self.my_tol_reached2d
    }

    /// `NbLines()`: the curves after the boundaries; `inter.nb_lines()`.
    pub fn nb_lines(&self) -> i32 {
        self.sline.len() as i32 - self.my_nbrestr
    }

    /// `Line(Index)`: the curve `index` (StdFail_NotDone, Standard_OutOfRange);
    /// `inter.line(1)?`.
    pub fn line(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.check_done("Line")?;
        seq_at(&self.sline, index + self.my_nbrestr, "GeomInt_IntSS::Line")
    }

    /// `HasLineOnS1(Index)`: whether the curve `index` has a 2d curve on S1;
    /// `inter.has_line_on_s1(1)?`.
    pub fn has_line_on_s1(&self, index: i32) -> Result<bool, Failure> {
        self.check_done("HasLineOnS1")?;
        Ok(!seq_at(&self.sline_s1, index, "GeomInt_IntSS::HasLineOnS1")?.is_null())
    }

    /// `LineOnS1(Index)`: the 2d curve on S1 of the curve `index` (null when not made);
    /// `inter.line_on_s1(1)?`.
    pub fn line_on_s1(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.check_done("LineOnS1")?;
        seq_at(&self.sline_s1, index, "GeomInt_IntSS::LineOnS1")
    }

    /// `HasLineOnS2(Index)`; `inter.has_line_on_s2(1)?`.
    pub fn has_line_on_s2(&self, index: i32) -> Result<bool, Failure> {
        self.check_done("HasLineOnS2")?;
        Ok(!seq_at(&self.sline_s2, index, "GeomInt_IntSS::HasLineOnS2")?.is_null())
    }

    /// `LineOnS2(Index)`; `inter.line_on_s2(1)?`.
    pub fn line_on_s2(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.check_done("LineOnS2")?;
        seq_at(&self.sline_s2, index, "GeomInt_IntSS::LineOnS2")
    }

    /// `NbBoundaries()` (StdFail_NotDone); `inter.nb_boundaries()?`.
    pub fn nb_boundaries(&self) -> Result<i32, Failure> {
        self.check_done("NbBoundaries()")?;
        Ok(self.my_nbrestr)
    }

    /// `Boundary(Index)`: StdFail_NotDone, Standard_OutOfRange outside
    /// `1..=NbBoundaries`; `inter.boundary(1)?`.
    pub fn boundary(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.check_done("Line")?;
        if index <= 0 || index > self.my_nbrestr {
            return Err(Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "GeomInt_IntSS::Boundary: index {index} outside 1..={}",
                    self.my_nbrestr
                ),
            ));
        }
        seq_at(&self.sline, index, "GeomInt_IntSS::Boundary")
    }

    /// `NbPoints()`: the isolated points of the intersector; `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.my_intersector.nb_pnts()
    }

    /// `Point(Index)`: the isolated point `index`; `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        Ok(self.my_intersector.point(index)?.value())
    }

    /// `Pnt2d(Index, OnFirst)`: its parameters on one surface; `inter.pnt2d(1, true)?`.
    pub fn pnt2d(&self, index: i32, on_first: bool) -> Result<Point2, Failure> {
        let thept = self.my_intersector.point(index)?;
        let (u, v) = if on_first {
            thept.parameters_on_s1()
        } else {
            thept.parameters_on_s2()
        };
        Ok(Point2::new(u, v))
    }
}
