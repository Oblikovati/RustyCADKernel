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

//! Direct translation of BRepGraph_Reverse_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). The `CheckCoEdgeOrder(wire, coedges)`
//! overload is `check_wire_co_edge_order`.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::editor::{CoEdgeOrderStatus, ReplaceEdgeStatus};
use rustycad_tkbrep::brep_graph::layer_topo_supplement::LayerTopoSupplement;
use rustycad_tkbrep::brep_graph::parent_explorer::{ParentExplorer, TraversalMode};
use rustycad_tkbrep::brep_graph::validate::{self, Options};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Shell};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use support::make_box;

fn buildBoxGraph() -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    let aRegisteredLayer = aGraph
        .layer_registry()
        .ensure::<LayerTopoSupplement>()
        .unwrap();
    assert!(!aRegisteredLayer.is_null());
    let _aRes = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    aGraph
}

fn addSegment(theGraph: &mut BRepGraph, theStart: VertexId, theEnd: VertexId) -> EdgeId {
    let aNoCurve: Handle<dyn GeomCurve> = Handle::default();
    theGraph
        .editor()
        .edges()
        .add(theStart, theEnd, &aNoCurve, 0.0, 1.0, 1.0e-7)
}

fn makeReversedFaceShell() -> Shape {
    let aBox: Shape = make_box(10.0, 20.0, 30.0).into();

    let aBuilder = BRepBuilder::default();
    let mut aShell = Shell::default();
    aBuilder.make_shell(&mut aShell);

    let mut anExp = Explorer::new(&aBox, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while anExp.more() {
        let mut aFace = anExp.current().clone();
        aFace.set_orientation(Orientation::Reversed);
        aBuilder.add(&mut aShell, &aFace).unwrap();
        anExp.next().unwrap();
    }

    aShell.into()
}

fn addVertex(aGraph: &mut BRepGraph, theX: f64) -> VertexId {
    aGraph
        .editor()
        .vertices()
        .add(Point3::new(theX, 0.0, 0.0), 1.0e-7)
}

fn addCoEdge(aGraph: &mut BRepGraph, theEdge: EdgeId) -> CoEdgeId {
    aGraph
        .editor()
        .co_edges()
        .add(theEdge, ParityOrientation::FORWARD)
}

fn isAuditValid(aGraph: &BRepGraph) -> bool {
    validate::perform_with_options(aGraph, &Options::audit()).is_valid()
}

fn isBoundaryValid(aGraph: &mut BRepGraph) -> bool {
    aGraph.editor().validate_mutation_boundary(None)
}

/// The graph built from the reversed-face shell (`Shapes().Add` must succeed).
fn reversedShellGraph() -> BRepGraph {
    let aShell = makeReversedFaceShell();
    let mut aGraph = BRepGraph::new();
    let aResult = aGraph.shapes_mut().add(&aShell).unwrap();
    assert!(aResult.is_ok());
    aGraph
}

#[test]
fn BRepGraph_ReverseTest__Wire_OrderAndOrientationFlipped() {
    let mut aGraph = buildBoxGraph();
    let aWire = WireId::new(0);
    assert!(aWire.is_valid_for(aGraph.topo().wires().nb()));

    let aCoEdgesBefore = aGraph.topo().wires().relations(aWire).co_edge_ids.clone();
    let mut aOrderBefore: Vec<CoEdgeId> = Vec::new();
    let mut aOriBefore: Vec<Orientation> = Vec::new();
    for aCoEdgeId in aCoEdgesBefore {
        aOrderBefore.push(aCoEdgeId);
        let aCoEdge = aGraph.topo().co_edges().definition(aCoEdgeId);
        aOriBefore.push(aCoEdge.orientation.orientation());
    }
    assert!(aOrderBefore.len() > 1);

    aGraph.editor().wires().reverse(aWire);

    let aCoEdgesAfter = &aGraph.topo().wires().relations(aWire).co_edge_ids;
    assert_eq!(aOrderBefore.len(), aCoEdgesAfter.len());
    for (i, &aCoEdgeId) in aCoEdgesAfter.iter().enumerate() {
        let aMirror = aOrderBefore.len() - 1 - i;
        assert_eq!(
            aCoEdgeId, aOrderBefore[aMirror],
            "CoEdgeId order should be reversed at slot {i}"
        );

        let aCoEdge = aGraph.topo().co_edges().definition(aCoEdgeId);
        assert_eq!(
            aCoEdge.orientation.orientation(),
            top_abs::reverse(aOriBefore[aMirror]),
            "CoEdge orientation should be flipped at slot {i}"
        );
    }
}

#[test]
fn BRepGraph_ReverseTest__WireOrder_SetCoEdgeOrderCanonicalizesUnorderedPermutation() {
    let mut aGraph = buildBoxGraph();
    let aWire = WireId::new(0);
    assert!(aWire.is_valid_for(aGraph.topo().wires().nb()));

    let aCoEdges = aGraph.topo().wires().relations(aWire).co_edge_ids.clone();
    assert!(aCoEdges.len() >= 4);

    let aBadOrder = [aCoEdges[0], aCoEdges[2], aCoEdges[1], aCoEdges[3]];

    assert!(aGraph.editor().wires().set_co_edge_order(aWire, &aBadOrder));

    let aCoEdgesAfter = &aGraph.topo().wires().relations(aWire).co_edge_ids;
    assert_eq!(aCoEdges.len(), aCoEdgesAfter.len());
    assert!(isBoundaryValid(&mut aGraph));
    assert!(isAuditValid(&aGraph));
}

#[test]
fn BRepGraph_ReverseTest__WireOrder_CheckCoEdgeOrderReportsCurrentAndReordered() {
    let mut aGraph = buildBoxGraph();
    let aWire = WireId::new(0);
    assert!(aWire.is_valid_for(aGraph.topo().wires().nb()));

    let aCoEdges = aGraph.topo().wires().relations(aWire).co_edge_ids.clone();
    assert!(aCoEdges.len() >= 4);

    let aCurrentOrder: Vec<CoEdgeId> = aCoEdges.clone();

    assert_eq!(
        aGraph
            .editor()
            .wires()
            .check_wire_co_edge_order(aWire, &aCurrentOrder),
        CoEdgeOrderStatus::AlreadyCurrent
    );
    assert!(
        aGraph
            .editor()
            .wires()
            .set_co_edge_order(aWire, &aCurrentOrder)
    );

    let anUnordered = [aCoEdges[0], aCoEdges[2], aCoEdges[1], aCoEdges[3]];
    assert_eq!(
        aGraph
            .editor()
            .wires()
            .check_wire_co_edge_order(aWire, &anUnordered),
        CoEdgeOrderStatus::Reordered
    );
    assert_eq!(
        aGraph
            .editor()
            .wires()
            .check_wire_co_edge_order(WireId::default(), &anUnordered),
        CoEdgeOrderStatus::InvalidWire
    );
}

#[test]
fn BRepGraph_ReverseTest__WireOrder_SetCoEdgeOrderCanonicalizesOpenWirePermutation() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0);
    let aV1 = addVertex(&mut aGraph, 1.0);
    let aV2 = addVertex(&mut aGraph, 2.0);
    let aV3 = addVertex(&mut aGraph, 3.0);

    let mut aCoEdges: Vec<CoEdgeId> = Vec::new();
    for (aStart, anEnd) in [(aV0, aV1), (aV1, aV2), (aV2, aV3)] {
        let anEdge = addSegment(&mut aGraph, aStart, anEnd);
        aCoEdges.push(addCoEdge(&mut aGraph, anEdge));
    }

    let aWire = aGraph.editor().wires().add(&aCoEdges);
    assert!(aWire.is_valid());

    let anUnordered = [aCoEdges[2], aCoEdges[0], aCoEdges[1]];

    assert!(aGraph.editor().wires().set_co_edge_order(aWire, &anUnordered));

    let anOrdered = &aGraph.topo().wires().relations(aWire).co_edge_ids;
    assert_eq!(anOrdered.len(), aCoEdges.len());
    assert_eq!(anOrdered[0], aCoEdges[0]);
    assert_eq!(anOrdered[1], aCoEdges[1]);
    assert_eq!(anOrdered[2], aCoEdges[2]);
    assert!(isBoundaryValid(&mut aGraph));
}

