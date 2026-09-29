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
// Rust translation of `GraphCopyContext`, `bindItemRemap`, `setRemovedLike`,
// `bindTypedMap`, `mappedNode` and `itemCapacityUpperBound` of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The memoised copy context the `ensure*` functions share.
//!
//! The native context holds a `const BRepGraph&` source and a `BRepGraph&` result that
//! are the same object for a self-copy. The Rust context owns the one mutable borrow:
//! [`CopyGraphs::Distinct`] for two graphs, [`CopyGraphs::Same`] for a self-copy, so the
//! source reads see the copies already made, as natively.
//!
//! The native context keeps one `NCollection_DataMap` per node kind; they are only
//! searched and bound (never iterated), and their keys never collide across kinds, so
//! one map keyed by the untyped node id holds them all.
use super::policy::{GeomPolicy, MeshPolicy};
use crate::brep_graph::copy_remap::ItemMap;
use crate::brep_graph::{BRepGraph, ItemId, NodeId, NodeTag, OccurrenceRefId, StoredTag, TypedId};
use crate::brep_graph_inc::Storage;
use std::collections::HashMap;

/// The source and result graphs of a copy.
pub(crate) enum CopyGraphs<'a> {
    /// An external copy: the source is read, the target written.
    Distinct {
        source: &'a BRepGraph,
        target: &'a mut BRepGraph,
    },
    /// A self-copy: one graph is both.
    Same(&'a mut BRepGraph),
}

/// `GraphCopyContext`.
pub(crate) struct CopyContext<'a> {
    graphs: CopyGraphs<'a>,
    /// `GeomPol`.
    pub(crate) geom: GeomPolicy,
    /// `MeshPol`.
    pub(crate) mesh: MeshPolicy,
    // `Vertices`...`Occurrences`: source node -> copied node.
    nodes: HashMap<NodeId, NodeId>,
    // `OccurrenceRefs`.
    occurrence_refs: HashMap<OccurrenceRefId, OccurrenceRefId>,
    // `!Products.IsEmpty()`.
    has_products: bool,
    /// `ItemRemap`: the source-to-result item map handed to the layers and caches.
    pub(crate) items: ItemMap,
}

impl<'a> CopyContext<'a> {
    /// The context of a copy, its item map reserved for every source item
    /// (`ItemRemap.Reserve(itemCapacityUpperBound(...))`).
    pub(crate) fn new(graphs: CopyGraphs<'a>, geom: GeomPolicy, mesh: MeshPolicy) -> Self {
        let mut context = Self {
            graphs,
            geom,
            mesh,
            nodes: HashMap::new(),
            occurrence_refs: HashMap::new(),
            has_products: false,
            items: ItemMap::default(),
        };
        let capacity = item_capacity_upper_bound(context.source().inc_storage());
        context.items.reserve(capacity);
        context
    }

    /// `Source`.
    pub(crate) fn source(&self) -> &BRepGraph {
        match &self.graphs {
            CopyGraphs::Distinct { source, .. } => source,
            CopyGraphs::Same(graph) => graph,
        }
    }

    /// `Result` (read access).
    pub(crate) fn target(&self) -> &BRepGraph {
        match &self.graphs {
            CopyGraphs::Distinct { target, .. } => target,
            CopyGraphs::Same(graph) => graph,
        }
    }

    /// `Result`.
    pub(crate) fn target_mut(&mut self) -> &mut BRepGraph {
        match &mut self.graphs {
            CopyGraphs::Distinct { target, .. } => target,
            CopyGraphs::Same(graph) => graph,
        }
    }

    /// `SrcStorage`.
    pub(crate) fn src_storage(&self) -> &Storage {
        self.source().inc_storage()
    }

    /// `DstStorage`.
    pub(crate) fn dst_storage(&mut self) -> &mut Storage {
        self.target_mut().inc_storage_mut()
    }

