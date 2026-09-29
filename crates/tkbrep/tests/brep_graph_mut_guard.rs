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

//! Direct translation of BRepGraph_MutGuard_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported).
//!
//! Regression coverage for BRepGraph_MutGuard<T>: move semantics, inert-state
//! detection (operator bool()), and exception-path notification safety.
//!
//! The C++ move constructor is `MutGuard::move_from`; a guard holds the graph's only
//! mutable borrow, so a second guard is taken through the first one's `graph_mut()`
//! and a guarded setter is the `*_guarded` associated function
//! (docs/topology-design.md section 11.4).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::editor::{EdgeOps, VertexOps};
use rustycad_tkbrep::brep_graph::mut_guard::MutGuard;
use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId, FaceRefId, VertexId};
use rustycad_tkmath::gp::Point3;
use support::make_box;

fn makeBoxGraph() -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes1 = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    aGraph
}

fn vertex_own_gen(aGraph: &BRepGraph, aVertex: VertexId) -> u32 {
    aGraph.topo().vertices().definition(aVertex).base.own_gen
}

#[test]
fn BRepGraph_MutGuardTest__OperatorBool_TrueWhenOwned_FalseAfterMove() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let mut aGuard = aGraph
        .editor()
        .vertices()
        .mutate(VertexId::start())
        .unwrap();
    assert!(aGuard.is_active());

    let aMoved = MutGuard::move_from(&mut aGuard);
    assert!(!aGuard.is_active(), "Moved-from guard must be inert");
    assert!(aMoved.is_active());
}

#[test]
fn BRepGraph_MutGuardTest__DereferenceAfterMove_Throws() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let mut aGuard = aGraph
        .editor()
        .vertices()
        .mutate(VertexId::start())
        .unwrap();

    let aMoved = MutGuard::move_from(&mut aGuard);
    let _ = &aMoved;
    // Using operator-> or operator* on the moved-from guard must raise.
    assert_eq!(
        aGuard.try_get().unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    assert_eq!(
        aGuard.try_get().unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}

#[test]
fn BRepGraph_MutGuardTest__MoveAssignmentFlushesThenTransfers() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aGenBefore = vertex_own_gen(&aGraph, VertexId::start());

    {
        let mut aFirst = aGraph
            .editor()
            .vertices()
            .mutate(VertexId::start())
            .unwrap();
        VertexOps::set_point_guarded(&mut aFirst, Point3::new(1.0, 2.0, 3.0));

        let mut aSecond = aFirst
            .graph_mut()
            .editor()
            .vertices()
            .mutate(VertexId::new(1))
            .unwrap();
        aSecond.mark_dirty();
        // `aSecond = std::move(aFirst)`: the assignment flushes aSecond's own item
        // first, which ends its borrow of aFirst, then takes aFirst's item over.
        drop(aSecond);
        let aSecond = MutGuard::move_from(&mut aFirst);
        // aFirst now inert; aSecond owns what was aFirst.
        assert!(!aFirst.is_active());
        assert!(aSecond.is_active());
    }

    // Both vertex 0 and vertex 1 must have had their OwnGen bumped exactly once each.
    let aGenAfterV0 = vertex_own_gen(&aGraph, VertexId::start());
    let aGenAfterV1 = vertex_own_gen(&aGraph, VertexId::new(1));
    assert!(aGenAfterV0 > aGenBefore);
    assert!(aGenAfterV1 > 0);
}

#[test]
fn BRepGraph_MutGuardTest__ExceptionInsideScope_StillNotifies() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aGenBefore = vertex_own_gen(&aGraph, VertexId::start());

    let aScope = |aGraph: &mut BRepGraph| -> Result<(), String> {
        let mut aGuard = aGraph
            .editor()
            .vertices()
            .mutate(VertexId::start())
            .unwrap();
        VertexOps::set_point_guarded(&mut aGuard, Point3::new(9.0, 9.0, 9.0));
        Err("test: throw mid-scope".to_owned())
    };
    // Fall through. The guard's destructor must have executed while the scope was
    // left and must have bumped OwnGen.
    assert!(aScope(&mut aGraph).is_err());

    let aGenAfter = vertex_own_gen(&aGraph, VertexId::start());
    assert!(
        aGenAfter > aGenBefore,
        "Notification must fire even when the scope exits via an exception"
    );
}

