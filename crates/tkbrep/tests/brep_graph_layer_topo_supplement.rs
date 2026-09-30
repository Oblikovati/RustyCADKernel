//! Direct translation of BRepGraph_LayerTopoSupplement_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. The shapes are built with the real
//! BRep_Builder; the suite includes BRepGraph_Compact but calls none. The layer's
//! `OnNodeReplaced` callback receives the graph it is attached to, and the
//! `NCollection_LinearVector` answers of `AttachedTo` are vectors.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::layer::BRepGraphLayer;
use rustycad_tkbrep::brep_graph::layer_topo_supplement::{AttachmentKind, LayerTopoSupplement};
use rustycad_tkbrep::brep_graph::refs_iterator::RefsVertexOfEdge;
use rustycad_tkbrep::brep_graph::shapes_view::Options;
use rustycad_tkbrep::brep_graph::supplement_editor::SupplementEditor;
use rustycad_tkbrep::brep_graph::supplement_iterator::SupplementIterator;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topods::{
    CompSolid, Compound, Edge, Face, Shape, ShapeIterator, Shell, Solid, Vertex, Wire,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;

fn makeVertexShape(thePoint: Point3) -> Vertex {
    let mut aVertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut aVertex, thePoint, 1.0e-7)
        .unwrap();
    aVertex
}

fn oriented(theShape: &Shape, theOrientation: Orientation) -> Shape {
    theShape.oriented(theOrientation)
}

fn makeEdgeWithSupplementVertices() -> Edge {
    let aBuilder = BRepBuilder::default();

    let aStart = makeVertexShape(Point3::new(0.0, 0.0, 0.0));
    let anExtraFwd = makeVertexShape(Point3::new(2.0, 0.0, 0.0));
    let anInternal = makeVertexShape(Point3::new(5.0, 0.0, 0.0));
    let anExternal = makeVertexShape(Point3::new(7.0, 0.0, 0.0));
    let anEnd = makeVertexShape(Point3::new(10.0, 0.0, 0.0));

    let mut anEdge = Edge::default();
    aBuilder.make_edge(&mut anEdge).unwrap();

    aBuilder
        .add(&mut anEdge, &oriented(&anExtraFwd, Orientation::Forward))
        .unwrap();
    aBuilder
        .add(&mut anEdge, &oriented(&aStart, Orientation::Forward))
        .unwrap();
    aBuilder
        .add(&mut anEdge, &oriented(&anInternal, Orientation::Internal))
        .unwrap();
    aBuilder
        .add(&mut anEdge, &oriented(&anExternal, Orientation::External))
        .unwrap();
    aBuilder
        .add(&mut anEdge, &oriented(&anEnd, Orientation::Reversed))
        .unwrap();
    anEdge
}

// The four edges of the 10x10 square in z=0, each from its corner to the next.
fn squareEdges() -> [Edge; 4] {
    let aBuilder = BRepBuilder::default();
    let aCorners = [
        makeVertexShape(Point3::new(0.0, 0.0, 0.0)),
        makeVertexShape(Point3::new(10.0, 0.0, 0.0)),
        makeVertexShape(Point3::new(10.0, 10.0, 0.0)),
        makeVertexShape(Point3::new(0.0, 10.0, 0.0)),
    ];
    std::array::from_fn(|anIndex| {
        let mut anEdge = Edge::default();
        aBuilder.make_edge(&mut anEdge).unwrap();
        let (aFirst, aLast) = (&aCorners[anIndex], &aCorners[(anIndex + 1) % 4]);
        aBuilder
            .add(&mut anEdge, &oriented(aFirst, Orientation::Forward))
            .unwrap();
        aBuilder
            .add(&mut anEdge, &oriented(aLast, Orientation::Reversed))
            .unwrap();
        anEdge
    })
}

fn makePlainFace() -> Face {
    let aBuilder = BRepBuilder::default();
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    for anEdge in squareEdges() {
        aBuilder.add(&mut aWire, &anEdge).unwrap();
    }
    aWire.set_closed(true).unwrap();

    let mut aFace = Face::default();
    aBuilder.make_face(&mut aFace);
    aBuilder.add(&mut aFace, &aWire).unwrap();
    aFace
}

fn makeFaceWithSupplementVertex() -> Face {
    let mut aFace = makePlainFace();
    let aLooseVertex = makeVertexShape(Point3::new(5.0, 5.0, 0.0));
    BRepBuilder::default()
        .add(&mut aFace, &oriented(&aLooseVertex, Orientation::Internal))
        .unwrap();
    aFace
}

fn makePlainShell() -> Shell {
    let aBuilder = BRepBuilder::default();
    let mut aShell = Shell::default();
    aBuilder.make_shell(&mut aShell);
    aBuilder.add(&mut aShell, &makePlainFace()).unwrap();
    aShell
}