    /// Consumes the context into its graphs and item map for the final layer copy.
    pub(crate) fn into_parts(self) -> (CopyGraphs<'a>, ItemMap) {
        (self.graphs, self.items)
    }

    /// The typed map `Seek`: the copy of an already processed node.
    pub(crate) fn existing<T: NodeTag>(&self, source: TypedId<T>) -> Option<TypedId<T>> {
        self.nodes
            .get(&NodeId::from(source))
            .map(|node| TypedId::from_node_id(*node))
    }

    /// `mappedNode(theCtx, theSrcId)`: the copy of any node kind, invalid when unmapped
    /// (and for the reference kinds the switch has no case for).
    pub(crate) fn mapped_node(&self, source: NodeId) -> NodeId {
        self.nodes.get(&source).copied().unwrap_or_default()
    }

    /// `bindTypedMap(theMap, theCtx, theSource, theTarget)`: binds the typed map, then
    /// the item map.
    pub(crate) fn bind_node<T: NodeTag>(&mut self, source: TypedId<T>, target: TypedId<T>) {
        self.nodes.insert(source.into(), target.into());
        if T::RAW_KIND == crate::brep_graph::NodeKind::PRODUCT.0 {
            self.has_products = true;
        }
        self.bind_item(NodeId::from(source).into(), NodeId::from(target).into());
    }

    /// `Products.IsEmpty()` negated.
    pub(crate) fn has_products(&self) -> bool {
        self.has_products
    }

    /// `OccurrenceRefs.Seek`.
    pub(crate) fn existing_occurrence_ref(&self, source: OccurrenceRefId) -> Option<OccurrenceRefId> {
        self.occurrence_refs.get(&source).copied()
    }

    /// `OccurrenceRefs.Bind`.
    pub(crate) fn bind_occurrence_ref(&mut self, source: OccurrenceRefId, target: OccurrenceRefId) {
        self.occurrence_refs.insert(source, target);
    }

    /// `bindItemRemap(theCtx, theSource, theTarget)`: binds (or rebinds) valid pairs only.
    pub(crate) fn bind_item(&mut self, source: ItemId, target: ItemId) {
        if !source.is_valid() || !target.is_valid() {
            return;
        }
        if let Some(bound) = self.items.change_seek(&source) {
            *bound = target;
            return;
        }
        self.items.bind(source, target);
    }

    /// `DstStorage->SetRemoved(theTarget, SrcStorage->IsRemoved(theSource))`, which is
    /// also `setRemovedLike`: the removed plane ignores ids out of range, and an invalid
    /// id is out of range.
    pub(crate) fn copy_removed<T: StoredTag>(&mut self, source: TypedId<T>, target: TypedId<T>) {
        let removed = self.src_storage().is_removed(source);
        self.dst_storage().set_removed(target, removed);
    }
}

// `itemCapacityUpperBound(theStorage)`: every definition, reference and representation
// slot of the source.
fn item_capacity_upper_bound(storage: &Storage) -> usize {
    let counts = storage.counts();
    [
        counts.nb_vertices,
        counts.nb_edges,
        counts.nb_co_edges,
        counts.nb_wires,
        counts.nb_faces,
        counts.nb_shells,
        counts.nb_solids,
        counts.nb_compounds,
        counts.nb_comp_solids,
        counts.nb_products,
        counts.nb_occurrences,
        counts.nb_shell_refs,
        counts.nb_face_refs,
        counts.nb_wire_refs,
        counts.nb_vertex_refs,
        counts.nb_solid_refs,
        counts.nb_child_refs,
        counts.nb_occurrence_refs,
        counts.nb_face_surface_reps,
        counts.nb_edge_curve3d_reps,
        counts.nb_co_edge_curve2d_reps,
        counts.nb_face_triangulation_reps,
        counts.nb_edge_polygon3d_reps,
        counts.nb_co_edge_polygon2d_reps,
        counts.nb_co_edge_polygon_on_tri_reps,
    ]
    .iter()
    .map(|count| *count as usize)
    .sum()
}
