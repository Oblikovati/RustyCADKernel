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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Cache.hxx and
// BRepGraph_Cache.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Cache`: the owner-bound base of the transient graph cache services.
use crate::brep_graph::layer::AttachedGraph;
use crate::brep_graph::marks::ref_parent_node;
use crate::brep_graph::{
    BRepGraph, EdgeId, FaceId, FaceRefId, ItemDomain, ItemId, NodeId, NodeKind, RefId, RefKind,
    VertexRefId,
};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Guid, RuntimeTyped, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::atomic::{AtomicU64, Ordering};

mod entries;

pub use entries::{ItemEntry, NodeEntry, RefEntry};

/// The protected state and helpers of `BRepGraph_Cache`: the attached graph (its
/// attachment token, 0 when detached) and the generation reads a concrete cache
/// validates its entries with. Each read takes the graph and uses it only when this
/// cache is attached to it (docs/topology-design.md 11.9).
///
/// ```
/// use rustycad_tkbrep::brep_graph::cache::CacheBase;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// let graph = BRepGraph::new();
/// let base = CacheBase::new();
/// assert!(!base.is_attached());
/// assert_eq!(base.node_own_gen(&graph, NodeId::new(NodeKind::FACE, 0)), None);
/// ```
#[derive(Debug, Default)]
pub struct CacheBase {
    graph_token: AtomicU64,
}

impl CacheBase {
    /// `BRepGraph_Cache()`: detached.
    pub fn new() -> Self {
        Self::default()
    }

    /// `IsAttached()`: registered in the registry of a graph.
    pub fn is_attached(&self) -> bool {
        self.graph_token() != 0
    }

    /// `AttachedGraph()`: `graph` when this cache is attached to it.
    pub fn attached_graph<'g>(&self, graph: &'g BRepGraph) -> AttachedGraph<'g> {
        let token = self.graph_token();
        (token != 0 && token == graph.attachment_token()).then_some(graph)
    }

    /// `Graph()`: the attached graph; Standard_ProgramError when detached.
    pub fn graph<'g>(&self, graph: &'g BRepGraph) -> Result<&'g BRepGraph, Failure> {
        self.attached_graph(graph).ok_or_else(|| {
            Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "BRepGraph_Cache: cache is detached from graph (attachment token {}, \
                     expected the token {} of the given graph)",
                    self.graph_token(),
                    graph.attachment_token()
                ),
                "",
            )
        })
    }

    /// `NodeSubtreeGen(theNode, theGen)`: the SubtreeGen of an active node.
    pub fn node_subtree_gen(&self, graph: &BRepGraph, node: NodeId) -> Option<u32> {
        let graph = self.attached_graph(graph)?;
        let def = graph.topo_entity(node)?;
        (!node.is_removed(graph)).then_some(def.subtree_gen)
    }

    /// `NodeOwnGen(theNode, theGen)`: the OwnGen of an active node.
    pub fn node_own_gen(&self, graph: &BRepGraph, node: NodeId) -> Option<u32> {
        let graph = self.attached_graph(graph)?;
        let def = graph.topo_entity(node)?;
        (!node.is_removed(graph)).then_some(def.own_gen)
    }

    /// `RefOwnGen(theRef, theGen)`: the OwnGen of the parent definition of an active
    /// reference (a vertex reference's parent is the first edge of its vertex that
    /// starts or ends with it).
    pub fn ref_own_gen(&self, graph: &BRepGraph, reference: RefId) -> Option<u32> {
        let graph = self.attached_graph(graph)?;
        if reference.is_removed(graph) {
            return None;
        }
        let parent = if reference.kind == RefKind::VERTEX {
            vertex_ref_parent(graph, VertexRefId::from_ref_id(reference))?
        } else {
            ref_parent_node(graph.inc_storage(), reference)
        };
        if !parent.is_valid() {
            return None;
        }
        graph.topo_entity(parent).map(|def| def.own_gen)
    }

    /// `ItemOwnGen(theItem, theGen)`: `NodeOwnGen` or `RefOwnGen` of a valid item.
    pub fn item_own_gen(&self, graph: &BRepGraph, item: ItemId) -> Option<u32> {
        if !item.is_valid() {
            return None;
        }
        match item.item_domain() {
            ItemDomain::Node => self.node_own_gen(graph, item.node_id()),
            ItemDomain::Reference => self.ref_own_gen(graph, item.ref_id()),
            ItemDomain::None => None,
        }
    }

    /// `ResolveActiveRefChild(theRef, theNode)`: the active child node of an active
    /// reference.
    pub fn resolve_active_ref_child(&self, graph: &BRepGraph, reference: RefId) -> Option<NodeId> {
        let graph = self.attached_graph(graph)?;
        graph.ref_entity(reference)?;
        if reference.is_removed(graph) {
            return None;
        }
        let node = graph.refs().generic().child_node(reference);
        graph.topo_entity(node)?;
        (!node.is_removed(graph)).then_some(node)
    }

    /// `ResolveActiveFaceRef(theRef, theFace)`: the active face of an active face
    /// reference.
    pub fn resolve_active_face_ref(
        &self,
        graph: &BRepGraph,
        reference: FaceRefId,
    ) -> Option<FaceId> {
        let node = self.resolve_active_ref_child(graph, reference.into())?;
        (node.kind == NodeKind::FACE).then(|| FaceId::from_node_id(node))
    }

    pub(crate) fn graph_token(&self) -> u64 {
        self.graph_token.load(Ordering::Acquire)
    }

    fn set_graph_token(&self, token: u64) {
        self.graph_token.store(token, Ordering::Release);
    }
}

