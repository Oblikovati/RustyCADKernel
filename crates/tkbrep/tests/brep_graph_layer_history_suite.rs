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

//! Direct translation of BRepGraph_LayerHistory_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` and
//! `BRepBuilderAPI_MakeVertex` are the fakes of the test support module (TKPrim and
//! TKTopAlgo are not ported). An `ApplyModification` modifier returns `Result`, the
//! Rust form of a modifier that throws. A standalone `BRepGraph_LayerHistory` is not
//! attached to any graph; its `Absorb` receives the fixture graph, which it ignores
//! as the detached native layer does.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
use rustycad_tkbrep::brep_graph::shapes_view::ShapeNodeMap;
use rustycad_tkbrep::brep_graph::tool::edge as tool_edge;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use support::{make_box, make_vertex};

/// The `BRepGraph_LayerHistoryTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    myGraph
}

/// `myGraph.LayerRegistry().Ensure<BRepGraph_LayerHistory>()`.
fn history(theGraph: &BRepGraph) -> Handle<LayerHistory> {
    theGraph.layer_registry().ensure::<LayerHistory>().unwrap()
}

fn label(theText: &str) -> AsciiString {
    AsciiString::new(theText)
}

fn edge(theIndex: u32) -> NodeId {
    NodeId::new(NodeKind::EDGE, theIndex)
}

/// `History->Record(label, original, replacements)` (the default Modified kind).
fn record(theGraph: &BRepGraph, theLabel: &str, theOriginal: NodeId, theReplacements: &[NodeId]) {
    record_kind(
        theGraph,
        theLabel,
        theOriginal,
        theReplacements,
        HistoryKind::Modified,
    );
}

fn record_kind(
    theGraph: &BRepGraph,
    theLabel: &str,
    theOriginal: NodeId,
    theReplacements: &[NodeId],
    theKind: HistoryKind,
) {
    let aHistory = history(theGraph);
    let aHistory = aHistory.get().unwrap();
    aHistory.record(
        theGraph,
        &label(theLabel),
        theOriginal,
        theReplacements,
        theKind,
    );
}

fn nb_records(theGraph: &BRepGraph) -> usize {
    history(theGraph).get().unwrap().nb_records()
}

fn record_name_and_kind(theHistory: &LayerHistory, theIndex: usize) -> (AsciiString, HistoryKind) {
    theHistory
        .record_at(theIndex, |anEvent| {
            (anEvent.operation_name.clone(), anEvent.record_kind)
        })
        .unwrap()
}

fn collectWiresOfEdge(theGraph: &BRepGraph, theEdge: EdgeId) -> Vec<WireId> {
    theGraph.topo().edges().wires_of(theEdge).collect()
}

/// `ApplyModification` of a modifier that appends the next edge index (the replacement
/// is written to `theNew`).
fn applyNextEdge(theGraph: &mut BRepGraph, theTarget: NodeId, theLabel: &str) -> NodeId {
    let mut aNew = NodeId::default();
    theGraph
        .editor()
        .generic()
        .apply_modification(
            theTarget,
            |theGraph, _theTarget| {
                // Simulate producing a new edge node at index NbEdgeDefs.
                aNew = NodeId::new(NodeKind::EDGE, theGraph.topo().edges().nb());
                Ok(vec![aNew])
            },
            &label(theLabel),
        )
        .unwrap();
    aNew
}

// ============================================================
// History chain tests
// ============================================================

#[test]
fn BRepGraph_LayerHistoryTest__FindOriginal_ChainABC_ReturnsA() {
    let mut myGraph = SetUp();
    // Build a chain: edge0 -> edge1 -> edge2 via two ApplyModification calls.
    let anEdge0 = edge(0);
    let anEdge1 = applyNextEdge(&mut myGraph, anEdge0, "Step1");
    let anEdge2 = applyNextEdge(&mut myGraph, anEdge1, "Step2");

    let anOriginal = history(&myGraph).get().unwrap().find_original(anEdge2);
    assert!(anOriginal.is_valid());
    assert_eq!(anOriginal, anEdge0);
}

#[test]
fn BRepGraph_LayerHistoryTest__FindDerived_ChainABC_ContainsBAndC() {
    let mut myGraph = SetUp();
    let anEdge0 = edge(0);
    let anEdge1 = applyNextEdge(&mut myGraph, anEdge0, "Step1");
    let anEdge2 = applyNextEdge(&mut myGraph, anEdge1, "Step2");

    let aDerived = history(&myGraph).get().unwrap().find_derived(anEdge0);
    assert!(aDerived.contains(&anEdge1));
    assert!(aDerived.contains(&anEdge2));
}

#[test]
fn BRepGraph_LayerHistoryTest__FindOriginal_UnmodifiedNode_ReturnsSelf() {
    let myGraph = SetUp();
    // FindOriginal returns the node itself when it is not derived from anything.
    let aFace = NodeId::new(NodeKind::FACE, 0);
    let anOriginal = history(&myGraph).get().unwrap().find_original(aFace);
    assert!(anOriginal.is_valid());
    assert_eq!(anOriginal, aFace);
}

