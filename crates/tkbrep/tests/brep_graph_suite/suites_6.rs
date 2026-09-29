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

// BRepGraph_Test.cxx, Groups 10 to 15 and the history-enabled group: from
// `Wire_IsClosed_BoxOuterWires` to `ApplyModification_HistoryDisabled_ModifierStillRuns`.

fn outerWire(theGraph: &BRepGraph, theFace: FaceId) -> WireId {
    BRepGraph_TestTools::outer_wire_of_face(theGraph, theFace).unwrap()
}

// ===================================================================
// Group 10: Topology Structure
// ===================================================================

#[test]
fn BRepGraphTest__Wire_IsClosed_BoxOuterWires() {
    let myGraph = SetUp();
    // All outer wires of a box face should be closed.
    for aFaceId in faceIds(&myGraph) {
        let anOuterWire = outerWire(&myGraph, aFaceId);
        assert!(anOuterWire.is_valid());
        assert!(
            tool::wire::is_closed(&myGraph, anOuterWire),
            "Outer wire of face {} should be closed",
            aFaceId.index
        );
    }
}

#[test]
fn BRepGraphTest__Face_InnerWireRefs_BoxHasNone() {
    let myGraph = SetUp();
    // Box faces have no holes, so each face should have one wire ref.
    for aFaceId in faceIds(&myGraph) {
        let aWireRefs = BRepGraph_TestTools::wire_refs_of_face(&myGraph, aFaceId);
        assert_eq!(
            aWireRefs.len(),
            1,
            "Box face {} should have one wire",
            aFaceId.index
        );
    }
}

#[test]
fn BRepGraphTest__Face_Orientation_ValidValue() {
    let myGraph = SetUp();
    // Verify face orientations in the shell's incidence refs.
    assert_eq!(myGraph.topo().shells().nb(), 1);
    let aFaceRefs = BRepGraph_TestTools::face_refs_of_shell(&myGraph, ShellId::start());
    for (aRefIdx, &aFaceRefId) in aFaceRefs.iter().enumerate() {
        let anOri = myGraph.refs().faces().entry(aFaceRefId).orientation.orientation();
        assert!(
            anOri == Orientation::Forward || anOri == Orientation::Reversed,
            "Face ref {aRefIdx} has unexpected orientation {anOri:?}"
        );
    }
}

#[test]
fn BRepGraphTest__Shell_ContainsSixFaces() {
    let myGraph = SetUp();
    assert_eq!(myGraph.topo().shells().nb(), 1);
    assert_eq!(
        BRepGraph_TestTools::count_face_refs_of_shell(&myGraph, ShellId::start()),
        6
    );
}

#[test]
fn BRepGraphTest__Solid_ContainsOneShell() {
    let myGraph = SetUp();
    assert_eq!(myGraph.topo().solids().nb(), 1);
    assert_eq!(
        BRepGraph_TestTools::count_shell_refs_of_solid(&myGraph, SolidId::start()),
        1
    );
}

#[test]
fn BRepGraphTest__Edge_ParamRange_ValidBounds() {
    let myGraph = SetUp();
    for anEdgeId in edgeIds(&myGraph) {
        if tool::edge::degenerated(&myGraph, anEdgeId) {
            continue;
        }
        let (aFirst, aLast) = tool::edge::range(&myGraph, anEdgeId);
        assert!(
            aFirst < aLast,
            "Edge {} has invalid parameter range [{aFirst}, {aLast}]",
            anEdgeId.index
        );
    }
}

#[test]
fn BRepGraphTest__Vertex_TolerancePositive() {
    let myGraph = SetUp();
    let mut aVertexIt = VertexIterator::new(&myGraph);
    while aVertexIt.more() {
        let aVertexId = aVertexIt.current_id();
        assert!(
            tool::vertex::tolerance(&myGraph, aVertexId) > 0.0,
            "Vertex {} has non-positive tolerance",
            aVertexId.index
        );
        aVertexIt.next();
    }
}

#[test]
fn BRepGraphTest__Edge_TolerancePositive() {
    let myGraph = SetUp();
    for anEdgeId in edgeIds(&myGraph) {
        assert!(
            tool::edge::tolerance(&myGraph, anEdgeId) > 0.0,
            "Edge {} has non-positive tolerance",
            anEdgeId.index
        );
    }
}

