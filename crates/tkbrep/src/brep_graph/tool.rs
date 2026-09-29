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
// Rust translation of BRepGraph_Tool::Edge::Curve(graph, edge), Edge::Range,
// CoEdge::PCurve(graph, coedge), CoEdge::Range and Face::Surface(graph, face) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Tool.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool`: the geometry access helpers over a graph.
//!
//! Phase 1 ports the five use-record reads that the derived edge state needs; the rest
//! of `BRepGraph_Tool` (adaptors, usages, lookups, wire and shell queries) belongs to
//! the algorithms area of phase 2 (docs/topology-design.md section 11.7).

/// `BRepGraph_Tool::Edge`.
pub mod edge {
    use crate::brep_graph::{BRepGraph, EdgeId};
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg3d::geom::GeomCurve;

    /// `Curve(theGraph, theEdge)`: the 3D curve of an active curve use, null otherwise
    /// (the edge itself must be in range; its removal is not checked).
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let graph = BRepGraph::new();
    /// # let _ = graph;
    /// ```
    pub fn curve(graph: &BRepGraph, edge: EdgeId) -> Handle<dyn GeomCurve> {
        let storage = graph.inc_storage();
        let rep = storage.edge(edge).curve3d_rep_id;
        if !rep.is_valid_for(storage.nb_edge_curves3d()) || storage.is_removed(rep) {
            return Handle::default();
        }
        storage.edge_curve3d_rep(rep).curve.clone()
    }

    /// `Range(theGraph, theEdge)`: the parameter range of the active curve use, (0, 0)
    /// otherwise.
    pub fn range(graph: &BRepGraph, edge: EdgeId) -> (f64, f64) {
        let storage = graph.inc_storage();
        let rep = storage.edge(edge).curve3d_rep_id;
        if !rep.is_valid()
            || !rep.is_valid_for(storage.nb_edge_curves3d())
            || storage.is_removed(rep)
        {
            return (0.0, 0.0);
        }
        let record = storage.edge_curve3d_rep(rep);
        (record.param_first, record.param_last)
    }
}

/// `BRepGraph_Tool::CoEdge`.
pub mod co_edge {
    use crate::brep_graph::{BRepGraph, CoEdgeId};
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::Geom2dCurve;

    /// `PCurve(theGraph, theCoEdge)`: the p-curve of an active curve use, null otherwise.
    pub fn pcurve(graph: &BRepGraph, co_edge: CoEdgeId) -> Handle<dyn Geom2dCurve> {
        let storage = graph.inc_storage();
        let rep = storage.co_edge(co_edge).curve2d_rep_id;
        if !rep.is_valid()
            || !rep.is_valid_for(storage.nb_co_edge_curves2d())
            || storage.is_removed(rep)
        {
            return Handle::default();
        }
        storage.co_edge_curve2d_rep(rep).curve.clone()
    }

    /// `Range(theGraph, theCoEdge)`: the p-curve range of the active curve use, (0, 0)
    /// otherwise.
    pub fn range(graph: &BRepGraph, co_edge: CoEdgeId) -> (f64, f64) {
        let storage = graph.inc_storage();
        let rep = storage.co_edge(co_edge).curve2d_rep_id;
        if !rep.is_valid()
            || !rep.is_valid_for(storage.nb_co_edge_curves2d())
            || storage.is_removed(rep)
        {
            return (0.0, 0.0);
        }
        let record = storage.co_edge_curve2d_rep(rep);
        (record.param_first, record.param_last)
    }
}

/// `BRepGraph_Tool::Face`.
pub mod face {
    use crate::brep_graph::{BRepGraph, FaceId};
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg3d::geom::GeomSurface;

    /// `Surface(theGraph, theFace)`: the surface of an active surface use, null otherwise.
    pub fn surface(graph: &BRepGraph, face: FaceId) -> Handle<dyn GeomSurface> {
        let storage = graph.inc_storage();
        let rep = storage.face(face).surface_rep_id;
        if !rep.is_valid_for(storage.nb_face_surfaces()) || storage.is_removed(rep) {
            return Handle::default();
        }
        storage.face_surface_rep(rep).surface.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::BRepGraph;

    #[test]
    fn missing_uses_read_as_null() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let (edge, face, co_edge) = (
            storage.append_edge(),
            storage.append_face(),
            storage.append_co_edge(),
        );
        let curve = storage.append_edge_curve3d_rep();
        storage.change_edge(edge).curve3d_rep_id = curve;
        storage.change_edge_curve3d_rep(curve).param_last = 2.0;
        assert_eq!(edge::range(&graph, edge), (0.0, 2.0));
        assert!(edge::curve(&graph, edge).is_null());
        assert_eq!(co_edge::range(&graph, co_edge), (0.0, 0.0));
        assert!(co_edge::pcurve(&graph, co_edge).is_null());
        assert!(face::surface(&graph, face).is_null());
    }
}
