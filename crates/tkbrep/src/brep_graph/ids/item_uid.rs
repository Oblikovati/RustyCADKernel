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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_ItemUID.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_ItemUID`: the durable identity of a node or a reference entry.
use super::hashing::{combine, hash_i32, hash_u32};
use super::item_id::ItemDomain;
use super::node_id::NodeKind;
use super::ref_id::RefKind;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// `BRepGraph_ItemUID`: a 32-bit counter (0 is invalid), the domain and the kind byte.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{ItemUid, NodeKind};
/// let uid = ItemUid::node(NodeKind::EDGE, 4);
/// assert!(uid.is_valid() && uid.is_node());
/// assert!(!ItemUid::node(NodeKind::EDGE, u64::from(u32::MAX) as usize + 1).is_valid());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ItemUid {
    counter: u32,
    domain: ItemDomain,
    kind: u8,
}

impl ItemUid {
    /// `Node(theKind, theCounter)`.
    pub fn node(kind: NodeKind, counter: usize) -> Self {
        Self::from_parts(ItemDomain::Node, kind.0 as u8, counter)
    }

    /// `Reference(theKind, theCounter)`.
    pub fn reference(kind: RefKind, counter: usize) -> Self {
        Self::from_parts(ItemDomain::Reference, kind.0 as u8, counter)
    }

    /// `Invalid()`.
    pub fn invalid() -> Self {
        Self::default()
    }

    // The private constructor: a zero or a counter beyond 32 bits keeps the invalid
    // counter 0 (the assertions are debug-only natively).
    fn from_parts(domain: ItemDomain, kind: u8, counter: usize) -> Self {
        let counter = u32::try_from(counter).unwrap_or(0);
        Self {
            counter,
            domain,
            kind,
        }
    }

    /// `IsValid()`: a non-zero counter and a valid kind of its domain.
    pub fn is_valid(self) -> bool {
        if self.counter == 0 {
            return false;
        }
        match self.domain {
            ItemDomain::Node => NodeKind::is_valid_kind(NodeKind(i32::from(self.kind))),
            ItemDomain::Reference => RefKind::is_valid_kind(RefKind(i32::from(self.kind))),
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

    /// `NodeKind()`: the stored kind byte (the domain check is debug-only natively).
    pub fn node_kind(self) -> NodeKind {
        NodeKind(i32::from(self.kind))
    }

    /// `RefKind()`: the stored kind byte (release behaviour, as `NodeKind()`).
    pub fn ref_kind(self) -> RefKind {
        RefKind(i32::from(self.kind))
    }

    /// `RawKind()`.
    pub fn raw_kind(self) -> u8 {
        self.kind
    }

    /// `Counter()`.
    pub fn counter(self) -> usize {
        self.counter as usize
    }

    /// `HashValue()`: `hash(0)` when invalid, else the domain, kind and counter hashes
    /// (the counter hashed as the `uint32_t` it is stored as).
    pub fn hash_value(self) -> usize {
        if self.counter == 0 {
            return hash_i32(0);
        }
        combine(&[
            hash_u32(self.domain as u32),
            hash_u32(u32::from(self.kind)),
            hash_u32(self.counter),
        ])
    }
}

impl PartialEq for ItemUid {
    fn eq(&self, other: &Self) -> bool {
        if self.counter == 0 || other.counter == 0 {
            return (self.counter == 0) == (other.counter == 0);
        }
        (self.domain, self.kind, self.counter) == (other.domain, other.kind, other.counter)
    }
}

impl Eq for ItemUid {}

impl PartialOrd for ItemUid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemUid {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.domain, self.kind, self.counter).cmp(&(other.domain, other.kind, other.counter))
    }
}

impl Hash for ItemUid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_value());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_uids_compare_by_domain_kind_counter() {
        let node = ItemUid::node(NodeKind::FACE, 2);
        let reference = ItemUid::reference(RefKind::FACE, 2);
        assert_ne!(node, reference);
        assert!(node < reference);
        assert_eq!(ItemUid::invalid(), ItemUid::node(NodeKind::FACE, 0));
        assert_eq!(node.counter(), 2);
        assert_eq!(reference.ref_kind(), RefKind::FACE);
        assert_eq!(ItemUid::invalid().hash_value(), hash_i32(0));
    }
}
