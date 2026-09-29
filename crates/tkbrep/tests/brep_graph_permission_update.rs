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

//! Direct translation of BRepGraph_PermissionUpdate_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). `BRepGraph_Copy::CopyNode` and
//! `BRepGraph_Transform::TransformNode`/`MoveRef` are the functions of the `copy` and
//! `transform` modules; the guarded setter is the `*_guarded` associated function
//! (docs/topology-design.md section 11.4).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::editor::OccurrenceOps;
use rustycad_tkbrep::brep_graph::iterator::{EdgeIterator, SolidIterator};
use rustycad_tkbrep::brep_graph::refs_iterator::{RefsFaceOfShell, RefsShellOfSolid};
use rustycad_tkbrep::brep_graph::reverse_iterator::{OccurrencesOfChild, SolidsOfShell};
use rustycad_tkbrep::brep_graph::transform;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::make_box;

fn makeBoxGraph() -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aRes = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    aGraph
}

fn xLine() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::X))).into_cast()
}

fn makeSelfLoopEdge(theGraph: &mut BRepGraph) -> EdgeId {
    let aV = theGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    let aLine = xLine();
    theGraph
        .editor()
        .edges()
        .add(aV, aV, &aLine, 0.0, 1.0, 1.0e-7)
}

fn containsCurrentId<T: PartialEq>(theIterator: impl Iterator<Item = T>, theId: T) -> bool {
    let mut theIterator = theIterator;
    theIterator.any(|anId| anId == theId)
}

/// `Products().Add()` then `AppendDocumentRoot`.
fn addRootProduct(aGraph: &mut BRepGraph) -> ProductId {
    let aProduct = aGraph.editor().products().add_empty();
    aGraph
        .editor()
        .products()
        .append_document_root(aProduct)
        .unwrap();
    aProduct
}

/// `Products().Append(theParent, theChild, TopLoc_Location(), OccurrenceId(), &ref)`.
fn appendOccurrence(
    aGraph: &mut BRepGraph,
    theParent: ProductId,
    theChild: ProductId,
) -> (OccurrenceId, OccurrenceRefId) {
    let aDefault = OccurrenceId::default();
    aGraph
        .editor()
        .products()
        .append(theParent, theChild, &Location::default(), aDefault)
        .unwrap()
}

/// `BRepGraph_OccurrencesOfChild(aGraph, Topo().Gen().OccurrenceRefIds(theChild))`.
fn occurrencesOfChild(theGraph: &BRepGraph, theChild: NodeId) -> Vec<OccurrenceId> {
    let aRefs = theGraph.topo().generic().occurrence_ref_ids(theChild);
    OccurrencesOfChild::new(theGraph, aRefs).collect()
}

fn translation(x: f64, y: f64, z: f64) -> Transform3 {
    Transform3::translation_vector(Vector3::new(x, y, z))
}

#[test]
fn BRepGraph_PermissionUpdateTest__RemoveRef_FaceRef_InvalidatesOwningShellBeforeUnbind() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aFaceRefs = RefsFaceOfShell::new(&aGraph, ShellId::start());
    assert!(aFaceRefs.more());
    let aFaceRefId = aFaceRefs.current_id();

    let aShellSubtreeGenBefore = aGraph
        .topo()
        .shells()
        .definition(ShellId::start())
        .base
        .subtree_gen;
    assert!(
        aGraph
            .editor()
            .generic()
            .remove_ref(aFaceRefId.into())
            .unwrap()
    );

    assert!(aFaceRefId.is_removed(&aGraph));
    let aAfter = aGraph.topo().shells().definition(ShellId::start()).base.subtree_gen;
    assert!(aAfter > aShellSubtreeGenBefore);
    assert!(aGraph.validate_relations());
}

