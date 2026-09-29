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
// Rust translation of the `NCollection_DefaultHasher` use of the BRepGraph id hashes
// (std::hash specialisations in BRepGraph_NodeId.hxx, BRepGraph_ItemId.hxx,
// BRepGraph_UID.hxx and BRepGraph_ItemUID.hxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native hashes of the BRepGraph ids as TKernel map keys.
//!
//! `NCollection_DataMap<BRepGraph_NodeId, ...>` and `NCollection_FlatMap<...>` hash
//! their keys with `NCollection_DefaultHasher`, which is `std::hash` of the id. The
//! layer maps whose iteration order reaches a result use the TKernel map ports, so
//! the ids hash as natively (docs/topology-design.md section 11.9).
use crate::brep_graph::{ItemId, ItemUid, NodeId, Uid};
use rustycad_tkernel::ncollection::HashKey;

impl HashKey for NodeId {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}

impl HashKey for ItemId {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}

impl HashKey for Uid {
    fn occt_hash(&self) -> usize {
        self.hash_value()
    }
}

impl HashKey for ItemUid {
    fn occt_hash(&self) -> usize {
        self.hash_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::NodeKind;

    #[test]
    fn keys_hash_with_the_native_id_hashes() {
        let node = NodeId::new(NodeKind::FACE, 3);
        assert_eq!(node.occt_hash(), node.hash_code());
        let item = ItemId::from(node);
        assert_eq!(item.occt_hash(), item.hash_code());
        let uid = Uid::new(NodeKind::EDGE, 7);
        assert_eq!(uid.occt_hash(), uid.hash_value());
        let item_uid = ItemUid::node(NodeKind::EDGE, 7);
        assert_eq!(item_uid.occt_hash(), item_uid.hash_value());
    }
}
