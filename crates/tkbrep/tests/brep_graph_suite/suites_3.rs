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

// BRepGraph_Test.cxx, Group 2 (History Tracking) and the start of Group 3 (Mutation
// APIs): from `RecordHistory_MultipleRecords_SequenceNumbers` to
// `RemoveChild_UsesDistinctRefsForSharedChild`.

#[test]
fn BRepGraphTest__RecordHistory_MultipleRecords_SequenceNumbers() {
    let myGraph = SetUp();
    let aBefore = nbRecords(&myGraph);

    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);
    let aRepl = [anEdge1];

    recordHistory(&myGraph, "OpA", anEdge0, &aRepl);
    recordHistory(&myGraph, "OpB", anEdge0, &aRepl);
    recordHistory(&myGraph, "OpC", anEdge0, &aRepl);

    assert_eq!(nbRecords(&myGraph), aBefore + 3);

    // Check monotonically increasing sequence numbers.
    for anIdx in aBefore + 1..aBefore + 3 {
        assert!(
            recordSequence(&myGraph, anIdx) > recordSequence(&myGraph, anIdx - 1),
            "SequenceNumber not monotonically increasing at index {anIdx}"
        );
    }

    assert!(recordNameIs(&myGraph, aBefore, "OpA"));
    assert!(recordNameIs(&myGraph, aBefore + 1, "OpB"));
    assert!(recordNameIs(&myGraph, aBefore + 2, "OpC"));
}

#[test]
fn BRepGraphTest__FindOriginal_SingleHop_ReturnsSource() {
    let mut myGraph = SetUp();
    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);

    applyModification(&mut myGraph, anEdge0, anEdge1, "TestHop");

    let anOriginal = history(&myGraph).get().unwrap().find_original(anEdge1);
    assert_eq!(anOriginal, anEdge0);
}

#[test]
fn BRepGraphTest__FindDerived_SingleHop_ContainsTarget() {
    let mut myGraph = SetUp();
    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);

    applyModification(&mut myGraph, anEdge0, anEdge1, "TestHop");

    let aDerived = history(&myGraph).get().unwrap().find_derived(anEdge0);
    let isFound = aDerived.contains(&anEdge1);
    assert!(isFound, "Edge1 not found in FindDerived(Edge0)");
}

#[test]
fn BRepGraphTest__ApplyModification_MultiStepChain_FindOriginalTracesBack() {
    let mut myGraph = SetUp();
    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);
    let anEdge2 = edgeNode(2);

    // Step 1: edge0 -> edge1
    applyModification(&mut myGraph, anEdge0, anEdge1, "Step1");

    // Step 2: edge1 -> edge2
    applyModification(&mut myGraph, anEdge1, anEdge2, "Step2");

    // FindOriginal from edge2 should trace back to edge0.
    let aHistory = history(&myGraph);
    let aLayer = aHistory.get().unwrap();
    let anOriginal = aLayer.find_original(anEdge2);
    assert_eq!(anOriginal, anEdge0);

    // FindDerived from edge0 returns leaf-only transitive descendants.
    // edge1 is an intermediate (it has further derived edge2), so only edge2 is returned.
    let aDerived = aLayer.find_derived(anEdge0);
    let isEdge2Found = aDerived.contains(&anEdge2);
    assert!(isEdge2Found, "Edge2 not found in FindDerived(Edge0)");

    // edge1 can be found by querying FindDerived on the intermediate step.
    let aDerived1 = aLayer.find_derived(anEdge1);
    let isEdge2FromEdge1 = aDerived1.contains(&anEdge2);
    assert!(isEdge2FromEdge1, "Edge2 not found in FindDerived(Edge1)");
}

// ===================================================================
// Group 3: Mutation APIs
// ===================================================================

#[test]
fn BRepGraphTest__AddPCurve_NewPCurve_RetrievableViaFindPCurveCoEdgeId() {
    let mut myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let aFaceId = FaceId::new(0);

    let anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let aCurve2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(anAxis)).into_cast();
    let _ = myGraph.editor().co_edges().add_with_pcurve(
        anEdgeId,
        aFaceId,
        &aCurve2d,
        0.0,
        1.0,
        ParityOrientation::FORWARD,
    );

    let aRetrievedId = tool::edge::find_pcurve_co_edge_id(&myGraph, anEdgeId, aFaceId);
    assert!(aRetrievedId.is_valid());
    if aRetrievedId.is_valid() {
        let aCoEdge = myGraph.topo().co_edges().definition(aRetrievedId);
        assert!(aCoEdge.curve2d_rep_id.is_valid());
        let isListedByEdge = containsId(
            myGraph.topo().edges().co_edges(anEdgeId).iter().copied(),
            aRetrievedId,
        );
        assert!(
            isListedByEdge,
            "Created face-bound coedge must be present in edge relations"
        );
    }
    assert!(myGraph.validate_relations());
}