#[test]
fn BRepGraph_LayerHistoryTest__FindDerived_UnmodifiedNode_ReturnsEmpty() {
    let myGraph = SetUp();
    let aFace = NodeId::new(NodeKind::FACE, 0);
    let aDerived = history(&myGraph).get().unwrap().find_derived(aFace);
    assert_eq!(aDerived.len(), 0);
}

#[test]
fn BRepGraph_LayerHistoryTest__Disabled_RecordHistory_NoRecordStored() {
    let myGraph = SetUp();
    let aNbBefore = nb_records(&myGraph);
    history(&myGraph).get().unwrap().set_enabled(false);

    let anEdge0 = edge(0);
    let aReplacements = [edge(1)];
    record(&myGraph, "Disabled", anEdge0, &aReplacements);

    assert_eq!(nb_records(&myGraph), aNbBefore);
}

#[test]
fn BRepGraph_LayerHistoryTest__Disabled_ApplyModification_ModifierStillRuns() {
    let mut myGraph = SetUp();
    history(&myGraph).get().unwrap().set_enabled(false);
    let mut aModifierRan = false;

    let anEdge0 = edge(0);
    myGraph
        .editor()
        .generic()
        .apply_modification(
            anEdge0,
            |_, _| {
                aModifierRan = true;
                Ok(Vec::new())
            },
            &label("DisabledOp"),
        )
        .unwrap();

    assert!(aModifierRan);
}

#[test]
fn BRepGraph_LayerHistoryTest__ReEnabled_RecordsAfterReEnable() {
    let myGraph = SetUp();
    history(&myGraph).get().unwrap().set_enabled(false);
    assert!(!history(&myGraph).get().unwrap().is_enabled());

    history(&myGraph).get().unwrap().set_enabled(true);
    assert!(history(&myGraph).get().unwrap().is_enabled());

    let aNbBefore = nb_records(&myGraph);
    let anEdge0 = edge(0);
    let aReplacements = [edge(1)];
    record(&myGraph, "ReEnabled", anEdge0, &aReplacements);

    assert_eq!(nb_records(&myGraph), aNbBefore + 1);
}

#[test]
fn BRepGraph_LayerHistoryTest__ApplyModification_EmptyReplacements() {
    let mut myGraph = SetUp();
    let anEdge0 = edge(0);
    let aNbBefore = nb_records(&myGraph);

    myGraph
        .editor()
        .generic()
        .apply_modification(anEdge0, |_, _| Ok(Vec::new()), &label("Delete"))
        .unwrap();

    assert_eq!(nb_records(&myGraph), aNbBefore + 1);
    let aHistory = history(&myGraph);
    let (aName, _) = record_name_and_kind(aHistory.get().unwrap(), nb_records(&myGraph) - 1);
    assert!(aName.is_equal(label("Delete")));
}

#[test]
fn BRepGraph_LayerHistoryTest__ApplyModification_MultipleReplacements() {
    let mut myGraph = SetUp();
    let anEdge0 = edge(0);
    let mut aNew1 = NodeId::default();
    let mut aNew2 = NodeId::default();

    myGraph
        .editor()
        .generic()
        .apply_modification(
            anEdge0,
            |theGraph, _| {
                let aBase = theGraph.topo().edges().nb();
                aNew1 = NodeId::new(NodeKind::EDGE, aBase);
                aNew2 = NodeId::new(NodeKind::EDGE, aBase + 1);
                Ok(vec![aNew1, aNew2])
            },
            &label("Split"),
        )
        .unwrap();

    let aDerived = history(&myGraph).get().unwrap().find_derived(anEdge0);
    assert!(aDerived.contains(&aNew1));
    assert!(aDerived.contains(&aNew2));
}

#[test]
fn BRepGraph_LayerHistoryTest__RecordHistory_EmptyReplacements_Stored() {
    let myGraph = SetUp();
    let aNbBefore = nb_records(&myGraph);
    let anEdge0 = edge(0);
    record(&myGraph, "Erase", anEdge0, &[]);

    assert_eq!(nb_records(&myGraph), aNbBefore + 1);
}

#[test]
fn BRepGraph_LayerHistoryTest__HistoryRecord_SequenceNumber_Monotonic() {
    let myGraph = SetUp();
    let anEdge0 = edge(0);
    let anEdge1 = edge(1);
    let anEdge2 = edge(2);

    record(&myGraph, "Op1", anEdge0, &[anEdge1]);
    record(&myGraph, "Op2", anEdge1, &[anEdge2]);

    let aNb = nb_records(&myGraph);
    assert!(aNb >= 2);
    let aHistory = history(&myGraph);
    let aHistory = aHistory.get().unwrap();
    let aSeq1 = aHistory.record_at(aNb - 2, |e| e.sequence_number).unwrap();
    let aSeq2 = aHistory.record_at(aNb - 1, |e| e.sequence_number).unwrap();
    assert!(aSeq1 < aSeq2);
}

