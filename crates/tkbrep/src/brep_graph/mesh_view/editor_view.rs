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
// Rust translation of BRepGraph::MeshView::EditorView (the cache writes) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_MeshView.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MeshView::EditorView`: writes to the default slot of the mesh cache. Persistent
//! representations are written by the graph editor, not here.
use super::cache_mesh_of;
use crate::brep_graph::cache_mesh::DEFAULT_DISPLAY_SLOT;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};

/// `MeshView::EditorView`: the face, edge and coedge cache mutations.
#[derive(Clone, Copy)]
pub struct EditorView<'g> {
    graph: &'g BRepGraph,
}

impl<'g> EditorView<'g> {
    pub(super) fn new(graph: &'g BRepGraph) -> Self {
        Self { graph }
    }

    /// `Faces()`.
    pub fn faces(self) -> FaceOps<'g> {
        FaceOps { graph: self.graph }
    }

    /// `Edges()`.
    pub fn edges(self) -> EdgeOps<'g> {
        EdgeOps { graph: self.graph }
    }

    /// `CoEdges()`.
    pub fn co_edges(self) -> CoEdgeOps<'g> {
        CoEdgeOps { graph: self.graph }
    }
}

/// `EditorView::FaceOps`.
#[derive(Clone, Copy)]
pub struct FaceOps<'g> {
    graph: &'g BRepGraph,
}

impl FaceOps<'_> {
    /// `SetCachedTriangulation(theFace, theTriangulation)`: a face in range gets the
    /// triangulation (null clears it), bound fresh, and its mesh generation bumped.
    pub fn set_cached_triangulation(
        self,
        face: FaceId,
        triangulation: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        if !face.is_valid_for(self.graph.inc_storage().nb_faces()) {
            return Ok(());
        }
        self.rewrite(face, |entry| entry.triangulation = triangulation.clone())
    }

    /// `Clear(theFace)`: drops the cached triangulation (the entry is created when
    /// absent), binds it fresh and bumps the mesh generation, so the coedge polygons on
    /// triangulation go stale; the 2D polygons stay valid. Standard_OutOfRange for the
    /// invalid id (natively a 2^32-entry allocation).
    pub fn clear(self, face: FaceId) -> Result<(), Failure> {
        self.rewrite(face, |entry| entry.clear_representation())
    }

    fn rewrite(
        self,
        face: FaceId,
        change: impl FnOnce(&mut crate::brep_graph::cache_mesh::FaceMeshEntry),
    ) -> Result<(), Failure> {
        let Some(cache) = cache_mesh_of(self.graph) else {
            return Ok(());
        };
        cache.write_face_fresh(self.graph, face, change)?;
        cache.bump_face_mesh_generation(face, DEFAULT_DISPLAY_SLOT)
    }
}

/// `EditorView::EdgeOps`.
#[derive(Clone, Copy)]
pub struct EdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl EdgeOps<'_> {
    /// `SetCachedPolygon3D(theEdge, thePolygon3D)`: an edge in range gets the polygon
    /// (null clears it), bound fresh.
    pub fn set_cached_polygon3d(
        self,
        edge: EdgeId,
        polygon: &Handle<Polygon3D>,
    ) -> Result<(), Failure> {
        if !edge.is_valid_for(self.graph.inc_storage().nb_edges()) {
            return Ok(());
        }
        let Some(cache) = cache_mesh_of(self.graph) else {
            return Ok(());
        };
        cache.write_edge_fresh(self.graph, edge, |entry| entry.polygon3d = polygon.clone())
    }

    /// `Clear(theEdge)`: resets the edge's entry.
    pub fn clear(self, edge: EdgeId) {
        if let Some(cache) = cache_mesh_of(self.graph) {
            cache.clear_edge_mesh(edge);
        }
    }
}

/// `EditorView::CoEdgeOps`.
#[derive(Clone, Copy)]
pub struct CoEdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl CoEdgeOps<'_> {
    /// `AppendCachedPolygonOnTri(theCoEdge, thePolygonOnTri)`: a coedge in range
    /// appends a non-null polygon, bound fresh.
    pub fn append_cached_polygon_on_tri(
        self,
        co_edge: CoEdgeId,
        polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        if !self.in_range(co_edge) || polygon.is_null() {
            return Ok(());
        }
        let Some(cache) = cache_mesh_of(self.graph) else {
            return Ok(());
        };
        cache.write_co_edge_fresh(self.graph, co_edge, |entry| {
            entry.polygons_on_tri.push(polygon.clone())
        })
    }

    /// `SetCachedPolygon2D(theCoEdge, thePolygon2D)`: a coedge in range gets the
    /// polygon (null clears it), bound fresh.
    pub fn set_cached_polygon2d(
        self,
        co_edge: CoEdgeId,
        polygon: &Handle<Polygon2D>,
    ) -> Result<(), Failure> {
        if !self.in_range(co_edge) {
            return Ok(());
        }
        let Some(cache) = cache_mesh_of(self.graph) else {
            return Ok(());
        };
        cache.write_co_edge_fresh(self.graph, co_edge, |entry| {
            entry.polygon2d = polygon.clone()
        })
    }

    /// `Clear(theCoEdge)`: resets the coedge's entry.
    pub fn clear(self, co_edge: CoEdgeId) {
        if let Some(cache) = cache_mesh_of(self.graph) {
            cache.clear_co_edge_mesh(co_edge);
        }
    }

    fn in_range(self, co_edge: CoEdgeId) -> bool {
        co_edge.is_valid_for(self.graph.inc_storage().nb_co_edges())
    }
}
