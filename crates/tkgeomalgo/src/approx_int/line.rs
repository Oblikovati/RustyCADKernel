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
// Rust translation of the TheLine calls of ModelingAlgorithms/TKGeomAlgo/ApproxInt/
// ApproxInt_MultiLine.gxx and ApproxInt_Approx.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TheLine` (`TheWLine`) of the ApproxInt templates: the walking line whose points
//! are approximated (docs/geomalgo-design.md section 20).
use crate::int_patch::{IntPatchPointLine, IntPatchWLine};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};

/// The calls the ApproxInt templates make on `TheLine`: `NbPnts()`, `Point(i)` and
/// `new TheLine(LineOn2S, false)`. `IntPatch_WLine` implements it for GeomInt, as
/// `BRepApprox_ApproxLine` does for BRepApprox.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeomalgo::approx_int::ApproxIntLine;
/// use rustycad_tkgeomalgo::int_patch::IntPatchWLine;
/// use rustycad_tkgeomalgo::int_surf::IntSurfLineOn2S;
/// let line = IntPatchWLine::from_points(&Handle::new(IntSurfLineOn2S::new(None)), false);
/// assert_eq!(ApproxIntLine::nb_pnts(&line)?, 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait ApproxIntLine: Transient + RuntimeTyped + Sized {
    /// `NbPnts()`.
    fn nb_pnts(&self) -> Result<i32, Failure>;
    /// `Point(Index)`: a copy of the point of range `index`.
    fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure>;
    /// `TheLine(Line, Tang)`: the line of the points of `line`.
    fn from_points(line: &Handle<IntSurfLineOn2S>, tang: bool) -> Self;
}

impl ApproxIntLine for IntPatchWLine {
    fn nb_pnts(&self) -> Result<i32, Failure> {
        IntPatchPointLine::nb_pnts(self)
    }
    fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        IntPatchPointLine::point(self, index)
    }
    fn from_points(line: &Handle<IntSurfLineOn2S>, tang: bool) -> Self {
        IntPatchWLine::new_undecided(line, tang)
    }
}
