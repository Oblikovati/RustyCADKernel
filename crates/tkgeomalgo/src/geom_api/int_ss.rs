// Created on: 1995-03-14
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomAPI/GeomAPI_IntSS.hxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_IntSS`: the intersection curves of two Geom surfaces, the approximated
//! 3d curves of `GeomInt_IntSS` without their 2d curves.
use crate::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};

/// `GeomAPI_IntSS`: the intersection of two surfaces with approximated walking lines
/// and no 2d curves. For most cases the tolerance `1.0e-7` is recommended.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::IntSS;
/// let inter = IntSS::new();
/// assert!(!inter.is_done());
/// assert_eq!(inter.nb_lines(), 0);
/// assert!(inter.line(1).is_err());
/// ```
#[derive(Default)]
pub struct IntSS {
    my_intersec: GeomIntIntSS,
}

/// The flags of `GeomInt_IntSS` that `GeomAPI_IntSS` passes (`Approx` true only).
fn approx_flags() -> GeomIntIntSSApprox {
    GeomIntIntSSApprox {
        approx: true,
        approx_s1: false,
        approx_s2: false,
    }
}

impl IntSS {
    /// `GeomAPI_IntSS()`: empty; `IntSS::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `GeomAPI_IntSS(S1, S2, Tol)`: the intersection curves of `s1` and `s2` computed
    /// with the precision `tol`; `IntSS::with_surfaces(&s1, &s2, 1.0e-7)?`.
    pub fn with_surfaces(
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        Ok(Self {
            my_intersec: GeomIntIntSS::with_surfaces(s1, s2, tol, approx_flags())?,
        })
    }

    /// `Perform(S1, S2, Tol)`: computes the intersection curves again;
    /// `inter.perform(&s1, &s2, 1.0e-7)?`.
    pub fn perform(
        &mut self,
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        tol: f64,
    ) -> Result<(), Failure> {
        self.my_intersec.perform(s1, s2, tol, approx_flags())
    }

    /// `IsDone()`: whether the intersection was successful; `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_intersec.is_done()
    }

    /// `NbLines()`: the number of intersection curves (not checked for a failed
    /// computation, as natively); `inter.nb_lines()`.
    pub fn nb_lines(&self) -> i32 {
        self.my_intersec.nb_lines()
    }

    /// `Line(Index)`: the intersection curve `index` (StdFail_NotDone when the
    /// computation failed, Standard_OutOfRange outside `1..=NbLines`);
    /// `inter.line(1)?`.
    pub fn line(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.my_intersec.line(index)
    }
}