#[test]
fn BRepGraphTest__Face_ToleranceNonNegative() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        assert!(
            tool::face::tolerance(&myGraph, aFaceId) >= 0.0,
            "Face {} has negative tolerance",
            aFaceId.index
        );
    }
}

// ===================================================================
// Group 13: Mutation (Extended)
// ===================================================================

#[test]
fn BRepGraphTest__Wire_IsClosed_DerivedFromCoedgeChain() {
    let myGraph = SetUp();
    // Wire closure is now derived from the ordered coedge chain.
    let anOuterWire = outerWire(&myGraph, FaceId::start());
    assert!(anOuterWire.is_valid());
    // Outer wires of a box face should be closed.
    assert!(tool::wire::is_closed(&myGraph, anOuterWire));
}

// ===================================================================
// Group 14: Build From Different Root Shapes
// ===================================================================

#[test]
fn BRepGraphTest__Build_SingleFace_CorrectCounts() {
    let _myGraph = SetUp();
    let aFace = firstOf(&boxShape(10.0, 20.0, 30.0), ShapeEnum::Face);

    let aGraph = graphOf(&aFace);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().solids().nb(), 0);
    assert_eq!(aGraph.topo().shells().nb(), 0);
    assert_eq!(aGraph.topo().faces().nb(), 1);
    assert_eq!(aGraph.topo().wires().nb(), 1);
    assert_eq!(aGraph.topo().edges().nb(), 4);
    assert_eq!(aGraph.topo().vertices().nb(), 4);
}

#[test]
fn BRepGraphTest__Build_Shell_CorrectCounts() {
    let _myGraph = SetUp();
    let aShell = firstOf(&boxShape(10.0, 20.0, 30.0), ShapeEnum::Shell);

    let aGraph = graphOf(&aShell);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().solids().nb(), 0);
    assert_eq!(aGraph.topo().shells().nb(), 1);
    assert_eq!(aGraph.topo().faces().nb(), 6);
    assert_eq!(aGraph.topo().edges().nb(), 12);
    assert_eq!(aGraph.topo().vertices().nb(), 8);
}

#[test]
fn BRepGraphTest__Build_CompoundOfTwoSolids() {
    let _myGraph = SetUp();
    let aCompound = compoundOf(&[boxShape(10.0, 10.0, 10.0), boxShape(20.0, 20.0, 20.0)]);

    let aGraph = graphOf(&aCompound);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().solids().nb(), 2);
    assert_eq!(aGraph.topo().shells().nb(), 2);
    assert_eq!(aGraph.topo().faces().nb(), 12);
}

#[test]
fn BRepGraphTest__ReconstructShape_ShellRoot_SameFaceCount() {
    let _myGraph = SetUp();
    let aShell = firstOf(&boxShape(10.0, 20.0, 30.0), ShapeEnum::Shell);

    let aGraph = graphOf(&aShell);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().shells().nb(), 1);

    let aShellId = NodeId::new(NodeKind::SHELL, 0);
    let aReconstructed = aGraph.shapes().reconstruct(aShellId).unwrap();

    let mut aFaceCount = 0;
    let mut aFaceExp = Explorer::new(&aReconstructed, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while aFaceExp.more() {
        aFaceCount += 1;
        aFaceExp.next().unwrap();
    }
    assert_eq!(aFaceCount, 6);
}

// ===================================================================
// Group 15: UID Properties
// ===================================================================

#[test]
fn BRepGraphTest__UID_FaceIsTopology() {
    let myGraph = SetUp();
    let aUID = myGraph.uids().of_node(NodeId::new(NodeKind::FACE, 0));
    assert!(aUID.is_topology());
}

#[test]
fn BRepGraphTest__UID_AllTopoNodesHaveValidUIDs() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        assert!(
            myGraph.uids().of_node(aFaceId.into()).is_valid(),
            "Face {} has invalid UID",
            aFaceId.index
        );
    }
    for anEdgeId in edgeIds(&myGraph) {
        assert!(
            myGraph.uids().of_node(anEdgeId.into()).is_valid(),
            "Edge {} has invalid UID",
            anEdgeId.index
        );
    }
}

