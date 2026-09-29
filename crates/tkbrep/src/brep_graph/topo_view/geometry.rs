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
// Rust translation of the face, edge, vertex and coedge members of
// ModelingData/TKBRep/BRepGraph/BRepGraph_TopoView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The representation and adjacency members of the face, edge, vertex and coedge groups.
use super::{CoEdgeOps, EdgeOps, FaceOps, VertexOps};
use crate::brep_graph::{CoEdgeId, EdgeId, FaceId, VertexId, WireId};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::poly::Triangulation;

impl<'g> FaceOps<'g> {
    /// `Surface(theFace)`: the surface of an active face with an active surface use,
    /// null otherwise.
    pub fn surface(self, face: FaceId) -> Handle<dyn GeomSurface> {
        let storage = self.graph.inc_storage();
        if !face.is_valid_for(storage.nb_faces()) || storage.is_removed(face) {
            return Handle::default();
        }
        let rep = storage.face(face).surface_rep_id;
        if !rep.is_valid_for(storage.nb_face_surfaces()) || storage.is_removed(rep) {
            return Handle::default();
        }
        storage.face_surface_rep(rep).surface.clone()
    }

    /// `ActiveTriangulation(theFace)`: the triangulation of an active face, null otherwise.
    pub fn active_triangulation(self, face: FaceId) -> Handle<Triangulation> {
        let storage = self.graph.inc_storage();
        if !face.is_valid_for(storage.nb_faces()) || storage.is_removed(face) {
            return Handle::default();
        }
        let rep = storage.face(face).triangulation_rep_id;
        if !rep.is_valid_for(storage.nb_face_triangulations()) || storage.is_removed(rep) {
            return Handle::default();
        }
        storage.face_triangulation_rep(rep).triangulation.clone()
    }
}

impl<'g> EdgeOps<'g> {
    /// `NbFaces(theEdge)`: the distinct active faces of the edge's active coedges.
    pub fn nb_faces(self, edge: EdgeId) -> u32 {
        let storage = self.graph.inc_storage();
        if !edge.is_valid_for(storage.nb_edges())
            || storage.is_removed(edge)
            || storage.nb_active_faces() == 0
        {
            return 0;
        }
        let mut seen: Vec<FaceId> = Vec::with_capacity(32);
        for co_edge in &storage.edge_relations(edge).co_edge_ids {
            if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(*co_edge) {
                continue;
            }
            let face = storage.co_edge(*co_edge).face_id;
            if face.is_valid_for(storage.nb_faces())
                && !storage.is_removed(face)
                && !seen.contains(&face)
            {
                seen.push(face);
            }
        }
        seen.len() as u32
    }

    /// `CoEdges(theEdge)`: the coedges that use the edge (empty for an id out of range).
    pub fn co_edges(self, edge: EdgeId) -> &'g [CoEdgeId] {
        let storage = self.graph.inc_storage();
        if !edge.is_valid_for(storage.nb_edges()) {
            return &[];
        }
        &storage.edge_relations(edge).co_edge_ids
    }

    /// `Curve3D(theEdge)`: the 3D curve of an active edge, null otherwise.
    pub fn curve3d(self, edge: EdgeId) -> Handle<dyn GeomCurve> {
        let storage = self.graph.inc_storage();
        if !edge.is_valid_for(storage.nb_edges()) || storage.is_removed(edge) {
            return Handle::default();
        }
        let rep = storage.edge(edge).curve3d_rep_id;
        if !rep.is_valid_for(storage.nb_edge_curves3d()) || storage.is_removed(rep) {
            return Handle::default();
        }
        storage.edge_curve3d_rep(rep).curve.clone()
    }
}

impl<'g> VertexOps<'g> {
    /// `Edges(theVertex)`: the edges sharing the vertex (empty for an id out of range).
    pub fn edges(self, vertex: VertexId) -> &'g [EdgeId] {
        let storage = self.graph.inc_storage();
        if !vertex.is_valid_for(storage.nb_vertices()) {
            return &[];
        }
        &storage.vertex_relations(vertex).edge_ids
    }
}

impl<'g> CoEdgeOps<'g> {
    /// `Edge(theCoEdge)`: the active edge of an active coedge, invalid otherwise.
    pub fn edge(self, co_edge: CoEdgeId) -> EdgeId {
        let storage = self.graph.inc_storage();
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
            return EdgeId::default();
        }
        let edge = storage.co_edge(co_edge).child_edge_id;
        if !edge.is_valid_for(storage.nb_edges()) || storage.is_removed(edge) {
            return EdgeId::default();
        }
        edge
    }

    /// `Face(theCoEdge)`: the active face of an active coedge, invalid otherwise.
    pub fn face(self, co_edge: CoEdgeId) -> FaceId {
        let storage = self.graph.inc_storage();
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
            return FaceId::default();
        }
        let face = storage.co_edge(co_edge).face_id;
        if !face.is_valid_for(storage.nb_faces()) || storage.is_removed(face) {
            return FaceId::default();
        }
        face
    }

    /// `Wire(theCoEdge)`: the active wire of an active coedge, invalid otherwise.
    pub fn wire(self, co_edge: CoEdgeId) -> WireId {
        let storage = self.graph.inc_storage();
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
            return WireId::default();
        }
        let wire = storage.co_edge(co_edge).parent_wire_id;
        if !wire.is_valid_for(storage.nb_wires()) || storage.is_removed(wire) {
            return WireId::default();
        }
        wire
    }

    /// `Curve2D(theCoEdge)`: the p-curve of an active coedge, null otherwise.
    pub fn curve2d(self, co_edge: CoEdgeId) -> Handle<dyn Geom2dCurve> {
        let storage = self.graph.inc_storage();
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
            return Handle::default();
        }
        let rep = storage.co_edge(co_edge).curve2d_rep_id;
        if !rep.is_valid_for(storage.nb_co_edge_curves2d()) || storage.is_removed(rep) {
            return Handle::default();
        }
        storage.co_edge_curve2d_rep(rep).curve.clone()
    }
}
