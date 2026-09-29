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

//! Direct translation of BRepGraph_ChildExplorer_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). The native constructor overloads are
//! the named `ChildExplorer` constructors (docs/topology-design.md section 11.9).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::child_explorer::{ChildExplorer, LinkKind, TraversalMode};
use rustycad_tkbrep::brep_graph::editor::OccurrenceOps;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::NodeInstance;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use support::make_box;

fn makeDirectChildExplorer(
    theGraph: &BRepGraph,
    theRoot: impl Into<NodeId>,
    theTargetKind: NodeKind,
) -> ChildExplorer<'_> {
    ChildExplorer::with_target_mode(
        theGraph,
        theRoot,
        theTargetKind,
        TraversalMode::DirectChildren,
    )
    .unwrap()
}

fn makeDirectChildExplorerAt<'g>(
    theGraph: &'g BRepGraph,
    theRoot: NodeId,
    theTargetKind: NodeKind,
    theLoc: &Location,
    theOri: Orientation,
) -> ChildExplorer<'g> {
    ChildExplorer::with_start(
        theGraph,
        theRoot,
        theTargetKind,
        theLoc,
        theOri,
        TraversalMode::DirectChildren,
    )
    .unwrap()
}

/// `BRepGraph aGraph; aGraph.Clear(); aGraph.Shapes().Add(shape)`.
fn graphOf(theShape: &Shape) -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph.shapes_mut().add(theShape).unwrap();
    aGraph
}

fn boxGraph() -> BRepGraph {
    graphOf(&make_box(10.0, 20.0, 30.0))
}

/// The count of a `More`/`Next` walk.
fn count(mut anExp: ChildExplorer<'_>) -> i32 {
    let mut aCount = 0;
    while anExp.more() {
        aCount += 1;
        anExp.next().unwrap();
    }
    aCount
}

/// `TopoDS_Compound` of the shapes (`BRep_Builder::MakeCompound` then `Add`).
fn compoundOf(theShapes: &[Shape]) -> Shape {
    let aBB = BRepBuilder::default();
    let mut aComp = Compound::default();
    aBB.make_compound(&mut aComp);
    for aShape in theShapes {
        aBB.add(&mut aComp, aShape).unwrap();
    }
    aComp.into()
}

fn translation(x: f64, y: f64, z: f64) -> Location {
    Location::new(Transform3::translation_vector(Vector3::new(x, y, z)))
}

// --- Explorer tests ---

#[test]
fn BRepGraph_ChildExplorerTest__Box_EdgeOccurrences_Count24() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aCount =
        count(ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::EDGE).unwrap());
    // 12 edges x 2 faces each = 24 edge occurrences.
    assert_eq!(aCount, 24);
}

#[test]
fn BRepGraph_ChildExplorerTest__Box_FaceOccurrences_Count6() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aFaceCount =
        count(ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::FACE).unwrap());
    assert_eq!(aFaceCount, 6);
}

#[test]
fn BRepGraph_ChildExplorerTest__Box_VertexOccurrences() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    // Count unique vertices.
    let mut aCount = 0;
    let mut aVisited: HashSet<u32> = HashSet::new();
    let mut anExp =
        ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::VERTEX).unwrap();
    while anExp.more() {
        aVisited.insert(anExp.current().def_id.index);
        aCount += 1;
        anExp.next().unwrap();
    }
    assert!(aCount > 0);
    assert_eq!(aVisited.len(), 8);
}

#[test]
fn BRepGraph_ChildExplorerTest__Face_EdgeOccurrences_4() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aCount =
        count(ChildExplorer::with_target(&aGraph, FaceId::start(), NodeKind::EDGE).unwrap());
    assert_eq!(aCount, 4);
}