// The vertex case of `RefOwnGen`: `None` when the reference or its vertex is out of
// range, else the first edge in range of the vertex using it (invalid when none).
fn vertex_ref_parent(graph: &BRepGraph, reference: VertexRefId) -> Option<NodeId> {
    let storage = graph.inc_storage();
    if !reference.is_valid_for(storage.nb_vertex_refs()) {
        return None;
    }
    let vertex = storage.vertex_ref(reference).child_vertex_id;
    if !vertex.is_valid_for(storage.nb_vertices()) {
        return None;
    }
    let uses_reference = |edge: &&EdgeId| {
        let def = storage.edge(**edge);
        def.start_vertex_ref_id == reference || def.end_vertex_ref_id == reference
    };
    let edge = storage
        .vertex_relations(vertex)
        .edge_ids
        .iter()
        .filter(|edge| edge.is_valid_for(storage.nb_edges()))
        .find(uses_reference);
    Some(edge.map_or_else(NodeId::default, |edge| (*edge).into()))
}

/// `BRepGraph_Cache`: a typed, recomputable, graph-local cache service. The registry
/// binds it to its graph; the service owns its data and validates it lazily against
/// the generation counters (`NodeEntry`, `RefEntry`, `ItemEntry`). `CopyFreshTo` waits
/// for `BRepGraph_CopyRemap` (docs/topology-design.md 11.9).
pub trait BRepGraphCache: Transient {
    /// The embedded `BRepGraph_Cache` state.
    fn cache_base(&self) -> &CacheBase;

    /// `ID()`: the cache service identity, unique within a registry.
    fn id(&self) -> Guid;

    /// `Name()`: the cache service display name.
    fn name(&self) -> &AsciiString;

    /// `Clear()`: drop every transient value (the base keeps nothing).
    fn clear(&self) {}

    /// `OnAttached()`: called after the registry attached this cache.
    fn on_attached(&self) {}

    /// `OnDetached()`: called before the registry detaches this cache.
    fn on_detached(&self) {}
}
impl_transient_interface!(dyn BRepGraphCache, "BRepGraph_Cache", StandardTransient);

/// A concrete cache class with a static GUID (`T::GetID()`) for the registry's typed
/// `Find<T>` and `Ensure<T>`; `Ensure` creates it with `Default`.
pub trait CacheType: BRepGraphCache + RuntimeTyped + Default + Sized {
    /// `GetID()`: the fixed cache type GUID.
    fn get_id() -> Guid;
}

/// `attachGraph(theGraph)` (friend of the registry): no-op for the same graph, else
/// detach from the previous one, store the token and `OnAttached` for a graph.
pub(crate) fn attach_graph(cache: &dyn BRepGraphCache, token: u64) {
    let base = cache.cache_base();
    if base.graph_token() == token {
        return;
    }
    if base.is_attached() {
        detach_graph(cache);
    }
    base.set_graph_token(token);
    if token != 0 {
        cache.on_attached();
    }
}

/// `detachGraph()` (friend of the registry): `Clear`, `OnDetached`, forget the graph.
pub(crate) fn detach_graph(cache: &dyn BRepGraphCache) {
    if !cache.cache_base().is_attached() {
        return;
    }
    cache.clear();
    cache.on_detached();
    cache.cache_base().set_graph_token(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_detached_cache_reads_no_generation() {
        let graph = BRepGraph::new();
        let base = CacheBase::new();
        assert!(base.graph(&graph).is_err());
        assert_eq!(base.ref_own_gen(&graph, RefId::new(RefKind::FACE, 0)), None);
        assert_eq!(base.item_own_gen(&graph, ItemId::default()), None);
        base.set_graph_token(graph.attachment_token());
        assert!(base.graph(&graph).is_ok());
        assert_eq!(
            base.node_subtree_gen(&graph, NodeId::new(NodeKind::FACE, 0)),
            None
        );
        assert_eq!(
            base.resolve_active_ref_child(&graph, RefId::new(RefKind::FACE, 0)),
            None
        );
    }
}
