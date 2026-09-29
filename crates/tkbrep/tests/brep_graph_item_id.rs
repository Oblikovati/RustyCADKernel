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

//! Direct translation of BRepGraph_ItemId_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::{ItemDomain, ItemId, NodeId, NodeKind, RefId, RefKind};

#[test]
fn BRepGraph_ItemIdTest__Default_IsInvalid() {
    let anItem = ItemId::default();

    assert!(!anItem.is_valid());
    assert!(!anItem.is_node());
    assert!(!anItem.is_reference());
}

#[test]
fn BRepGraph_ItemIdTest__NodeRoundTrip() {
    let aNode = NodeId::new(NodeKind::FACE, 3u32);
    let anItem = ItemId::from(aNode);

    assert!(anItem.is_valid());
    assert!(anItem.is_node());
    assert!(!anItem.is_reference());
    assert_eq!(anItem.item_domain(), ItemDomain::Node);
    assert_eq!(anItem.node_kind(), NodeKind::FACE);
    assert_eq!(anItem.node_id(), aNode);
    assert!(!anItem.ref_id().is_valid());
}

#[test]
fn BRepGraph_ItemIdTest__ReferenceRoundTrip() {
    let aRef = RefId::new(RefKind::WIRE, 4u32);
    let anItem = ItemId::from(aRef);

    assert!(anItem.is_valid());
    assert!(!anItem.is_node());
    assert!(anItem.is_reference());
    assert_eq!(anItem.item_domain(), ItemDomain::Reference);
    assert_eq!(anItem.ref_kind(), RefKind::WIRE);
    assert_eq!(anItem.ref_id(), aRef);
    assert!(!anItem.node_id().is_valid());
}

#[test]
fn BRepGraph_ItemIdTest__InvalidSourceKindProducesInvalidItem() {
    const THE_RESERVED_KIND: i32 = 9;
    let aNode = NodeId::new(NodeKind(THE_RESERVED_KIND), 0u32);
    let anItem = ItemId::from(aNode);

    assert!(!aNode.is_valid());
    assert!(!anItem.is_valid());
    assert_eq!(anItem.item_domain(), ItemDomain::None);
}
