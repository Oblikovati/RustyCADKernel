// Created on: 1992-05-07
// Created by: Jacques GOUSSARD
// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.hxx
// /.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_ImpPrmIntersection`: the intersection of a quadric (plane, cylinder, sphere
//! or cone) and a parametric surface inside their domains (docs/geomalgo-design.md
//! section 17): the walking of the implicit function of the quadric on the parametric
//! surface, the restriction segments, then the decomposition at seams and poles.
mod cleanup;
mod decompose;
mod perform;
mod segments;
mod setup;
mod tangency;
mod walk_lines;

use super::line::IntPatchLine;
use super::point::IntPatchPoint;
use super::{IntPatchTheSOnBounds, IntPatchTheSearchInside};
use crate::int_start::TheTopolTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// `IntPatch_ImpPrmIntersection`: the lines and points of a quadric and a parametric
/// patch.
///
/// ```ignore
/// let inter = IntPatchImpPrmIntersection::with_surfaces(&s1, &*d1, &s2, &*d2, 1e-7, 1e-6, 0.01, 0.1)?;
/// let n = inter.nb_lines()?;
/// ```
pub struct IntPatchImpPrmIntersection {
    done: bool,
    empt: bool,
    spnt: Vec<IntPatchPoint>,
    slin: Vec<Handle<dyn IntPatchLine>>,
    solrst: IntPatchTheSOnBounds,
    solins: IntPatchTheSearchInside,
    my_is_start_pnt: bool,
    my_u_start: f64,
    my_v_start: f64,
}

impl Default for IntPatchImpPrmIntersection {
    fn default() -> Self {
        Self::new()
    }
}

/// StdFail_NotDone of a consultation member before a successful computation.
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("IntPatch_ImpPrmIntersection::{member}() - the intersection is not done"),
    )
}

impl IntPatchImpPrmIntersection {
    /// `IntPatch_ImpPrmIntersection()`: not done, no start point.
    /// `IntPatchImpPrmIntersection::new().is_done() == false`.
    pub fn new() -> Self {
        Self {
            done: false,
            empt: false,
            spnt: Vec::new(),
            slin: Vec::new(),
            solrst: IntPatchTheSOnBounds::new(),
            solins: IntPatchTheSearchInside::new(),
            my_is_start_pnt: false,
            my_u_start: 0.0,
            my_v_start: 0.0,
        }
    }

    /// `IntPatch_ImpPrmIntersection(Surf1, D1, Surf2, D2, TolArc, TolTang, Fleche,
    /// Pas)`: runs `Perform`.
    /// `IntPatchImpPrmIntersection::with_surfaces(&s1, &*d1, &s2, &*d2, 1e-7, 1e-6, 0.01, 0.1)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_surfaces(
        surf1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        surf2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        tol_arc: f64,
        tol_tang: f64,
        fleche: f64,
        pas: f64,
    ) -> Result<Self, Failure> {
        let mut inter = Self::new();
        inter.perform(surf1, d1, surf2, d2, tol_arc, tol_tang, fleche, pas)?;
        Ok(inter)
    }

    /// `SetStartPoint(U, V)`: the interior search starts from `(u, v)` on the parametric
    /// surface only. `inter.set_start_point(0.5, 0.5)`.
    pub fn set_start_point(&mut self, u: f64, v: f64) {
        self.my_is_start_pnt = true;
        self.my_u_start = u;
        self.my_v_start = v;
    }

    /// `IsDone()`. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.done {
            return Err(not_done(member));
        }
        Ok(())
    }

    /// `IsEmpty()`: StdFail_NotDone unless done. `inter.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        self.check_done("IsEmpty")?;
        Ok(self.empt)
    }

    /// `NbPnts()`. `inter.nb_pnts()?`.
    pub fn nb_pnts(&self) -> Result<i32, Failure> {
        self.check_done("NbPnts")?;
        Ok(self.spnt.len() as i32)
    }

    /// `Point(Index)`: Standard_OutOfRange outside `1..=NbPnts()`. `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<&IntPatchPoint, Failure> {
        self.check_done("Point")?;
        crate::int_start::failures::sequence_value(&self.spnt, index, "IntPatch_ImpPrmIntersection::Point")
    }

    /// `NbLines()`. `inter.nb_lines()?`.
    pub fn nb_lines(&self) -> Result<i32, Failure> {
        self.check_done("NbLines")?;
        Ok(self.slin.len() as i32)
    }

    /// `Line(Index)`: Standard_OutOfRange outside `1..=NbLines()`. `inter.line(1)?`.
    pub fn line(&self, index: i32) -> Result<&Handle<dyn IntPatchLine>, Failure> {
        self.check_done("Line")?;
        crate::int_start::failures::sequence_value(&self.slin, index, "IntPatch_ImpPrmIntersection::Line")
    }
}
