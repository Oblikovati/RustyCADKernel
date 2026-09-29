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
// Rust translation of BRepGraph_Cache::NodeEntry, RefEntry and ItemEntry
// (ModelingData/TKBRep/BRepGraph/BRepGraph_Cache.hxx and .cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The value bases of cache entries: an identity plus the generation it was computed
//! at, fresh while the graph still reports that generation.
use super::CacheBase;
use crate::brep_graph::{BRepGraph, ItemId, NodeId, RefId};

/// Which node generation a `NodeEntry` is bound to (`GenKind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum GenKind {
    #[default]
    None,
    Own,
    Subtree,
}

/// `BRepGraph_Cache::NodeEntry`: a node-derived entry bound to the node's OwnGen or
/// SubtreeGen.
///
/// ```
/// use rustycad_tkbrep::brep_graph::cache::{CacheBase, NodeEntry};
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
/// let graph = BRepGraph::new();
/// let mut entry = NodeEntry::default();
/// assert!(!entry.bind_own_gen(&CacheBase::new(), &graph, NodeId::new(NodeKind::FACE, 0)));
/// assert!(!entry.is_bound());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeEntry {
    node: NodeId,
    generation: u32,
    kind: GenKind,
}

impl NodeEntry {
    /// `Reset()`: unbound and stale.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// `BindOwnGen(theCache, theNode)`: bound to the current OwnGen; false (and reset)
    /// when the cache is detached or the node inactive.
    pub fn bind_own_gen(&mut self, cache: &CacheBase, graph: &BRepGraph, node: NodeId) -> bool {
        self.bind(cache, graph, node, GenKind::Own)
    }

    /// `BindSubtreeGen(theCache, theNode)`: bound to the current SubtreeGen.
    pub fn bind_subtree_gen(&mut self, cache: &CacheBase, graph: &BRepGraph, node: NodeId) -> bool {
        self.bind(cache, graph, node, GenKind::Subtree)
    }

    /// `IsFreshOwn(theCache, theNode)`: bound to this node's unchanged OwnGen.
    pub fn is_fresh_own(&self, cache: &CacheBase, graph: &BRepGraph, node: NodeId) -> bool {
        self.is_fresh_for(cache, graph, node, GenKind::Own)
    }

    /// `IsFreshSubtree(theCache, theNode)`: bound to this node's unchanged SubtreeGen.
    pub fn is_fresh_subtree(&self, cache: &CacheBase, graph: &BRepGraph, node: NodeId) -> bool {
        self.is_fresh_for(cache, graph, node, GenKind::Subtree)
    }

    /// `IsFresh(theCache)`: fresh for the node and generation kind of the bind.
    pub fn is_fresh(&self, cache: &CacheBase, graph: &BRepGraph) -> bool {
        self.is_fresh_for(cache, graph, self.node, self.kind)
    }

    /// `IsBound()`: the last bind succeeded.
    pub fn is_bound(&self) -> bool {
        self.kind != GenKind::None
    }

    /// `Node()`: the node captured at bind time.
    pub fn node(&self) -> NodeId {
        self.node
    }

    fn bind(&mut self, cache: &CacheBase, graph: &BRepGraph, node: NodeId, kind: GenKind) -> bool {
        let Some(generation) = generation_of(cache, graph, node, kind) else {
            self.reset();
            return false;
        };
        *self = Self {
            node,
            generation,
            kind,
        };
        true
    }

    fn is_fresh_for(
        &self,
        cache: &CacheBase,
        graph: &BRepGraph,
        node: NodeId,
        kind: GenKind,
    ) -> bool {
        if self.kind != kind || self.node != node {
            return false;
        }
        generation_of(cache, graph, node, kind) == Some(self.generation)
    }
}

// The generation a node entry of the given kind reads (`None` for GenKind::None).
fn generation_of(cache: &CacheBase, graph: &BRepGraph, node: NodeId, kind: GenKind) -> Option<u32> {
    match kind {
        GenKind::Own => cache.node_own_gen(graph, node),
        GenKind::Subtree => cache.node_subtree_gen(graph, node),
        GenKind::None => None,
    }
}

/// `BRepGraph_Cache::RefEntry`: a reference-derived entry bound to the OwnGen of the
/// reference's parent definition.
#[derive(Clone, Copy, Debug, Default)]
pub struct RefEntry {
    reference: RefId,
    generation: u32,
    bound: bool,
}

impl RefEntry {
    /// `Reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// `BindOwnGen(theCache, theRef)`: false (and reset) when detached or inactive.
    pub fn bind_own_gen(&mut self, cache: &CacheBase, graph: &BRepGraph, reference: RefId) -> bool {
        let Some(generation) = cache.ref_own_gen(graph, reference) else {
            self.reset();
            return false;
        };
        *self = Self {
            reference,
            generation,
            bound: true,
        };
        true
    }

    /// `IsFreshOwn(theCache, theRef)`: bound to this reference's unchanged OwnGen.
    pub fn is_fresh_own(&self, cache: &CacheBase, graph: &BRepGraph, reference: RefId) -> bool {
        if !self.bound || self.reference != reference {
            return false;
        }
        cache.ref_own_gen(graph, reference) == Some(self.generation)
    }

    /// `IsFresh(theCache)`: `IsFreshOwn` for the captured reference.
    pub fn is_fresh(&self, cache: &CacheBase, graph: &BRepGraph) -> bool {
        self.is_fresh_own(cache, graph, self.reference)
    }

    /// `IsBound()`.
    pub fn is_bound(&self) -> bool {
        self.bound
    }

    /// `Ref()`: the reference captured at bind time.
    pub fn reference(&self) -> RefId {
        self.reference
    }
}

/// `BRepGraph_Cache::ItemEntry`: an item-derived entry bound to the item's OwnGen.
#[derive(Clone, Copy, Debug, Default)]
pub struct ItemEntry {
    item: ItemId,
    generation: u32,
    bound: bool,
}

impl ItemEntry {
    /// `Reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// `BindOwnGen(theCache, theItem)`: false (and reset) when detached or inactive.
    pub fn bind_own_gen(&mut self, cache: &CacheBase, graph: &BRepGraph, item: ItemId) -> bool {
        let Some(generation) = cache.item_own_gen(graph, item) else {
            self.reset();
            return false;
        };
        *self = Self {
            item,
            generation,
            bound: true,
        };
        true
    }

    /// `IsFreshOwn(theCache, theItem)`: bound to this item's unchanged OwnGen.
    pub fn is_fresh_own(&self, cache: &CacheBase, graph: &BRepGraph, item: ItemId) -> bool {
        if !self.bound || self.item != item {
            return false;
        }
        cache.item_own_gen(graph, item) == Some(self.generation)
    }

    /// `IsFresh(theCache)`: `IsFreshOwn` for the captured item.
    pub fn is_fresh(&self, cache: &CacheBase, graph: &BRepGraph) -> bool {
        self.is_fresh_own(cache, graph, self.item)
    }

    /// `IsBound()`.
    pub fn is_bound(&self) -> bool {
        self.bound
    }

    /// `Item()`: the item captured at bind time.
    pub fn item(&self) -> ItemId {
        self.item
    }
}
