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

//! Direct translation of BRepGraph_DeferredInvalidation_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). A `DeferredScope` holds the graph's
//! mutable borrow, so the scoped edits and queries go through it.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::deferred_scope::DeferredScope;
use rustycad_tkbrep::brep_graph::iterator::*;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::osd::Parallel;
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use std::sync::Mutex;
use support::make_box;
use support::ref_test_tools as BRepGraph_TestTools;

/// The `BRepGraph_DeferredInvalidationTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!myGraph.is_empty());
    myGraph
}

fn set_edge_tolerance(aGraph: &mut BRepGraph, anEdge: EdgeId, aTolerance: f64) {
    aGraph
        .editor()
        .edges()
        .set_tolerance(anEdge, aTolerance)
        .unwrap();
}

fn edge_own_gen(aGraph: &BRepGraph) -> u32 {
    aGraph
        .topo()
        .edges()
        .definition(EdgeId::start())
        .base
        .own_gen
}

fn wire_subtree_gen(aGraph: &BRepGraph, aWire: WireId) -> u32 {
    aGraph.topo().wires().definition(aWire).base.subtree_gen
}

fn face_subtree_gen(aGraph: &BRepGraph, aFace: FaceId) -> u32 {
    aGraph.topo().faces().definition(aFace).base.subtree_gen
}

fn shell_subtree_gen(aGraph: &BRepGraph) -> u32 {
    aGraph
        .topo()
        .shells()
        .definition(ShellId::start())
        .base
        .subtree_gen
}

fn solid_subtree_gen(aGraph: &BRepGraph) -> u32 {
    aGraph
        .topo()
        .solids()
        .definition(SolidId::start())
        .base
        .subtree_gen
}

