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

//! Direct translation of BRepGraph_Sharing_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported); the `BRepGraph_SharingTest` fixture is
//! [`SetUp`], and `BRepGraph_TestTools` is `support::ref_test_tools`.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::iterator::{
    EdgeIterator, FaceIterator, VertexIterator, WireIterator,
};
use rustycad_tkbrep::brep_graph::tool;
use rustycad_tkbrep::brep_graph::validate::{self, Options, Severity};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::make_box;
use support::ref_test_tools as BRepGraph_TestTools;

// =========================================================================
// Fixture: builds a box 10x20x30 and its BRepGraph
// =========================================================================

/// `BRepGraph_SharingTest::SetUp`: the graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let aBox: Shape = make_box(10.0, 20.0, 30.0).into();
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph.shapes_mut().add(&aBox).unwrap();
    myGraph
}

/// `BRep_Builder::MakeCompound` then `Add` of each shape.
fn compoundOf(theShapes: &[&Shape]) -> Compound {
    let aBuilder = BRepBuilder::default();
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    for aShape in theShapes {
        aBuilder.add(&mut aCompound, aShape).unwrap();
    }
    aCompound
}

fn graphOf(theShape: &Shape) -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph.shapes_mut().add(theShape).unwrap();
    assert!(!aGraph.is_empty());
    aGraph
}

fn translated(theShape: &Shape, theX: f64, theY: f64, theZ: f64) -> Shape {
    let aTrsf = Transform3::translation_vector(Vector3::new(theX, theY, theZ));
    theShape.moved(&Location::new(aTrsf), false).unwrap()
}

// =========================================================================
// Edge sharing via incidence: each box edge appears in 2 wires (faces)
// =========================================================================

#[test]
fn BRepGraph_SharingTest__EdgeDef_EachSharedByTwoFaces() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    assert_eq!(myGraph.topo().edges().nb(), 12);
    // In a box, each edge is shared by exactly 2 faces.
    let mut anEdgeIt = EdgeIterator::new(&myGraph);
    while anEdgeIt.more() {
        let anEdgeId = anEdgeIt.current_id();
        let aFaceCount = myGraph.topo().edges().nb_faces(anEdgeId);
        assert_eq!(
            aFaceCount, 2,
            "Edge def {} expected to be shared by 2 faces, got {aFaceCount}",
            anEdgeId.index
        );
        anEdgeIt.next();
    }
}

#[test]
fn BRepGraph_SharingTest__FaceDef_EachHasValidSurface() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    assert_eq!(myGraph.topo().faces().nb(), 6);
    let mut aFaceIt = FaceIterator::new(&myGraph);
    while aFaceIt.more() {
        let aFaceId = aFaceIt.current_id();
        let aDef = aFaceIt.current();
        assert!(
            aDef.surface_rep_id.is_valid(),
            "Face def {} has no surface rep",
            aFaceId.index
        );
        aFaceIt.next();
    }
}

#[test]
fn BRepGraph_SharingTest__SolidDef_HasOneShellRef() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    assert_eq!(myGraph.topo().solids().nb(), 1);
    assert_eq!(
        BRepGraph_TestTools::count_shell_refs_of_solid(&myGraph, SolidId::start()),
        1
    );
}

#[test]
fn BRepGraph_SharingTest__ShellDef_HasSixFaceRefs() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    assert_eq!(myGraph.topo().shells().nb(), 1);
    assert_eq!(
        BRepGraph_TestTools::count_face_refs_of_shell(&myGraph, ShellId::start()),
        6
    );
}

// =========================================================================
// Containment hierarchy
// =========================================================================

#[test]
fn BRepGraph_SharingTest__SolidDef_ContainsOneShellRef() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    assert_eq!(myGraph.topo().solids().nb(), 1);
    assert_eq!(
        BRepGraph_TestTools::count_shell_refs_of_solid(&myGraph, SolidId::start()),
        1
    );
}

#[test]
fn BRepGraph_SharingTest__ShellDef_ContainsSixFaceRefs() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    assert_eq!(myGraph.topo().shells().nb(), 1);
    assert_eq!(
        BRepGraph_TestTools::count_face_refs_of_shell(&myGraph, ShellId::start()),
        6
    );
}

#[test]
fn BRepGraph_SharingTest__FaceDef_OuterWireIdx_Valid() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    let mut aFaceIt = FaceIterator::new(&myGraph);
    while aFaceIt.more() {
        let aFaceId = aFaceIt.current_id();
        let anOuterWire = BRepGraph_TestTools::outer_wire_of_face(&myGraph, aFaceId).unwrap();
        assert!(
            anOuterWire.is_valid(),
            "Face def {} has no outer wire",
            aFaceId.index
        );
        aFaceIt.next();
    }
}

