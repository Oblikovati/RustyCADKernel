// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of BRepGraph::MeshView::PolyOps of
// ModelingData/TKBRep/BRepGraph/BRepGraph_MeshView.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MeshView::PolyOps`: the counts of the mesh representation stores.
use crate::brep_graph::BRepGraph;

/// `MeshView::PolyOps`: slot counts (removed records included) and active counts.
#[derive(Clone, Copy)]
pub struct PolyOps<'g> {
    graph: &'g BRepGraph,
}

impl<'g> PolyOps<'g> {
    pub(super) fn new(graph: &'g BRepGraph) -> Self {
        Self { graph }
    }

    /// `NbFaceTriangulations()`.
    pub fn nb_face_triangulations(self) -> u32 {
        self.graph.inc_storage().nb_face_triangulations()
    }

    /// `NbEdgePolygons3D()`.
    pub fn nb_edge_polygons3d(self) -> u32 {
        self.graph.inc_storage().nb_edge_polygons3d()
    }

    /// `NbCoEdgePolygons2D()`.
    pub fn nb_co_edge_polygons2d(self) -> u32 {
        self.graph.inc_storage().nb_co_edge_polygons2d()
    }

    /// `NbCoEdgePolygonsOnTri()`.
    pub fn nb_co_edge_polygons_on_tri(self) -> u32 {
        self.graph.inc_storage().nb_co_edge_polygons_on_tri()
    }

    /// `NbActiveTriangulations()`.
    pub fn nb_active_triangulations(self) -> u32 {
        self.graph.inc_storage().nb_active_face_triangulations()
    }

    /// `NbActivePolygons3D()`.
    pub fn nb_active_polygons3d(self) -> u32 {
        self.graph.inc_storage().nb_active_edge_polygons3d()
    }

    /// `NbActivePolygons2D()`.
    pub fn nb_active_polygons2d(self) -> u32 {
        self.graph.inc_storage().nb_active_co_edge_polygons2d()
    }

    /// `NbActivePolygonsOnTri()`.
    pub fn nb_active_polygons_on_tri(self) -> u32 {
        self.graph.inc_storage().nb_active_co_edge_polygons_on_tri()
    }
}
