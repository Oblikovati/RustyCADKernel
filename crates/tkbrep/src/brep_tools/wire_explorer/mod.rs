// Created on: 1993-01-21
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
// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_WireExplorer.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_WireExplorer`: the edges of a wire in their connection order.
mod init;
mod next;
mod select;

use crate::topods::{Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use crate::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// `NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>`:
/// the edges that start at each vertex, in wire order.
type EdgesByVertex = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;

/// Explores the edges of a wire following the connectivity of their vertices
/// (`BRepTools_WireExplorer`). With a face, the p-curves on the face choose the next
/// edge at a vertex shared by several edges.
///
/// ```
/// use rustycad_tkbrep::brep_tools::WireExplorer;
/// use rustycad_tkbrep::topods::Wire;
/// let explorer = WireExplorer::with_wire(&Wire::default())?;
/// assert!(!explorer.more());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct WireExplorer {
    map: EdgesByVertex,
    edge: Edge,
    vertex: Vertex,
    face: Face,
    doubles: MapOfShape,
    reverse: bool,
    tol_u: f64,
    tol_v: f64,
}

impl WireExplorer {
    /// `BRepTools_WireExplorer()`: an explorer with nothing to explore;
    /// `WireExplorer::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepTools_WireExplorer(W)`; `WireExplorer::with_wire(&wire)?`.
    pub fn with_wire(wire: &Wire) -> Result<Self, Failure> {
        let mut explorer = Self::default();
        explorer.init(wire)?;
        Ok(explorer)
    }

    /// `BRepTools_WireExplorer(W, F)`; `WireExplorer::with_wire_on_face(&wire, &face)?`.
    pub fn with_wire_on_face(wire: &Wire, face: &Face) -> Result<Self, Failure> {
        let mut explorer = Self::default();
        explorer.init_on_face(wire, face)?;
        Ok(explorer)
    }

    /// `More()`: the current edge is not null; `explorer.more()`.
    pub fn more(&self) -> bool {
        !self.edge.is_null()
    }

    /// `Current()`: the current edge; `explorer.current()`.
    pub fn current(&self) -> &Edge {
        &self.edge
    }

    /// `CurrentVertex()`: the vertex the current edge starts from; `explorer.current_vertex()`.
    pub fn current_vertex(&self) -> &Vertex {
        &self.vertex
    }

    /// `Orientation()`: the orientation of the current vertex in the current edge
    /// (FORWARD for an infinite edge without vertex); Standard_NoSuchObject when the
    /// edge does not contain it; `explorer.orientation()?`.
    pub fn orientation(&self) -> Result<Orientation, Failure> {
        if self.vertex.is_null() && !self.edge.is_null() {
            return Ok(Orientation::Forward);
        }
        for child in ShapeIterator::new(&self.edge, false, true) {
            if self.vertex.is_same(&child) {
                return Ok(child.orientation());
            }
        }
        Err(Failure::with_type(
            "Standard_NoSuchObject",
            "BRepTools_WireExplorer::Orientation: the current vertex is not in the current edge; expected a vertex of the edge",
            "",
        ))
    }

    /// `Clear()`: forgets the wire, the face and the current edge and vertex;
    /// `explorer.clear()`.
    pub fn clear(&mut self) {
        self.map.clear(true);
        self.doubles.clear(true);
        self.edge = Edge::default();
        self.face = Face::default();
        self.vertex = Vertex::default();
    }
}
