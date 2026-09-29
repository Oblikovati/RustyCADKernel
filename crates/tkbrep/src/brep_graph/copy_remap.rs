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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_CopyRemap.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_CopyRemap`: the context a structural copy or compaction hands to the
//! layer and cache copy callbacks: the source and target graphs, the source-to-target
//! item map (or the identity mapping) and the migration mode.
//!
//! The native context holds a `const BRepGraph*` source and a `BRepGraph*` target that
//! alias during compaction (the layers migrate into the same, rebuilt graph), and
//! hands out the mutable target from a `const` context. The Rust context owns the one
//! mutable borrow of the target: [`CopyRemap::new`] and [`CopyRemap::with_mapping`]
//! take distinct graphs, [`CopyRemap::in_place`] one graph that is both, and
//! [`CopyRemap::target_graph`] takes `&mut self`, so the copy callbacks receive
//! `&mut CopyRemap`.
use crate::brep_graph::{BRepGraph, ItemId, ItemUid};
use rustycad_tkernel::ncollection::FlatDataMap;

/// `BRepGraph_CopyRemap::Mode`: copy or compaction semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Mode {
    /// `Copy`: source and target are distinct graphs.
    #[default]
    Copy = 0,
    /// `Compact`: layers migrate into the same, rebuilt graph.
    Compact = 1,
}

/// `BRepGraph_CopyRemap::MappingKind`: an explicit item map or the identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MappingKind {
    /// `Explicit`: the item map resolves source to target items.
    #[default]
    Explicit = 0,
    /// `Identity`: source and target ids are equal (a full identity copy).
    Identity = 1,
}

/// `BRepGraph_CopyRemap::ItemMap`: `NCollection_FlatDataMap<ItemId, ItemId>`.
pub type ItemMap = FlatDataMap<ItemId, ItemId>;

// The graphs of a context: two distinct graphs, or one graph for a compaction.
enum Graphs<'a> {
    Distinct {
        source: &'a BRepGraph,
        target: &'a mut BRepGraph,
    },
    Same(&'a mut BRepGraph),
}

/// `BRepGraph_CopyRemap`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::copy_remap::{CopyRemap, ItemMap, Mode};
/// let (source, mut target, items) = (BRepGraph::new(), BRepGraph::new(), ItemMap::default());
/// let remap = CopyRemap::new(&source, &mut target, &items, Mode::Copy);
/// assert!(!remap.is_compact());
/// ```
pub struct CopyRemap<'a> {
    graphs: Graphs<'a>,
    items: Option<&'a ItemMap>,
    mode: Mode,
    mapping_kind: MappingKind,
}

impl<'a> CopyRemap<'a> {
    /// `BRepGraph_CopyRemap(theSource, theTarget, theItemRemap, theMode)`: an explicit
    /// item map.
    pub fn new(
        source: &'a BRepGraph,
        target: &'a mut BRepGraph,
        items: &'a ItemMap,
        mode: Mode,
    ) -> Self {
        Self {
            graphs: Graphs::Distinct { source, target },
            items: Some(items),
            mode,
            mapping_kind: MappingKind::Explicit,
        }
    }

    /// `BRepGraph_CopyRemap(theSource, theTarget, theMappingKind, theMode)`: no item
    /// map (the identity constructor; an explicit kind then maps nothing).
    pub fn with_mapping(
        source: &'a BRepGraph,
        target: &'a mut BRepGraph,
        mapping_kind: MappingKind,
        mode: Mode,
    ) -> Self {
        Self {
            graphs: Graphs::Distinct { source, target },
            items: None,
            mode,
            mapping_kind,
        }
    }

    /// The native constructors with the same graph as source and target (the
    /// compaction call sites): `items` is the explicit map, or `None` for the
    /// identity-mapping constructor with `mapping_kind`.
    pub fn in_place(
        graph: &'a mut BRepGraph,
        items: Option<&'a ItemMap>,
        mapping_kind: MappingKind,
        mode: Mode,
    ) -> Self {
        Self {
            graphs: Graphs::Same(graph),
            items,
            mode,
            mapping_kind: if items.is_some() {
                MappingKind::Explicit
            } else {
                mapping_kind
            },
        }
    }