#[test]
fn BRepGraph_ReverseTest__WireOrder_CheckAppendCoEdgeReportsReadyAndAlreadyContained() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0);
    let aV1 = addVertex(&mut aGraph, 1.0);
    let aV2 = addVertex(&mut aGraph, 2.0);

    let aFirstEdge = addSegment(&mut aGraph, aV0, aV1);
    let aInitialCoEdges = [addCoEdge(&mut aGraph, aFirstEdge)];
    let aWire = aGraph.editor().wires().add(&aInitialCoEdges);
    assert!(aWire.is_valid());

    let aSecondEdge = addSegment(&mut aGraph, aV1, aV2);
    let aSecondCoEdge = addCoEdge(&mut aGraph, aSecondEdge);

    assert_eq!(
        aGraph
            .editor()
            .wires()
            .check_append_co_edge(aWire, aSecondCoEdge),
        CoEdgeOrderStatus::Ready
    );
    assert_eq!(
        aGraph
            .editor()
            .wires()
            .check_append_co_edge(aWire, aInitialCoEdges[0]),
        CoEdgeOrderStatus::AlreadyContained
    );
}

#[test]
fn BRepGraph_ReverseTest__WireOrder_CheckReplaceEdgeReportsReadyAndDisconnected() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0);
    let aV1 = addVertex(&mut aGraph, 1.0);
    let aV2 = addVertex(&mut aGraph, 2.0);
    let aV3 = addVertex(&mut aGraph, 3.0);
    let aV4 = addVertex(&mut aGraph, 4.0);

    let anOldEdge = addSegment(&mut aGraph, aV0, aV1);
    let aNextEdge = addSegment(&mut aGraph, aV1, aV2);
    let aCoEdges = [
        addCoEdge(&mut aGraph, anOldEdge),
        addCoEdge(&mut aGraph, aNextEdge),
    ];
    let aWire = aGraph.editor().wires().add(&aCoEdges);
    assert!(aWire.is_valid());

    let aCompatibleEdge = addSegment(&mut aGraph, aV0, aV1);
    let aDisconnectedEdge = addSegment(&mut aGraph, aV3, aV4);

    let aWires = aGraph.editor().wires();
    assert_eq!(
        aWires.check_replace_edge(aWire, anOldEdge, aCompatibleEdge, false),
        ReplaceEdgeStatus::Ready
    );
    assert_eq!(
        aWires.check_replace_edge(aWire, anOldEdge, anOldEdge, false),
        ReplaceEdgeStatus::AlreadyCurrent
    );
    assert_eq!(
        aWires.check_replace_edge(aWire, anOldEdge, aDisconnectedEdge, false),
        ReplaceEdgeStatus::Disconnected
    );
}