#[test]
fn BRepGraph_LayerHistoryTest__HistoryRecord_OperationName_Stored() {
    let myGraph = SetUp();
    let anEdge0 = edge(0);
    record(&myGraph, "MyCustomOp", anEdge0, &[edge(1)]);

    let aHistory = history(&myGraph);
    let (aName, _) = record_name_and_kind(aHistory.get().unwrap(), nb_records(&myGraph) - 1);
    assert!(aName.is_equal(label("MyCustomOp")));
}

#[test]
fn BRepGraph_LayerHistoryTest__NbHistoryRecords_AfterMultipleOps_Correct() {
    let myGraph = SetUp();
    let aNbBefore = nb_records(&myGraph);

    let anEdge0 = edge(0);
    let aRepl = [edge(1)];

    record(&myGraph, "A", anEdge0, &aRepl);
    record(&myGraph, "B", anEdge0, &aRepl);
    record(&myGraph, "C", anEdge0, &aRepl);

    assert_eq!(nb_records(&myGraph), aNbBefore + 3);
}

/// `ApplyModification` of a modifier that appends the next vertex index.
fn applyNextVertex(theGraph: &mut BRepGraph, theTarget: NodeId, theLabel: &str) -> NodeId {
    let mut aNew = NodeId::default();
    theGraph
        .editor()
        .generic()
        .apply_modification(
            theTarget,
            |theGraph, _| {
                aNew = NodeId::new(NodeKind::VERTEX, theGraph.topo().vertices().nb());
                Ok(vec![aNew])
            },
            &label(theLabel),
        )
        .unwrap();
    aNew
}

#[test]
fn BRepGraph_LayerHistoryTest__FindOriginal_TwoApply_TransitiveTrace() {
    let mut myGraph = SetUp();
    let aVtx0 = NodeId::new(NodeKind::VERTEX, 0);
    let aVtx1 = applyNextVertex(&mut myGraph, aVtx0, "Move1");
    let aVtx2 = applyNextVertex(&mut myGraph, aVtx1, "Move2");

    // FindOriginal from the end of a 2-step chain should reach the root.
    let anOriginal = history(&myGraph).get().unwrap().find_original(aVtx2);
    assert!(anOriginal.is_valid());
    assert_eq!(anOriginal, aVtx0);

    // Intermediate node should also trace back to root.
    let anOriginal1 = history(&myGraph).get().unwrap().find_original(aVtx1);
    assert!(anOriginal1.is_valid());
    assert_eq!(anOriginal1, aVtx0);
}

#[test]
fn BRepGraph_LayerHistoryTest__ApplyModification_WhenModifierThrows_DoesNotRecordHistory() {
    let mut myGraph = SetUp();
    let aNbRecordsBefore = nb_records(&myGraph);

    let anEdge = edge(0);

    let aResult = myGraph.editor().generic().apply_modification(
        anEdge,
        |_, _| -> Result<Vec<NodeId>, Failure> {
            Err(Failure::with_type(
                "Standard_Failure",
                "Synthetic failure",
                "",
            ))
        },
        &label("ThrowingModification"),
    );
    assert_eq!(aResult.unwrap_err().exception_type(), "Standard_Failure");

    assert_eq!(nb_records(&myGraph), aNbRecordsBefore);
}

/// The split parameter at the middle of the edge's range.
fn midParameter(theGraph: &BRepGraph, theEdge: EdgeId) -> f64 {
    let (aFirst, aLast) = tool_edge::range(theGraph, theEdge);
    0.5 * (aFirst + aLast)
}

#[test]
fn BRepGraph_LayerHistoryTest__SplitEdge_RewritesAllContainingWires() {
    let mut myGraph = SetUp();
    assert!(myGraph.topo().edges().nb() > 0);

    let anEdgeId = EdgeId::new(0);
    let aSplitParam = midParameter(&myGraph, anEdgeId);

    let aNbVerticesBefore = myGraph.topo().vertices().nb();
    let aSplitPoint = Point3::new(1.0, 2.0, 3.0);
    let aSplitVertex = myGraph.editor().vertices().add(aSplitPoint, 1.0e-7);

    assert!(aSplitVertex.is_valid());
    assert_eq!(myGraph.topo().vertices().nb(), aNbVerticesBefore + 1);

    let aWireIndices = collectWiresOfEdge(&myGraph, anEdgeId);
    assert!(!aWireIndices.is_empty());

    let aNbEdgesBefore = myGraph.topo().edges().nb();
    let aNbActiveEdgesBefore = myGraph.topo().edges().nb_active();

    let (aSubA, aSubB) = myGraph
        .editor()
        .edges()
        .split(anEdgeId, aSplitVertex, aSplitParam)
        .unwrap();

    assert!(aSubA.is_valid());
    assert!(aSubB.is_valid());
    assert_eq!(myGraph.topo().edges().nb(), aNbEdgesBefore + 2);
    assert_eq!(myGraph.topo().edges().nb_active(), aNbActiveEdgesBefore + 1);

    for aWireId in aWireIndices {
        let aCoEdgeIds = &myGraph.topo().wires().relations(aWireId).co_edge_ids;

        let mut hasOld = false;
        let mut aSubAOrd = None;
        let mut aSubBOrd = None;

        for (anIdx, aCoEdgeId) in aCoEdgeIds.iter().enumerate() {
            let aCoEdge = myGraph.topo().co_edges().definition(*aCoEdgeId);
            let anId = aCoEdge.child_edge_id;
            if anId == anEdgeId {
                hasOld = true;
            } else if anId == aSubA {
                aSubAOrd = Some(anIdx);
            } else if anId == aSubB {
                aSubBOrd = Some(anIdx);
            }
        }

        assert!(!hasOld);
        let (Some(aSubAOrd), Some(aSubBOrd)) = (aSubAOrd, aSubBOrd) else {
            panic!("both sub-edges must be used by wire {aWireId:?}");
        };
        let aDistance = aSubAOrd.abs_diff(aSubBOrd);
        assert!(aDistance == 1 || aDistance + 1 == aCoEdgeIds.len());
    }
}