#[test]
fn BRepGraph_ChildExplorerTest__InvalidRoot_Empty() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let anExp = ChildExplorer::with_target(&aGraph, NodeId::default(), NodeKind::EDGE).unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ChildExplorerTest__RootEqualsTarget_ReturnsSelf() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut anExp = ChildExplorer::with_target(&aGraph, FaceId::start(), NodeKind::FACE).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(FaceId::start()));
    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ChildExplorerTest__AvoidKind_Shell_SkipsContainedFaces() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let anExp = ChildExplorer::with_target_avoid(
        &aGraph,
        SolidId::start().into(),
        NodeKind::FACE,
        Some(NodeKind::SHELL),
        false,
        TraversalMode::Recursive,
    )
    .unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ChildExplorerTest__AvoidKind_EmitBoundary_ReturnsFacesInsteadOfEdges() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut aFaceCount = 0;
    let mut anExp = ChildExplorer::with_target_avoid(
        &aGraph,
        SolidId::start().into(),
        NodeKind::EDGE,
        Some(NodeKind::FACE),
        true,
        TraversalMode::Recursive,
    )
    .unwrap();
    while anExp.more() {
        assert_eq!(anExp.current().def_id.kind, NodeKind::FACE);
        aFaceCount += 1;
        anExp.next().unwrap();
    }
    assert_eq!(aFaceCount, 6);
}

#[test]
fn BRepGraph_ChildExplorerTest__AvoidKind_SameAsTarget_IsIgnored() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut aFaceCount = 0;
    let mut anExp = ChildExplorer::with_target_avoid(
        &aGraph,
        SolidId::start().into(),
        NodeKind::FACE,
        Some(NodeKind::FACE),
        false,
        TraversalMode::Recursive,
    )
    .unwrap();
    while anExp.more() {
        assert_eq!(anExp.current().def_id.kind, NodeKind::FACE);
        aFaceCount += 1;
        anExp.next().unwrap();
    }
    assert_eq!(aFaceCount, 6);
}

#[test]
fn BRepGraph_ChildExplorerTest__AllDescendants_Recursive_YieldsAllKinds() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut aCounts: HashMap<NodeKind, i32> = HashMap::new();
    let mut anExp = ChildExplorer::new(&aGraph, SolidId::start().into()).unwrap();
    while anExp.more() {
        let aKind = anExp.current().def_id.kind;
        let anExpected = [
            NodeKind::SHELL,
            NodeKind::FACE,
            NodeKind::WIRE,
            NodeKind::CO_EDGE,
            NodeKind::EDGE,
            NodeKind::VERTEX,
        ];
        assert!(
            anExpected.contains(&aKind),
            "Unexpected kind in unfiltered child traversal"
        );
        *aCounts.entry(aKind).or_insert(0) += 1;
        anExp.next().unwrap();
    }

    let aCount = |aKind: NodeKind| aCounts.get(&aKind).copied().unwrap_or(0);
    assert_eq!(aCount(NodeKind::SHELL), 1);
    assert_eq!(aCount(NodeKind::FACE), 6);
    assert_eq!(aCount(NodeKind::WIRE), 6);
    assert_eq!(aCount(NodeKind::CO_EDGE), 24); // 4 CoEdges per face, 6 faces
    assert_eq!(aCount(NodeKind::EDGE), 24); // Each CoEdge leads to one Edge (occurrence-based)
    assert_eq!(aCount(NodeKind::VERTEX), 48); // Each Edge visited with 2 vertices
}

#[test]
fn BRepGraph_ChildExplorerTest__AllDescendants_AvoidFaceBoundary_StopsBelowFaces() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut aShellCount = 0;
    let mut aFaceCount = 0;
    let mut anExp = ChildExplorer::with_avoid(
        &aGraph,
        SolidId::start().into(),
        Some(NodeKind::FACE),
        true,
        TraversalMode::Recursive,
    )
    .unwrap();
    while anExp.more() {
        if anExp.current().def_id.kind == NodeKind::SHELL {
            aShellCount += 1;
            anExp.next().unwrap();
            continue;
        }

        assert_eq!(anExp.current().def_id.kind, NodeKind::FACE);
        aFaceCount += 1;
        anExp.next().unwrap();
    }

    assert_eq!(aShellCount, 1);
    assert_eq!(aFaceCount, 6);
}

