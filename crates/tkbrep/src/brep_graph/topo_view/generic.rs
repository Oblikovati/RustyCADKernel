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
// Rust translation of the GenOps and GeometryOps members of
// ModelingData/TKBRep/BRepGraph/BRepGraph_TopoView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The generic node queries and the representation counts of the topology view.
use crate::brep_graph::{BRepGraph, ChildRefId, NodeId, NodeKind, OccurrenceRefId};
use crate::brep_graph_inc::{BaseDef, check_untyped_index};
use rustycad_tkernel::standard::Failure;

/// `TopoView::GenOps`: generic topology and assembly queries.
#[derive(Clone, Copy, Debug)]
pub struct GenOps<'g> {
    pub(super) graph: &'g BRepGraph,
}

impl<'g> GenOps<'g> {
    /// `TopoEntity(theId)`: the base of any node in range, `None` otherwise.
    pub fn topo_entity(self, node: NodeId) -> Option<&'g BaseDef> {
        self.graph.topo_entity(node)
    }

    /// `CompoundRefIds(theChild)`: the compound references that name the node.
    pub fn compound_ref_ids(self, child: NodeId) -> &'g [ChildRefId] {
        self.graph.inc_storage().compound_refs_of_node(child)
    }

    /// `OccurrenceRefIds(theChild)`: the occurrence references that reach the node.
    pub fn occurrence_ref_ids(self, child: NodeId) -> &'g [OccurrenceRefId] {
        self.graph.inc_storage().occurrence_refs_of_node(child)
    }

    /// `HasCompoundParents(theNode)`.
    pub fn has_compound_parents(self, node: NodeId) -> bool {
        self.graph.inc_storage().has_compound_parent(node)
    }

    /// `HasOccurrenceParents(theNode)`.
    pub fn has_occurrence_parents(self, node: NodeId) -> bool {
        self.graph.inc_storage().has_occurrence_parent(node)
    }

    /// `NbNodes()`: every definition of every kind, soft-removed ones included.
    pub fn nb_nodes(self) -> u32 {
        let storage = self.graph.inc_storage();
        storage.nb_solids()
            + storage.nb_shells()
            + storage.nb_faces()
            + storage.nb_wires()
            + storage.nb_co_edges()
            + storage.nb_edges()
            + storage.nb_vertices()
            + storage.nb_compounds()
            + storage.nb_comp_solids()
            + storage.nb_products()
            + storage.nb_occurrences()
    }

    /// `Nb(theKind)`: the definitions of one kind (0 for an unsupported kind).
    pub fn nb(self, kind: NodeKind) -> u32 {
        let storage = self.graph.inc_storage();
        match kind {
            NodeKind::VERTEX => storage.nb_vertices(),
            NodeKind::EDGE => storage.nb_edges(),
            NodeKind::CO_EDGE => storage.nb_co_edges(),
            NodeKind::WIRE => storage.nb_wires(),
            NodeKind::FACE => storage.nb_faces(),
            NodeKind::SHELL => storage.nb_shells(),
            NodeKind::SOLID => storage.nb_solids(),
            NodeKind::COMPOUND => storage.nb_compounds(),
            NodeKind::COMP_SOLID => storage.nb_comp_solids(),
            NodeKind::PRODUCT => storage.nb_products(),
            NodeKind::OCCURRENCE => storage.nb_occurrences(),
            _ => 0,
        }
    }

    /// `IsValid(theNode)`: a valid id inside its kind's range.
    pub fn is_valid(self, node: NodeId) -> bool {
        node.is_valid() && node.index < self.nb(node.kind)
    }

    /// The checked form of [`is_valid`](Self::is_valid): the node's slot, else
    /// Standard_OutOfRange naming the kind, the index and the valid range (an
    /// unsupported kind has the empty range).
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId, NodeId};
    /// let graph = BRepGraph::new();
    /// assert!(graph.topo().generic().check_in_range(NodeId::from(FaceId::new(0))).is_err());
    /// ```
    pub fn check_in_range(self, node: NodeId) -> Result<usize, Failure> {
        check_untyped_index(
            "BRepGraph_NodeId",
            node.kind.0,
            node.index,
            self.nb(node.kind),
        )
    }

    /// `IsActive(theNode)`: valid and not soft-removed.
    pub fn is_active(self, node: NodeId) -> bool {
        self.is_valid(node) && !node.is_removed(self.graph)
    }

    /// `IsRemoved(theNode)`: invalid or soft-removed.
    pub fn is_removed(self, node: NodeId) -> bool {
        !self.is_valid(node) || node.is_removed(self.graph)
    }
}

/// `TopoView::GeometryOps`: the analytic representation counts.
#[derive(Clone, Copy, Debug)]
pub struct GeometryOps<'g> {
    pub(super) graph: &'g BRepGraph,
}

impl GeometryOps<'_> {
    /// `NbFaceSurfaces()`.
    pub fn nb_face_surfaces(self) -> u32 {
        self.graph.inc_storage().nb_face_surfaces()
    }

    /// `NbEdgeCurves3D()`.
    pub fn nb_edge_curves3d(self) -> u32 {
        self.graph.inc_storage().nb_edge_curves3d()
    }

    /// `NbCoEdgeCurves2D()`.
    pub fn nb_co_edge_curves2d(self) -> u32 {
        self.graph.inc_storage().nb_co_edge_curves2d()
    }

    /// `NbActiveFaceSurfaces()`.
    pub fn nb_active_face_surfaces(self) -> u32 {
        self.graph.inc_storage().nb_active_face_surfaces()
    }

    /// `NbActiveEdgeCurves3D()`.
    pub fn nb_active_edge_curves3d(self) -> u32 {
        self.graph.inc_storage().nb_active_edge_curves3d()
    }

    /// `NbActiveCoEdgeCurves2D()`.
    pub fn nb_active_co_edge_curves2d(self) -> u32 {
        self.graph.inc_storage().nb_active_co_edge_curves2d()
    }
}

#[cfg(test)]
mod tests {
    use crate::brep_graph::{BRepGraph, FaceId, NodeId, NodeKind};

    #[test]
    fn generic_queries_check_ranges() {
        let mut graph = BRepGraph::new();
        let face = graph.inc_storage_mut().append_face();
        let generic = graph.topo().generic();
        assert_eq!(generic.nb_nodes(), 1);
        assert!(generic.is_active(face.into()));
        assert!(generic.is_removed(FaceId::new(1).into()));
        assert_eq!(generic.nb(NodeKind(9)), 0);
        assert!(generic.topo_entity(NodeId::default()).is_none());
        assert!(generic.compound_ref_ids(face.into()).is_empty());
        assert_eq!(graph.topo().geometry().nb_face_surfaces(), 0);
    }
}