#[test]
fn BRepGraph_LayerHistoryTest__SplitEdge_IgnoresRemovedCoEdgeEntries() {
    let mut myGraph = SetUp();
    assert!(myGraph.topo().edges().nb() > 0);

    let anEdgeId = EdgeId::new(0);
    let aSplitParam = midParameter(&myGraph, anEdgeId);

    let aWireIndices = collectWiresOfEdge(&myGraph, anEdgeId);
    assert!(aWireIndices.len() > 1);

    let aWireId = aWireIndices[0];
    let aWireCoEdgesBefore = myGraph
        .topo()
        .wires()
        .relations(aWireId)
        .co_edge_ids
        .clone();
    assert!(!aWireCoEdgesBefore.is_empty());

    let aCoEdgeToRemove = aWireCoEdgesBefore
        .iter()
        .copied()
        .find(|aCoEdgeId| {
            myGraph
                .topo()
                .co_edges()
                .definition(*aCoEdgeId)
                .child_edge_id
                == anEdgeId
        })
        .unwrap_or_default();
    assert!(aCoEdgeToRemove.is_valid_for(myGraph.topo().co_edges().nb()));

    myGraph
        .editor()
        .generic()
        .remove_node(aCoEdgeToRemove.into())
        .unwrap();
    assert!(aCoEdgeToRemove.is_removed(&myGraph));
    let aRemovedCoEdgeId = aCoEdgeToRemove;

    let aRemovedWireNbActiveBefore = myGraph.topo().wires().relations(aWireId).co_edge_ids.len();

    let aSplitVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(4.0, 5.0, 6.0), 1.0e-7);
    assert!(aSplitVertex.is_valid());

    let (aSubA, aSubB) = myGraph
        .editor()
        .edges()
        .split(anEdgeId, aSplitVertex, aSplitParam)
        .unwrap();
    assert!(aSubA.is_valid());
    assert!(aSubB.is_valid());

    assert!(aCoEdgeToRemove.is_removed(&myGraph));
    assert_eq!(
        myGraph.topo().wires().relations(aWireId).co_edge_ids.len(),
        aRemovedWireNbActiveBefore
    );
    let aRemovedCoEdgeAfter = myGraph.topo().co_edges().definition(aRemovedCoEdgeId);
    assert_eq!(aRemovedCoEdgeAfter.child_edge_id, anEdgeId);

    let mut hasSubA = false;
    let mut hasSubB = false;
    let mut aCoEdgeId = CoEdgeId::start();
    while aCoEdgeId.is_valid_for(myGraph.topo().co_edges().nb()) {
        let aCoEdge = myGraph.topo().co_edges().definition(aCoEdgeId);
        if !aCoEdgeId.is_removed(&myGraph) {
            hasSubA |= aCoEdge.child_edge_id == aSubA;
            hasSubB |= aCoEdge.child_edge_id == aSubB;
        }
        aCoEdgeId.pre_increment();
    }
    assert!(hasSubA);
    assert!(hasSubB);
}

#[test]
fn BRepGraph_LayerHistoryTest__ApplyModification_SplitEdge_RecordsBothDerivedNodes() {
    let mut myGraph = SetUp();
    assert!(myGraph.topo().edges().nb() > 0);

    let anEdgeId = EdgeId::new(0);
    let aSplitParam = midParameter(&myGraph, anEdgeId);

    let aSplitVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(4.0, 5.0, 6.0), 1.0e-7);
    assert!(aSplitVertex.is_valid());

    let aNbRecordsBefore = nb_records(&myGraph);

    myGraph
        .editor()
        .generic()
        .apply_modification(
            anEdgeId.into(),
            |theGraph, theTarget| {
                let (aSubA, aSubB) = theGraph.editor().edges().split(
                    EdgeId::from_node_id(theTarget),
                    aSplitVertex,
                    aSplitParam,
                )?;
                Ok(vec![NodeId::from(aSubA), NodeId::from(aSubB)])
            },
            &label("Split"),
        )
        .unwrap();

    assert_eq!(nb_records(&myGraph), aNbRecordsBefore + 1);

    let aDerived = history(&myGraph)
        .get()
        .unwrap()
        .find_derived(anEdgeId.into());
    assert!(aDerived.len() >= 2);
}

