// Rust translation of BRepGraph::MeshView::EffectiveView of
// ModelingData/TKBRep/BRepGraph/BRepGraph_MeshView.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MeshView::EffectiveView`: the active display slot of the cache first, the
//! persistent representations second.
use super::cache_mesh_of;
use crate::brep_graph::cache_mesh::{CacheMesh, CoEdgeMeshEntry, SlotId};
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};

/// `MeshView::EffectiveView`: the face, edge and coedge effective queries.
#[derive(Clone, Copy)]
pub struct EffectiveView<'g> {
    graph: &'g BRepGraph,
}

impl<'g> EffectiveView<'g> {
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

// The mesh cache and its active display slot.
fn active_cache(graph: &BRepGraph) -> Option<(Handle<CacheMesh>, SlotId)> {
    let cache = cache_mesh_of(graph)?;
    let slot = cache.active_display_slot();
    Some((cache, slot))
}

/// `EffectiveView::FaceOps`.
#[derive(Clone, Copy)]
pub struct FaceOps<'g> {
    graph: &'g BRepGraph,
}

impl FaceOps<'_> {
    /// `Has(theFace)`: a fresh entry in the active slot, or a persistent triangulation.
    pub fn has(self, face: FaceId) -> bool {
        let cached = active_cache(self.graph)
            .is_some_and(|(cache, slot)| cache.find_face_mesh_in(self.graph, slot, face).is_some());
        cached || self.graph.mesh().persistent().faces().has(face)
    }

    /// `Triangulation(theFace)`: the cached triangulation, else the persistent one.
    pub fn triangulation(self, face: FaceId) -> Handle<Triangulation> {
        let cached = active_cache(self.graph)
            .and_then(|(cache, slot)| cache.find_face_mesh_in(self.graph, slot, face))
            .map(|entry| entry.triangulation)
            .filter(Handle::is_some);
        cached.unwrap_or_else(|| self.graph.mesh().persistent().faces().triangulation(face))
    }
}

/// `EffectiveView::EdgeOps`.
#[derive(Clone, Copy)]
pub struct EdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl EdgeOps<'_> {
    /// `Has(theEdge)`: a fresh entry in the active slot, or a persistent 3D polygon.
    pub fn has(self, edge: EdgeId) -> bool {
        let cached = active_cache(self.graph)
            .is_some_and(|(cache, slot)| cache.find_edge_mesh_in(self.graph, slot, edge).is_some());
        cached || self.graph.mesh().persistent().edges().has(edge)
    }

    /// `Polygon3D(theEdge)`: the cached 3D polygon, else the persistent one.
    pub fn polygon3d(self, edge: EdgeId) -> Handle<Polygon3D> {
        let cached = active_cache(self.graph)
            .and_then(|(cache, slot)| cache.find_edge_mesh_in(self.graph, slot, edge))
            .map(|entry| entry.polygon3d)
            .filter(Handle::is_some);
        cached.unwrap_or_else(|| self.graph.mesh().persistent().edges().polygon3d(edge))
    }
}

/// `EffectiveView::CoEdgeOps`.
#[derive(Clone, Copy)]
pub struct CoEdgeOps<'g> {
    graph: &'g BRepGraph,
}

impl CoEdgeOps<'_> {
    /// `Has(theCoEdge)`: a polygon on surface or a polygon on triangulation.
    pub fn has(self, co_edge: CoEdgeId) -> bool {
        self.has_polygon_on_surface(co_edge) || self.has_polygon_on_triangulation(co_edge)
    }

    /// `HasPolygonOnSurface(theCoEdge)`: a fresh cached 2D polygon, or a persistent one.
    pub fn has_polygon_on_surface(self, co_edge: CoEdgeId) -> bool {
        self.cached_polygon2d(co_edge).is_some()
            || self.graph.mesh().persistent().co_edges().has(co_edge)
    }

    /// `PolygonOnSurface(theCoEdge)`: the cached 2D polygon, else the persistent one.
    pub fn polygon_on_surface(self, co_edge: CoEdgeId) -> Handle<Polygon2D> {
        let persistent = || {
            self.graph
                .mesh()
                .persistent()
                .co_edges()
                .polygon_on_surface(co_edge)
        };
        self.cached_polygon2d(co_edge).unwrap_or_else(persistent)
    }

    /// `HasPolygonOnTriangulation(theCoEdge)`: fresh cached polygons on triangulation,
    /// or a persistent one.
    pub fn has_polygon_on_triangulation(self, co_edge: CoEdgeId) -> bool {
        let cached = self
            .cached_on_tri(co_edge)
            .is_some_and(|entry| !entry.polygons_on_tri.is_empty());
        cached
            || self
                .graph
                .mesh()
                .persistent()
                .co_edges()
                .has_polygon_on_triangulation(co_edge)
    }

    /// `PolygonOnTriangulation(theCoEdge)`: the first cached polygon on triangulation
    /// when it is not null, else the persistent one.
    pub fn polygon_on_triangulation(self, co_edge: CoEdgeId) -> Handle<PolygonOnTriangulation> {
        let cached = self
            .cached_on_tri(co_edge)
            .and_then(|entry| entry.polygons_on_tri.first().cloned())
            .filter(Handle::is_some);
        let persistent = || {
            let co_edges = self.graph.mesh().persistent().co_edges();
            co_edges.polygon_on_triangulation(co_edge)
        };
        cached.unwrap_or_else(persistent)
    }

    // The non-null 2D polygon of a fresh active-slot entry.
    fn cached_polygon2d(self, co_edge: CoEdgeId) -> Option<Handle<Polygon2D>> {
        let (cache, slot) = active_cache(self.graph)?;
        let entry = cache.find_co_edge_polygon2d_in(self.graph, slot, co_edge)?;
        Some(entry.polygon2d).filter(Handle::is_some)
    }

    // The active-slot entry whose polygons on triangulation are fresh.
    fn cached_on_tri(self, co_edge: CoEdgeId) -> Option<CoEdgeMeshEntry> {
        let (cache, slot) = active_cache(self.graph)?;
        cache.find_co_edge_polygon_on_tri_in(self.graph, slot, co_edge)
    }
}