#[test]
fn BRepGraph_ReverseTest__WireOrder_RemoveInternalCoEdgeFromClosedWireKeepsConnectedOrder() {
    let mut aGraph = buildBoxGraph();
    let aWire = WireId::new(0);
    assert!(aWire.is_valid_for(aGraph.topo().wires().nb()));

    let aCoEdgesBefore = aGraph.topo().wires().relations(aWire).co_edge_ids.clone();
    assert!(aCoEdgesBefore.len() >= 4);

    assert!(
        aGraph
            .editor()
            .wires()
            .remove_co_edge(aWire, aCoEdgesBefore[1])
            .unwrap()
    );

    let aCoEdgesAfter = &aGraph.topo().wires().relations(aWire).co_edge_ids;
    assert_eq!(aCoEdgesAfter.len(), aCoEdgesBefore.len() - 1);
    assert!(isBoundaryValid(&mut aGraph));
    assert!(isAuditValid(&aGraph));
}

#[test]
fn BRepGraph_ReverseTest__Edge_StartEndVertexRefsSwapped() {
    let mut aGraph = buildBoxGraph();
    let anEdge = EdgeId::new(0);
    assert!(anEdge.is_valid_for(aGraph.topo().edges().nb()));

    let aStartBefore = aGraph.topo().edges().definition(anEdge).start_vertex_ref_id;
    let aEndBefore = aGraph.topo().edges().definition(anEdge).end_vertex_ref_id;
    assert!(aStartBefore.is_valid());
    assert!(aEndBefore.is_valid());
    assert_ne!(aStartBefore, aEndBefore);

    aGraph.editor().edges().reverse(anEdge);

    let anEdgeDef = aGraph.topo().edges().definition(anEdge);
    assert_eq!(anEdgeDef.start_vertex_ref_id, aEndBefore);
    assert_eq!(anEdgeDef.end_vertex_ref_id, aStartBefore);
}