#[test]
fn BRepGraph_LayerHistoryTest__FindModified_PerKindLookup_ReturnsOnlyModified() {
    let mut myGraph = SetUp();
    let anOrig = edge(0);
    let aRepl = applyNextEdge(&mut myGraph, anOrig, "Modify");

    let aHistory = history(&myGraph);
    let aMod = aHistory.get().unwrap().find_modified(anOrig);
    let aMod = aMod.expect("FindModified must find the modification");
    assert_eq!(aMod.len(), 1);
    assert_eq!(aMod[0], aRepl);

    let aGen = aHistory.get().unwrap().find_generated(anOrig);
    assert!(aGen.is_none());
}

#[test]
fn BRepGraph_LayerHistoryTest__FindGenerated_PerKindLookup_ReturnsOnlyGenerated() {
    let myGraph = SetUp();
    let anOrig = edge(0);
    let anRepl = edge(1);

    record_kind(
        &myGraph,
        "Generate",
        anOrig,
        &[anRepl],
        HistoryKind::Generated,
    );

    let aHistory = history(&myGraph);
    let aGen = aHistory.get().unwrap().find_generated(anOrig);
    let aGen = aGen.expect("FindGenerated must find the generation");
    assert_eq!(aGen.len(), 1);
    assert_eq!(aGen[0], anRepl);

    let aMod = aHistory.get().unwrap().find_modified(anOrig);
    assert!(aMod.is_none());
}

#[test]
fn BRepGraph_LayerHistoryTest__RecordDeleted_MarksNodesAsDeleted() {
    let myGraph = SetUp();
    let anEdgeA = edge(0);
    let anEdgeB = edge(1);

    let aDeleted = [anEdgeA, anEdgeB];

    let aNbBefore = nb_records(&myGraph);
    let aHistory = history(&myGraph);
    let aHistory = aHistory.get().unwrap();
    aHistory.record_deleted(&myGraph, &label("DeleteEdges"), &aDeleted);
    assert_eq!(aHistory.nb_records(), aNbBefore + 1);

    assert!(aHistory.is_deleted(anEdgeA));
    assert!(aHistory.is_deleted(anEdgeB));

    let aDeletedSet = aHistory.deleted_nodes();
    assert!(aDeletedSet.contains(&anEdgeA));
    assert!(aDeletedSet.contains(&anEdgeB));
}

#[test]
fn BRepGraph_LayerHistoryTest__RecordDeleted_EmptyList_IsNoop() {
    let myGraph = SetUp();
    let aNbBefore = nb_records(&myGraph);
    let aHistory = history(&myGraph);
    aHistory
        .get()
        .unwrap()
        .record_deleted(&myGraph, &label("Noop"), &[]);
    assert_eq!(nb_records(&myGraph), aNbBefore);
}

#[test]
fn BRepGraph_LayerHistoryTest__Record_EmptyReplacements_AutoDeleted() {
    let myGraph = SetUp();
    let anOrig = edge(0);

    record_kind(&myGraph, "Consumed", anOrig, &[], HistoryKind::Generated);

    let aHistory = history(&myGraph);
    let aHistory = aHistory.get().unwrap();
    assert!(aHistory.is_deleted(anOrig));
    assert_eq!(aHistory.nb_records(), 1);
    assert_eq!(record_name_and_kind(aHistory, 0).1, HistoryKind::Deleted);
}

#[test]
fn BRepGraph_LayerHistoryTest__FindDerived_ModifiedAndGenerated_CollectsBoth() {
    let mut myGraph = SetUp();
    let anOrig = edge(0);
    let aModRepl = applyNextEdge(&mut myGraph, anOrig, "FirstModified");

    // Record a Generated event on the Modified result.
    let aGenRepl = edge(2);
    record_kind(
        &myGraph,
        "ThenGenerated",
        aModRepl,
        &[aGenRepl],
        HistoryKind::Generated,
    );

    let aHistory = history(&myGraph);
    let aHistory = aHistory.get().unwrap();
    let aAll = aHistory.find_derived(anOrig);
    assert!(!aAll.is_empty());
    assert!(aAll.contains(&aGenRepl));
    // Verify the Generated record is in the per-kind map.
    assert!(aHistory.find_generated(aModRepl).is_some());
    assert!(aHistory.find_modified(anOrig).is_some());
}

#[test]
fn BRepGraph_LayerHistoryTest__RecordBatch_ExplicitGeneratedKind_StoredCorrectly() {
    let myGraph = SetUp();
    let anOrig = edge(0);
    let anRepl = edge(1);

    let aOriginals = [anOrig];
    let aReplacements = [anRepl];

    let aHistory = history(&myGraph);
    let aHistory = aHistory.get().unwrap();
    aHistory
        .record_batch(
            &myGraph,
            &label("BatchGen"),
            &aOriginals,
            &aReplacements,
            &AsciiString::default(),
            HistoryKind::Generated,
        )
        .unwrap();

    assert_eq!(aHistory.nb_records(), 1);
    assert_eq!(record_name_and_kind(aHistory, 0).1, HistoryKind::Generated);
    assert!(aHistory.find_generated(anOrig).is_some());
    assert!(aHistory.find_modified(anOrig).is_none());
}