#[test]
fn BRepGraph_PermissionUpdateTest__RemoveShell_SameSolidSiblingRef_PreservesRelations() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aSolidId = SolidId::start();
    let aShellRefs = RefsShellOfSolid::new(&aGraph, aSolidId);
    assert!(aShellRefs.more());
    let aFirstRefId = aShellRefs.current_id();
    let aShellId = aGraph.refs().shells().entry(aFirstRefId).child_shell_id;

    let aSecondRefId = aGraph
        .editor()
        .solids()
        .append(aSolidId, aShellId, ParityOrientation::REVERSED)
        .unwrap();
    assert!(aSecondRefId.is_valid());
    assert_ne!(aSecondRefId, aFirstRefId);
    let solidsOf = |theGraph: &BRepGraph| {
        let aParents = &theGraph.topo().shells().relations(aShellId).parent_shell_ref_ids;
        containsCurrentId(SolidsOfShell::new(theGraph, aParents), aSolidId)
    };
    assert!(solidsOf(&aGraph));

    assert!(
        aGraph
            .editor()
            .solids()
            .remove_shell(aSolidId, aFirstRefId)
            .unwrap()
    );

    assert!(aFirstRefId.is_removed(&aGraph));
    assert!(!aSecondRefId.is_removed(&aGraph));
    assert!(
        solidsOf(&aGraph),
        "A sibling ShellRef still connects the same Solid to the same Shell"
    );
    assert!(aGraph.validate_relations());
}

// B1: RemoveRef on a Product->Occurrence ref must unbind the relation-table entry
// keyed by the referenced product (OccurrenceDef::ChildNodeId), not by the parent.
#[test]
fn BRepGraph_PermissionUpdateTest__RemoveRef_OccurrenceRef_PreservesProductToOccurrencesIndex() {
    // Targeted regression for the wrong-key Unbind in GenOps::RemoveRef. Walks the
    // relation tables directly because BRepGraphInc_Relations::Validate currently
    // covers only topology references.
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aParent = addRootProduct(&mut aGraph);
    let aChild = addRootProduct(&mut aGraph);
    assert!(aParent.is_valid());
    assert!(aChild.is_valid());

    let (aOccId, aOccRefId) = appendOccurrence(&mut aGraph, aParent, aChild);
    assert!(aOccId.is_valid());
    assert!(aOccRefId.is_valid());

    let aFoundBefore = occurrencesOfChild(&aGraph, aChild.into()).contains(&aOccId);
    assert!(
        aFoundBefore,
        "Occurrence must be indexed under referenced product before remove"
    );

    assert!(
        aGraph
            .editor()
            .generic()
            .remove_ref(RefId::from(aOccRefId))
            .unwrap()
    );

    for anId in occurrencesOfChild(&aGraph, aChild.into()) {
        assert_ne!(
            anId, aOccId,
            "RemoveRef must drop the referenced-product entry"
        );
    }

    // The wrong-key bug would also leave a stale entry under the parent product id.
    for anId in occurrencesOfChild(&aGraph, aParent.into()) {
        assert_ne!(
            anId, aOccId,
            "Bug: Unbind happened against parent product key"
        );
    }
}

// B2: removing one coedge usage must leave the wire's other usages of the same edge
// reflected in the deduplicated Edge->Wire relation tables.
//
// Constructs a wire with two coedge usages whose CoEdges share the same edge so the
// dedup-aware unbind path is exercised; removing one ref must keep the
// (Edge -> Wire) reverse entry alive because the sibling still references it.
#[test]
fn BRepGraph_PermissionUpdateTest__RemoveRef_CoEdge_PreservesEdgeToWireWhenSiblingPresent() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();

    let aV1 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    let aV2 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1.0e-7);
    let aLine = xLine();
    let aEdge = aGraph
        .editor()
        .edges()
        .add(aV1, aV2, &aLine, 0.0, 1.0, 1.0e-7);
    assert!(aEdge.is_valid());

    let aCoEdgeIds = [
        aGraph
            .editor()
            .co_edges()
            .add(aEdge, ParityOrientation::FORWARD),
        aGraph
            .editor()
            .co_edges()
            .add(aEdge, ParityOrientation::REVERSED),
    ];
    let aWireId = aGraph.editor().wires().add(&aCoEdgeIds);
    assert!(aWireId.is_valid());
    assert!(aGraph.validate_relations());

    let aWireRelations = &aGraph.topo().wires().relations(aWireId).co_edge_ids;
    assert_eq!(aWireRelations.len(), 2);
    let aFirstCoEdge = aWireRelations[0];
    assert!(aFirstCoEdge.is_valid());

    let containsWire =
        |theGraph: &BRepGraph, theE: EdgeId| containsCurrentId(theGraph.topo().edges().wires_of(theE), aWireId);
    assert!(
        containsWire(&aGraph, aEdge),
        "Edge->Wire reverse must be present before removal"
    );

    assert!(
        aGraph
            .editor()
            .wires()
            .remove_co_edge(aWireId, aFirstCoEdge)
            .unwrap()
    );
    assert!(aGraph.validate_relations());
    assert!(
        containsWire(&aGraph, aEdge),
        "Sibling coedge still references the edge: Edge->Wire entry must survive"
    );
}

