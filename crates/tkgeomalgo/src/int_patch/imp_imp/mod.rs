// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.hxx
// /.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_ImpImpIntersection`: the intersection of two quadrics (planes, cylinders,
//! cones, spheres and tori) inside their domains (docs/geomalgo-design.md section 17).
//! The pair solvers of the `.cxx` are private modules, one per pair or helper group.
mod arc;
mod bounds;
mod cone_cone;
mod cone_sphere;
mod cyl_cone;
mod cyl_cyl;
mod cyl_sphere;
mod find_line;
mod lines;
mod multiple;
mod perform;
mod plane;
mod plane_cone;
mod post;
mod put_points;
mod rline;
mod seam;
mod segments;
mod sphere_sphere;
mod torus;
mod transitions;

use super::line::IntPatchLine;
use super::point::IntPatchPoint;
use super::IntPatchTheSOnBounds;
use crate::int_start::TheTopolTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// `IntPatch_ImpImpIntersection::IntStatus`: the status of the computation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IntPatchImpImpStatus {
    /// `IntStatus_OK`: a good intersection result.
    Ok,
    /// `IntStatus_InfiniteSectionCurve`: the intersection curve is too long (bug #26894)
    /// to compute its values and derivatives precisely in IEEE 754 arithmetic.
    InfiniteSectionCurve,
    /// `IntStatus_Fail`: the algorithm cannot finish correctly.
    #[default]
    Fail,
}

/// `IntPatch_ImpImpIntersection`: the lines and isolated points of two quadric patches.
///
/// ```ignore
/// let inter = IntPatchImpImpIntersection::with_surfaces(&s1, &*d1, &s2, &*d2, 1e-7, 1e-6, false)?;
/// assert!(inter.is_done());
/// let n = inter.nb_lines()?;
/// ```
pub struct IntPatchImpImpIntersection {
    my_done: IntPatchImpImpStatus,
    empt: bool,
    tgte: bool,
    oppo: bool,
    spnt: Vec<IntPatchPoint>,
    slin: Vec<Handle<dyn IntPatchLine>>,
    solrst: IntPatchTheSOnBounds,
}

impl Default for IntPatchImpImpIntersection {
    fn default() -> Self {
        Self::new()
    }
}

/// StdFail_NotDone of a consultation member of a failed computation.
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("IntPatch_ImpImpIntersection::{member}() - the intersection is not done"),
    )
}

impl IntPatchImpImpIntersection {
    /// `IntPatch_ImpImpIntersection()`: failed and empty.
    /// `IntPatchImpImpIntersection::new().is_done() == false`.
    pub fn new() -> Self {
        Self {
            my_done: IntPatchImpImpStatus::Fail,
            empt: true,
            tgte: false,
            oppo: false,
            spnt: Vec::new(),
            slin: Vec::new(),
            solrst: IntPatchTheSOnBounds::new(),
        }
    }

    /// `IntPatch_ImpImpIntersection(S1, D1, S2, D2, TolArc, TolTang,
    /// theIsReqToKeepRLine)` (the native default of the flag is false): runs `Perform`.
    /// `IntPatchImpImpIntersection::with_surfaces(&s1, &*d1, &s2, &*d2, 1e-7, 1e-6, false)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_surfaces(
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        tol_arc: f64,
        tol_tang: f64,
        is_req_to_keep_rline: bool,
    ) -> Result<Self, Failure> {
        let mut inter = Self::new();
        inter.perform(s1, d1, s2, d2, tol_arc, tol_tang, is_req_to_keep_rline)?;
        Ok(inter)
    }

    /// `IsDone()`: the status is not Fail. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.get_status() != IntPatchImpImpStatus::Fail
    }

    /// `GetStatus()`. `inter.get_status()`.
    pub fn get_status(&self) -> IntPatchImpImpStatus {
        self.my_done
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.is_done() {
            return Err(not_done(member));
        }
        Ok(())
    }

    /// `IsEmpty()`: StdFail_NotDone unless done. `inter.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        self.check_done("IsEmpty")?;
        Ok(self.empt)
    }

    /// `TangentFaces()`: every restriction of each patch is on the other surface;
    /// StdFail_NotDone unless done. `inter.tangent_faces()?`.
    pub fn tangent_faces(&self) -> Result<bool, Failure> {
        self.check_done("TangentFaces")?;
        Ok(self.tgte)
    }

    /// `OppositeFaces()`: the tangent faces have opposite normals; StdFail_NotDone
    /// unless done, Standard_DomainError unless the faces are tangent.
    /// `inter.opposite_faces()?`.
    pub fn opposite_faces(&self) -> Result<bool, Failure> {
        self.check_done("OppositeFaces")?;
        if !self.tgte {
            return Err(Failure::with_message(
                "Standard_DomainError",
                "IntPatch_ImpImpIntersection::OppositeFaces() - the faces are not tangent",
            ));
        }
        Ok(self.oppo)
    }

    /// `NbPnts()`: the number of isolated points. `inter.nb_pnts()?`.
    pub fn nb_pnts(&self) -> Result<i32, Failure> {
        self.check_done("NbPnts")?;
        Ok(self.spnt.len() as i32)
    }

    /// `Point(Index)`: Standard_OutOfRange outside `1..=NbPnts()`. `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<&IntPatchPoint, Failure> {
        self.check_done("Point")?;
        crate::int_start::failures::sequence_value(&self.spnt, index, "IntPatch_ImpImpIntersection::Point")
    }

    /// `NbLines()`: the number of lines. `inter.nb_lines()?`.
    pub fn nb_lines(&self) -> Result<i32, Failure> {
        self.check_done("NbLines")?;
        Ok(self.slin.len() as i32)
    }

    /// `Line(Index)`: Standard_OutOfRange outside `1..=NbLines()`. `inter.line(1)?`.
    pub fn line(&self, index: i32) -> Result<&Handle<dyn IntPatchLine>, Failure> {
        self.check_done("Line")?;
        crate::int_start::failures::sequence_value(&self.slin, index, "IntPatch_ImpImpIntersection::Line")
    }
}
