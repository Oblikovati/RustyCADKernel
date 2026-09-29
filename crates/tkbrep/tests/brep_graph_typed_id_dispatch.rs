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

//! Direct translation of BRepGraph_TypedIdDispatch_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. The generic lambdas are visitor structs
//! and `std::is_same_v` is a `TypeId` comparison of the kind tags.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::{
    FaceTag, NodeId, NodeKind, NodeTag, NodeVisitor, RefId, RefKind, RefTag, RefVisitor, TypedId,
    VertexRefTag,
};
use std::any::TypeId;

struct FaceProbe<'a> {
    isFace: &'a mut bool,
}

impl NodeVisitor for FaceProbe<'_> {
    type Output = i32;
    fn visit<T: NodeTag>(self, theTypedId: TypedId<T>) -> i32 {
        *self.isFace = TypeId::of::<T>() == TypeId::of::<FaceTag>();
        theTypedId.index as i32
    }
}

struct VertexRefProbe<'a> {
    isVertex: &'a mut bool,
}

impl RefVisitor for VertexRefProbe<'_> {
    type Output = i32;
    fn visit<T: RefTag>(self, theTypedId: TypedId<T>) -> i32 {
        *self.isVertex = TypeId::of::<T>() == TypeId::of::<VertexRefTag>();
        theTypedId.index as i32
    }
}

#[test]
fn BRepGraph_TypedIdDispatchTest__VisitNodeId_ConvertsToMatchingTypedId() {
    let aNodeId = NodeId::new(NodeKind::FACE, 4);

    let mut isFace = false;
    let anIdx = aNodeId.visit(FaceProbe {
        isFace: &mut isFace,
    });

    assert!(isFace);
    assert_eq!(anIdx, 4);
}

#[test]
fn BRepGraph_TypedIdDispatchTest__VisitRefId_ConvertsToMatchingTypedId() {
    let aRefId = RefId::new(RefKind::VERTEX, 6);

    let mut isVertex = false;
    let anIdx = aRefId.visit(VertexRefProbe {
        isVertex: &mut isVertex,
    });

    assert!(isVertex);
    assert_eq!(anIdx, 6);
}