#[test]
fn BRepGraphTest__Wire_OuterWireIdx_MatchesFaceDef() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        let anOuterWire = outerWire(&myGraph, aFaceId);
        assert!(
            anOuterWire.is_valid(),
            "Face {} has no outer wire",
            aFaceId.index
        );
        if !anOuterWire.is_valid() {
            continue;
        }
        assert!(
            anOuterWire.index < myGraph.topo().wires().nb(),
            "Face {} outer wire index out of range",
            aFaceId.index
        );
    }
}

#[test]
fn BRepGraphTest__Wire_CoEdges_FourEdgesPerBoxFace() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        let anOuterWire = outerWire(&myGraph, aFaceId);
        assert!(anOuterWire.is_valid());
        assert_eq!(
            BRepGraph_TestTools::count_co_edges_of_wire(&myGraph, anOuterWire),
            4,
            "Box face {} should have 4 coedges in its outer wire",
            aFaceId.index
        );
    }
}

// ===================================================================
// Group: History Enabled Flag
// ===================================================================

fn setHistoryEnabled(theGraph: &BRepGraph, theEnabled: bool) {
    history(theGraph).get().unwrap().set_enabled(theEnabled);
}

fn isHistoryEnabled(theGraph: &BRepGraph) -> bool {
    history(theGraph).get().unwrap().is_enabled()
}

#[test]
fn BRepGraphTest__SetHistoryEnabled_DefaultTrue() {
    let myGraph = SetUp();
    assert!(isHistoryEnabled(&myGraph));
}

#[test]
fn BRepGraphTest__SetHistoryEnabled_DisableAndQuery() {
    let myGraph = SetUp();
    setHistoryEnabled(&myGraph, false);
    assert!(!isHistoryEnabled(&myGraph));
    setHistoryEnabled(&myGraph, true);
    assert!(isHistoryEnabled(&myGraph));
}

#[test]
fn BRepGraphTest__RecordHistory_Disabled_NoRecordAdded() {
    let myGraph = SetUp();
    let aBefore = nbRecords(&myGraph);

    setHistoryEnabled(&myGraph, false);

    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);
    let aRepl = [anEdge1];
    recordHistory(&myGraph, "ShouldNotRecord", anEdge0, &aRepl);

    assert_eq!(nbRecords(&myGraph), aBefore);
}

#[test]
fn BRepGraphTest__RecordHistory_ReEnabled_RecordsAgain() {
    let myGraph = SetUp();
    setHistoryEnabled(&myGraph, false);

    let aBefore = nbRecords(&myGraph);

    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);
    let aRepl = [anEdge1];
    recordHistory(&myGraph, "Skipped", anEdge0, &aRepl);
    assert_eq!(nbRecords(&myGraph), aBefore);

    setHistoryEnabled(&myGraph, true);
    recordHistory(&myGraph, "Recorded", anEdge0, &aRepl);
    assert_eq!(nbRecords(&myGraph), aBefore + 1);
    assert!(recordNameIs(&myGraph, aBefore, "Recorded"));
}

#[test]
fn BRepGraphTest__ApplyModification_HistoryDisabled_NoHistoryNoDerivedEdges() {
    let mut myGraph = SetUp();
    setHistoryEnabled(&myGraph, false);

    let aNbHistBefore = nbRecords(&myGraph);

    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);

    applyModification(&mut myGraph, anEdge0, anEdge1, "NoHistory");

    // No history records should be added.
    assert_eq!(nbRecords(&myGraph), aNbHistBefore);
}

#[test]
fn BRepGraphTest__ApplyModification_HistoryDisabled_ModifierStillRuns() {
    let mut myGraph = SetUp();
    setHistoryEnabled(&myGraph, false);

    let mut isModifierCalled = false;
    let anEdge0 = edgeNode(0);

    myGraph
        .editor()
        .generic()
        .apply_modification(
            anEdge0,
            |_, _| {
                isModifierCalled = true;
                Ok(vec![anEdge0])
            },
            &AsciiString::new("CheckModifier"),
        )
        .unwrap();
    assert!(isModifierCalled);
}