#[test]
fn BRepGraph_LayerHistoryTest__Record_ExplicitModifiedKind_DefaultMatches() {
    let myGraph = SetUp();
    let aRepls = [edge(1)];

    let aNbBefore = nb_records(&myGraph);
    record_kind(
        &myGraph,
        "ExplicitMod",
        edge(0),
        &aRepls,
        HistoryKind::Modified,
    );
    assert_eq!(nb_records(&myGraph), aNbBefore + 1);
    let aHistory = history(&myGraph);
    assert_eq!(
        record_name_and_kind(aHistory.get().unwrap(), aNbBefore).1,
        HistoryKind::Modified
    );
}

#[test]
fn BRepGraph_LayerHistoryTest__IsDeleted_UnrelatedNode_ReturnsFalse() {
    let myGraph = SetUp();
    let aNode = edge(0);
    assert!(!history(&myGraph).get().unwrap().is_deleted(aNode));
}

#[test]
fn BRepGraph_LayerHistoryTest__RecordReplaced_MapsImageAndMarksDeleted() {
    let myGraph = SetUp();
    let anOrig = NodeId::new(NodeKind::FACE, 0);
    let aReplacement = NodeId::new(NodeKind::FACE, 1);

    let aHistory = history(&myGraph);
    let aHistory = aHistory.get().unwrap();
    aHistory.record_replaced(&myGraph, &label("ReplaceFace"), anOrig, aReplacement);

    assert_eq!(aHistory.nb_records(), 1);
    assert_eq!(record_name_and_kind(aHistory, 0).1, HistoryKind::Replaced);
    assert!(aHistory.is_deleted(anOrig));

    let aModified = aHistory.find_modified(anOrig).expect("the image is mapped");
    assert_eq!(aModified.len(), 1);
    assert_eq!(aModified[0], aReplacement);

    let anOrigins = aHistory
        .find_originals(aReplacement)
        .expect("the origin is mapped");
    assert_eq!(anOrigins.len(), 1);
    assert_eq!(anOrigins[0], anOrig);
}

#[test]
fn BRepGraph_LayerHistoryTest__StructuralReplacement_DoesNotEmitSemanticHistory() {
    let mut myGraph = SetUp();
    assert!(myGraph.topo().faces().nb() > 1);
    history(&myGraph).get().unwrap().clear_records();

    let aRemoved = FaceId::new(1);
    let aReplacement = FaceId::new(0);
    myGraph
        .editor()
        .generic()
        .replace_node(aRemoved.into(), aReplacement.into())
        .unwrap();

    let aHistory = history(&myGraph);
    let aHistory = aHistory.get().unwrap();
    assert_eq!(aHistory.nb_records(), 0);
    assert!(!aHistory.is_deleted(aRemoved.into()));
    assert!(aHistory.find_modified(aRemoved.into()).is_none());
}

#[test]
fn BRepGraph_LayerHistoryTest__FindOriginals_DerivedWithTwoParents_ReturnsBoth() {
    let myGraph = SetUp();
    let anOrigA = NodeId::new(NodeKind::FACE, 0);
    let anOrigB = NodeId::new(NodeKind::FACE, 1);
    let aDerived = edge(0);

    record(&myGraph, "FirstParent", anOrigA, &[aDerived]);
    record(&myGraph, "SecondParent", anOrigB, &[aDerived]);

    let anOrigins = history(&myGraph)
        .get()
        .unwrap()
        .find_originals(aDerived)
        .expect("both origins are mapped");
    assert_eq!(anOrigins.len(), 2);
    assert!(anOrigins.contains(&anOrigA));
    assert!(anOrigins.contains(&anOrigB));
}

#[test]
fn BRepGraph_LayerHistoryTest__RecordUid_Modified_AppendsAuditRecord() {
    let _myGraph = SetUp();
    let anOrig = Uid::new(NodeKind::FACE, 10);
    let aRepl = Uid::new(NodeKind::FACE, 11);

    let aReplacements = [aRepl];

    let aHist = LayerHistory::new();
    aHist.record_uid(
        &label("UidMod"),
        anOrig,
        &aReplacements,
        HistoryKind::Modified,
    );

    assert_eq!(aHist.nb_records(), 1);
    aHist
        .record_at(0, |aRecord| {
            assert!(aRecord.operation_name.is_equal(label("UidMod")));
            assert_eq!(aRecord.record_kind, HistoryKind::Modified);
            assert!(aRecord.uid_mapping.is_bound(&anOrig));
            let aMapped = aRecord.uid_mapping.find(&anOrig).unwrap();
            assert_eq!(aMapped.len(), 1);
            assert_eq!(aMapped[0], aRepl);
            assert!(aRecord.mapping.is_empty());
        })
        .unwrap();
    assert!(aHist.has_known_input_uid(&anOrig));

    let aMod = aHist
        .find_modified_uid(&anOrig)
        .expect("the UID image is mapped");
    assert_eq!(aMod.len(), 1);
    assert_eq!(aMod[0], aRepl);
}