// B3: SetRefChildVertexId on a self-loop edge (start == end == V_old) must keep
// the V_old->edge entry while the sibling slot still references it.
#[test]
fn BRepGraph_PermissionUpdateTest__SetRefChildVertexId_SelfLoopSibling_KeepsRevIndexValid() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aEdge = makeSelfLoopEdge(&mut aGraph);
    assert!(aEdge.is_valid());
    assert!(aGraph.validate_relations());

    let aStartRefId = aGraph.topo().edges().definition(aEdge).start_vertex_ref_id;
    assert!(aStartRefId.is_valid());

    let aNewV = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1.0e-7);

    aGraph
        .editor()
        .vertices()
        .set_ref_child_vertex_id(aStartRefId, aNewV)
        .unwrap();
    assert!(aGraph.validate_relations());

    // EndVertexRef still references the original vertex, so the (Vold -> aEdge)
    // reverse entry must survive the start-side rebind.
    let anEndRef = aGraph.topo().edges().definition(aEdge).end_vertex_ref_id;
    let aOldV = if anEndRef.is_valid() {
        aGraph.refs().vertices().entry(anEndRef).child_vertex_id
    } else {
        VertexId::default()
    };
    assert!(aOldV.is_valid());
    let aOldStillIndexed = aGraph.topo().vertices().edges(aOldV).contains(&aEdge);
    assert!(
        aOldStillIndexed,
        "Vold must remain in EdgesOfVertex(Vold) after start-side rebind"
    );
}

// CopyNode must not loop infinitely on a self-referencing compound.
#[test]
fn BRepGraph_PermissionUpdateTest__CopyNode_SelfReferencingCompound_Terminates() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aV = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    let aRoot = aGraph.editor().compounds().add(&[NodeId::from(aV)]);
    assert!(aRoot.is_valid());

    // Splice the compound into itself by rewriting its first child ref.
    let aRelations = &aGraph.topo().compounds().relations(aRoot).child_ref_ids;
    assert!(!aRelations.is_empty());
    let aChildRefId = aRelations[0];
    aGraph
        .editor()
        .generic()
        .set_child_ref_child_node_id(aChildRefId, NodeId::from(aRoot))
        .unwrap();

    let mut aCopy = BRepGraph::new();
    let aRootId = copy::copy_node(
        &aGraph,
        &mut aCopy,
        NodeId::from(aRoot),
        GeomPolicy::Copy,
        MeshPolicy::Drop,
        CachePolicy::Drop,
    )
    .unwrap();
    assert!(aRootId.is_valid());
    assert!(!aCopy.is_empty());
    assert!(aCopy.validate_relations());
}

/// `TransformNode(aGraph, aResult, theNode, translation(1, 0, 0), Copy, Drop)`.
fn transformNodeWithCopyGeom(aGraph: &BRepGraph, theNode: NodeId) -> NodeId {
    let aT = translation(1.0, 0.0, 0.0);
    let mut aResult = BRepGraph::new();
    transform::transform_node(
        aGraph,
        &mut aResult,
        theNode,
        &aT,
        GeomPolicy::Copy,
        MeshPolicy::Drop,
    )
    .unwrap()
}