fn first_wire_of_first_edge(aGraph: &BRepGraph) -> WireId {
    let aWireIt = aGraph.topo().edges().wires_of(EdgeId::start());
    assert!(aWireIt.more());
    aWireIt.current_id()
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_EdgeMutation_IncrementsOwnGen() {
    let mut myGraph = SetUp();
    assert_eq!(edge_own_gen(&myGraph), 0);

    myGraph.editor().begin_deferred_invalidation();
    set_edge_tolerance(&mut myGraph, EdgeId::start(), 0.5);
    // In deferred mode, the entity's OwnGen is incremented.
    assert!(edge_own_gen(&myGraph) > 0);
    myGraph.editor().end_deferred_invalidation();
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_PropagatesUpOnFlush() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();
    set_edge_tolerance(&mut myGraph, EdgeId::start(), 0.5);

    // During deferred mode: edge is mutated, but parent wire/face are NOT yet.
    let aWireId = first_wire_of_first_edge(&myGraph);
    assert_eq!(wire_subtree_gen(&myGraph, aWireId), 0);

    myGraph.editor().end_deferred_invalidation();

    // After flush: wire and face SubtreeGen should be propagated.
    assert!(wire_subtree_gen(&myGraph, aWireId) > 0);

    // Check propagation to face.
    let mut aFaceIt = FaceIterator::new(&myGraph);
    while aFaceIt.more() {
        if BRepGraph_TestTools::face_uses_wire(&myGraph, aFaceIt.current_id(), aWireId) {
            assert!(aFaceIt.current().base.subtree_gen > 0);
            break;
        }
        aFaceIt.next();
    }

    // Check propagation to shell and solid.
    assert!(shell_subtree_gen(&myGraph) > 0);
    assert!(solid_subtree_gen(&myGraph) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_DirectFaceMutation_PropagatesUp() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();
    myGraph
        .editor()
        .faces()
        .mutate(FaceId::start())
        .unwrap()
        .mark_dirty();

    // Face was directly mutated: OwnGen incremented.
    assert!(
        myGraph
            .topo()
            .faces()
            .definition(FaceId::start())
            .base
            .own_gen
            > 0
    );
    // Shell not yet propagated during deferred mode.
    assert_eq!(shell_subtree_gen(&myGraph), 0);

    myGraph.editor().end_deferred_invalidation();

    // After flush: shell and solid SubtreeGen should be propagated.
    assert!(shell_subtree_gen(&myGraph) > 0);
    assert!(solid_subtree_gen(&myGraph) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_DirectShellMutation_PropagatesUp() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();
    myGraph
        .editor()
        .shells()
        .mutate(ShellId::start())
        .unwrap()
        .mark_dirty();

    // Shell was directly mutated: OwnGen incremented.
    assert!(
        myGraph
            .topo()
            .shells()
            .definition(ShellId::start())
            .base
            .own_gen
            > 0
    );
    // Solid not yet propagated during deferred mode.
    assert_eq!(solid_subtree_gen(&myGraph), 0);

    myGraph.editor().end_deferred_invalidation();

    // After flush: solid SubtreeGen should be propagated.
    assert!(solid_subtree_gen(&myGraph) > 0);
}

/// The SubtreeGen of every active node of one kind.
fn all_subtree_gens<T: NodeTag>(aGraph: &BRepGraph) -> Vec<u32> {
    use rustycad_tkbrep::brep_graph_inc::HasBaseDef;
    DefIterator::<T>::new(aGraph)
        .map(|aDef| aDef.base().subtree_gen)
        .collect()
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_MultipleEdges_BatchPropagation() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();

    let anEdges: Vec<EdgeId> = {
        let mut anEdgeIt = EdgeIterator::new(&myGraph);
        let mut anIds = Vec::new();
        while anEdgeIt.more() {
            anIds.push(anEdgeIt.current_id());
            anEdgeIt.next();
        }
        anIds
    };
    for anEdge in anEdges {
        set_edge_tolerance(&mut myGraph, anEdge, 0.1);
    }

    // During deferred mode: all edges mutated, but no parent propagation yet.
    for aGen in all_subtree_gens::<WireTag>(&myGraph) {
        assert_eq!(aGen, 0);
    }

    myGraph.editor().end_deferred_invalidation();

    // After flush: all wires, faces, shells, solids should have SubtreeGen propagated.
    for aGen in all_subtree_gens::<WireTag>(&myGraph) {
        assert!(aGen > 0);
    }
    for aGen in all_subtree_gens::<FaceTag>(&myGraph) {
        assert!(aGen > 0);
    }
    assert!(shell_subtree_gen(&myGraph) > 0);
    assert!(solid_subtree_gen(&myGraph) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_ReconstructAfterFlush_Succeeds() {
    let mut myGraph = SetUp();
    // Modify an edge in deferred mode and verify reconstruction still works.
    myGraph.editor().begin_deferred_invalidation();
    set_edge_tolerance(&mut myGraph, EdgeId::start(), 0.5);
    myGraph.editor().end_deferred_invalidation();

    // Reconstruction should succeed (shape cache was cleared on flush).
    let aSolidId: NodeId = SolidId::start().into();
    let aShape = myGraph.shapes().reconstruct(aSolidId);
    assert!(aShape.is_ok());
    assert!(!aShape.unwrap().is_null());
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_ParallelMutation_WithExternalSync() {
    let mut myGraph = SetUp();
    let aNbEdges = myGraph.topo().edges().nb();
    assert!(aNbEdges > 1);

    // Deferred mode is NOT internally thread-safe. Parallel callers must
    // provide external synchronization around each MutGuard usage.
    myGraph.editor().begin_deferred_invalidation();
    {
        let aMutex = Mutex::new(&mut myGraph);
        Parallel::for_range(
            0,
            aNbEdges as i32,
            |theIdx| {
                let mut aLock = aMutex.lock().unwrap();
                let aTolerance = 0.1 + f64::from(theIdx) * 0.01;
                set_edge_tolerance(&mut aLock, EdgeId::new(theIdx as u32), aTolerance);
            },
            false,
        )
        .unwrap();
    }
    myGraph.editor().end_deferred_invalidation();

    // All edges should be mutated (directly: OwnGen).
    let mut anEdgeIt = EdgeIterator::new(&myGraph);
    while anEdgeIt.more() {
        assert!(anEdgeIt.current().base.own_gen > 0);
        let anExpected = 0.1 + f64::from(anEdgeIt.current_id().index) * 0.01;
        assert!((anEdgeIt.current().tolerance - anExpected).abs() <= precision::confusion());
        anEdgeIt.next();
    }

    // Propagation should have happened on flush (parents: SubtreeGen).
    assert!(shell_subtree_gen(&myGraph) > 0);
    assert!(solid_subtree_gen(&myGraph) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_NoMutations_FlushIsSafe() {
    let mut myGraph = SetUp();
    // Begin/End with no mutations in between should be a no-op.
    myGraph.editor().begin_deferred_invalidation();
    myGraph.editor().end_deferred_invalidation();

    // Nothing should be mutated.
    let aTopo = myGraph.topo();
    assert_eq!(aTopo.edges().definition(EdgeId::start()).base.own_gen, 0);
    assert_eq!(aTopo.wires().definition(WireId::start()).base.own_gen, 0);
    assert_eq!(aTopo.faces().definition(FaceId::start()).base.own_gen, 0);
    assert_eq!(aTopo.shells().definition(ShellId::start()).base.own_gen, 0);
    assert_eq!(aTopo.solids().definition(SolidId::start()).base.own_gen, 0);
    assert_eq!(
        aTopo.edges().definition(EdgeId::start()).base.subtree_gen,
        0
    );
    assert_eq!(
        aTopo.wires().definition(WireId::start()).base.subtree_gen,
        0
    );
    assert_eq!(
        aTopo.faces().definition(FaceId::start()).base.subtree_gen,
        0
    );
    assert_eq!(
        aTopo.shells().definition(ShellId::start()).base.subtree_gen,
        0
    );
    assert_eq!(
        aTopo.solids().definition(SolidId::start()).base.subtree_gen,
        0
    );
}

#[test]
fn BRepGraph_DeferredInvalidationTest__EndWithoutBegin_IsIdempotent() {
    let mut myGraph = SetUp();
    // Calling End without Begin should be a safe no-op.
    myGraph.editor().end_deferred_invalidation();

    // Nothing should be mutated or cleared.
    assert_eq!(edge_own_gen(&myGraph), 0);
    assert_eq!(
        myGraph
            .topo()
            .solids()
            .definition(SolidId::start())
            .base
            .own_gen,
        0
    );
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredScope_NestedGuards_FlushOnlyOnOuterDestruction() {
    let mut myGraph = SetUp();
    let aWireId = first_wire_of_first_edge(&myGraph);

    {
        let mut anOuterScope = DeferredScope::new(&mut myGraph);
        assert!(anOuterScope.editor_state().is_deferred_mode());

        {
            let mut anInnerScope = DeferredScope::new(&mut anOuterScope);
            assert!(anInnerScope.editor_state().is_deferred_mode());
            set_edge_tolerance(&mut anInnerScope, EdgeId::start(), 0.5);
        }

        assert!(anOuterScope.editor_state().is_deferred_mode());
        assert_eq!(wire_subtree_gen(&anOuterScope, aWireId), 0);
    }

    assert!(!myGraph.editor_state().is_deferred_mode());
    assert!(wire_subtree_gen(&myGraph, aWireId) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_DoubleEnd_IsIdempotent() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();
    set_edge_tolerance(&mut myGraph, EdgeId::start(), 0.5);
    myGraph.editor().end_deferred_invalidation();

    // Second End should be a safe no-op.
    myGraph.editor().end_deferred_invalidation();
    assert!(edge_own_gen(&myGraph) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_SameEdgeMutatedTwice() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();
    set_edge_tolerance(&mut myGraph, EdgeId::start(), 0.1);
    set_edge_tolerance(&mut myGraph, EdgeId::start(), 0.5);
    myGraph.editor().end_deferred_invalidation();

    // Last write wins.
    let aTolerance = myGraph.topo().edges().definition(EdgeId::start()).tolerance;
    assert!((aTolerance - 0.5).abs() <= precision::confusion());
    assert!(edge_own_gen(&myGraph) > 0);
    assert!(shell_subtree_gen(&myGraph) > 0);
    assert!(solid_subtree_gen(&myGraph) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_DirectWireMutation_PropagatesUp() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();
    myGraph
        .editor()
        .wires()
        .mutate(WireId::start())
        .unwrap()
        .mark_dirty();

    // Wire was directly mutated: OwnGen incremented.
    assert!(
        myGraph
            .topo()
            .wires()
            .definition(WireId::start())
            .base
            .own_gen
            > 0
    );
    // Face not yet propagated during deferred mode.
    assert_eq!(face_subtree_gen(&myGraph, FaceId::start()), 0);

    myGraph.editor().end_deferred_invalidation();

    // After flush: face, shell, solid SubtreeGen should be propagated.
    let mut aFacePropagated = false;
    let mut aFaceIt = FaceIterator::new(&myGraph);
    while aFaceIt.more() {
        if BRepGraph_TestTools::face_uses_wire(&myGraph, aFaceIt.current_id(), WireId::start())
            && aFaceIt.current().base.subtree_gen > 0
        {
            aFacePropagated = true;
            break;
        }
        aFaceIt.next();
    }
    assert!(aFacePropagated);
    assert!(shell_subtree_gen(&myGraph) > 0);
    assert!(solid_subtree_gen(&myGraph) > 0);
}

#[test]
fn BRepGraph_DeferredInvalidationTest__DeferredMode_OccurrenceMutation_PropagatesSubtreeGenToProduct()
 {
    let mut myGraph = SetUp();
    // Build an assembly: root product + child occurrence referencing it.
    let aPartId = ProductId::start();
    let aAssemblyId = myGraph.editor().products().add_empty();
    myGraph
        .editor()
        .products()
        .append_document_root(aAssemblyId)
        .unwrap();
    let (anOccId, _) = myGraph
        .editor()
        .products()
        .append(
            aAssemblyId,
            aPartId,
            &Location::default(),
            OccurrenceId::default(),
        )
        .unwrap();
    assert!(anOccId.is_valid());

    // LinkProducts is a structural mutation in immediate mode, so SubtreeGen
    // may already be non-zero.  Capture the baseline after linking.
    let aBaselineSubtreeGen = myGraph
        .topo()
        .products()
        .definition(aAssemblyId)
        .base
        .subtree_gen;
    let aBaselineOwnGen = myGraph
        .topo()
        .products()
        .definition(aAssemblyId)
        .base
        .own_gen;

    // Find the OccurrenceRefId for the occurrence.
    let aOccRefs = myGraph.refs().occurrences().ids_of(aAssemblyId);
    assert_eq!(aOccRefs.len(), 1);
    let anOccRefId = aOccRefs[0];

    // Mutate occurrence ref placement in deferred mode.
    myGraph.editor().begin_deferred_invalidation();
    {
        let aTrsf = Transform3::translation_vector(Vector3::new(100.0, 0.0, 0.0));
        myGraph
            .editor()
            .occurrences()
            .set_ref_local_location(anOccRefId, &Location::new(aTrsf))
            .unwrap();
    }

    // During deferred mode: ref modified - parent product's OwnGen should be bumped.
    let aParentProductId = myGraph
        .refs()
        .occurrences()
        .entry(anOccRefId)
        .parent_product_id;
    assert_eq!(
        myGraph
            .topo()
            .products()
            .definition(aParentProductId)
            .base
            .own_gen,
        aBaselineOwnGen + 1
    );

    let aSubtreeGenBeforeFlush = aBaselineSubtreeGen;

    myGraph.editor().end_deferred_invalidation();

    // After flush: parent assembly product should have SubtreeGen >= before flush.
    assert!(
        myGraph
            .topo()
            .products()
            .definition(aAssemblyId)
            .base
            .subtree_gen
            >= aSubtreeGenBeforeFlush
    );
    // Parent's OwnGen is bumped by exactly 1 because ref mutations now propagate to the parent node.
    assert_eq!(
        myGraph
            .topo()
            .products()
            .definition(aAssemblyId)
            .base
            .own_gen,
        aBaselineOwnGen + 1
    );
}

#[test]
fn BRepGraph_DeferredInvalidationTest__CoEdgeMutation_ImmediatePropagatesToWireAndFace() {
    let mut myGraph = SetUp();
    let aWireId = WireId::start();

    // Find the first coedge of the wire and its owning face.
    let aFaceId = FaceId::start();

    let aCoEdgeIds = &myGraph.topo().wires().relations(aWireId).co_edge_ids;
    assert!(!aCoEdgeIds.is_empty());
    let aCoEdgeId = aCoEdgeIds[0];
    assert!(aCoEdgeId.is_valid());

    // Record SubtreeGen before mutation.
    let aWireGenBefore = wire_subtree_gen(&myGraph, aWireId);
    let aFaceGenBefore = face_subtree_gen(&myGraph, aFaceId);

    // Mutate CoEdge orientation in immediate mode.
    myGraph
        .editor()
        .co_edges()
        .set_orientation(aCoEdgeId, ParityOrientation::REVERSED)
        .unwrap();

    // Verify parent Wire and Face SubtreeGen incremented.
    assert!(
        wire_subtree_gen(&myGraph, aWireId) > aWireGenBefore,
        "CoEdge mutation did not propagate SubtreeGen to parent Wire in immediate mode"
    );
    assert!(
        face_subtree_gen(&myGraph, aFaceId) > aFaceGenBefore,
        "CoEdge mutation did not propagate SubtreeGen to parent Face in immediate mode"
    );
}

#[test]
fn BRepGraph_DeferredInvalidationTest__CoEdgeMutation_DeferredPropagatesToWireAndFaceOnFlush() {
    let mut myGraph = SetUp();
    let aWireId = WireId::start();

    let aCoEdgeIds = &myGraph.topo().wires().relations(aWireId).co_edge_ids;
    assert!(!aCoEdgeIds.is_empty());
    let aCoEdgeId = aCoEdgeIds[0];
    assert!(aCoEdgeId.is_valid());

    let aFaceId = FaceId::start();

    myGraph.editor().begin_deferred_invalidation();

    // Record SubtreeGen during deferred mode (before flush).
    let aWireGenBefore = wire_subtree_gen(&myGraph, aWireId);
    let aFaceGenBefore = face_subtree_gen(&myGraph, aFaceId);

    myGraph
        .editor()
        .co_edges()
        .set_orientation(aCoEdgeId, ParityOrientation::REVERSED)
        .unwrap();

    // During deferred mode: parent Wire and Face NOT yet propagated.
    assert_eq!(
        wire_subtree_gen(&myGraph, aWireId),
        aWireGenBefore,
        "Wire SubtreeGen should not change during deferred mode"
    );
    assert_eq!(
        face_subtree_gen(&myGraph, aFaceId),
        aFaceGenBefore,
        "Face SubtreeGen should not change during deferred mode"
    );

    myGraph.editor().end_deferred_invalidation();

    // After flush: parent Wire and Face SubtreeGen should be incremented.
    assert!(
        wire_subtree_gen(&myGraph, aWireId) > aWireGenBefore,
        "CoEdge mutation did not propagate SubtreeGen to parent Wire on deferred flush"
    );
    assert!(
        face_subtree_gen(&myGraph, aFaceId) > aFaceGenBefore,
        "CoEdge mutation did not propagate SubtreeGen to parent Face on deferred flush"
    );
}