    /// `CopyMode()`.
    pub fn copy_mode(&self) -> Mode {
        self.mode
    }

    /// `IsCompact()`.
    pub fn is_compact(&self) -> bool {
        self.mode == Mode::Compact
    }

    /// `SourceGraph()`.
    pub fn source_graph(&self) -> &BRepGraph {
        match &self.graphs {
            Graphs::Distinct { source, .. } => source,
            Graphs::Same(graph) => graph,
        }
    }

    /// `TargetGraph()`: the mutable target.
    pub fn target_graph(&mut self) -> &mut BRepGraph {
        match &mut self.graphs {
            Graphs::Distinct { target, .. } => target,
            Graphs::Same(graph) => graph,
        }
    }

    /// `TargetGraphConst()`.
    pub fn target_graph_const(&self) -> &BRepGraph {
        match &self.graphs {
            Graphs::Distinct { target, .. } => target,
            Graphs::Same(graph) => graph,
        }
    }

    /// `Items()`: the source-to-target item map; `None` for the identity mapping,
    /// whose native context holds a null map.
    pub fn items(&self) -> Option<&'a ItemMap> {
        self.items
    }

    /// `TargetItem(theSourceItem)`: the source item itself for the identity mapping,
    /// else the mapped item; invalid for an invalid or unmapped source item.
    pub fn target_item(&self, source_item: ItemId) -> ItemId {
        if !source_item.is_valid() {
            return ItemId::default();
        }
        if self.mapping_kind == MappingKind::Identity {
            return source_item;
        }
        self.items
            .and_then(|items| items.seek(&source_item).copied())
            .unwrap_or_default()
    }

    /// `TargetItemOrInvalid(theSourceItem)`: the same as [`target_item`](Self::target_item).
    pub fn target_item_or_invalid(&self, source_item: ItemId) -> ItemId {
        self.target_item(source_item)
    }

    /// `HasTargetItem(theSourceItem)`.
    pub fn has_target_item(&self, source_item: ItemId) -> bool {
        self.target_item(source_item).is_valid()
    }

    /// `SourceUID(theSourceItem)`: the item's UID in the source graph.
    pub fn source_uid(&self, source_item: ItemId) -> ItemUid {
        self.source_graph().uids().of_item(source_item)
    }

    /// `TargetUID(theTargetItem)`: the item's UID in the target graph.
    pub fn target_uid(&self, target_item: ItemId) -> ItemUid {
        self.target_graph_const().uids().of_item(target_item)
    }

    /// `TargetUIDFromSource(theSourceItem)`: the target UID of the mapped item.
    pub fn target_uid_from_source(&self, source_item: ItemId) -> ItemUid {
        self.target_uid(self.target_item_or_invalid(source_item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{FaceId, NodeId};

    #[test]
    fn explicit_and_identity_mappings() {
        let mut source = BRepGraph::new();
        let face = source.inc_storage_mut().append_face();
        let item = ItemId::from(NodeId::from(face));
        let mut items = ItemMap::default();
        items.bind(item, ItemId::from(NodeId::from(FaceId::new(4))));
        let mut target = BRepGraph::new();
        let remap = CopyRemap::new(&source, &mut target, &items, Mode::Copy);
        assert_eq!(remap.target_item(item).index(), 4);
        assert!(!remap.has_target_item(ItemId::from(NodeId::from(FaceId::new(1)))));
        assert!(!remap.target_item(ItemId::default()).is_valid());
        let identity =
            CopyRemap::with_mapping(&source, &mut target, MappingKind::Identity, Mode::Copy);
        assert_eq!(identity.target_item(item), item);
        assert!(identity.items().is_none());
        let mut graph = BRepGraph::new();
        let mut compact = CopyRemap::in_place(
            &mut graph,
            Some(&items),
            MappingKind::Identity,
            Mode::Compact,
        );
        assert!(compact.is_compact() && compact.target_graph().is_empty());
        assert_eq!(compact.target_item(item).index(), 4);
    }
}
