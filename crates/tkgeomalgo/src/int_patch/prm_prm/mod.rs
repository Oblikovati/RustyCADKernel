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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.hxx/
// .lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_PrmPrmIntersection`: the intersection of two bi-parametric surfaces by
//! marching from the start points of a polyhedral interference (docs/geomalgo-design.md
//! section 18). The grid members of the header (`GrilleInteger`, `CodeReject`...) are
//! here; the grid filling, `PointDepart`, `NewLine`, the static helpers and the
//! `Perform` overloads are in the submodules.
mod grid;
mod new_line;
mod point_depart;

use super::IntPatchLine;
use crate::int_start::failures::sequence_value;
use rustycad_tkernel::standard::{Failure, Handle};

/// `_DECAL`: the bits of one grid coordinate.
const DECAL: i32 = 7;
/// `_DECAL2`.
const DECAL2: i32 = 14;
/// `_BASE`: the number of cells of the grid in each direction.
const BASE: i32 = 128;
/// `_BASEM1`.
const BASEM1: i32 = 127;

/// `IntPatch_PrmPrmIntersection`: the intersection lines of two bi-parametric surfaces
/// (walking lines), computed from the polyhedra of the surfaces, from given start
/// points, or from a grid of the space.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchPrmPrmIntersection;
/// let inter = IntPatchPrmPrmIntersection::new();
/// assert!(!inter.is_done());
/// assert!(inter.nb_lines().is_err());
/// assert_eq!(inter.grille_integer(1, 2, 3), 1 | (2 << 7) | (3 << 14));
/// ```
pub struct IntPatchPrmPrmIntersection {
    done: bool,
    empt: bool,
    slin: Vec<Handle<dyn IntPatchLine>>,
}

impl Default for IntPatchPrmPrmIntersection {
    fn default() -> Self {
        Self::new()
    }
}

/// The StdFail_NotDone of a result read before a Perform.
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!(
            "IntPatch_PrmPrmIntersection::{member}() - the intersection is not done; expected a Perform"
        ),
    )
}

impl IntPatchPrmPrmIntersection {
    /// The empty constructor: not done, empty. `IntPatchPrmPrmIntersection::new()`.
    pub fn new() -> Self {
        Self {
            done: false,
            empt: true,
            slin: Vec::new(),
        }
    }

    /// `IsDone()`: whether the computation was successful. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `IsEmpty()`: whether there is no intersection; StdFail_NotDone unless done.
    /// `inter.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("IsEmpty"));
        }
        Ok(self.empt)
    }

    /// `NbLines()`: StdFail_NotDone unless done. `inter.nb_lines()?`.
    pub fn nb_lines(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbLines"));
        }
        Ok(self.slin.len() as i32)
    }

    /// `Line(Index)`: StdFail_NotDone unless done, Standard_OutOfRange outside
    /// `1..=NbLines()`. `inter.line(1)?`.
    pub fn line(&self, index: i32) -> Result<Handle<dyn IntPatchLine>, Failure> {
        if !self.done {
            return Err(not_done("Line"));
        }
        sequence_value(&self.slin, index, "IntPatch_PrmPrmIntersection::Line").cloned()
    }

    /// `GrilleInteger(ix, iy, iz)`: the code of the cell `(ix, iy, iz)` of the grid.
    /// `inter.grille_integer(1, 2, 3)`.
    pub fn grille_integer(&self, ix: i32, iy: i32, iz: i32) -> i32 {
        let tz: i32 = iz.wrapping_shl(DECAL2 as u32);
        let ty: i32 = iy.wrapping_shl(DECAL as u32);
        let mut t: i32 = ix;
        t |= ty;
        t |= tz;
        t
    }

    /// `IntegerGrille(t, ix, iy, iz)`: the cell of the code `t`.
    /// `let (ix, iy, iz) = inter.integer_grille(t);`.
    pub fn integer_grille(&self, tt: i32) -> (i32, i32, i32) {
        let mut t: i32 = tt;
        let ix: i32 = t & BASEM1;
        t >>= DECAL;
        let iy: i32 = t & BASEM1;
        t >>= DECAL;
        (ix, iy, t)
    }

    /// `DansGrille(t)`: 1 when `0 <= t < 128`, else 0. `inter.dans_grille(5)`.
    pub fn dans_grille(&self, t: i32) -> i32 {
        i32::from((0..BASE).contains(&t))
    }

    /// `NbPointsGrille()`: 128. `inter.nb_points_grille()`.
    pub fn nb_points_grille(&self) -> i32 {
        BASE
    }

    /// `CodeReject(x0, y0, z0, x1, y1, z1, x, y, z)`: the bits of the sides of the box
    /// `[x0, x1] x [y0, y1] x [z0, z1]` that the point `(x, y, z)` is out of (1, 2, 4
    /// below, 8, 16, 32 above). `inter.code_reject([0.0; 3], [1.0; 3], [2.0, 0.5, -1.0])`.
    pub fn code_reject(
        &self,
        [x0, y0, z0]: [f64; 3],
        [x1, y1, z1]: [f64; 3],
        [x, y, z]: [f64; 3],
    ) -> i32 {
        let mut code: i32 = 0;
        if x < x0 {
            code = 1;
        }
        if y < y0 {
            code |= 2;
        }
        if z < z0 {
            code |= 4;
        }
        if x > x1 {
            code |= 8;
        }
        if y > y1 {
            code |= 16;
        }
        if z > z1 {
            code |= 32;
        }
        code
    }
}

/// `StartParams(1..4)`: the start parameters `u1 v1 u2 v2` of the walking, a native
/// 1-based array. `start_params([0.5, 0.5, 0.25, 0.25])?`.
pub(super) fn start_params(
    uv: [f64; 4],
) -> Result<rustycad_tkernel::ncollection::Array1<'static, f64>, Failure> {
    let mut array = rustycad_tkernel::ncollection::Array1::with_bounds(1, 4)
        .map_err(|e| e.into_failure("IntPatch_PrmPrmIntersection"))?;
    for (i, v) in uv.iter().enumerate() {
        array[i as i32 + 1] = *v;
    }
    Ok(array)
}