#[test]
fn BRepGraph_SharingTest__WireDef_CoEdgesCount_FourPerBoxFace() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    let mut aWireIt = WireIterator::new(&myGraph);
    while aWireIt.more() {
        let aWireId = aWireIt.current_id();
        let aNbCoEdges = BRepGraph_TestTools::count_co_edges_of_wire(&myGraph, aWireId);
        assert!(aNbCoEdges > 0, "Wire def {} has no coedges", aWireId.index);
        // Box face wires have 4 edges
        assert_eq!(
            aNbCoEdges, 4,
            "Wire def {} expected 4 coedges for box face",
            aWireId.index
        );
        aWireIt.next();
    }
}

#[test]
fn BRepGraph_SharingTest__EdgeDef_VertexDefs_BothValid() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    let mut anEdgeIt = EdgeIterator::new(&myGraph);
    while anEdgeIt.more() {
        let anEdgeId = anEdgeIt.current_id();
        assert!(
            tool::edge::start_vertex_id(&myGraph, anEdgeId).is_valid(),
            "Edge def {} has invalid start vertex def",
            anEdgeId.index
        );
        assert!(
            tool::edge::end_vertex_id(&myGraph, anEdgeId).is_valid(),
            "Edge def {} has invalid end vertex def",
            anEdgeId.index
        );
        anEdgeIt.next();
    }
}

// =========================================================================
// Orientation via incidence refs
// =========================================================================

#[test]
fn BRepGraph_SharingTest__SharedEdge_IncidenceRefs_DifferentOrientation() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    // In a box, shared edges between adjacent faces have coedges on
    // different face definitions. Check that at least some edges have
    // coedges referencing more than one face.
    let mut aMultiFaceEdgeCount = 0;
    let mut anEdgeIt = EdgeIterator::new(&myGraph);
    while anEdgeIt.more() {
        let anEdgeId = anEdgeIt.current_id();
        let aCoEdgeIdxs = myGraph.topo().edges().co_edges(anEdgeId);
        anEdgeIt.next();
        if aCoEdgeIdxs.len() < 2 {
            continue;
        }
        // Check if coedges reference different faces.
        let aFace0 = myGraph.topo().co_edges().definition(aCoEdgeIdxs[0]).face_id;
        let aDiffers = aCoEdgeIdxs[1..]
            .iter()
            .any(|&aCoEdge| myGraph.topo().co_edges().definition(aCoEdge).face_id != aFace0);
        if aDiffers {
            aMultiFaceEdgeCount += 1;
        }
    }
    assert!(
        aMultiFaceEdgeCount > 0,
        "Expected at least some shared edges with coedges on different faces"
    );
}

#[test]
fn BRepGraph_SharingTest__NonClosedEdge_StartEnd_Different() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    let mut anEdgeIt = EdgeIterator::new(&myGraph);
    while anEdgeIt.more() {
        let anEdgeId = anEdgeIt.current_id();
        anEdgeIt.next();
        if tool::edge::degenerated(&myGraph, anEdgeId) {
            continue;
        }
        // Box edges are not closed, so start and end vertex defs must differ
        let aStartVtx = tool::edge::start_vertex_id(&myGraph, anEdgeId);
        let anEndVtx = tool::edge::end_vertex_id(&myGraph, anEdgeId);
        assert_ne!(
            aStartVtx, anEndVtx,
            "Non-degenerate edge def {} has identical start and end vertex def ids",
            anEdgeId.index
        );
    }
}

#[test]
fn BRepGraph_SharingTest__VertexDef_Points_MatchExpectedBoxCorners() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
    // For a simple 10x20x30 box, all 8 vertex points should be valid.
    assert_eq!(myGraph.topo().vertices().nb(), 8);
    let aTol = precision::confusion();
    let mut aVertexIt = VertexIterator::new(&myGraph);
    while aVertexIt.more() {
        let aDef = aVertexIt.current();
        // Verify coordinates are within the box bounds.
        assert!(aDef.point.x() >= -aTol);
        assert!(aDef.point.x() <= 10.0 + aTol);
        assert!(aDef.point.y() >= -aTol);
        assert!(aDef.point.y() <= 20.0 + aTol);
        assert!(aDef.point.z() >= -aTol);
        assert!(aDef.point.z() <= 30.0 + aTol);
        aVertexIt.next();
    }
}

// =========================================================================
// TShape sharing in compounds (standalone tests)
// =========================================================================

#[test]
fn BRepGraph_SharingTest__CompoundTwoIdenticalBoxes() {
    let _myGraph = SetUp();
    let aBox: Shape = make_box(10.0, 20.0, 30.0).into();

    let aCompound: Shape = compoundOf(&[&aBox, &aBox]).into();

    let aGraph = graphOf(&aCompound);

    // Same TShape added twice to compound: definition is shared (1 solid def),
    // compound has 2 ChildRefs pointing to the same solid index.
    assert_eq!(aGraph.topo().solids().nb(), 1);

    // All defs are shared (same TShape), counts stay at single-box levels.
    assert_eq!(aGraph.topo().faces().nb(), 6);
    assert_eq!(aGraph.topo().edges().nb(), 12);
    assert_eq!(aGraph.topo().vertices().nb(), 8);

    // Compound has 2 child references to the same solid.
    let aChildRefs = BRepGraph_TestTools::child_refs_of_parent(&aGraph, CompoundId::start().into());
    assert_eq!(aChildRefs.len(), 2);
    assert_ne!(
        aChildRefs[0], aChildRefs[1],
        "Repeated child occurrences in one parent must keep separate slots"
    );
    assert_eq!(
        aGraph
            .refs()
            .children()
            .entry(aChildRefs[0])
            .child_node_id
            .index,
        aGraph
            .refs()
            .children()
            .entry(aChildRefs[1])
            .child_node_id
            .index
    );
}