#[test]
fn BRepGraph_LayerHistoryTest__RecordDeletedUid_AppendsAuditRecord() {
    let _myGraph = SetUp();
    let anOrigA = Uid::new(NodeKind::EDGE, 20);
    let anOrigB = Uid::new(NodeKind::EDGE, 21);

    let aDeleted = [anOrigA, anOrigB];

    let aHist = LayerHistory::new();
    aHist.record_deleted_uid(&label("UidDelete"), &aDeleted);

    assert_eq!(aHist.nb_records(), 1);
    aHist
        .record_at(0, |aRecord| {
            assert!(aRecord.operation_name.is_equal(label("UidDelete")));
            assert_eq!(aRecord.record_kind, HistoryKind::Deleted);
            assert!(aRecord.uid_mapping.is_bound(&anOrigA));
            assert!(aRecord.uid_mapping.is_bound(&anOrigB));
            assert!(aRecord.uid_mapping.find(&anOrigA).unwrap().is_empty());
            assert!(aRecord.uid_mapping.find(&anOrigB).unwrap().is_empty());
        })
        .unwrap();
    assert!(aHist.is_deleted_uid(&anOrigA));
    assert!(aHist.is_deleted_uid(&anOrigB));
    assert!(aHist.has_known_input_uid(&anOrigA));
    assert!(aHist.has_known_input_uid(&anOrigB));
}

// ============================================================
// Absorb (BRepTools_History -> BRepGraph_LayerHistory bridge) tests
// ============================================================

fn MakeVertexShape(theX: f64, theY: f64, theZ: f64) -> Shape {
    make_vertex(Point3::new(theX, theY, theZ)).into()
}

#[test]
fn BRepGraph_LayerHistoryTest__Absorb_NullSource_NoOp() {
    let myGraph = SetUp();
    let aHist = LayerHistory::new();
    let anInputs = ShapeNodeMap::default();
    let anOutputs = ShapeNodeMap::default();
    let aNullSrc: Handle<History> = Handle::default();
    aHist
        .absorb(
            &myGraph,
            &anInputs,
            &anOutputs,
            &aNullSrc,
            &label("NullSrc"),
        )
        .unwrap();
    assert_eq!(aHist.nb_records(), 0);
}

#[test]
fn BRepGraph_LayerHistoryTest__Absorb_EmptyInputs_NoOp() {
    let myGraph = SetUp();
    let aHist = LayerHistory::new();
    let anInputs = ShapeNodeMap::default();
    let anOutputs = ShapeNodeMap::default();
    let aSrc = Handle::new(History::new());
    aHist
        .absorb(
            &myGraph,
            &anInputs,
            &anOutputs,
            &aSrc,
            &label("EmptyInputs"),
        )
        .unwrap();
    assert_eq!(aHist.nb_records(), 0);
}

// The absorbed history of one input and one output shape bound to the given nodes
// (an output without node is left unbound).
fn absorbOne(
    myGraph: &BRepGraph,
    aSrc: &Handle<History>,
    (anInShape, anInNode): (&Shape, NodeId),
    (anOutShape, anOutNode): (&Shape, Option<NodeId>),
    aLabel: &str,
) -> LayerHistory {
    let mut anInputs = ShapeNodeMap::default();
    anInputs.bind(anInShape.clone(), anInNode);
    let mut anOutputs = ShapeNodeMap::default();
    if let Some(anOutNode) = anOutNode {
        anOutputs.bind(anOutShape.clone(), anOutNode);
    }

    let aHist = LayerHistory::new();
    aHist
        .absorb(myGraph, &anInputs, &anOutputs, aSrc, &label(aLabel))
        .unwrap();
    aHist
}

#[test]
fn BRepGraph_LayerHistoryTest__Absorb_ModifiedOnly_EmitsModifiedRecord() {
    let myGraph = SetUp();
    let anInShape = MakeVertexShape(0.0, 0.0, 0.0);
    let anOutShape = MakeVertexShape(1.0, 0.0, 0.0);
    let anInNode = NodeId::new(NodeKind::VERTEX, 100);
    let anOutNode = NodeId::new(NodeKind::VERTEX, 101);

    let aSrc = Handle::new(History::new());
    aSrc.add_modified(&anInShape, &anOutShape).unwrap();

    let aHist = absorbOne(
        &myGraph,
        &aSrc,
        (&anInShape, anInNode),
        (&anOutShape, Some(anOutNode)),
        "Mod",
    );

    assert_eq!(aHist.nb_records(), 1);
    assert_eq!(record_name_and_kind(&aHist, 0).1, HistoryKind::Modified);

    let aMod = aHist.find_modified(anInNode).expect("the image is mapped");
    assert_eq!(aMod.len(), 1);
    assert_eq!(aMod[0], anOutNode);
    assert!(aHist.find_generated(anInNode).is_none());
    assert!(!aHist.is_deleted(anInNode));
}

