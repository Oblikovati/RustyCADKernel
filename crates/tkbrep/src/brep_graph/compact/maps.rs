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
// Rust translation of the index maps of `BRepGraph_Compact::Perform` with
// `collectReachableNodesFromRootProducts`, `remapNodeId`, `countActiveDefs` and
// `bindActiveIndexMap` (ModelingData/TKBRep/BRepGraph/BRepGraph_Compact.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The old-to-new maps of a compaction.
//!
//! The per-kind maps are `NCollection_DataMap`s sized with the kind's slot count, and
//! their `Items()` order builds the item remap and the history batch, so they are the
//! TKernel map port with the native id hashes and sizes.
use crate::brep_graph::child_explorer::ChildExplorer;
use crate::brep_graph::copy_remap::ItemMap;
use crate::brep_graph::id_walk::active_ids;
use crate::brep_graph::iterator::RootProductIterator;
use crate::brep_graph::*;
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy};
use rustycad_tkernel::standard::Failure;
use std::collections::HashSet;

/// `NCollection_DataMap<Typed, Typed>`: an old-to-new map of one kind.
pub(super) type IdMap<T> = DataMap<TypedId<T>, TypedId<T>, DefaultHashPolicy>;

fn id_map<T: KindTag>(count: u32) -> IdMap<T> {
    DataMap::with_hasher(DefaultHashPolicy, count as usize)
}

/// The node maps (`aVertexMap` ... `anOccurrenceMap`).
pub(super) struct NodeMaps {
    pub(super) vertices: IdMap<VertexTag>,
    pub(super) edges: IdMap<EdgeTag>,
    pub(super) co_edges: IdMap<CoEdgeTag>,
    pub(super) wires: IdMap<WireTag>,
    pub(super) faces: IdMap<FaceTag>,
    pub(super) shells: IdMap<ShellTag>,
    pub(super) solids: IdMap<SolidTag>,
    pub(super) compounds: IdMap<CompoundTag>,
    pub(super) comp_solids: IdMap<CompSolidTag>,
    pub(super) products: IdMap<ProductTag>,
    pub(super) occurrences: IdMap<OccurrenceTag>,
}

/// The reference maps (`aVertexRefMap` ... `anOccurrenceRefMap`).
pub(super) struct RefMaps {
    pub(super) vertex_refs: IdMap<VertexRefTag>,
    pub(super) wire_refs: IdMap<WireRefTag>,
    pub(super) face_refs: IdMap<FaceRefTag>,
    pub(super) shell_refs: IdMap<ShellRefTag>,
    pub(super) child_refs: IdMap<ChildRefTag>,
    pub(super) solid_refs: IdMap<SolidRefTag>,
    pub(super) occurrence_refs: IdMap<OccurrenceRefTag>,
}

impl NodeMaps {
    /// The empty maps, each sized with its kind's slot count.
    pub(super) fn new(graph: &BRepGraph) -> Self {
        let storage = graph.inc_storage();
        Self {
            vertices: id_map(storage.nb_vertices()),
            edges: id_map(storage.nb_edges()),
            co_edges: id_map(storage.nb_co_edges()),
            wires: id_map(storage.nb_wires()),
            faces: id_map(storage.nb_faces()),
            shells: id_map(storage.nb_shells()),
            solids: id_map(storage.nb_solids()),
            compounds: id_map(storage.nb_compounds()),
            comp_solids: id_map(storage.nb_comp_solids()),
            products: id_map(storage.nb_products()),
            occurrences: id_map(storage.nb_occurrences()),
        }
    }

    /// `remapNodeId(theId, ...)`: the new node of an old topology node, invalid when
    /// unmapped; products and occurrences are never remapped here.
    pub(super) fn remap(&self, node: NodeId) -> NodeId {
        if !node.is_valid() {
            return NodeId::default();
        }
        let index = node.index;
        match node.kind {
            NodeKind::VERTEX => seek_node(&self.vertices, index),
            NodeKind::EDGE => seek_node(&self.edges, index),
            NodeKind::WIRE => seek_node(&self.wires, index),
            NodeKind::FACE => seek_node(&self.faces, index),
            NodeKind::SHELL => seek_node(&self.shells, index),
            NodeKind::SOLID => seek_node(&self.solids, index),
            NodeKind::COMPOUND => seek_node(&self.compounds, index),
            NodeKind::COMP_SOLID => seek_node(&self.comp_solids, index),
            NodeKind::CO_EDGE => seek_node(&self.co_edges, index),
            _ => NodeId::default(),
        }
    }

    /// The typed remap of [`remap`](Self::remap) (`Typed::FromNodeId(remapId(...))`).
    pub(super) fn remap_typed<T: NodeTag>(&self, node: impl Into<NodeId>) -> TypedId<T> {
        TypedId::from_node_id(self.remap(node.into()))
    }

    /// The mapped topology definitions (`aVertexMap.Size() + ... + aCompSolidMap.Size()`).
    pub(super) fn nb_mapped_topology(&self) -> usize {
        self.vertices.len()
            + self.edges.len()
            + self.wires.len()
            + self.faces.len()
            + self.shells.len()
            + self.solids.len()
            + self.compounds.len()
            + self.comp_solids.len()
    }