// --- Cached location/orientation tests ---

#[test]
fn BRepGraph_ChildExplorerTest__NoCumLoc_IdentityLocation() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut anExp = ChildExplorer::with_accumulation(
        &aGraph,
        SolidId::start(),
        NodeKind::EDGE,
        false,
        true,
        TraversalMode::Recursive,
    )
    .unwrap();
    while anExp.more() {
        assert!(anExp.current().location.is_identity());
        anExp.next().unwrap();
    }
}

#[test]
fn BRepGraph_ChildExplorerTest__NoCumOri_ForwardOrientation() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut anExp = ChildExplorer::with_accumulation(
        &aGraph,
        SolidId::start(),
        NodeKind::EDGE,
        true,
        false,
        TraversalMode::Recursive,
    )
    .unwrap();
    while anExp.more() {
        assert_eq!(anExp.current().orientation, Orientation::Forward);
        anExp.next().unwrap();
    }
}

// --- Path-based composition tests ---

#[test]
fn BRepGraph_ChildExplorerTest__GlobalLocation_Box_Identity() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    // All paths in a simple box should compose to identity.
    let mut anExp = ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::EDGE).unwrap();
    while anExp.more() {
        assert!(anExp.current().location.is_identity());
        anExp.next().unwrap();
    }
}

#[test]
fn BRepGraph_ChildExplorerTest__GlobalOrientation_BoxEdges_ForwardOrReversed() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut anExp = ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::EDGE).unwrap();
    while anExp.more() {
        let anOri = anExp.current().orientation;
        assert!(anOri == Orientation::Forward || anOri == Orientation::Reversed);
        anExp.next().unwrap();
    }
}

// --- Compound tests ---

#[test]
fn BRepGraph_ChildExplorerTest__Compound_FaceCount() {
    // Build a compound of two boxes.
    let aComp = compoundOf(&[
        make_box(10.0, 10.0, 10.0).into(),
        make_box(20.0, 20.0, 20.0).into(),
    ]);

    let aGraph = graphOf(&aComp);
    assert!(!aGraph.is_empty());

    let aCount =
        count(ChildExplorer::with_target(&aGraph, CompoundId::start(), NodeKind::FACE).unwrap());
    assert_eq!(aCount, 12); // 6 + 6 faces
}

// --- Explorer convenience tests ---

#[test]
fn BRepGraph_ChildExplorerTest__NodeOf_Kind_Face() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let anExp = ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::EDGE).unwrap();
    assert!(anExp.more());

    let aFace = anExp.node_of(NodeKind::FACE);
    assert!(aFace.is_valid());
    assert_eq!(aFace.kind, NodeKind::FACE);
}

// --- Depth guard and re-initialization tests ---

#[test]
fn BRepGraph_ChildExplorerTest__DeepCompound_NoStackOverflow() {
    // Build a deeply nested compound: Compound(Compound(Compound(...(Box)...))).
    let mut aInner: Shape = make_box(1.0, 1.0, 1.0).into();
    const THE_NESTING_DEPTH: i32 = 100;
    for _ in 0..THE_NESTING_DEPTH {
        aInner = compoundOf(&[aInner]);
    }
    let aGraph = graphOf(&aInner);
    assert!(!aGraph.is_empty());

    // Should not crash (stack overflow) and should find the box's faces.
    let aCount =
        count(ChildExplorer::with_target(&aGraph, CompoundId::start(), NodeKind::FACE).unwrap());
    assert_eq!(aCount, 6);
}

#[test]
fn BRepGraph_ChildExplorerTest__Recreate_ResetAndReexplore() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aFaceCount =
        count(ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::FACE).unwrap());
    assert_eq!(aFaceCount, 6);

    // Recreate targeting edges.
    let aEdgeCount =
        count(ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::EDGE).unwrap());
    assert_eq!(aEdgeCount, 24);
}

// ============ Regression tests: CoEdge/Occurrence target reachability ============