// TransformNode with copyGeom on an assembly node must reject (returns invalid graph)
// rather than silently produce inconsistent geometry/location state.
#[test]
fn BRepGraph_PermissionUpdateTest__TransformNode_AssemblyWithCopyGeom_Rejected() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aProd = addRootProduct(&mut aGraph);
    assert!(aProd.is_valid());

    let aRootId = transformNodeWithCopyGeom(&aGraph, NodeId::from(aProd));
    assert!(!aRootId.is_valid());
}

// Companion to the above: Occurrence node with copyGeom must be rejected too.
#[test]
fn BRepGraph_PermissionUpdateTest__TransformNode_OccurrenceWithCopyGeom_Rejected() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aParent = addRootProduct(&mut aGraph);
    let aChild = addRootProduct(&mut aGraph);
    assert!(aParent.is_valid());
    assert!(aChild.is_valid());
    let (aOccId, _) = appendOccurrence(&mut aGraph, aParent, aChild);
    assert!(aOccId.is_valid());

    let aRootId = transformNodeWithCopyGeom(&aGraph, NodeId::from(aOccId));
    assert!(!aRootId.is_valid());
}

// LinkProducts with the unified signature returns the freshly inserted OccurrenceRef
// id through the optional out pointer.
#[test]
fn BRepGraph_PermissionUpdateTest__LinkProducts_OutOccurrenceRefId_Populated() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aParent = addRootProduct(&mut aGraph);
    let aChild = addRootProduct(&mut aGraph);
    assert!(aParent.is_valid());
    assert!(aChild.is_valid());

    let (aOccId, aOccRefId) = appendOccurrence(&mut aGraph, aParent, aChild);
    assert!(aOccId.is_valid());
    assert!(aOccRefId.is_valid());
    assert_eq!(
        aGraph.refs().occurrences().entry(aOccRefId).child_occurrence_id,
        aOccId
    );

    // Default out-pointer = nullptr is also legal.
    let (aOccId2, _) = appendOccurrence(&mut aGraph, aParent, aChild);
    assert!(aOccId2.is_valid());
}

#[test]
fn BRepGraph_PermissionUpdateTest__SetOccurrenceChildNodeId_RejectsOccurrenceChild() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aParent = addRootProduct(&mut aGraph);
    let aChild = addRootProduct(&mut aGraph);
    assert!(aParent.is_valid());
    assert!(aChild.is_valid());

    let (aOccId, _) = appendOccurrence(&mut aGraph, aParent, aChild);
    assert!(aOccId.is_valid());
    let anOldChild = aGraph.topo().occurrences().definition(aOccId).child_node_id;

    aGraph
        .editor()
        .occurrences()
        .set_child_node_id(aOccId, NodeId::from(aOccId))
        .unwrap();

    assert_eq!(
        aGraph.topo().occurrences().definition(aOccId).child_node_id,
        anOldChild
    );
    assert!(occurrencesOfChild(&aGraph, anOldChild).contains(&aOccId));
    assert!(!occurrencesOfChild(&aGraph, NodeId::from(aOccId)).contains(&aOccId));
    assert!(aGraph.validate_relations());
}

#[test]
fn BRepGraph_PermissionUpdateTest__SetOccurrenceChildNodeId_InvalidOccurrenceNoOp() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aChild = addRootProduct(&mut aGraph);
    assert!(aChild.is_valid());

    let aResult = aGraph
        .editor()
        .occurrences()
        .set_child_node_id(OccurrenceId::new(100000), NodeId::from(aChild));
    assert!(aResult.is_ok());
    assert!(aGraph.validate_relations());
}