fn makePlainSolid() -> Solid {
    let aBuilder = BRepBuilder::default();
    let mut aSolid = Solid::default();
    aBuilder.make_solid(&mut aSolid);
    aBuilder.add(&mut aSolid, &makePlainShell()).unwrap();
    aSolid
}

fn addEmptyCompSolid(theGraph: &mut BRepGraph) -> CompSolidId {
    theGraph.editor().comp_solids().add(&[])
}

fn makePlainCompSolid() -> CompSolid {
    let aBuilder = BRepBuilder::default();
    let mut aCompSolid = CompSolid::default();
    aBuilder.make_comp_solid(&mut aCompSolid);
    aBuilder.add(&mut aCompSolid, &makePlainSolid()).unwrap();
    aCompSolid
}

fn countDirectChildren(theShape: &Shape, theType: ShapeEnum, theOrientation: Orientation) -> i32 {
    let mut aCount = 0;
    let mut aChildIt = ShapeIterator::new(theShape, false, false);
    while aChildIt.more() {
        let aChild = aChildIt.value().unwrap();
        if aChild.shape_type().unwrap() == theType && aChild.orientation() == theOrientation {
            aCount += 1;
        }
        aChildIt.next();
    }
    aCount
}

fn no_auto_product() -> Options {
    Options {
        create_auto_product: false,
        ..Options::default()
    }
}

fn find_layer(theGraph: &BRepGraph) -> Handle<LayerTopoSupplement> {
    theGraph.layer_registry().find::<LayerTopoSupplement>()
}

fn ensure_layer(theGraph: &BRepGraph) -> Handle<LayerTopoSupplement> {
    theGraph
        .layer_registry()
        .ensure::<LayerTopoSupplement>()
        .unwrap()
}

