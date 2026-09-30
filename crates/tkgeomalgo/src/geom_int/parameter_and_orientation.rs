// Created on: 1995-02-08
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_ParameterAndOrientation.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomInt_ParameterAndOrientation`: a parameter on a restriction line with the
//! orientations of the line on both surfaces there.
use rustycad_tkg3d::top_abs::Orientation;

/// `GeomInt_ParameterAndOrientation`: a parameter and two orientations, FORWARD and 0
/// by default.
///
/// ```
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkgeomalgo::geom_int::GeomIntParameterAndOrientation;
/// let mut p = GeomIntParameterAndOrientation::with_values(0.5, Orientation::Forward, Orientation::Reversed);
/// p.set_orientation1(Orientation::Internal);
/// assert_eq!((p.parameter(), p.orientation1()), (0.5, Orientation::Internal));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeomIntParameterAndOrientation {
    prm: f64,
    or1: Orientation,
    or2: Orientation,
}

impl Default for GeomIntParameterAndOrientation {
    fn default() -> Self {
        Self::new()
    }
}

impl GeomIntParameterAndOrientation {
    /// `GeomInt_ParameterAndOrientation()`: 0 and FORWARD twice;
    /// `GeomIntParameterAndOrientation::new()`.
    pub fn new() -> Self {
        Self::with_values(0.0, Orientation::Forward, Orientation::Forward)
    }

    /// `GeomInt_ParameterAndOrientation(P, Or1, Or2)`;
    /// `GeomIntParameterAndOrientation::with_values(1.0, or1, or2)`.
    pub fn with_values(p: f64, or1: Orientation, or2: Orientation) -> Self {
        Self { prm: p, or1, or2 }
    }

    /// `SetOrientation1(Or)`; `p.set_orientation1(Orientation::Internal)`.
    pub fn set_orientation1(&mut self, or: Orientation) {
        self.or1 = or;
    }

    /// `SetOrientation2(Or)`; `p.set_orientation2(Orientation::Internal)`.
    pub fn set_orientation2(&mut self, or: Orientation) {
        self.or2 = or;
    }

    /// `Parameter()`; `p.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.prm
    }

    /// `Orientation1()`; `p.orientation1()`.
    pub fn orientation1(&self) -> Orientation {
        self.or1
    }

    /// `Orientation2()`; `p.orientation2()`.
    pub fn orientation2(&self) -> Orientation {
        self.or2
    }
}
