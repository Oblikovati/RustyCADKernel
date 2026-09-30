// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Line.hxx/.cxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Line`: the line of a skinning, the indices of its sections.
use crate::app_blend::TheLine;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::StandardTransient;

/// The `TheLine` of `GeomFill_AppSurf`: `NbPoints` sections whose points are their
/// indices (`GeomFill_Line`); `Handle::new(GeomFillLine::with_nb_points(2))`.
#[derive(Default)]
pub struct GeomFillLine {
    parent: StandardTransient,
    my_nb_points: i32,
}
impl_transient!(GeomFillLine, "GeomFill_Line", StandardTransient, parent);

impl GeomFillLine {
    /// A line without points (`GeomFill_Line()`); `GeomFillLine::new().nb_points() == 0`.
    pub fn new() -> Self {
        Self::default()
    }
    /// A line of `nb_points` sections (`GeomFill_Line(NbPoints)`);
    /// `GeomFillLine::with_nb_points(3).point(2) == 2`.
    pub fn with_nb_points(nb_points: i32) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_nb_points: nb_points,
        }
    }
    /// `NbPoints()`; `line.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.my_nb_points
    }
    /// `Point(Index)`: the index itself, whatever its range; `line.point(5) == 5`.
    pub fn point(&self, index: i32) -> i32 {
        index
    }
}

impl TheLine for GeomFillLine {
    type Point = i32;
    fn nb_points(&self) -> i32 {
        self.my_nb_points
    }
    fn point(&self, index: i32) -> i32 {
        index
    }
}