#[test]
fn BRepGraph_PermissionUpdateTest__SetOccurrenceChildNodeId_MutGuardRejectsRemovedChild() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    let aParent = addRootProduct(&mut aGraph);
    let aChild = addRootProduct(&mut aGraph);
    let aRemovedChild = addRootProduct(&mut aGraph);
    assert!(aParent.is_valid());
    assert!(aChild.is_valid());
    assert!(aRemovedChild.is_valid());

    let (aOccId, _) = appendOccurrence(&mut aGraph, aParent, aChild);
    assert!(aOccId.is_valid());
    let anOldChild = aGraph.topo().occurrences().definition(aOccId).child_node_id;
    aGraph
        .editor()
        .generic()
        .remove_node(aRemovedChild.into())
        .unwrap();

    {
        let mut aMut = aGraph.editor().occurrences().mutate(aOccId).unwrap();
        OccurrenceOps::set_child_node_id_guarded(&mut aMut, NodeId::from(aRemovedChild));
        assert!(!aMut.is_dirty());
    }

    assert_eq!(
        aGraph.topo().occurrences().definition(aOccId).child_node_id,
        anOldChild
    );
    assert!(occurrencesOfChild(&aGraph, anOldChild).contains(&aOccId));
    assert!(!occurrencesOfChild(&aGraph, NodeId::from(aRemovedChild)).contains(&aOccId));
    assert!(aGraph.validate_relations());
}

// MoveRef: a pure rotation/translation must succeed; a scaled trsf must fail with no
// state change.
#[test]
fn BRepGraph_PermissionUpdateTest__MoveRef_ChildRefScaledTrsf_RejectedWithoutMutation() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aCompound = aGraph
        .editor()
        .compounds()
        .add(&[NodeId::from(SolidId::start())]);
    assert!(aCompound.is_valid());
    assert_eq!(aGraph.refs().children().ids_of(aCompound).len(), 1);
    let aChildRef = aGraph.refs().children().ids_of(aCompound)[0];

    let mut aScaled = Transform3::default();
    aScaled.set_scale_factor(2.0).unwrap();
    assert!(!transform::move_child_ref(&mut aGraph, aChildRef, &aScaled).unwrap());
    let aLocation = &aGraph.refs().children().entry(aChildRef).local_location;
    assert!(aLocation.is_identity());

    let aTrans = translation(1.0, 2.0, 3.0);
    assert!(transform::move_child_ref(&mut aGraph, aChildRef, &aTrans).unwrap());
    let aLocation = &aGraph.refs().children().entry(aChildRef).local_location;
    assert!(!aLocation.is_identity());
}

// RemoveSubgraph cascade rebuilds the relation tables exactly once at the outermost
// scope; nested calls during recursion must not trigger intermediate rebuilds, and
// the final state must validate.
#[test]
fn BRepGraph_PermissionUpdateTest__RemoveSubgraph_NestedCascade_FinalStateValid() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    assert!(aGraph.validate_relations());

    let aIt = SolidIterator::new(&aGraph);
    let aSolidId = if aIt.more() {
        aIt.current_id()
    } else {
        SolidId::default()
    };
    assert!(aSolidId.is_valid());

    aGraph
        .editor()
        .generic()
        .remove_subgraph(NodeId::from(aSolidId))
        .unwrap();
    assert!(aGraph.validate_relations());
}

// MutGuard: a guard that observes but never writes must not bump OwnGen on destruction;
// MarkDirty() must force the bump even when no setter was called.
#[test]
fn BRepGraph_PermissionUpdateTest__MutGuard_DirtyFlag_RespectsExplicitMarkAndCleanScope() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let anIt = EdgeIterator::new(&aGraph);
    let aEdgeId = if anIt.more() {
        anIt.current_id()
    } else {
        EdgeId::default()
    };
    assert!(aEdgeId.is_valid());

    let aOwnGenBefore = aGraph.topo().edges().definition(aEdgeId).base.own_gen;
    {
        let aGuard = aGraph.editor().edges().mutate(aEdgeId).unwrap();
        let _ = aGuard.tolerance; // read-only access only
    }
    assert_eq!(
        aOwnGenBefore,
        aGraph.topo().edges().definition(aEdgeId).base.own_gen,
        "Read-only guard scope must not bump OwnGen"
    );

    {
        let mut aGuard = aGraph.editor().edges().mutate(aEdgeId).unwrap();
        aGuard.mark_dirty();
    }
    assert!(
        aGraph.topo().edges().definition(aEdgeId).base.own_gen > aOwnGenBefore,
        "MarkDirty() must bump OwnGen on guard destruction"
    );
}
