// Created on: 1993-07-23
// Created by: Remi LEQUETTE
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
// Rust translation of BRepLib_MakeVertex.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeVertex`: a vertex at a point with the default precision.
use super::globals;
use super::make_shape::BRepLibMakeShape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use std::ops::{Deref, DerefMut};

/// `BRepLib_MakeVertex` (docs/topalgo-design.md section 7): a vertex at a point, of
/// tolerance `BRepLib::Precision()`; always done.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;
/// let maker = BRepLibMakeVertex::new(Point3::new(1.0, 2.0, 3.0)).unwrap();
/// assert!(maker.is_done());
/// assert!(!maker.vertex().unwrap().is_null());
/// ```
#[derive(Clone)]
pub struct BRepLibMakeVertex {
    base: BRepLibMakeShape,
}

impl Deref for BRepLibMakeVertex {
    type Target = BRepLibMakeShape;
    fn deref(&self) -> &BRepLibMakeShape {
        &self.base
    }
}

impl DerefMut for BRepLibMakeVertex {
    fn deref_mut(&mut self) -> &mut BRepLibMakeShape {
        &mut self.base
    }
}

impl BRepLibMakeVertex {
    /// `BRepLib_MakeVertex(P)`: `BRep_Builder::MakeVertex(V, P, BRepLib::Precision())`;
    /// `BRepLibMakeVertex::new(point)?`.
    pub fn new(point: Point3) -> Result<Self, Failure> {
        let mut base = BRepLibMakeShape::new();
        let mut vertex = Vertex::default();
        BRepBuilder::default().make_vertex_at(&mut vertex, point, globals::precision())?;
        base.my_shape = vertex.into_shape();
        base.set_done();
        Ok(Self { base })
    }

    /// `Vertex()` and `operator TopoDS_Vertex()`: the vertex; `maker.vertex()?`.
    pub fn vertex(&self) -> Result<Vertex, Failure> {
        topods::vertex(self.shape()?)
    }
}