#[test]
fn BRepGraph_LayerHistoryTest__Absorb_GeneratedOnly_EmitsGeneratedRecord() {
    let myGraph = SetUp();
    let anInShape = MakeVertexShape(0.0, 0.0, 0.0);
    let anOutShape = MakeVertexShape(2.0, 0.0, 0.0);
    let anInNode = NodeId::new(NodeKind::VERTEX, 200);
    let anOutNode = NodeId::new(NodeKind::VERTEX, 201);

    let aSrc = Handle::new(History::new());
    aSrc.add_generated(&anInShape, &anOutShape).unwrap();

    let aHist = absorbOne(
        &myGraph,
        &aSrc,
        (&anInShape, anInNode),
        (&anOutShape, Some(anOutNode)),
        "Gen",
    );

    assert_eq!(aHist.nb_records(), 1);
    assert_eq!(record_name_and_kind(&aHist, 0).1, HistoryKind::Generated);

    let aGen = aHist.find_generated(anInNode).expect("the image is mapped");
    assert_eq!(aGen.len(), 1);
    assert_eq!(aGen[0], anOutNode);
    assert!(aHist.find_modified(anInNode).is_none());
}

#[test]
fn BRepGraph_LayerHistoryTest__Absorb_RemovedTakesPrecedenceOverModified() {
    let myGraph = SetUp();
    // The OCCT bug being guarded against: a shape can show up as both
    // IsRemoved() and in Modified()/Generated().  Absorb must classify it
    // as a deletion event regardless.
    let anInShape = MakeVertexShape(0.0, 0.0, 0.0);
    let aGhostOut = MakeVertexShape(3.0, 0.0, 0.0);
    let anInNode = NodeId::new(NodeKind::VERTEX, 300);
    let aGhostNode = NodeId::new(NodeKind::VERTEX, 301);

    let aSrc = Handle::new(History::new());
    aSrc.add_modified(&anInShape, &aGhostOut).unwrap();
    aSrc.remove(&anInShape).unwrap();

    let aHist = absorbOne(
        &myGraph,
        &aSrc,
        (&anInShape, anInNode),
        (&aGhostOut, Some(aGhostNode)),
        "RemovedWins",
    );

    assert!(aHist.is_deleted(anInNode));
    assert!(aHist.find_modified(anInNode).is_none());
    assert_eq!(aHist.nb_records(), 1);
    assert_eq!(record_name_and_kind(&aHist, 0).1, HistoryKind::Deleted);
}

#[test]
fn BRepGraph_LayerHistoryTest__Absorb_OutputMissingFromMap_DroppedSilently() {
    let myGraph = SetUp();
    let anInShape = MakeVertexShape(0.0, 0.0, 0.0);
    let anOutShape = MakeVertexShape(4.0, 0.0, 0.0);
    let anInNode = NodeId::new(NodeKind::VERTEX, 400);

    let aSrc = Handle::new(History::new());
    aSrc.add_modified(&anInShape, &anOutShape).unwrap();

    // Intentionally do not bind anOutShape in anOutputs.
    let aHist = absorbOne(
        &myGraph,
        &aSrc,
        (&anInShape, anInNode),
        (&anOutShape, None),
        "Drop",
    );

    assert_eq!(aHist.nb_records(), 0);
    assert!(aHist.find_modified(anInNode).is_none());
}

#[test]
fn BRepGraph_LayerHistoryTest__ClearReleasesHistoryAndAllowsFreshUidRecording() {
    let _myGraph = SetUp();
    let aHist = LayerHistory::new();

    for anIdx in 0..1000_u32 {
        let anOriginal = Uid::new(NodeKind::EDGE, anIdx + 1);
        let aReplacements = [Uid::new(NodeKind::EDGE, anIdx + 1001)];
        aHist.record_uid(
            &label("Stress"),
            anOriginal,
            &aReplacements,
            HistoryKind::Modified,
        );
    }
    assert_eq!(aHist.nb_records(), 1000);

    aHist.clear_records();
    assert_eq!(aHist.nb_records(), 0);

    let aFreshOriginal = Uid::new(NodeKind::FACE, 20);
    let aFreshReplacement = Uid::new(NodeKind::FACE, 21);
    let aFreshReplacements = [aFreshReplacement];
    aHist.record_uid(
        &label("AfterClear"),
        aFreshOriginal,
        &aFreshReplacements,
        HistoryKind::Modified,
    );

    assert_eq!(aHist.nb_records(), 1);
    let aModified = aHist
        .find_modified_uid(&aFreshOriginal)
        .expect("the UID image is mapped");
    assert_eq!(aModified.len(), 1);
    assert_eq!(aModified[0], aFreshReplacement);
    assert!(!aHist.is_deleted_uid(&aFreshOriginal));
}