#[test]
fn BRepGraph_ReverseTest__Edge_RoundTripRestoresOriginal() {
    let mut aGraph = buildBoxGraph();
    let anEdge = EdgeId::new(0);
    assert!(anEdge.is_valid_for(aGraph.topo().edges().nb()));

    let aStart = aGraph.topo().edges().definition(anEdge).start_vertex_ref_id;
    let aEnd = aGraph.topo().edges().definition(anEdge).end_vertex_ref_id;

    aGraph.editor().edges().reverse(anEdge);
    aGraph.editor().edges().reverse(anEdge);

    let anEdgeDef = aGraph.topo().edges().definition(anEdge);
    assert_eq!(anEdgeDef.start_vertex_ref_id, aStart);
    assert_eq!(anEdgeDef.end_vertex_ref_id, aEnd);
}

#[test]
fn BRepGraph_ReverseTest__LayerTopoSupplement_AddVertexToFaceGoesToSupplementLayer() {
    let mut aGraph = buildBoxGraph();
    let aFaceId = FaceId::new(0);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    let aVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.5, 0.5, 0.5), 1.0e-7);
    assert!(aVertex.is_valid());

    // Faces().AddVertex() removed; use Shapes().Add(vertexShape, faceNode) instead.
    // const BRepGraph_VertexRefId aRefId =
    //   aGraph.Editor().Faces().AddVertex(aFaceId, aVertex, TopAbs_INTERNAL);
    // EXPECT_FALSE(aRefId.IsValid());

    let aLayer = aGraph.layer_registry().find::<LayerTopoSupplement>();
    assert!(!aLayer.is_null());
    let anAttached = aLayer.attached_to(NodeId::from(aFaceId));
    // No vertex added -> no supplement attachments.
    assert_eq!(anAttached.len(), 0);
    // Skip removal test since no attachments exist.
}

#[test]
fn BRepGraph_ReverseTest__LayerTopoSupplement_ShellStartsWithoutSupplementAttachments() {
    let aGraph = buildBoxGraph();
    assert!(aGraph.topo().shells().nb() >= 1);
    let aShellId = ShellId::new(0);
    let anEdge = EdgeId::new(0);
    assert!(anEdge.is_valid_for(aGraph.topo().edges().nb()));

    let aLayer = aGraph.layer_registry().find::<LayerTopoSupplement>();
    if !aLayer.is_null() {
        assert_eq!(aLayer.attached_to(NodeId::from(aShellId)).len(), 0);
    }
}

