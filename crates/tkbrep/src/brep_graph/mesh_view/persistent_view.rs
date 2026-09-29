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
// Rust translation of BRepGraph::MeshView::PersistentView of
// ModelingData/TKBRep/BRepGraph/BRepGraph_MeshView.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MeshView::PersistentView`: the mesh representations the definitions own.
use crate::brep_graph::tool;
use crate::brep_graph::{
    BRepGraph, CoEdgeId, CoEdgePolygon2DRepId, CoEdgePolygonOnTriRepId, EdgeId, EdgePolygon3DRepId,
    FaceId, FaceTriangulationRepId,
};
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};

/// `MeshView::PersistentView`: the face, edge and coedge persistent queries.
#[derive(Clone, Copy)]
pub struct PersistentView<'g> {
    graph: &'g BRepGraph,
}

impl<'g> PersistentView<'g> {
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

/// `PersistentView::FaceOps`.
#[derive(Clone, Copy)]
pub struct FaceOps<'g> {
    graph: &'g BRepGraph,
}

impl FaceOps<'_> {
    /// `Has(theFace)`: an active face with an active triangulation representation.
    pub fn has(self, face: FaceId) -> bool {
        self.rep(face).is_some()
    }

    /// `Triangulation(theFace)`: the persistent triangulation, a null handle when
    /// absent.
    pub fn triangulation(self, face: FaceId) -> Handle<Triangulation> {
        let storage = self.graph.inc_storage();
        self.rep(face)
            .map(|rep| storage.face_triangulation_rep(rep).triangulation.clone())
            .unwrap_or_default()
    }

    fn rep(self, face: FaceId) -> Option<FaceTriangulationRepId> {
        let storage = self.graph.inc_storage();
        if !face.is_valid_for(storage.nb_faces()) || storage.is_removed(face) {
            return None;
        }
        let rep = storage.face(face).triangulation_rep_id;
        active_rep(storage, rep, storage.nb_face_triangulations())
    }
}

/// `PersistentView::EdgeOps`.
#[derive(Clone, Copy)]
pub struct EdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl EdgeOps<'_> {
    /// `Has(theEdge)`: an active edge with an active 3D polygon representation.
    pub fn has(self, edge: EdgeId) -> bool {
        self.rep(edge).is_some()
    }

    /// `Polygon3D(theEdge)`: the persistent 3D polygon, a null handle when absent.
    pub fn polygon3d(self, edge: EdgeId) -> Handle<Polygon3D> {
        let storage = self.graph.inc_storage();
        self.rep(edge)
            .map(|rep| storage.edge_polygon3d_rep(rep).polygon.clone())
            .unwrap_or_default()
    }

    /// `HasPolygonOnTriangulation(theEdge, theFace)`: the (edge, face) coedge has an
    /// active polygon-on-triangulation representation.
    pub fn has_polygon_on_triangulation(self, edge: EdgeId, face: FaceId) -> bool {
        self.polygon_on_tri_rep(edge, face).is_some()
    }

    /// `PolygonOnTriangulation(theEdge, theFace)`: the (edge, face) coedge's polygon on
    /// triangulation, a null handle when absent.
    pub fn polygon_on_triangulation(
        self,
        edge: EdgeId,
        face: FaceId,
    ) -> Handle<PolygonOnTriangulation> {
        let storage = self.graph.inc_storage();
        self.polygon_on_tri_rep(edge, face)
            .map(|rep| storage.co_edge_polygon_on_tri_rep(rep).polygon.clone())
            .unwrap_or_default()
    }

    fn rep(self, edge: EdgeId) -> Option<EdgePolygon3DRepId> {
        let storage = self.graph.inc_storage();
        if !edge.is_valid_for(storage.nb_edges()) || storage.is_removed(edge) {
            return None;
        }
        let rep = storage.edge(edge).polygon3d_rep_id;
        active_rep(storage, rep, storage.nb_edge_polygons3d())
    }

    fn polygon_on_tri_rep(self, edge: EdgeId, face: FaceId) -> Option<CoEdgePolygonOnTriRepId> {
        let co_edge = tool::edge::find_co_edge_id(self.graph, edge, face);
        if !co_edge.is_valid() {
            return None;
        }
        let storage = self.graph.inc_storage();
        let rep = storage.co_edge(co_edge).polygon_on_tri_rep_id;
        active_rep(storage, rep, storage.nb_co_edge_polygons_on_tri())
    }
}

/// `PersistentView::CoEdgeOps`: the coedge itself is only range-checked, as natively.
#[derive(Clone, Copy)]
pub struct CoEdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl CoEdgeOps<'_> {
    /// `Has(theCoEdge)`: an active polygon-on-surface representation.
    pub fn has(self, co_edge: CoEdgeId) -> bool {
        self.polygon2d_rep(co_edge).is_some()
    }

    /// `PolygonOnSurface(theCoEdge)`: the persistent 2D polygon, a null handle when
    /// absent.
    pub fn polygon_on_surface(self, co_edge: CoEdgeId) -> Handle<Polygon2D> {
        let storage = self.graph.inc_storage();
        self.polygon2d_rep(co_edge)
            .map(|rep| storage.co_edge_polygon2d_rep(rep).polygon.clone())
            .unwrap_or_default()
    }

    /// `HasPolygonOnTriangulation(theCoEdge)`.
    pub fn has_polygon_on_triangulation(self, co_edge: CoEdgeId) -> bool {
        self.polygon_on_tri_rep(co_edge).is_some()
    }

    /// `PolygonOnTriangulation(theCoEdge)`: the persistent polygon on triangulation, a
    /// null handle when absent.
    pub fn polygon_on_triangulation(self, co_edge: CoEdgeId) -> Handle<PolygonOnTriangulation> {
        let storage = self.graph.inc_storage();
        self.polygon_on_tri_rep(co_edge)
            .map(|rep| storage.co_edge_polygon_on_tri_rep(rep).polygon.clone())
            .unwrap_or_default()
    }

    fn polygon2d_rep(self, co_edge: CoEdgeId) -> Option<CoEdgePolygon2DRepId> {
        let storage = self.graph.inc_storage();
        if !co_edge.is_valid_for(storage.nb_co_edges()) {
            return None;
        }
        let rep = storage.co_edge(co_edge).polygon2d_rep_id;
        active_rep(storage, rep, storage.nb_co_edge_polygons2d())
    }

    fn polygon_on_tri_rep(self, co_edge: CoEdgeId) -> Option<CoEdgePolygonOnTriRepId> {
        let storage = self.graph.inc_storage();
        if !co_edge.is_valid_for(storage.nb_co_edges()) {
            return None;
        }
        let rep = storage.co_edge(co_edge).polygon_on_tri_rep_id;
        active_rep(storage, rep, storage.nb_co_edge_polygons_on_tri())
    }
}

// A representation id in range and not removed.
fn active_rep<T: crate::brep_graph::RepTag>(
    storage: &Storage,
    rep: crate::brep_graph::TypedId<T>,
    count: u32,
) -> Option<crate::brep_graph::TypedId<T>> {
    (rep.is_valid_for(count) && !storage.is_removed(rep)).then_some(rep)
}