#[test]
fn BRepGraph_MutGuardTest__MovedFrom_DoesNotDoubleNotify() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aGenBefore = vertex_own_gen(&aGraph, VertexId::start());

    {
        let mut aSrc = aGraph
            .editor()
            .vertices()
            .mutate(VertexId::start())
            .unwrap();
        aSrc.mark_dirty();
        let _aDst = MutGuard::move_from(&mut aSrc);
        // Only aDst's destructor should notify; aSrc's destructor must be a no-op.
    }

    let aGenAfter = vertex_own_gen(&aGraph, VertexId::start());
    assert_eq!(
        aGenAfter,
        aGenBefore + 1,
        "Move must transfer the notification obligation - no double-bump"
    );
}

#[test]
fn BRepGraph_MutGuardTest__RefGuard_SameMoveSemantics() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    assert!(aGraph.refs().faces().nb() > 0);

    let aRefId = FaceRefId::new(0);
    let mut aGuard = aGraph.editor().faces().mutate_ref(aRefId).unwrap();
    assert!(aGuard.is_active());

    let aMoved = MutGuard::move_from(&mut aGuard);
    assert!(!aGuard.is_active());
    assert!(aMoved.is_active());
}

#[test]
fn BRepGraph_MutGuardTest__DuplicateGuard_Rejected() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let mut aGuard1 = aGraph
        .editor()
        .vertices()
        .mutate(VertexId::start())
        .unwrap();
    assert!(aGuard1.is_active());

    // Attempting to acquire a second guard on the same vertex must throw.
    let aDuplicate = aGuard1
        .graph_mut()
        .editor()
        .vertices()
        .mutate(VertexId::start());
    assert_eq!(
        aDuplicate.unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}

#[test]
fn BRepGraph_MutGuardTest__GuardAllowsGuardedSetter() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    {
        let mut aGuard = aGraph.editor().edges().mutate(EdgeId::new(0)).unwrap();
        // Guarded setter must NOT self-block via requireUnlocked.
        EdgeOps::set_tolerance_guarded(&mut aGuard, 0.5);
        assert!(aGuard.is_dirty());
    }
}

#[test]
fn BRepGraph_MutGuardTest__GuardBlocksStructuralRemoval() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let mut aGuard = aGraph
        .editor()
        .vertices()
        .mutate(VertexId::start())
        .unwrap();
    assert!(aGuard.is_active());

    // Structural removal must throw while a guard is active on the item.
    let aRemoval = aGuard
        .graph_mut()
        .editor()
        .generic()
        .remove_node(VertexId::start().into());
    assert_eq!(
        aRemoval.unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}

#[test]
fn BRepGraph_MutGuardTest__GuardReleasedOnDestruction_CanReacquire() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    {
        let aGuard = aGraph
            .editor()
            .vertices()
            .mutate(VertexId::start())
            .unwrap();
        assert!(aGuard.is_active());
    }
    // Guard destroyed - should be able to acquire again.
    let aGuard2 = aGraph
        .editor()
        .vertices()
        .mutate(VertexId::start())
        .unwrap();
    assert!(aGuard2.is_active());
}

#[test]
fn BRepGraph_MutGuardTest__ClearRejectsActiveGuard() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    {
        let mut aGuard = aGraph
            .editor()
            .vertices()
            .mutate(VertexId::start())
            .unwrap();
        assert!(aGuard.is_active());
        assert_eq!(
            aGuard.graph_mut().clear().unwrap_err().exception_type(),
            "Standard_ProgramError"
        );
    }

    assert!(aGraph.clear().is_ok());
}