#[test]
fn BRepGraph_ChildExplorerTest__CoEdgeTarget_Reachable() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    // CoEdge target from Solid must find all coedges (24 edge occurrences = 24 coedges).
    let mut aCount = 0;
    let mut anExp =
        ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::CO_EDGE).unwrap();
    while anExp.more() {
        assert_eq!(anExp.current().def_id.kind, NodeKind::CO_EDGE);
        aCount += 1;
        anExp.next().unwrap();
    }
    assert_eq!(aCount, 24);
}

#[test]
fn BRepGraph_ChildExplorerTest__CoEdgeTarget_FromFace_Count4() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut aCount = 0;
    let mut anExp =
        ChildExplorer::with_target(&aGraph, FaceId::start(), NodeKind::CO_EDGE).unwrap();
    while anExp.more() {
        assert_eq!(anExp.current().def_id.kind, NodeKind::CO_EDGE);
        aCount += 1;
        anExp.next().unwrap();
    }
    assert_eq!(aCount, 4);
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_ShellFaces_CountAndOrder() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aShellId = ShellId::new(0);
    let aFaceRefIds = aGraph.refs().faces().ids_of(aShellId);

    let mut anExpectedFaceIds: Vec<u32> = Vec::new();
    for aFaceRefId in aFaceRefIds {
        let aRef = aGraph.refs().faces().entry(*aFaceRefId);
        if !aFaceRefId.is_removed(&aGraph) {
            anExpectedFaceIds.push(aRef.child_face_id.index);
        }
    }

    let mut anActualFaceIds: Vec<u32> = Vec::new();
    let mut anIt = makeDirectChildExplorer(&aGraph, aShellId, NodeKind::FACE);
    while anIt.more() {
        assert_eq!(anIt.current().def_id.kind, NodeKind::FACE);
        anActualFaceIds.push(anIt.current().def_id.index);
        anIt.next().unwrap();
    }

    assert_eq!(anActualFaceIds.len(), anExpectedFaceIds.len());
    for i in 0..anExpectedFaceIds.len() {
        assert_eq!(anActualFaceIds[i], anExpectedFaceIds[i]);
    }
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_ShellFaces_ExposeParentAndRef() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aShellId = ShellId::new(0);
    let mut anOrdinal = 0;
    let mut anIt = makeDirectChildExplorer(&aGraph, aShellId, NodeKind::FACE);
    while anIt.more() {
        let aFaceRefId = aGraph.refs().faces().ids_of(aShellId)[anOrdinal];
        assert_eq!(anIt.current_parent(), NodeId::from(aShellId));
        assert_eq!(anIt.current_link_kind(), LinkKind::Reference);
        assert_eq!(anIt.current_ref(), RefId::from(aFaceRefId));
        anIt.next().unwrap();
        anOrdinal += 1;
    }
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_ProductShapeRoot_ViaOccurrenceRef() {
    let mut aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aProductId = aGraph
        .editor()
        .products()
        .add(SolidId::start().into(), &Location::default());
    aGraph
        .editor()
        .products()
        .append_document_root(aProductId)
        .unwrap();
    assert!(aProductId.is_valid());

    // In the new model, products reference children through OccurrenceRefIds.
    // The shape root is reached via an occurrence whose ChildNodeId is the topology node.
    let mut anIt =
        ChildExplorer::with_mode(&aGraph, aProductId.into(), TraversalMode::DirectChildren)
            .unwrap();
    assert!(anIt.more());
    // The first child should be an occurrence node (not the solid directly).
    assert_eq!(anIt.current().def_id.kind, NodeKind::OCCURRENCE);
    assert_eq!(anIt.current_parent(), NodeId::from(aProductId));
    // The occurrence is linked via a reference.
    assert!(anIt.current_ref().is_valid());

    anIt.next().unwrap();
    assert!(!anIt.more());
}

#[test]
fn BRepGraph_ChildExplorerTest__RootEqualsTarget_LinkKindNone() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let anExp = ChildExplorer::with_target(&aGraph, FaceId::start(), NodeKind::FACE).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current_link_kind(), LinkKind::None);
    assert!(!anExp.current_parent().is_valid());
    assert!(!anExp.current_ref().is_valid());
}