#[test]
fn BRepGraph_ReverseTest__Relations_SupplementLayerClearedOnGraphClear() {
    let mut aGraph = buildBoxGraph();
    assert!(aGraph.topo().shells().nb() >= 1);

    let aFaceId = FaceId::new(0);

    // Faces().AddVertex() removed; use Shapes().Add(vertexShape, faceNode) instead.
    // [[maybe_unused]] const BRepGraph_VertexRefId aVertexRef =
    //   aGraph.Editor().Faces().AddVertex(aFaceId, aVertex, TopAbs_INTERNAL);

    let aLayerBefore = aGraph.layer_registry().find::<LayerTopoSupplement>();
    assert!(!aLayerBefore.is_null());
    // No vertex added -> no supplement attachments before clear.
    assert_eq!(aLayerBefore.attached_to(NodeId::from(aFaceId)).len(), 0);

    aGraph.clear().unwrap();

    assert_eq!(aGraph.topo().faces().nb(), 0);
    assert_eq!(aGraph.topo().vertices().nb(), 0);

    let aLayerAfter = aGraph.layer_registry().find::<LayerTopoSupplement>();
    assert!(!aLayerAfter.is_null());
    assert_eq!(aLayerAfter.attached_to(NodeId::from(aFaceId)).len(), 0);
}

#[test]
fn BRepGraph_ReverseTest__ReversedShell_GraphStoresFaceRefOrientation() {
    let aGraph = reversedShellGraph();

    assert_eq!(aGraph.topo().faces().nb(), 6);

    let aFaceRefs = aGraph.refs().faces().ids_of(ShellId::start());
    assert_eq!(aFaceRefs.len(), 6);

    for &aFaceRefId in aFaceRefs {
        let aFaceRef = aGraph.refs().faces().entry(aFaceRefId);
        assert_eq!(
            aFaceRef.orientation.orientation(),
            Orientation::Reversed,
            "Face {} should have REVERSED orientation",
            aFaceRef.child_face_id.index
        );
    }
}

#[test]
fn BRepGraph_ReverseTest__ReversedShell_ParentExplorerReportsShell() {
    let aGraph = reversedShellGraph();

    let mut aFaceId = FaceId::new(0);
    while aFaceId.is_valid_for(6) {
        let anExp =
            ParentExplorer::with_mode(&aGraph, aFaceId.into(), TraversalMode::DirectParents)
                .unwrap();
        assert!(
            anExp.more(),
            "Face {} should have a parent shell",
            aFaceId.index
        );
        assert_eq!(anExp.current().def_id, NodeId::from(ShellId::start()));
        assert_eq!(anExp.leaf_orientation(), Orientation::Reversed);
        aFaceId.pre_increment();
    }
}

#[test]
fn BRepGraph_ReverseTest__ReversedShell_ReconstructPreservesOrientation() {
    let aGraph = reversedShellGraph();

    let mut aFaceId = FaceId::new(0);
    while aFaceId.is_valid_for(6) {
        let aReconShape = aGraph.shapes().reconstruct(aFaceId.into()).unwrap();
        assert!(!aReconShape.is_null());
        assert_eq!(aReconShape.shape_type().unwrap(), ShapeEnum::Face);
        aFaceId.pre_increment();
    }
}

#[test]
fn BRepGraph_ReverseTest__ReversedShell_ReconstructShellPreservesFaceOrientations() {
    let aGraph = reversedShellGraph();

    let aReconShell = aGraph
        .shapes()
        .reconstruct(NodeId::new(NodeKind::SHELL, 0))
        .unwrap();
    assert!(!aReconShell.is_null());
    assert_eq!(aReconShell.shape_type().unwrap(), ShapeEnum::Shell);

    let mut aFaceCount = 0;
    let mut anExp = Explorer::new(&aReconShell, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while anExp.more() {
        assert_eq!(anExp.current().orientation(), Orientation::Reversed);
        aFaceCount += 1;
        anExp.next().unwrap();
    }

    assert_eq!(aFaceCount, 6);
}