    /// The node pairs in the native order: kind by kind (vertex, edge, coedge, wire,
    /// face, shell, solid, compound, compsolid, product, occurrence), each map's
    /// `Items()` order within a kind.
    pub(super) fn pairs(&self) -> Vec<(NodeId, NodeId)> {
        let mut pairs = Vec::new();
        extend_pairs(&mut pairs, &self.vertices);
        extend_pairs(&mut pairs, &self.edges);
        extend_pairs(&mut pairs, &self.co_edges);
        extend_pairs(&mut pairs, &self.wires);
        extend_pairs(&mut pairs, &self.faces);
        extend_pairs(&mut pairs, &self.shells);
        extend_pairs(&mut pairs, &self.solids);
        extend_pairs(&mut pairs, &self.compounds);
        extend_pairs(&mut pairs, &self.comp_solids);
        extend_pairs(&mut pairs, &self.products);
        extend_pairs(&mut pairs, &self.occurrences);
        pairs
    }
}

impl RefMaps {
    /// The empty maps, each sized with its kind's slot count.
    pub(super) fn new(graph: &BRepGraph) -> Self {
        let storage = graph.inc_storage();
        Self {
            vertex_refs: id_map(storage.nb_vertex_refs()),
            wire_refs: id_map(storage.nb_wire_refs()),
            face_refs: id_map(storage.nb_face_refs()),
            shell_refs: id_map(storage.nb_shell_refs()),
            child_refs: id_map(storage.nb_child_refs()),
            solid_refs: id_map(storage.nb_solid_refs()),
            occurrence_refs: id_map(storage.nb_occurrence_refs()),
        }
    }

    /// The reference pairs in the native order (vertex, wire, face, shell, child, solid,
    /// occurrence).
    pub(super) fn pairs(&self) -> Vec<(RefId, RefId)> {
        let mut pairs = Vec::new();
        extend_pairs(&mut pairs, &self.vertex_refs);
        extend_pairs(&mut pairs, &self.wire_refs);
        extend_pairs(&mut pairs, &self.face_refs);
        extend_pairs(&mut pairs, &self.shell_refs);
        extend_pairs(&mut pairs, &self.child_refs);
        extend_pairs(&mut pairs, &self.solid_refs);
        extend_pairs(&mut pairs, &self.occurrence_refs);
        pairs
    }
}

/// The full item remap (nodes then references; representations are session-local).
pub(super) fn item_remap(nodes: &NodeMaps, refs: &RefMaps) -> ItemMap {
    let mut items = ItemMap::default();
    for (old, new) in nodes.pairs() {
        items.bind(old.into(), new.into());
    }
    for (old, new) in refs.pairs() {
        items.bind(old.into(), new.into());
    }
    items
}

fn seek_node<T: NodeTag>(map: &IdMap<T>, index: u32) -> NodeId {
    map.seek(&TypedId::new(index))
        .map_or_else(NodeId::default, |new| NodeId::from(*new))
}

fn extend_pairs<T: KindTag, U: From<TypedId<T>>>(pairs: &mut Vec<(U, U)>, map: &IdMap<T>) {
    pairs.extend(map.items().map(|(old, new)| (U::from(*old), U::from(*new))));
}

/// `countActiveDefs<Def>(theGraph)`.
pub(super) fn count_active<T: NodeTag>(graph: &BRepGraph) -> u32 {
    active_ids::<T>(graph).len() as u32
}

/// `bindActiveIndexMap<Def>(theGraph, theMap, theReachableNodes)`: the active nodes (the
/// reachable ones when a filter is given) numbered densely from 0.
pub(super) fn bind_active_index_map<T: NodeTag>(
    graph: &BRepGraph,
    map: &mut IdMap<T>,
    reachable: Option<&HashSet<NodeId>>,
) {
    let mut next = TypedId::<T>::start();
    for node in active_ids::<T>(graph) {
        if reachable.is_some_and(|nodes| !nodes.contains(&node.into())) {
            continue;
        }
        map.bind(node, next);
        next.pre_increment();
    }
}

/// `collectReachableNodesFromRootProducts(theGraph, theReachableNodes)`: each valid root
/// product and every definition its child explorer visits (the explorer's location
/// composition raises through the error).
pub(super) fn reachable_from_root_products(graph: &BRepGraph) -> Result<HashSet<NodeId>, Failure> {
    let mut reachable = HashSet::new();
    let mut roots = RootProductIterator::new(graph);
    while roots.more() {
        let root = roots.current();
        roots.next();
        if !root.is_valid_for(graph.topo().products().nb()) {
            continue;
        }
        reachable.insert(root.into());
        let mut children = ChildExplorer::new(graph, root.into())?;
        while children.more() {
            let node = children.current().def_id;
            if node.is_valid() {
                reachable.insert(node);
            }
            children.next()?;
        }
    }
    Ok(reachable)
}