// The part product of the box solid and an assembly product, both document roots.
fn partAndAssembly(aGraph: &mut BRepGraph) -> (ProductId, ProductId) {
    let aPart = aGraph
        .editor()
        .products()
        .add(SolidId::start().into(), &Location::default());
    aGraph
        .editor()
        .products()
        .append_document_root(aPart)
        .unwrap();
    let anAssembly = aGraph.editor().products().add_empty();
    aGraph
        .editor()
        .products()
        .append_document_root(anAssembly)
        .unwrap();
    assert!(aPart.is_valid());
    assert!(anAssembly.is_valid());
    (aPart, anAssembly)
}

fn appendOccurrence(
    aGraph: &mut BRepGraph,
    aParent: ProductId,
    aChild: ProductId,
    aLoc: &Location,
) -> OccurrenceId {
    aGraph
        .editor()
        .products()
        .append(aParent, aChild, aLoc, OccurrenceId::default())
        .unwrap()
        .0
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_ProductOccurrences_ExposeOccurrenceRefs() {
    let mut aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let (aPart, anAssembly) = partAndAssembly(&mut aGraph);

    let anOcc0 = appendOccurrence(&mut aGraph, anAssembly, aPart, &Location::default());
    let anOcc1 = appendOccurrence(&mut aGraph, anAssembly, aPart, &Location::default());
    assert!(anOcc0.is_valid());
    assert!(anOcc1.is_valid());

    let anOccurrenceRefs = aGraph.refs().occurrences().ids_of(anAssembly);
    assert_eq!(anOccurrenceRefs.len(), 2);

    let mut anOrdinal = 0;
    let mut anIt =
        ChildExplorer::with_mode(&aGraph, anAssembly.into(), TraversalMode::DirectChildren)
            .unwrap();
    while anIt.more() {
        assert_eq!(anIt.current_parent(), NodeId::from(anAssembly));
        assert_eq!(anIt.current_ref(), RefId::from(anOccurrenceRefs[anOrdinal]));
        anIt.next().unwrap();
        anOrdinal += 1;
    }
    assert_eq!(anOrdinal, 2);
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_RemovedFaceRef_IsSkipped() {
    let mut aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aShellId = ShellId::new(0);
    let aFaceRefIds = aGraph.refs().faces().ids_of(aShellId);
    assert!(!aFaceRefIds.is_empty());
    let aNbFaceRefsBefore = aFaceRefIds.len();

    let aRemovedRef = aFaceRefIds[0];
    let aRemovedFaceId = aGraph.refs().faces().entry(aRemovedRef).child_face_id;

    aGraph
        .editor()
        .generic()
        .remove_ref(aRemovedRef.into())
        .unwrap();

    let mut aCount = 0;
    let mut anIt = makeDirectChildExplorer(&aGraph, aShellId, NodeKind::FACE);
    while anIt.more() {
        assert_ne!(anIt.current().def_id, NodeId::from(aRemovedFaceId));
        aCount += 1;
        anIt.next().unwrap();
    }

    assert_eq!(aCount, aNbFaceRefsBefore - 1);
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_WireChildren_AreCoEdges() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aFaceId = FaceId::new(0);
    let aWireRefs = aGraph.refs().wires().ids_of(aFaceId);
    assert!(!aWireRefs.is_empty());

    let aWireId = aGraph.refs().wires().entry(aWireRefs[0]).child_wire_id;
    let aCoEdges = &aGraph.topo().wires().relations(aWireId).co_edge_ids;

    // Wire's direct children are CoEdges (no 1:1 collapse).
    let mut aCount = 0;
    let mut anIt = makeDirectChildExplorer(&aGraph, aWireId, NodeKind::CO_EDGE);
    while anIt.more() {
        assert_eq!(anIt.current().def_id.kind, NodeKind::CO_EDGE);
        aCount += 1;
        anIt.next().unwrap();
    }

    assert_eq!(aCount, aCoEdges.len());
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_CompoundChildren_Basic() {
    let aComp = compoundOf(&[
        make_box(10.0, 10.0, 10.0).into(),
        make_box(20.0, 20.0, 20.0).into(),
    ]);

    let aGraph = graphOf(&aComp);
    assert!(!aGraph.is_empty());

    let mut aCount = 0;
    let mut anIt = makeDirectChildExplorer(&aGraph, CompoundId::start(), NodeKind::SOLID);
    while anIt.more() {
        assert!(NodeKind::is_topology_kind(anIt.current().def_id.kind));
        aCount += 1;
        anIt.next().unwrap();
    }

    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_ChainedTraversal_ParityWithRecursiveTraversal() {
    let mut aBox: Shape = make_box(10.0, 20.0, 30.0).into();
    aBox.move_by(&translation(100.0, 0.0, 0.0), true).unwrap();

    let aComp = compoundOf(&[aBox]);

    let aGraph = graphOf(&aComp);
    assert!(!aGraph.is_empty());

    let mut aExpectedLoc: HashMap<u32, Location> = HashMap::new();
    let mut aExpectedOri: HashMap<u32, Orientation> = HashMap::new();
    let mut anExp =
        ChildExplorer::with_target(&aGraph, CompoundId::start(), NodeKind::FACE).unwrap();
    while anExp.more() {
        let aFaceIdx = anExp.current().def_id.index;
        if let Entry::Vacant(aSlot) = aExpectedLoc.entry(aFaceIdx) {
            aSlot.insert(anExp.current().location);
            aExpectedOri.insert(aFaceIdx, anExp.current().orientation);
        }
        anExp.next().unwrap();
    }

    let mut aVisited = 0;
    let mut aSolidIt = makeDirectChildExplorer(&aGraph, CompoundId::start(), NodeKind::SOLID);
    while aSolidIt.more() {
        let aSolidUsage: NodeInstance = aSolidIt.current();
        let mut aShellIt = makeDirectChildExplorerAt(
            &aGraph,
            aSolidUsage.def_id,
            NodeKind::SHELL,
            &aSolidUsage.location,
            aSolidUsage.orientation,
        );
        while aShellIt.more() {
            let aShellUsage = aShellIt.current();
            let mut aFaceIt = makeDirectChildExplorerAt(
                &aGraph,
                aShellUsage.def_id,
                NodeKind::FACE,
                &aShellUsage.location,
                aShellUsage.orientation,
            );
            while aFaceIt.more() {
                let aFaceUsage = aFaceIt.current();
                assert_eq!(aFaceUsage.def_id.kind, NodeKind::FACE);
                let aFaceIdx = aFaceUsage.def_id.index;
                assert!(aExpectedLoc.contains_key(&aFaceIdx));
                assert!(aFaceUsage.location.is_equal(&aExpectedLoc[&aFaceIdx]));
                assert_eq!(aFaceUsage.orientation, aExpectedOri[&aFaceIdx]);
                aVisited += 1;
                aFaceIt.next().unwrap();
            }
            aShellIt.next().unwrap();
        }
        aSolidIt.next().unwrap();
    }

    assert_eq!(aVisited, aExpectedLoc.len());
}

#[test]
fn BRepGraph_ChildExplorerTest__Recursive_SharedProduct_ChildrenHaveDistinctContexts() {
    let mut aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let (aPart, anAssembly) = partAndAssembly(&mut aGraph);

    let aT1 = translation(10.0, 0.0, 0.0);
    let aT2 = translation(20.0, 0.0, 0.0);

    let anOcc1 = appendOccurrence(&mut aGraph, anAssembly, aPart, &aT1);
    let anOcc2 = appendOccurrence(&mut aGraph, anAssembly, aPart, &aT2);
    assert!(anOcc1.is_valid());
    assert!(anOcc2.is_valid());

    // Recursive traversal through Assembly->Occurrence->Product(part)->Solid.
    let mut aCount = 0;
    let mut aLoc1 = Location::default();
    let mut aLoc2 = Location::default();
    let mut anIt = ChildExplorer::with_target(&aGraph, anAssembly, NodeKind::SOLID).unwrap();
    while anIt.more() {
        assert_eq!(anIt.current().def_id, NodeId::from(SolidId::start()));
        if aCount == 0 {
            aLoc1 = anIt.current().location;
        } else if aCount == 1 {
            aLoc2 = anIt.current().location;
        }
        aCount += 1;
        anIt.next().unwrap();
    }

    assert_eq!(aCount, 2);
    assert!(!aLoc1.is_equal(&aLoc2));
}

#[test]
fn BRepGraph_ChildExplorerTest__Recursive_ProductOccurrenceChain_ReachesNestedOccurrences() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();

    let aPart = aGraph.editor().products().add_empty();
    let aSubAssembly = aGraph.editor().products().add_empty();
    let aTopAssembly = aGraph.editor().products().add_empty();
    assert!(aPart.is_valid());
    assert!(aSubAssembly.is_valid());
    assert!(aTopAssembly.is_valid());

    let aSubOccurrence = appendOccurrence(
        &mut aGraph,
        aTopAssembly,
        aSubAssembly,
        &Location::default(),
    );
    let aPartOccurrence = appendOccurrence(&mut aGraph, aSubAssembly, aPart, &Location::default());
    assert!(aSubOccurrence.is_valid());
    assert!(aPartOccurrence.is_valid());

    let mut hasSubOccurrence = false;
    let mut hasPartOccurrence = false;
    let mut aCount = 0;
    let mut anIt = ChildExplorer::with_target(&aGraph, aTopAssembly, NodeKind::OCCURRENCE).unwrap();
    while anIt.more() {
        hasSubOccurrence |= anIt.current().def_id == NodeId::from(aSubOccurrence);
        hasPartOccurrence |= anIt.current().def_id == NodeId::from(aPartOccurrence);
        aCount += 1;
        anIt.next().unwrap();
    }

    assert_eq!(aCount, 2);
    assert!(hasSubOccurrence);
    assert!(hasPartOccurrence);
}

#[test]
fn BRepGraph_ChildExplorerTest__Recursive_ProductPartRootContext_ComposedWithOccurrence() {
    let mut aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let aPart = aGraph
        .editor()
        .products()
        .add(SolidId::start().into(), &Location::default());
    aGraph
        .editor()
        .products()
        .append_document_root(aPart)
        .unwrap();
    assert!(aPart.is_valid());
    let anAssembly = aGraph.editor().products().add_empty();
    aGraph
        .editor()
        .products()
        .append_document_root(anAssembly)
        .unwrap();
    assert!(anAssembly.is_valid());

    let aOccTrsf = translation(10.0, 0.0, 0.0);
    let anOcc = appendOccurrence(&mut aGraph, anAssembly, aPart, &aOccTrsf);
    assert!(anOcc.is_valid());

    let aRootTrsf = translation(0.0, 20.0, 0.0);
    // Set the root location on the topology-root occurrence ref of the part product.
    {
        let aPartRelations = aGraph
            .topo()
            .products()
            .relations(aPart)
            .occurrence_ref_ids
            .clone();
        for aRefId in aPartRelations {
            let aOccRef = aGraph.refs().occurrences().entry(aRefId);
            let anOccDef = aGraph
                .topo()
                .occurrences()
                .definition(aOccRef.child_occurrence_id);
            if NodeKind::is_topology_kind(anOccDef.child_node_id.kind) {
                let mut aMutRef = aGraph.editor().occurrences().mutate_ref(aRefId).unwrap();
                OccurrenceOps::set_ref_local_location_guarded(&mut aMutRef, &aRootTrsf);
                break;
            }
        }
    }

    // Recursive traversal through Assembly->Occurrence->Product(part)->Solid.
    let mut anIt = ChildExplorer::with_target(&aGraph, anAssembly, NodeKind::SOLID).unwrap();
    assert!(anIt.more());
    assert_eq!(anIt.current().def_id, NodeId::from(SolidId::start()));

    let aUsage = anIt.current();
    let anExpectedLoc = (&aOccTrsf * &aRootTrsf).unwrap();
    let anActualTrsf = aUsage.location.transformation();
    let anExpectedTrsf = anExpectedLoc.transformation();
    let aTolerance = precision::confusion();
    assert!(
        (anActualTrsf.translation_part().x() - anExpectedTrsf.translation_part().x()).abs()
            <= aTolerance
    );
    assert!(
        (anActualTrsf.translation_part().y() - anExpectedTrsf.translation_part().y()).abs()
            <= aTolerance
    );
    assert!(
        (anActualTrsf.translation_part().z() - anExpectedTrsf.translation_part().z()).abs()
            <= aTolerance
    );

    anIt.next().unwrap();
    assert!(!anIt.more());
}

#[test]
fn BRepGraph_ChildExplorerTest__DirectChildren_HighFanout_DirectChildrenComplete() {
    const THE_NB_CHILDREN: i32 = 80;
    let aBoxes: Vec<Shape> = (0..THE_NB_CHILDREN)
        .map(|i| make_box(1.0 + f64::from(i), 2.0, 3.0).into())
        .collect();
    let aComp = compoundOf(&aBoxes);

    let aGraph = graphOf(&aComp);
    assert!(!aGraph.is_empty());

    let aCount = count(makeDirectChildExplorer(
        &aGraph,
        CompoundId::start(),
        NodeKind::SOLID,
    ));

    assert_eq!(aCount, THE_NB_CHILDREN);
}

#[test]
fn BRepGraph_ChildExplorerTest__HighFanout_CompletesAllChildren() {
    // Build a compound with many children to verify no premature traversal termination.
    const THE_NB_CHILDREN: i32 = 500;
    let aBoxes: Vec<Shape> = (0..THE_NB_CHILDREN)
        .map(|_| make_box(1.0, 1.0, 1.0).into())
        .collect();
    let aComp = compoundOf(&aBoxes);

    let aGraph = graphOf(&aComp);
    assert!(!aGraph.is_empty());

    let aFaceCount =
        count(ChildExplorer::with_target(&aGraph, CompoundId::start(), NodeKind::FACE).unwrap());
    // Each box has 6 faces.
    assert_eq!(aFaceCount, THE_NB_CHILDREN * 6);
}

#[test]
fn BRepGraph_ChildExplorerTest__StructuredBindings_NodeInstance() {
    let mut aBox: Shape = make_box(10.0, 20.0, 30.0).into();
    aBox.move_by(&translation(100.0, 0.0, 0.0), true).unwrap();

    let aComp = compoundOf(&[aBox]);

    let aGraph = graphOf(&aComp);
    assert!(!aGraph.is_empty());

    let mut aCount = 0;
    let mut anExp =
        ChildExplorer::with_target(&aGraph, CompoundId::start(), NodeKind::FACE).unwrap();
    while anExp.more() {
        let NodeInstance {
            def_id: aNodeId,
            location: aLoc,
            orientation: anOri,
        } = anExp.current();
        assert_eq!(aNodeId.kind, NodeKind::FACE);
        assert!(!aLoc.is_identity());
        assert!(anOri == Orientation::Forward || anOri == Orientation::Reversed);
        aCount += 1;
        anExp.next().unwrap();
    }
    assert_eq!(aCount, 6);
}

#[test]
fn BRepGraph_ChildExplorerTest__RangeFor_NodeInstance() {
    let aGraph = boxGraph();
    assert!(!aGraph.is_empty());

    let mut aCount = 0;
    for aUsage in ChildExplorer::with_target(&aGraph, SolidId::start(), NodeKind::FACE).unwrap() {
        assert_eq!(aUsage.unwrap().def_id.kind, NodeKind::FACE);
        aCount += 1;
    }
    assert_eq!(aCount, 6);
}