fn add_vertex(theGraph: &mut BRepGraph, theX: f64) -> VertexId {
    theGraph
        .editor()
        .vertices()
        .add(Point3::new(theX, 0.0, 0.0), 1.0e-7)
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__AttachAndRemoveVertexSupplement() {
    let mut aGraph = BRepGraph::new();
    let aOwner = add_vertex(&mut aGraph, 0.0);

    let mut anEditor = SupplementEditor::new(&mut aGraph);
    let anAttachment = anEditor
        .attach_to_vertex(aOwner, &makeVertexShape(Point3::new(1.0, 2.0, 3.0)), None)
        .unwrap();
    assert_ne!(anAttachment, 0);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();

    let anAttached = aLayer.attached_to(aOwner.into());
    assert_eq!(anAttached.len(), 1);
    assert_eq!(anAttached[0], anAttachment);

    let anEntry = aLayer.find_by_uid(anAttachment);
    assert!(anEntry.is_some());
    let anEntry = anEntry.unwrap();
    assert_eq!(anEntry.base_owner, NodeId::from(aOwner));
    assert_eq!(anEntry.kind, AttachmentKind::VertexSupplementShape);
    assert_eq!(anEntry.shape.shape_type().unwrap(), ShapeEnum::Vertex);

    assert!(SupplementEditor::new(&mut aGraph).remove_attachment(anAttachment));
    assert_eq!(aLayer.attached_to(aOwner.into()).len(), 0);
    assert!(aLayer.find_by_uid(anAttachment).is_none());
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__SupplementIteratorPreservesOwnerOrderAndUidLookup() {
    let mut aGraph = BRepGraph::new();
    let aOwner = add_vertex(&mut aGraph, 0.0);

    let mut anEditor = SupplementEditor::new(&mut aGraph);
    let aFirst = anEditor
        .attach_to_vertex(aOwner, &makeVertexShape(Point3::new(1.0, 0.0, 0.0)), None)
        .unwrap();
    let aSecond = anEditor
        .attach_to_vertex(aOwner, &makeVertexShape(Point3::new(2.0, 0.0, 0.0)), None)
        .unwrap();
    assert_ne!(aFirst, 0);
    assert_ne!(aSecond, 0);

    let mut aSeen = Vec::new();
    let mut anIt = SupplementIterator::new(&aGraph, aOwner.into());
    while anIt.more() {
        aSeen.push(anIt.uid());
        let aValue = anIt.value().unwrap();
        assert_eq!(aValue.base_owner, NodeId::from(aOwner));
        assert_eq!(aValue.shape.shape_type().unwrap(), ShapeEnum::Vertex);
        anIt.next_attachment();
    }

    assert_eq!(aSeen.len(), 2);
    assert_eq!(aSeen[0], aFirst);
    assert_eq!(aSeen[1], aSecond);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();
    assert!(aLayer.find_by_uid(aFirst).is_some());
    assert!(aLayer.find_by_uid(aSecond).is_some());
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__OnNodeReplacedMigratesAttachments() {
    let mut aGraph = BRepGraph::new();
    let anOldOwner = add_vertex(&mut aGraph, 0.0);
    let aNewOwner = add_vertex(&mut aGraph, 5.0);

    let mut anEditor = SupplementEditor::new(&mut aGraph);
    let anAttachment = anEditor
        .attach_to_vertex(
            anOldOwner,
            &makeVertexShape(Point3::new(0.0, 1.0, 0.0)),
            None,
        )
        .unwrap();
    assert_ne!(anAttachment, 0);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();

    let anAttachedGraph = aLayer.layer_base().attached_graph(&aGraph);
    aLayer.on_node_replaced(anAttachedGraph, anOldOwner.into(), aNewOwner.into());

    assert_eq!(aLayer.attached_to(anOldOwner.into()).len(), 0);
    let anAttachedNew = aLayer.attached_to(aNewOwner.into());
    assert_eq!(anAttachedNew.len(), 1);
    assert_eq!(anAttachedNew[0], anAttachment);

    let anEntry = aLayer.find_by_uid(anAttachment);
    assert!(anEntry.is_some());
    assert_eq!(anEntry.unwrap().base_owner, NodeId::from(aNewOwner));
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__RemoveNodeDropsOwnedAttachments() {
    let mut aGraph = BRepGraph::new();
    let aOwner = add_vertex(&mut aGraph, 0.0);

    let mut anEditor = SupplementEditor::new(&mut aGraph);
    let anAttachment = anEditor
        .attach_to_vertex(aOwner, &makeVertexShape(Point3::new(0.0, 1.0, 0.0)), None)
        .unwrap();
    assert_ne!(anAttachment, 0);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    assert_eq!(aLayer.get().unwrap().attached_to(aOwner.into()).len(), 1);

    aGraph
        .editor()
        .generic()
        .remove_node(aOwner.into())
        .unwrap();

    let aLayer = aLayer.get().unwrap();
    assert_eq!(aLayer.attached_to(aOwner.into()).len(), 0);
    assert!(aLayer.find_by_uid(anAttachment).is_none());
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__LegacyEdgeInternalVertexWriterUsesSupplementLayer() {
    let mut aGraph = BRepGraph::new();
    let aStart = add_vertex(&mut aGraph, 0.0);
    let anEnd = add_vertex(&mut aGraph, 10.0);
    let anEdge = aGraph
        .editor()
        .edges()
        .add(aStart, anEnd, &Handle::default(), 0.0, 1.0, 1.0e-7);
    assert!(anEdge.is_valid());

    // Edges().AddInternalVertex() removed; use Shapes().Add(vertexShape, edgeNode) instead.

    let aLayer = find_layer(&aGraph);
    // No internal vertex added -> supplement layer may not be created.
    if let Some(aLayer) = aLayer.get() {
        let anAttached = aLayer.attached_to(anEdge.into());
        assert_eq!(anAttached.len(), 0);
    }

    let mut aNbCoreVertexRefs = 0;
    let mut aRefIt = RefsVertexOfEdge::new(&aGraph, anEdge);
    while aRefIt.more() {
        aNbCoreVertexRefs += 1;
        aRefIt.next();
    }
    // Only start and end vertices (no internal vertex added).
    assert_eq!(aNbCoreVertexRefs, 2);
}

// The vertex children of a round trip counted per orientation (forward, reversed,
// internal, external).
fn count_vertex_orientations(theShape: &Shape) -> [i32; 4] {
    let mut aCounts = [0; 4];
    let mut aChildIt = ShapeIterator::new(theShape, false, false);
    while aChildIt.more() {
        let aChild = aChildIt.value().unwrap();
        assert_eq!(aChild.shape_type().unwrap(), ShapeEnum::Vertex);
        aCounts[aChild.orientation() as usize] += 1;
        aChildIt.next();
    }
    aCounts
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__AddAndReconstructPreservesSupplementEdgeVertices() {
    let mut aGraph = BRepGraph::new();
    let anOptions = no_auto_product();
    let aRegisteredLayer = ensure_layer(&aGraph);
    assert!(!aRegisteredLayer.is_null());

    let anInputEdge = makeEdgeWithSupplementVertices();
    let aResult = aGraph
        .shapes_mut()
        .add_with_options(&anInputEdge, &anOptions)
        .unwrap();
    assert!(aResult.is_ok());
    assert_eq!(aResult.topology_root.kind, NodeKind::EDGE);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();

    let anAttached = aLayer.attached_to(aResult.topology_root);
    assert_eq!(anAttached.len(), 3);

    let (mut aNbInternal, mut aNbExternal, mut aNbForward) = (0, 0, 0);
    for aUid in anAttached {
        let anEntry = aLayer.find_by_uid(aUid);
        assert!(anEntry.is_some());
        let anEntry = anEntry.unwrap();
        assert_eq!(anEntry.kind, AttachmentKind::EdgeInternalVertex);
        match anEntry.shape.orientation() {
            Orientation::Internal => aNbInternal += 1,
            Orientation::External => aNbExternal += 1,
            Orientation::Forward => aNbForward += 1,
            Orientation::Reversed => {}
        }
    }
    assert_eq!(aNbInternal, 1);
    assert_eq!(aNbExternal, 1);
    assert_eq!(aNbForward, 1);

    let aRoundTrip = aGraph.shapes().reconstruct(aResult.topology_root).unwrap();
    assert_eq!(aRoundTrip.shape_type().unwrap(), ShapeEnum::Edge);

    let [
        aNbRoundTripForward,
        aNbRoundTripReversed,
        aNbRoundTripInternal,
        aNbRoundTripExternal,
    ] = count_vertex_orientations(&aRoundTrip);
    assert_eq!(aNbRoundTripForward, 2);
    assert_eq!(aNbRoundTripReversed, 1);
    assert_eq!(aNbRoundTripInternal, 1);
    assert_eq!(aNbRoundTripExternal, 1);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__AddAndReconstructPreservesSupplementFaceVertex() {
    let mut aGraph = BRepGraph::new();
    let anOptions = no_auto_product();
    let aRegisteredLayer = ensure_layer(&aGraph);
    assert!(!aRegisteredLayer.is_null());

    let aInputFace = makeFaceWithSupplementVertex();
    let aResult = aGraph
        .shapes_mut()
        .add_with_options(&aInputFace, &anOptions)
        .unwrap();
    assert!(aResult.is_ok());
    assert_eq!(aResult.topology_root.kind, NodeKind::FACE);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();

    let anAttached = aLayer.attached_to(aResult.topology_root);
    assert_eq!(anAttached.len(), 1);

    let anEntry = aLayer.find_by_uid(anAttached[0]);
    assert!(anEntry.is_some());
    let anEntry = anEntry.unwrap();
    assert_eq!(anEntry.kind, AttachmentKind::FaceDirectVertex);
    assert_eq!(anEntry.shape.shape_type().unwrap(), ShapeEnum::Vertex);
    assert_eq!(anEntry.shape.orientation(), Orientation::Internal);

    let aRoundTrip = aGraph.shapes().reconstruct(aResult.topology_root).unwrap();
    assert_eq!(aRoundTrip.shape_type().unwrap(), ShapeEnum::Face);

    let mut aNbWires = 0;
    let mut aNbInternalVertices = 0;
    let mut aChildIt = ShapeIterator::new(&aRoundTrip, false, false);
    while aChildIt.more() {
        let aChild = aChildIt.value().unwrap();
        let aType = aChild.shape_type().unwrap();
        if aType == ShapeEnum::Wire {
            aNbWires += 1;
        } else if aType == ShapeEnum::Vertex && aChild.orientation() == Orientation::Internal {
            aNbInternalVertices += 1;
        }
        aChildIt.next();
    }

    assert_eq!(aNbWires, 1);
    assert_eq!(aNbInternalVertices, 1);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__ShellReconstructionReplaysFaceSupplementVertex() {
    let aBuilder = BRepBuilder::default();
    let mut aShell = Shell::default();
    aBuilder.make_shell(&mut aShell);
    aBuilder
        .add(&mut aShell, &makeFaceWithSupplementVertex())
        .unwrap();

    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aRegisteredLayer = ensure_layer(&aGraph);
    let anOptions = no_auto_product();
    let aResult = aGraph
        .shapes_mut()
        .add_with_options(&aShell, &anOptions)
        .unwrap();
    assert!(aResult.topology_root.is_valid());

    let aRoundTrip = aGraph.shapes().reconstruct(aResult.topology_root).unwrap();
    assert!(!aRoundTrip.is_null());
    assert_eq!(aRoundTrip.shape_type().unwrap(), ShapeEnum::Shell);

    let mut aNbFaces = 0;
    let mut aNbInternalVertices = 0;
    let mut aFaceIt = ShapeIterator::new(&aRoundTrip, false, false);
    while aFaceIt.more() {
        let aFace = aFaceIt.value().unwrap().clone();
        aFaceIt.next();
        if aFace.shape_type().unwrap() != ShapeEnum::Face {
            continue;
        }
        aNbFaces += 1;
        aNbInternalVertices +=
            countDirectChildren(&aFace, ShapeEnum::Vertex, Orientation::Internal);
    }

    assert_eq!(aNbFaces, 1);
    assert_eq!(aNbInternalVertices, 1);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__SolidAddChild_ReconstructPreservesSupplementEdge() {
    let mut aGraph = BRepGraph::new();
    let anOptions = no_auto_product();
    let aRegisteredLayer = ensure_layer(&aGraph);
    assert!(!aRegisteredLayer.is_null());

    let aInputSolid = makePlainSolid();
    let aResult = aGraph
        .shapes_mut()
        .add_with_options(&aInputSolid, &anOptions)
        .unwrap();
    assert!(aResult.is_ok());
    assert_eq!(aResult.topology_root.kind, NodeKind::SOLID);

    assert!(aGraph.topo().edges().nb() > 0);
    let aEdgeId = EdgeId::start();
    assert!(aEdgeId.is_valid());

    // Solids().AddChild() removed; use Shapes().Add(childShape, solidNode) instead.
    // Since the API call is commented out, no supplement attachment is created.

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());

    let anAttached = aLayer.get().unwrap().attached_to(aResult.topology_root);
    // No child added -> no supplement attachments.
    assert_eq!(anAttached.len(), 0);

    // Since no supplement attachment was created, the rest of the test is skipped.
    // The reconstruction test is covered by AddAndReconstructPreservesSupplementEdgeVertices.
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__ShellOwner_IsAcceptedBySupplementLayer() {
    let mut aGraph = BRepGraph::new();
    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    let aLayer = ensure_layer(&aGraph);
    let aLayer = aLayer.get().unwrap();
    assert_ne!(
        aLayer.add_attachment(
            &aGraph,
            aShell.into(),
            AttachmentKind::ShellAuxShape,
            &makeVertexShape(Point3::new(1.0, 2.0, 3.0)),
        ),
        0
    );

    assert_eq!(aLayer.attached_to(aShell.into()).len(), 1);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__CompSolidOwner_IsAcceptedBySupplementLayer() {
    let mut aGraph = BRepGraph::new();
    let aCompSolid = addEmptyCompSolid(&mut aGraph);
    assert!(aCompSolid.is_valid());

    let aLayer = ensure_layer(&aGraph);
    let aLayer = aLayer.get().unwrap();
    assert_ne!(
        aLayer.add_attachment(
            &aGraph,
            aCompSolid.into(),
            AttachmentKind::CompSolidAuxShape,
            &makePlainSolid(),
        ),
        0
    );
    assert_ne!(
        aLayer.add_attachment(
            &aGraph,
            aCompSolid.into(),
            AttachmentKind::GenericSupplementShape,
            &makeVertexShape(Point3::new(1.0, 2.0, 3.0)),
        ),
        0
    );

    assert_eq!(aLayer.attached_to(aCompSolid.into()).len(), 2);
}

// The core Add of a plain container, then an INTERNAL child added under it: the child
// is routed to the supplement layer with `theKind` and replayed by reconstruction.
fn check_internal_child_routed(
    theContainer: &Shape,
    theRootKind: NodeKind,
    theChild: Shape,
    theChildType: ShapeEnum,
    theKind: AttachmentKind,
    theCount: fn(&BRepGraph) -> u32,
    theExactlyOne: bool,
) {
    let mut aGraph = BRepGraph::new();
    let anOptions = no_auto_product();

    let aContainerResult = aGraph
        .shapes_mut()
        .add_with_options(theContainer, &anOptions)
        .unwrap();
    assert!(aContainerResult.is_ok());
    assert_eq!(aContainerResult.topology_root.kind, theRootKind);

    let aParent = aContainerResult.topology_root;
    let aNbBefore = theCount(&aGraph);
    let anInternalChild = theChild.oriented(Orientation::Internal);

    let aAddResult = aGraph
        .shapes_mut()
        .add_under(&anInternalChild, aParent)
        .unwrap();
    assert!(aAddResult.is_ok());
    assert!(!aAddResult.topology_root.is_valid());
    assert!(!aAddResult.inserted_ref.is_valid());
    assert_eq!(theCount(&aGraph), aNbBefore);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();

    let anAttached = aLayer.attached_to(aParent);
    if theExactlyOne {
        assert_eq!(anAttached.len(), 1);
    } else {
        assert!(!anAttached.is_empty());
    }

    let anEntry = aLayer.find_by_uid(*anAttached.last().unwrap());
    assert!(anEntry.is_some());
    let anEntry = anEntry.unwrap();
    assert_eq!(anEntry.kind, theKind);
    assert_eq!(anEntry.shape.shape_type().unwrap(), theChildType);
    assert_eq!(anEntry.shape.orientation(), Orientation::Internal);

    let aRoundTrip = aGraph.shapes().reconstruct(aParent).unwrap();
    assert_eq!(
        countDirectChildren(&aRoundTrip, theChildType, Orientation::Internal),
        1
    );
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__ShellAddFace_Internal_RoutedToSupplement() {
    check_internal_child_routed(
        &makePlainShell(),
        NodeKind::SHELL,
        makePlainFace().into(),
        ShapeEnum::Face,
        AttachmentKind::ShellAuxShape,
        |theGraph| theGraph.topo().faces().nb(),
        false,
    );
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__SolidAddShell_Internal_RoutedToSupplement() {
    check_internal_child_routed(
        &makePlainSolid(),
        NodeKind::SOLID,
        makePlainShell().into(),
        ShapeEnum::Shell,
        AttachmentKind::SolidAuxShape,
        |theGraph| theGraph.topo().shells().nb(),
        false,
    );
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__CompSolidAddSolid_Internal_RoutedToSupplement() {
    check_internal_child_routed(
        &makePlainCompSolid(),
        NodeKind::COMP_SOLID,
        makePlainSolid().into(),
        ShapeEnum::Solid,
        AttachmentKind::CompSolidAuxShape,
        |theGraph| theGraph.topo().solids().nb(),
        true,
    );
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__CompoundAddChild_MixedOrientations_CoreRefAndSupplement() {
    let mut aGraph = BRepGraph::new();
    let anOptions = no_auto_product();

    let aBB = BRepBuilder::default();
    let mut aCompound = Compound::default();
    aBB.make_compound(&mut aCompound);
    let aCompResult = aGraph
        .shapes_mut()
        .add_with_options(&aCompound, &anOptions)
        .unwrap();
    assert!(aCompResult.is_ok());
    let aCompoundId = CompoundId::from_node_id(aCompResult.topology_root);

    let aForwardFace = makePlainFace().oriented(Orientation::Forward);
    let aFwdResult = aGraph
        .shapes_mut()
        .add_under(&aForwardFace, aCompoundId.into())
        .unwrap();
    assert!(aFwdResult.is_ok());
    assert!(aFwdResult.inserted_ref.is_valid());

    let aInternalFace = makePlainFace().oriented(Orientation::Internal);
    let aIntResult = aGraph
        .shapes_mut()
        .add_under(&aInternalFace, aCompoundId.into())
        .unwrap();
    assert!(aIntResult.is_ok());
    assert!(!aIntResult.inserted_ref.is_valid());

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();

    let anAttached = aLayer.attached_to(aCompoundId.into());
    assert_eq!(anAttached.len(), 1);

    let anEntry = aLayer.find_by_uid(anAttached[0]);
    assert!(anEntry.is_some());
    assert_eq!(anEntry.unwrap().kind, AttachmentKind::CompoundAuxShape);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__CopyToAllocatesFreshUidOnCollision() {
    let mut aSource = BRepGraph::new();
    let mut aTarget = BRepGraph::new();

    let aSourceVertex = add_vertex(&mut aSource, 0.0);
    let aTargetVertex = add_vertex(&mut aTarget, 10.0);
    let aTargetVertexCount = aTarget.topo().vertices().nb();

    let aSourceLayer = ensure_layer(&aSource);
    let mut aTargetLayer = ensure_layer(&aTarget);

    assert!(aSourceLayer.get().unwrap().add_attachment_with_uid(
        &aSource,
        aSourceVertex.into(),
        1,
        AttachmentKind::VertexSupplementShape,
        &makeVertexShape(Point3::new(1.0, 0.0, 0.0)),
    ));
    assert!(aTargetLayer.get().unwrap().add_attachment_with_uid(
        &aTarget,
        aTargetVertex.into(),
        1,
        AttachmentKind::VertexSupplementShape,
        &makeVertexShape(Point3::new(11.0, 0.0, 0.0)),
    ));

    assert!(
        copy::perform(
            &aSource,
            &mut aTarget,
            GeomPolicy::Copy,
            MeshPolicy::Copy,
            CachePolicy::Drop,
        )
        .unwrap()
    );

    aTargetLayer = find_layer(&aTarget);
    assert!(!aTargetLayer.is_null());
    let aTargetLayer = aTargetLayer.get().unwrap();
    let aCopiedVertex = VertexId::new(aTargetVertexCount);
    let aCopiedAttachments = aTargetLayer.attached_to(aCopiedVertex.into());
    assert_eq!(aCopiedAttachments.len(), 1);
    assert_ne!(aCopiedAttachments[0], 1);
    assert!(aTargetLayer.find_by_uid(aCopiedAttachments[0]).is_some());
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__SelfCopyDoesNotMutateEntriesDuringIteration() {
    let mut aGraph = BRepGraph::new();
    let aSourceVertex = add_vertex(&mut aGraph, 0.0);

    let mut aLayer = ensure_layer(&aGraph);
    assert!(aLayer.get().unwrap().add_attachment_with_uid(
        &aGraph,
        aSourceVertex.into(),
        1,
        AttachmentKind::VertexSupplementShape,
        &makeVertexShape(Point3::new(1.0, 0.0, 0.0)),
    ));

    let aCopiedNode = copy::copy_node_in_place(
        &mut aGraph,
        aSourceVertex.into(),
        GeomPolicy::Copy,
        MeshPolicy::Copy,
        CachePolicy::Drop,
    )
    .unwrap();
    assert!(aCopiedNode.is_valid());
    assert_eq!(aCopiedNode.kind, NodeKind::VERTEX);
    assert_ne!(aCopiedNode.index, aSourceVertex.index);

    aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();

    let aSourceAttachments = aLayer.attached_to(aSourceVertex.into());
    assert_eq!(aSourceAttachments.len(), 1);
    assert_eq!(aSourceAttachments[0], 1);

    let aCopiedAttachments = aLayer.attached_to(aCopiedNode);
    assert_eq!(aCopiedAttachments.len(), 1);
    assert_ne!(aCopiedAttachments[0], 1);
    assert!(aLayer.find_by_uid(aCopiedAttachments[0]).is_some());
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__IncompatibleAttachmentKindIsRejected() {
    let mut aGraph = BRepGraph::new();
    let aV0 = add_vertex(&mut aGraph, 0.0);
    let aV1 = add_vertex(&mut aGraph, 1.0);
    let anEdge = aGraph
        .editor()
        .edges()
        .add(aV0, aV1, &Handle::default(), 0.0, 1.0, 1.0e-7);
    let aFaceResult = aGraph.shapes_mut().add(&makePlainFace()).unwrap();
    let aFace = FaceId::from_node_id(aFaceResult.topology_root);
    assert!(anEdge.is_valid());
    assert!(aFaceResult.is_ok());
    assert!(aFace.is_valid());

    let aLayer = ensure_layer(&aGraph);
    let aLayer = aLayer.get().unwrap();

    let attach = |theOwner: NodeId, theKind: AttachmentKind, theShape: &Shape| {
        aLayer.add_attachment(&aGraph, theOwner, theKind, theShape)
    };
    let aVertex: Shape = makeVertexShape(Point3::new(1.0, 2.0, 3.0)).into();
    assert_eq!(
        attach(anEdge.into(), AttachmentKind::FaceDirectVertex, &aVertex),
        0
    );
    let aVertex: Shape = makeVertexShape(Point3::new(4.0, 5.0, 6.0)).into();
    assert_eq!(
        attach(aFace.into(), AttachmentKind::SolidAuxShape, &aVertex),
        0
    );
    let aCompSolid = addEmptyCompSolid(&mut aGraph);
    assert!(aCompSolid.is_valid());
    let aShell: Shape = makePlainShell().into();
    assert_eq!(
        aLayer.add_attachment(
            &aGraph,
            aCompSolid.into(),
            AttachmentKind::ShellAuxShape,
            &aShell
        ),
        0
    );
    let aSolid: Shape = makePlainSolid().into();
    assert_eq!(
        aLayer.add_attachment(
            &aGraph,
            aFace.into(),
            AttachmentKind::CompSolidAuxShape,
            &aSolid
        ),
        0
    );

    assert_eq!(aLayer.attached_to(anEdge.into()).len(), 0);
    assert_eq!(aLayer.attached_to(aFace.into()).len(), 0);
    assert_eq!(aLayer.attached_to(aCompSolid.into()).len(), 0);
}

// An edge without vertices or curve (`BRep_Builder::MakeEdge`).
fn emptyEdge() -> Edge {
    let mut aEdge = Edge::default();
    BRepBuilder::default().make_edge(&mut aEdge).unwrap();
    aEdge
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__ShapesView_AddToShellRoutesInvalidChildWithoutOrphanAppend() {
    let mut aGraph = BRepGraph::new();
    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    let aNbEdgesBefore = aGraph.topo().edges().nb();
    let aNbVerticesBefore = aGraph.topo().vertices().nb();

    let aResult = aGraph
        .shapes_mut()
        .add_under(&emptyEdge(), aShell.into())
        .unwrap();
    assert!(aResult.is_ok());
    assert!(!aResult.topology_root.is_valid());
    assert!(!aResult.inserted_ref.is_valid());

    assert_eq!(aGraph.topo().edges().nb(), aNbEdgesBefore);
    assert_eq!(aGraph.topo().vertices().nb(), aNbVerticesBefore);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();
    let anAttached = aLayer.attached_to(aShell.into());
    assert_eq!(anAttached.len(), 1);
    let anEntry = aLayer.find_by_uid(anAttached[0]);
    assert!(anEntry.is_some());
    let anEntry = anEntry.unwrap();
    assert_eq!(anEntry.kind, AttachmentKind::ShellAuxShape);
    assert_eq!(anEntry.shape.shape_type().unwrap(), ShapeEnum::Edge);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__ShapesView_AddToSolidRoutesInvalidChildWithoutOrphanAppend() {
    let mut aGraph = BRepGraph::new();
    let aSolid = aGraph.editor().solids().add();
    assert!(aSolid.is_valid());

    let aNbFacesBefore = aGraph.topo().faces().nb();
    let aNbVerticesBefore = aGraph.topo().vertices().nb();

    let aFace = makePlainFace();

    let aResult = aGraph
        .shapes_mut()
        .add_under(&aFace, aSolid.into())
        .unwrap();
    assert!(aResult.is_ok());
    assert!(!aResult.topology_root.is_valid());
    assert!(!aResult.inserted_ref.is_valid());

    assert_eq!(aGraph.topo().faces().nb(), aNbFacesBefore);
    assert_eq!(aGraph.topo().vertices().nb(), aNbVerticesBefore);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();
    let anAttached = aLayer.attached_to(aSolid.into());
    assert_eq!(anAttached.len(), 1);
    let anEntry = aLayer.find_by_uid(anAttached[0]);
    assert!(anEntry.is_some());
    let anEntry = anEntry.unwrap();
    assert_eq!(anEntry.kind, AttachmentKind::SolidAuxShape);
    assert_eq!(anEntry.shape.shape_type().unwrap(), ShapeEnum::Face);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__ShapesView_AddToFaceRejectsUnsupportedChildWithoutOrphanAppend()
 {
    let mut aGraph = BRepGraph::new();
    let aFaceResult = aGraph.shapes_mut().add(&makePlainFace()).unwrap();
    assert!(aFaceResult.is_ok());
    assert!(aFaceResult.topology_root.is_valid());
    assert_eq!(aFaceResult.topology_root.kind, NodeKind::FACE);

    let aNbEdgesBefore = aGraph.topo().edges().nb();
    let aNbVerticesBefore = aGraph.topo().vertices().nb();

    let aResult = aGraph
        .shapes_mut()
        .add_under(&emptyEdge(), aFaceResult.topology_root)
        .unwrap();
    assert!(!aResult.is_ok());
    assert!(!aResult.topology_root.is_valid());
    assert!(!aResult.inserted_ref.is_valid());

    assert_eq!(aGraph.topo().edges().nb(), aNbEdgesBefore);
    assert_eq!(aGraph.topo().vertices().nb(), aNbVerticesBefore);
}

#[test]
fn BRepGraph_LayerTopoSupplementTest__ShapesView_AddNonSolidToCompSolidRoutesInvalidChildWithoutOrphanAppend()
 {
    let mut aGraph = BRepGraph::new();
    let aBuilder = BRepBuilder::default();
    let mut aCompSolidShape = CompSolid::default();
    aBuilder.make_comp_solid(&mut aCompSolidShape);
    aBuilder
        .add(&mut aCompSolidShape, &makePlainSolid())
        .unwrap();

    let aCompSolidResult = aGraph.shapes_mut().add(&aCompSolidShape).unwrap();
    assert!(aCompSolidResult.is_ok());
    assert!(aCompSolidResult.topology_root.is_valid());
    assert_eq!(aCompSolidResult.topology_root.kind, NodeKind::COMP_SOLID);

    let aNbEdgesBefore = aGraph.topo().edges().nb();
    let aNbVerticesBefore = aGraph.topo().vertices().nb();

    let aResult = aGraph
        .shapes_mut()
        .add_under(&emptyEdge(), aCompSolidResult.topology_root)
        .unwrap();
    assert!(aResult.is_ok());
    assert!(!aResult.topology_root.is_valid());
    assert!(!aResult.inserted_ref.is_valid());

    assert_eq!(aGraph.topo().edges().nb(), aNbEdgesBefore);
    assert_eq!(aGraph.topo().vertices().nb(), aNbVerticesBefore);

    let aLayer = find_layer(&aGraph);
    assert!(!aLayer.is_null());
    let aLayer = aLayer.get().unwrap();
    let anAttached = aLayer.attached_to(aCompSolidResult.topology_root);
    assert_eq!(anAttached.len(), 1);
    let anEntry = aLayer.find_by_uid(anAttached[0]);
    assert!(anEntry.is_some());
    let anEntry = anEntry.unwrap();
    assert_eq!(anEntry.kind, AttachmentKind::CompSolidAuxShape);
    assert_eq!(anEntry.shape.shape_type().unwrap(), ShapeEnum::Edge);
}
