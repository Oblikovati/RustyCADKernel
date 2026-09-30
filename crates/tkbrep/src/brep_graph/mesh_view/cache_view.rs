// Rust translation of BRepGraph::MeshView::CacheView of
// ModelingData/TKBRep/BRepGraph/BRepGraph_MeshView.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MeshView::CacheView`: reads of fresh default-slot mesh cache entries.
use super::cache_mesh_of;
use crate::brep_graph::cache_mesh::{
    CoEdgeMeshEntry, DEFAULT_DISPLAY_SLOT, EdgeMeshEntry, FaceMeshEntry,
};
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::poly::{Polygon3D, Triangulation};

/// `MeshView::CacheView`: the face, edge and coedge cache queries.
#[derive(Clone, Copy)]
pub struct CacheView<'g> {
    graph: &'g BRepGraph,
}

impl<'g> CacheView<'g> {
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

/// `CacheView::FaceOps`.
#[derive(Clone, Copy)]
pub struct FaceOps<'g> {
    graph: &'g BRepGraph,
}

impl FaceOps<'_> {
    /// `Has(theFace)`: `Entry(theFace)` is not null.
    pub fn has(self, face: FaceId) -> bool {
        self.entry(face).is_some()
    }

    /// `Triangulation(theFace)`: the cached triangulation, a null handle when absent.
    pub fn triangulation(self, face: FaceId) -> Handle<Triangulation> {
        self.entry(face)
            .map(|entry| entry.triangulation)
            .unwrap_or_default()
    }

    /// `Entry(theFace)`: a copy of the fresh default-slot entry.
    pub fn entry(self, face: FaceId) -> Option<FaceMeshEntry> {
        cache_mesh_of(self.graph)?.find_face_mesh(self.graph, face)
    }
}

/// `CacheView::EdgeOps`.
#[derive(Clone, Copy)]
pub struct EdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl EdgeOps<'_> {
    /// `Has(theEdge)`: a fresh entry with a 3D polygon.
    pub fn has(self, edge: EdgeId) -> bool {
        self.entry(edge)
            .is_some_and(|entry| entry.polygon3d.is_some())
    }

    /// `Polygon3D(theEdge)`: the cached 3D polygon, a null handle when absent.
    pub fn polygon3d(self, edge: EdgeId) -> Handle<Polygon3D> {
        self.entry(edge)
            .map(|entry| entry.polygon3d)
            .unwrap_or_default()
    }

    /// `Entry(theEdge)`: a copy of the fresh default-slot entry.
    pub fn entry(self, edge: EdgeId) -> Option<EdgeMeshEntry> {
        cache_mesh_of(self.graph)?.find_edge_mesh(self.graph, edge)
    }
}

/// `CacheView::CoEdgeOps`.
#[derive(Clone, Copy)]
pub struct CoEdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl CoEdgeOps<'_> {
    /// `Has(theCoEdge)`: `HasCoEdgeMesh` of the default slot.
    pub fn has(self, co_edge: CoEdgeId) -> bool {
        cache_mesh_of(self.graph).is_some_and(|cache| cache.has_co_edge_mesh(self.graph, co_edge))
    }

    /// `FindPolygon2D(theCoEdge)`: the entry when its 2D polygon side is fresh.
    pub fn find_polygon2d(self, co_edge: CoEdgeId) -> Option<CoEdgeMeshEntry> {
        cache_mesh_of(self.graph)?.find_co_edge_polygon2d(self.graph, co_edge)
    }

    /// `FindPolygonOnTri(theCoEdge)`: the entry when its polygons on triangulation are
    /// fresh.
    pub fn find_polygon_on_tri(self, co_edge: CoEdgeId) -> Option<CoEdgeMeshEntry> {
        cache_mesh_of(self.graph)?.find_co_edge_polygon_on_tri(self.graph, co_edge)
    }

    /// `FindRaw(theCoEdge)`: the default-slot entry with a representation, fresh or not.
    pub fn find_raw(self, co_edge: CoEdgeId) -> Option<CoEdgeMeshEntry> {
        cache_mesh_of(self.graph)?.find_co_edge_entry_raw(DEFAULT_DISPLAY_SLOT, co_edge)
    }
}
