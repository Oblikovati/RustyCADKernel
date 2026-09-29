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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_ItemId.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_ItemId`: a generic address of a definition node or a reference entry.
use super::hashing::{combine, hash_u32};
use super::node_id::{NodeId, NodeKind};
use super::ref_id::{RefId, RefKind};

/// `BRepGraph_ItemId::Domain` (and the identical `BRepGraph_ItemUID::Domain`): the
/// addressed item domain, stored as a `uint8_t`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ItemDomain {
    /// `Domain::None`.
    #[default]
    None = 0,
    /// `Domain::Node`.
    Node = 1,
    /// `Domain::Reference`.
    Reference = 2,
}

/// `BRepGraph_ItemId`: the domain, the per-kind index and the kind byte of a node or a
/// reference; representation records are not items. The default is invalid.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{ItemDomain, ItemId, NodeId, NodeKind};
/// let node = NodeId::new(NodeKind::FACE, 3);
/// let item = ItemId::from(node);
/// assert_eq!(item.item_domain(), ItemDomain::Node);
/// assert_eq!(item.node_id(), node);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemId {
    domain: ItemDomain,
    index: u32,
    kind: u8,
}

impl Default for ItemId {
    fn default() -> Self {
        Self {
            domain: ItemDomain::None,
            index: Self::INVALID_INDEX,
            kind: 0,
        }
    }
}

impl From<NodeId> for ItemId {
    /// `BRepGraph_ItemId(theNode)`: an invalid node gives the invalid item.
    fn from(node: NodeId) -> Self {
        if !node.is_valid() {
            return Self::default();
        }
        Self {
            domain: ItemDomain::Node,
            index: node.index,
            kind: node.kind.0 as u8,
        }
    }
}

impl From<RefId> for ItemId {
    /// `BRepGraph_ItemId(theRef)`: an invalid reference gives the invalid item.
    fn from(reference: RefId) -> Self {
        if !reference.is_valid() {
            return Self::default();
        }
        Self {
            domain: ItemDomain::Reference,
            index: reference.index,
            kind: reference.kind.0 as u8,
        }
    }
}

impl ItemId {
    const INVALID_INDEX: u32 = u32::MAX;

    /// `IsValid()`: an allocated index and a valid kind of its domain.
    pub fn is_valid(self) -> bool {
        if self.index == Self::INVALID_INDEX {
            return false;
        }
        match self.domain {
            ItemDomain::Node => NodeKind::is_valid_kind(self.raw_node_kind()),
            ItemDomain::Reference => RefKind::is_valid_kind(self.raw_ref_kind()),
            ItemDomain::None => false,
        }
    }

    /// `ItemDomain()`.
    pub fn item_domain(self) -> ItemDomain {
        self.domain
    }

    /// `IsNode()`.
    pub fn is_node(self) -> bool {
        self.domain == ItemDomain::Node
    }

    /// `IsReference()`.
    pub fn is_reference(self) -> bool {
        self.domain == ItemDomain::Reference
    }

    /// `NodeId()`: the node id, invalid for another domain.
    pub fn node_id(self) -> NodeId {
        if self.is_node() && NodeKind::is_valid_kind(self.raw_node_kind()) {
            return NodeId::new(self.raw_node_kind(), self.index);
        }
        NodeId::default()
    }

    /// `RefId()`: the reference id, invalid for another domain.
    pub fn ref_id(self) -> RefId {
        if self.is_reference() && RefKind::is_valid_kind(self.raw_ref_kind()) {
            return RefId::new(self.raw_ref_kind(), self.index);
        }
        RefId::default()
    }

    /// `NodeKind()`: the stored kind as a node kind (the check is a debug assertion
    /// natively, so the release build returns the stored byte for any domain).
    pub fn node_kind(self) -> NodeKind {
        self.raw_node_kind()
    }

    /// `RefKind()`: the stored kind as a reference kind (release behaviour, as
    /// [`node_kind`](Self::node_kind)).
    pub fn ref_kind(self) -> RefKind {
        self.raw_ref_kind()
    }

    /// `RawKind()` and `Kind()`: the kind byte in its domain's enumeration.
    pub fn raw_kind(self) -> u8 {
        self.kind
    }

    /// `Index()`: the per-kind index.
    pub fn index(self) -> u32 {
        self.index
    }

    /// `std::hash<BRepGraph_ItemId>`: `hashBytes({hash(domain), hash(kind), hash(index)})`.
    pub fn hash_code(self) -> usize {
        combine(&[
            hash_u32(self.domain as u32),
            hash_u32(u32::from(self.kind)),
            hash_u32(self.index),
        ])
    }

    fn raw_node_kind(self) -> NodeKind {
        NodeKind(i32::from(self.kind))
    }

    fn raw_ref_kind(self) -> RefKind {
        RefKind(i32::from(self.kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_keep_their_domain() {
        let reference = RefId::new(RefKind::WIRE, 4);
        let item = ItemId::from(reference);
        assert!(item.is_reference() && item.is_valid());
        assert_eq!(item.ref_id(), reference);
        assert_eq!(item.raw_kind(), 2);
        assert!(!item.node_id().is_valid());
        assert_eq!(item.index(), 4);
        assert_eq!(item.ref_kind(), RefKind::WIRE);
        assert_ne!(item.hash_code(), ItemId::default().hash_code());
    }
}
