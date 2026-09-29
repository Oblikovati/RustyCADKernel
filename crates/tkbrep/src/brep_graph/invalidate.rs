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
// Rust translation of BRepGraph::invalidateSubgraphImpl of
// ModelingData/TKBRep/BRepGraph/BRepGraph.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `invalidateSubgraphImpl`: the generation bump of a whole subgraph after an
//! `ApplyModification`.
use super::defs_iterator::{
    DefsChildOfCompound, DefsEdgeOfWire, DefsFaceOfShell, DefsOfParent, DefsShellOfSolid,
    DefsSolidOfCompSolid, DefsVertexOfEdge, DefsWireOfFace,
};
use super::parent_traits::ParentTraits;
use super::refs_iterator::RefsOccurrenceOfProduct;
use super::*;
use std::collections::HashSet;

impl BRepGraph {
    /// `invalidateSubgraphImpl(theNode)` (private natively): a depth-first walk from the
    /// node (children pushed in iterator order, popped last first, each node visited
    /// once, depth bounded by the node count) that bumps OwnGen and SubtreeGen of
    /// every node reached, so generation-based caches see the change. Nothing happens
    /// for an invalid node or one out of range.
    pub(crate) fn invalidate_subgraph_impl(&mut self, node: NodeId) {
        if !node.is_valid() || self.topo_entity(node).is_none() {
            return;
        }
        let max_depth = walk_node_count(self.inc_storage()).max(1);
        let mut stack: Vec<(NodeId, u32)> = vec![(node, 0)];
        let mut visited: HashSet<NodeId> = HashSet::new();
        while let Some((current, depth)) = stack.pop() {
            if depth > max_depth || !current.is_valid() || !visited.insert(current) {
                continue;
            }
            let Some(entity) = self.change_topo_entity(current) else {
                continue;
            };
            entity.own_gen = entity.own_gen.wrapping_add(1);
            entity.subtree_gen = entity.subtree_gen.wrapping_add(1);
            let children = self.invalidation_children(current);
            stack.extend(
                children
                    .into_iter()
                    .filter(|child| child.is_valid())
                    .map(|child| (child, depth + 1)),
            );
        }
    }

    // The children the walk pushes for one node, in iterator order.
    fn invalidation_children(&self, node: NodeId) -> Vec<NodeId> {
        match node.kind {
            NodeKind::COMPOUND => defs_children::<_>(DefsChildOfCompound::new(
                self,
                CompoundId::from_node_id(node),
            )),
            NodeKind::COMP_SOLID => defs_children(DefsSolidOfCompSolid::new(
                self,
                CompSolidId::from_node_id(node),
            )),
            NodeKind::SOLID => {
                defs_children(DefsShellOfSolid::new(self, SolidId::from_node_id(node)))
            }
            NodeKind::SHELL => {
                defs_children(DefsFaceOfShell::new(self, ShellId::from_node_id(node)))
            }
            NodeKind::FACE => defs_children(DefsWireOfFace::new(self, FaceId::from_node_id(node))),
            NodeKind::WIRE => defs_children(DefsEdgeOfWire::new(self, WireId::from_node_id(node))),
            NodeKind::EDGE => edge_vertices(self, EdgeId::from_node_id(node)),
            NodeKind::PRODUCT => self.product_occurrences(ProductId::from_node_id(node)),
            NodeKind::OCCURRENCE => {
                vec![
                    self.inc_storage()
                        .occurrence(OccurrenceId::from_node_id(node))
                        .child_node_id,
                ]
            }
            _ => Vec::new(),
        }
    }

    fn product_occurrences(&self, product: ProductId) -> Vec<NodeId> {
        let storage = self.inc_storage();
        RefsOccurrenceOfProduct::new(self, product)
            .map(|reference| storage.occurrence_ref(reference).child_occurrence_id.into())
            .collect()
    }
}

// The depth bound of the walk: the node count of every kind but the coedges, as the
// native sum lists them.
fn walk_node_count(storage: &crate::brep_graph_inc::Storage) -> u32 {
    [
        storage.nb_solids(),
        storage.nb_shells(),
        storage.nb_faces(),
        storage.nb_wires(),
        storage.nb_edges(),
        storage.nb_vertices(),
        storage.nb_compounds(),
        storage.nb_comp_solids(),
        storage.nb_products(),
        storage.nb_occurrences(),
    ]
    .into_iter()
    .fold(0_u32, u32::wrapping_add)
}

// The `CurrentId()` of every step of a defs iterator.
fn defs_children<P: ParentTraits>(mut iterator: DefsOfParent<'_, P>) -> Vec<NodeId>
where
    P::Child: Into<NodeId>,
{
    let mut children = Vec::new();
    while iterator.more() {
        children.push(iterator.current_id().into());
        iterator.next();
    }
    children
}

fn edge_vertices(graph: &BRepGraph, edge: EdgeId) -> Vec<NodeId> {
    let mut iterator = DefsVertexOfEdge::new(graph, edge);
    let mut vertices = Vec::new();
    while iterator.more() {
        vertices.push(iterator.current_id().into());
        iterator.next();
    }
    vertices
}
