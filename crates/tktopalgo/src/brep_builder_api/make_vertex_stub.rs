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
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepBuilderAPI/BRepBuilderAPI_MakeVertex.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! STUB of `BRepBuilderAPI_MakeVertex`, not its port: the minimal vertex builder that
//! the TKernel tests use (see the `brep_builder_api` module documentation). The real
//! port of BRepBuilderAPI replaces this file.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Shape, Vertex},
};
use rustycad_tkmath::gp::Point3;
/// STUB: builds a vertex using BRepLib's default precision;
/// `MakeVertex::new(Point3::default())`.
#[derive(Clone, Debug)]
pub struct MakeVertex {
    vertex: Vertex,
}
impl MakeVertex {
    /// Uses the native default construction precision; `MakeVertex::new(point)`.
    pub fn new(point: Point3) -> Self {
        Self::with_precision(point, 1e-7)
    }
    /// Injects the BRepLib precision instead of mutating process globals:
    /// `BRep_Builder::MakeVertex(V, P, BRepLib::Precision())`; `MakeVertex::with_precision(point,1e-6)`.
    pub fn with_precision(point: Point3, precision: f64) -> Self {
        let mut vertex = Vertex::default();
        // MakeVertex fails only for a locked TShape or a singular vertex location; a new
        // vertex has neither.
        BRepBuilder::default()
            .make_vertex_at(&mut vertex, point, precision)
            .expect("a new vertex is unlocked and at the identity location");
        Self { vertex }
    }
    /// Vertex construction completes synchronously; `builder.is_done()`.
    pub const fn is_done(&self) -> bool {
        true
    }
    /// Borrows the built vertex; `builder.vertex()`.
    pub const fn vertex(&self) -> &Vertex {
        &self.vertex
    }
    /// Borrows the shared shape; `builder.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.vertex
    }
}
