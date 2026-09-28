// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of the vertex members of ModelingData/TKBRep/BRep/BRep_Builder.cxx,
// BRep_TVertex.cxx and BRep_Tool.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex subset of BRep that the TKernel legacy TopLoc test uses.
//!
//! This module predates the TKBRep port (commit fde0c95) and is not the BRep package:
//! it keeps only a vertex TShape with a point and a tolerance, now derived from
//! [`topods::TVertex`], so that `BRepBuilderAPI_MakeVertex` of the TKTopAlgo crate and
//! the TopLoc legacy test run on the real TopoDS types. The BRep package itself
//! (representations, `BRep_Builder`, `BRep_Tool`) is not ported yet.
use crate::topods::{self, Builder as TopoDSBuilder, Shape, TShape, TShapeBase, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;

/// The point and tolerance of `BRep_TVertex` (without its point representations).
pub struct TVertex {
    parent: topods::TVertex,
    point: Point3,
    tolerance: f64,
}
impl_transient!(TVertex, "BRep_TVertex", topods::TVertex, parent, interfaces: [dyn TShape]);

impl TVertex {
    fn new(point: Point3, tolerance: f64) -> Self {
        Self {
            parent: topods::TVertex::new(),
            point,
            tolerance,
        }
    }
}

impl TShape for TVertex {
    fn tshape_base(&self) -> &TShapeBase {
        self.parent.tshape_base()
    }
    /// `BRep_TVertex::EmptyCopy`: a new vertex with the same point and tolerance.
    fn empty_copy(&self) -> Handle<dyn TShape> {
        Handle::new(Self::new(self.point, self.tolerance)).into_cast()
    }
}

/// BRep_Builder's vertex construction entry point; `Builder::make_vertex(point, 1e-7)`.
pub struct Builder;

impl Builder {
    /// `BRep_Builder::MakeVertex(V, P, Tol)` with the tolerance floored at the machine
    /// epsilon; `Builder::make_vertex(Point3::default(), 1e-7)`.
    pub fn make_vertex(point: Point3, tolerance: f64) -> Vertex {
        let tolerance = if tolerance > f64::EPSILON {
            tolerance
        } else {
            f64::EPSILON
        };
        let tshape: Handle<dyn TShape> = Handle::new(TVertex::new(point, tolerance)).into_cast();
        let mut vertex = Vertex::default();
        TopoDSBuilder.make_shape(&mut vertex, &tshape);
        vertex
    }
}

/// BRep_Tool vertex queries; `Tool::point(&vertex)?`.
pub struct Tool;

impl Tool {
    /// `BRep_Tool::Pnt(V)`: the point moved by the shape location; `Tool::point(&vertex)?`.
    pub fn point(vertex: &Vertex) -> Result<Point3, Failure> {
        let geometry = vertex_geometry(vertex)?;
        Ok(vertex.location().transformation().apply(geometry.point))
    }

    /// `BRep_Tool::Tolerance(V)`: the stored tolerance, floored at `Precision::Confusion()`;
    /// `Tool::tolerance(&vertex)?`.
    pub fn tolerance(vertex: &Vertex) -> Result<f64, Failure> {
        Ok(vertex_geometry(vertex)?
            .tolerance
            .max(rustycad_tkernel::precision::confusion()))
    }
}

fn vertex_geometry(vertex: &Shape) -> Result<&TVertex, Failure> {
    let tshape = vertex.tshape().get();
    let geometry = tshape.and_then(|tshape| {
        let object: &dyn rustycad_tkernel::standard::Transient = tshape;
        rustycad_tkernel::standard::project_concrete::<TVertex>(object)
    });
    geometry.ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRep_Tool:: TopoDS_Vertex hasn't gp_Pnt",
            "",
        )
    })
}