/// The first coedge of the first wire.
fn firstWireCoEdge(theGraph: &BRepGraph) -> CoEdgeId {
    let aCoEdgeIds = BRepGraph_TestTools::co_edges_of_wire(theGraph, WireId::start());
    assert!(!aCoEdgeIds.is_empty());
    aCoEdgeIds[0]
}

#[test]
fn BRepGraphTest__ReplaceEdge_Reversed_OrientationFlipped() {
    let mut myGraph = SetUp();
    let anOrigCoEdge = myGraph.topo().co_edges().definition(firstWireCoEdge(&myGraph));
    let anOldEdgeId = anOrigCoEdge.child_edge_id;
    let anOrigOrientation = anOrigCoEdge.orientation.orientation();

    let aNewEdgeId = addCompatibleReplacementEdge(&mut myGraph, anOldEdgeId, true);
    assert!(aNewEdgeId.is_valid());

    myGraph
        .editor()
        .wires()
        .replace_edge(WireId::start(), anOldEdgeId, aNewEdgeId, true)
        .unwrap();

    let aNewCoEdge = myGraph.topo().co_edges().definition(firstWireCoEdge(&myGraph));
    assert_eq!(aNewCoEdge.child_edge_id.index, aNewEdgeId.index);

    // Orientation should be flipped relative to original.
    let anExpected = if anOrigOrientation == Orientation::Forward {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    assert_eq!(aNewCoEdge.orientation.orientation(), anExpected);
}

#[test]
fn BRepGraphTest__ReplaceEdge_UpdatesEdgeToCoEdgeRelations() {
    let mut myGraph = SetUp();
    let aCoEdgeId = firstWireCoEdge(&myGraph);
    let anOldEdgeId = myGraph.topo().co_edges().definition(aCoEdgeId).child_edge_id;
    let aNewEdgeId = addCompatibleReplacementEdge(&mut myGraph, anOldEdgeId, false);
    assert!(aNewEdgeId.is_valid());

    let hasCoEdge = |theGraph: &BRepGraph, theEdgeId: EdgeId| {
        containsId(
            theGraph.topo().edges().co_edges(theEdgeId).iter().copied(),
            aCoEdgeId,
        )
    };

    assert!(hasCoEdge(&myGraph, anOldEdgeId));

    myGraph
        .editor()
        .wires()
        .replace_edge(WireId::start(), anOldEdgeId, aNewEdgeId, false)
        .unwrap();

    assert!(!hasCoEdge(&myGraph, anOldEdgeId));
    assert!(hasCoEdge(&myGraph, aNewEdgeId));
}

fn isBoundaryValid(aGraph: &mut BRepGraph) -> bool {
    aGraph.editor().validate_mutation_boundary(None)
}

#[test]
fn BRepGraphTest__RemoveCoEdge_PrunesOrphanAndKeepsRelationsConsistent() {
    let mut myGraph = SetUp();
    let aCoEdgeIdsBefore = myGraph
        .topo()
        .wires()
        .relations(WireId::start())
        .co_edge_ids
        .clone();
    assert!(!aCoEdgeIdsBefore.is_empty());

    let aCoEdgeId = aCoEdgeIdsBefore[0];
    let anEdgeId = myGraph.topo().co_edges().definition(aCoEdgeId).child_edge_id;

    assert!(
        myGraph
            .editor()
            .wires()
            .remove_co_edge(WireId::start(), aCoEdgeId)
            .unwrap()
    );

    let aCoEdgeIdsAfter = &myGraph.topo().wires().relations(WireId::start()).co_edge_ids;
    assert_eq!(aCoEdgeIdsAfter.len(), aCoEdgeIdsBefore.len() - 1);
    assert!(aCoEdgeId.is_removed(&myGraph));
    assert!(isBoundaryValid(&mut myGraph));

    for aWireId in myGraph.topo().edges().wires_of(anEdgeId) {
        assert_ne!(aWireId, WireId::start());
    }

    for &aEdgeCoEdgeId in myGraph.topo().edges().co_edges(anEdgeId) {
        assert_ne!(aEdgeCoEdgeId, aCoEdgeId);
    }
}

#[test]
fn BRepGraphTest__CleanupRemovedRefs_ManuallyRemovedEdge_UnbindsEdgeWireRelations() {
    let mut myGraph = SetUp();
    let aWireId = WireId::start();
    let aCoEdgeIds = &myGraph.topo().wires().relations(aWireId).co_edge_ids;
    assert!(!aCoEdgeIds.is_empty());

    let aCoEdgeId = aCoEdgeIds[0];
    let anEdgeId = myGraph.topo().co_edges().definition(aCoEdgeId).child_edge_id;
    assert!(containsId(myGraph.topo().edges().wires_of(anEdgeId), aWireId));

    myGraph
        .editor()
        .generic()
        .remove_node(NodeId::from(anEdgeId))
        .unwrap();

    myGraph.editor().generic().cleanup_removed_references();

    assert!(aCoEdgeId.is_removed(&myGraph));
    assert!(!containsId(myGraph.topo().edges().wires_of(anEdgeId), aWireId));
    assert!(isBoundaryValid(&mut myGraph));
}

#[test]
fn BRepGraphTest__RemoveChild_PreservesSharedChildAndKeepsRelationsConsistent() {
    let mut myGraph = SetUp();
    let aChildren = [NodeId::from(SolidId::start())];
    let aCompoundId = myGraph.editor().compounds().add(&aChildren);
    assert!(aCompoundId.is_valid());

    let aChildRefsBefore =
        BRepGraph_TestTools::child_refs_of_parent(&myGraph, NodeId::from(aCompoundId));
    assert_eq!(aChildRefsBefore.len(), 1);

    let aChildRefId = aChildRefsBefore[0];
    let aChildId = myGraph.refs().children().entry(aChildRefId).child_node_id;

    assert!(
        myGraph
            .editor()
            .compounds()
            .remove_child(aCompoundId, aChildRefId)
            .unwrap()
    );

    assert_eq!(
        BRepGraph_TestTools::count_child_refs_of_parent(&myGraph, NodeId::from(aCompoundId)),
        0
    );
    assert!(!myGraph.topo().generic().is_removed(aChildId));
    assert!(isBoundaryValid(&mut myGraph));
    assert_eq!(
        myGraph
            .topo()
            .generic()
            .compound_ref_ids(NodeId::from(aCompoundId))
            .len(),
        0
    );
}

/// `BRepGraph_CompoundsOfChild(theGraph, Topo().Gen().CompoundRefIds(theChild))`.
fn compoundsOfChild(theGraph: &BRepGraph, theChild: NodeId) -> Vec<CompoundId> {
    let aRefs = theGraph.topo().generic().compound_ref_ids(theChild);
    CompoundsOfChild::new(theGraph, aRefs).collect()
}

#[test]
fn BRepGraphTest__RemoveChild_UsesDistinctRefsForSharedChild() {
    let mut myGraph = SetUp();
    assert!(myGraph.topo().solids().nb() > 0);
    let aSolidId = SolidId::start();

    let aFirstCompound = myGraph.editor().compounds().add(&[NodeId::from(aSolidId)]);
    assert!(aFirstCompound.is_valid());

    let aSecondCompound = myGraph.editor().compounds().add(&[]);
    assert!(aSecondCompound.is_valid());

    let aFirstRefs =
        BRepGraph_TestTools::child_refs_of_parent(&myGraph, NodeId::from(aFirstCompound));
    assert_eq!(aFirstRefs.len(), 1);
    let aFirstRefId = aFirstRefs[0];

    let aSecondRefId = myGraph
        .editor()
        .compounds()
        .append(aSecondCompound, aSolidId.into(), ParityOrientation::FORWARD)
        .unwrap();
    assert!(aSecondRefId.is_valid());
    assert_ne!(aFirstRefId, aSecondRefId);
    myGraph.editor().generic().cleanup_removed_references();

    let aValidDistinctRefs = validate::perform_with_options(&myGraph, &validate::Options::audit());
    assert!(aValidDistinctRefs.is_valid());
    let aCompoundsBefore = compoundsOfChild(&myGraph, aSolidId.into());
    assert!(containsId(aCompoundsBefore.iter().copied(), aFirstCompound));
    assert!(containsId(aCompoundsBefore.iter().copied(), aSecondCompound));

    let mut aCompounds = myGraph.editor().compounds();
    assert!(aCompounds.remove_child(aFirstCompound, aFirstRefId).unwrap());
    assert!(aFirstRefId.is_removed(&myGraph));
    assert!(!aSecondRefId.is_removed(&myGraph));
    let aCompoundsAfterFirstRemove = compoundsOfChild(&myGraph, aSolidId.into());
    assert!(!containsId(aCompoundsAfterFirstRemove.iter().copied(), aFirstCompound));
    assert!(containsId(aCompoundsAfterFirstRemove.iter().copied(), aSecondCompound));
    let aCount = |theGraph: &BRepGraph, theCompound: CompoundId| {
        BRepGraph_TestTools::count_child_refs_of_parent(theGraph, NodeId::from(theCompound))
    };
    assert_eq!(aCount(&myGraph, aFirstCompound), 0);
    assert_eq!(aCount(&myGraph, aSecondCompound), 1);

    let mut aCompounds = myGraph.editor().compounds();
    assert!(aCompounds.remove_child(aSecondCompound, aSecondRefId).unwrap());
    assert!(aSecondRefId.is_removed(&myGraph));
    let aCompoundsAfterSecondRemove = compoundsOfChild(&myGraph, aSolidId.into());
    assert!(!containsId(aCompoundsAfterSecondRemove.iter().copied(), aSecondCompound));
    assert!(isBoundaryValid(&mut myGraph));
}