#[test]
fn BRepGraph_SharingTest__SameReferenceRepresentationAcrossDifferentParents_UsesDistinctChildRefs()
{
    let _myGraph = SetUp();
    let aBox: Shape = make_box(10.0, 20.0, 30.0).into();

    let aParentA: Shape = compoundOf(&[&aBox]).into();
    let aParentB: Shape = compoundOf(&[&aBox]).into();
    let aRoot: Shape = compoundOf(&[&aParentA, &aParentB]).into();

    let mut aGraph = BRepGraph::new();
    let _aBuildRes = aGraph.shapes_mut().add(&aRoot).unwrap();
    assert!(!aGraph.is_empty());

    let aParentANode = aGraph.shapes().find_node(&aParentA);
    let aParentBNode = aGraph.shapes().find_node(&aParentB);
    assert_eq!(aParentANode.kind, NodeKind::COMPOUND);
    assert_eq!(aParentBNode.kind, NodeKind::COMPOUND);

    let aParentARefs = BRepGraph_TestTools::child_refs_of_parent(&aGraph, aParentANode);
    let aParentBRefs = BRepGraph_TestTools::child_refs_of_parent(&aGraph, aParentBNode);
    assert_eq!(aParentARefs.len(), 1);
    assert_eq!(aParentBRefs.len(), 1);
    assert_ne!(
        aParentARefs[0], aParentBRefs[0],
        "Each parent usage must have its own child-reference entry"
    );
    assert_eq!(
        aGraph
            .refs()
            .children()
            .entry(aParentARefs[0])
            .child_node_id,
        aGraph
            .refs()
            .children()
            .entry(aParentBRefs[0])
            .child_node_id
    );

    let anAuditResult = validate::perform_with_options(&aGraph, &Options::audit());
    assert_eq!(anAuditResult.nb_issues(Severity::Error), 0);
}

#[test]
fn BRepGraph_SharingTest__CompoundTwoDistinctBoxes() {
    let _myGraph = SetUp();
    let aBox1: Shape = make_box(10.0, 20.0, 30.0).into();
    let aBox2: Shape = make_box(5.0, 15.0, 25.0).into();

    let aCompound: Shape = compoundOf(&[&aBox1, &aBox2]).into();

    let aGraph = graphOf(&aCompound);

    // Two different TShapes: no sharing, definitions are independent
    assert_eq!(aGraph.topo().solids().nb(), 2);
    assert_eq!(aGraph.topo().faces().nb(), 12);
    assert_eq!(aGraph.topo().edges().nb(), 24);
    assert_eq!(aGraph.topo().vertices().nb(), 16);
}

/// The two `CompoundWithLocation` and `TranslatedCopy` cases: a box and its moved copy
/// in one compound share every definition, the moved usage keeping its placement on
/// the second child reference.
fn expectSharedDefsWithLocatedUsage(theX: f64, theY: f64, theZ: f64) {
    let _myGraph = SetUp();
    let aBox: Shape = make_box(10.0, 20.0, 30.0).into();
    let aMovedBox = translated(&aBox, theX, theY, theZ);

    let aCompound: Shape = compoundOf(&[&aBox, &aMovedBox]).into();

    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let aBuildRes = aGraph.shapes_mut().add(&aCompound).unwrap();
    assert!(aBuildRes.is_ok());
    assert!(!aGraph.is_empty());

    assert_eq!(aGraph.topo().solids().nb(), 1);
    assert_eq!(aGraph.topo().faces().nb(), 6);
    assert_eq!(aGraph.topo().edges().nb(), 12);
    assert_eq!(aGraph.topo().vertices().nb(), 8);
    let aRoot = CompoundId::from_node_id(aBuildRes.topology_root);
    let aChildRefs = &aGraph.topo().compounds().relations(aRoot).child_ref_ids;
    assert_eq!(aChildRefs.len(), 2);
    let aLocation = aGraph.refs().generic().local_location(aChildRefs[1].into());
    assert!(!aLocation.is_identity());
}

#[test]
fn BRepGraph_SharingTest__CompoundWithLocation_MoreUsagesThanDefs() {
    // Same TShape with different compound usages: topology definitions are shared
    // and usage placement is stored on ChildRef.
    expectSharedDefsWithLocatedUsage(100.0, 0.0, 0.0);
}

#[test]
fn BRepGraph_SharingTest__TranslatedCopy_SameTShape_SharedDefs() {
    // Moved() preserves TShape, so the definition stays shared while the moved
    // usage is represented by a located compound child reference.
    expectSharedDefsWithLocatedUsage(50.0, 50.0, 50.0);
}
