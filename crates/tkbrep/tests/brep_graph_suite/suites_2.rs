// BRepGraph_Test.cxx, Group 1 (Shape Round-Trip): from
// `ReconstructFace_EachBoxFace_SameSubShapeCounts` to
// `BRepGraph_UIDsViewTest.ReverseLookupStaysCurrentAfterProgrammaticAdd`.

/// `TopExp::MapShapes(theShape, theKind, theMap)` then `Extent()`.
fn nbMapped(theShape: &Shape, theKind: ShapeEnum) -> usize {
    let mut aMap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(theShape, theKind, &mut aMap).unwrap();
    aMap.extent()
}

/// The surface object of `BRep_Tool::Surface(face, location)`.
fn surfaceAddress(theFace: &Shape) -> *const () {
    let aFace = topods::face(theFace).unwrap();
    let (aSurface, _aLoc) = BRepTool::surface_with_location(&aFace).unwrap();
    aSurface.object_address()
}

/// The number of faces a `TopExp_Explorer` visits.
fn countFaces(theShape: &Shape) -> i32 {
    let mut aCount = 0;
    let mut anExp = Explorer::new(theShape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while anExp.more() {
        aCount += 1;
        anExp.next().unwrap();
    }
    aCount
}

#[test]
fn BRepGraphTest__ReconstructFace_EachBoxFace_SameSubShapeCounts() {
    let myGraph = SetUp();
    for aFace in faceIds(&myGraph) {
        let aFaceId = NodeId::from(aFace);
        let anOrigFace = myGraph.shapes().original(aFaceId);
        assert!(!anOrigFace.is_null());
        let aReconstructed = myGraph.shapes().reconstruct(aFaceId).unwrap();

        for (aKind, aName) in [
            (ShapeEnum::Vertex, "Vertex"),
            (ShapeEnum::Edge, "Edge"),
            (ShapeEnum::Wire, "Wire"),
        ] {
            assert_eq!(
                nbMapped(&aReconstructed, aKind),
                nbMapped(&anOrigFace, aKind),
                "{aName} count mismatch for face {}",
                aFace.index
            );
        }

        // Verify same surface handle.
        assert_eq!(
            surfaceAddress(&anOrigFace),
            surfaceAddress(&aReconstructed),
            "Surface handle differs for face {}",
            aFace.index
        );
    }
}

/// The first coedge's edge of the first wire.
fn firstWireEdge(theGraph: &BRepGraph) -> EdgeId {
    let aCoEdgeIds = BRepGraph_TestTools::co_edges_of_wire(theGraph, WireId::start());
    assert!(!aCoEdgeIds.is_empty());
    theGraph.topo().co_edges().definition(aCoEdgeIds[0]).child_edge_id
}

fn replaceFirstWireEdge(theGraph: &mut BRepGraph, theOld: EdgeId, theNew: EdgeId) {
    theGraph
        .editor()
        .wires()
        .replace_edge(WireId::start(), theOld, theNew, false)
        .unwrap();
}

#[test]
fn BRepGraphTest__ReconstructFace_AfterEdgeReplace_ContainsNewEdge() {
    let mut myGraph = SetUp();
    let anOldEdgeId = firstWireEdge(&myGraph);

    let aNewEdgeId = addCompatibleReplacementEdge(&mut myGraph, anOldEdgeId, false);
    assert!(aNewEdgeId.is_valid());

    // Get 3D curve handles from graph for old/new edges.
    let aNewCurve = tool::edge::curve(&myGraph, aNewEdgeId);
    let anOldCurve = tool::edge::curve(&myGraph, anOldEdgeId);

    replaceFirstWireEdge(&mut myGraph, anOldEdgeId, aNewEdgeId);

    // Reconstruct face 0 (the face owning wire 0).
    assert!(BRepGraph_TestTools::face_uses_wire(
        &myGraph,
        FaceId::start(),
        WireId::start()
    ));
    let aFaceIdx = 0;
    let aReconstructed = myGraph
        .shapes()
        .reconstruct(FaceId::new(aFaceIdx).into())
        .unwrap();

    // Check via 3D curve handle identity (reconstructed edges have new TShapes).
    let mut isNewFound = false;
    let mut isOldFound = false;
    let mut anExp = Explorer::new(&aReconstructed, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    while anExp.more() {
        let anEdge: Edge = topods::edge(anExp.current()).unwrap();
        let aCurve = BRepTool::curve_with_location(&anEdge).unwrap().curve;
        let isSame = |theOther: &Handle<dyn GeomCurve>| {
            !aCurve.is_null()
                && !theOther.is_null()
                && aCurve.object_address() == theOther.object_address()
        };
        isNewFound |= isSame(&aNewCurve);
        isOldFound |= isSame(&anOldCurve);
        anExp.next().unwrap();
    }
    assert!(isNewFound, "New edge curve not found in reconstructed face");
    assert!(!isOldFound, "Old edge curve still present in reconstructed face");
}

#[test]
fn BRepGraphTest__ReconstructShape_SolidRoot_SameFaceCount() {
    let myGraph = SetUp();
    let aSolidId = NodeId::new(NodeKind::SOLID, 0);
    let aReconstructed = myGraph.shapes().reconstruct(aSolidId).unwrap();

    assert_eq!(countFaces(&aReconstructed), 6);
}

#[test]
fn BRepGraphTest__ReconstructShape_FaceRoot_ReturnsSameShape() {
    let myGraph = SetUp();
    let aFaceId = NodeId::new(NodeKind::FACE, 0);
    let aReconstructed = myGraph.shapes().reconstruct(aFaceId).unwrap();
    let anOriginal = myGraph.shapes().original(aFaceId);
    assert!(!anOriginal.is_null());

    // Reconstructed face should have the same surface handle.
    assert_eq!(surfaceAddress(&anOriginal), surfaceAddress(&aReconstructed));
}

#[test]
fn BRepGraphTest__Shape_Unmodified_ReturnsSameShape() {
    let myGraph = SetUp();
    let aFaceId = NodeId::new(NodeKind::FACE, 0);
    let aShape = myGraph.shapes().shape(aFaceId).unwrap();
    let anOrig = myGraph.shapes().original(aFaceId);
    assert!(!anOrig.is_null());
    assert!(aShape.is_same(&anOrig));
}

#[test]
fn BRepGraphTest__Shape_AfterReplaceEdge_DiffersFromOriginal() {
    let mut myGraph = SetUp();
    let anOldEdgeId = firstWireEdge(&myGraph);
    let aNewEdgeId = addCompatibleReplacementEdge(&mut myGraph, anOldEdgeId, false);
    assert!(aNewEdgeId.is_valid());

    replaceFirstWireEdge(&mut myGraph, anOldEdgeId, aNewEdgeId);

    // Find the face that owns wire 0.
    let aFaceIdx = faceIds(&myGraph)
        .into_iter()
        .find(|&aFace| BRepGraph_TestTools::face_uses_wire(&myGraph, aFace, WireId::start()))
        .map(|aFace| aFace.index);
    assert!(aFaceIdx.is_some());
    let aFaceId = NodeId::new(NodeKind::FACE, aFaceIdx.unwrap());
    let aShape = myGraph.shapes().shape(aFaceId).unwrap();
    let anOrig = myGraph.shapes().original(aFaceId);
    assert!(!anOrig.is_null());
    assert!(!aShape.is_same(&anOrig));
}

/// `Shapes().Shape(NodeId(theKind, 0))` is not null and has the shape type.
fn expectShapeKind(theKind: NodeKind, theType: ShapeEnum) {
    let myGraph = SetUp();
    let aShape = myGraph.shapes().shape(NodeId::new(theKind, 0)).unwrap();
    assert!(!aShape.is_null());
    assert_eq!(aShape.shape_type().unwrap(), theType);
}

#[test]
fn BRepGraphTest__Shape_WireKind_Valid() {
    expectShapeKind(NodeKind::WIRE, ShapeEnum::Wire);
}

#[test]
fn BRepGraphTest__Shape_EdgeKind_Valid() {
    expectShapeKind(NodeKind::EDGE, ShapeEnum::Edge);
}

#[test]
fn BRepGraphTest__Shape_VertexKind_Valid() {
    expectShapeKind(NodeKind::VERTEX, ShapeEnum::Vertex);
}

#[test]
fn BRepGraphTest__OwnGen_MutableEdge_PropagatesSubtreeGenUp() {
    let mut myGraph = SetUp();
    let aStart = EdgeId::start();
    assert_eq!(myGraph.topo().edges().definition(aStart).base.own_gen, 0);

    let mut aGuard = myGraph.editor().edges().mutate(aStart).unwrap();
    aGuard.mark_dirty();
    drop(aGuard);

    // Edge was directly mutated: OwnGen incremented.
    assert!(myGraph.topo().edges().definition(aStart).base.own_gen > 0);

    // Check propagation up to parent wire and face (SubtreeGen).
    if aStart.is_valid_for(myGraph.topo().edges().nb()) {
        // Find a wire containing this edge.
        let aWireIt = myGraph.topo().edges().wires_of(aStart);
        if aWireIt.more() {
            let aWireId = aWireIt.current_id();
            assert!(myGraph.topo().wires().definition(aWireId).base.subtree_gen > 0);
            // Check propagation to owning face.
            let anOwner = faceIds(&myGraph)
                .into_iter()
                .find(|&aFace| BRepGraph_TestTools::face_uses_wire(&myGraph, aFace, aWireId));
            if let Some(aFace) = anOwner {
                assert!(myGraph.topo().faces().definition(aFace).base.subtree_gen > 0);
            }
        }
    }
}

#[test]
fn BRepGraphTest__ReconstructShape_WireKind_NoThrow() {
    let myGraph = SetUp();
    let aWireId = NodeId::new(NodeKind::WIRE, 0);
    let aShape = myGraph.shapes().reconstruct(aWireId);
    assert!(aShape.is_ok());
    assert!(!aShape.unwrap().is_null());
}

#[test]
fn BRepGraphTest__HasOriginalShape_AfterBuild_True() {
    let myGraph = SetUp();
    for aFace in faceIds(&myGraph) {
        assert!(
            myGraph.shapes().has_original(aFace.into()),
            "Face {} should have original shape after BRepGraph::ShapesView::Add()",
            aFace.index
        );
    }
    for anEdge in edgeIds(&myGraph) {
        assert!(
            myGraph.shapes().has_original(anEdge.into()),
            "Edge {} should have original shape after BRepGraph::ShapesView::Add()",
            anEdge.index
        );
    }
}

#[test]
fn BRepGraphTest__Shape_CachedOnSecondCall() {
    let myGraph = SetUp();
    let aVtxId = NodeId::new(NodeKind::VERTEX, 0);
    let aFirst = myGraph.shapes().shape(aVtxId).unwrap();
    let aSecond = myGraph.shapes().shape(aVtxId).unwrap();
    assert!(aFirst.is_same(&aSecond));
}

#[test]
fn BRepGraphTest__Shape_InvalidatedAfterMutation() {
    let mut myGraph = SetUp();
    let anEdgeId = NodeId::new(NodeKind::EDGE, 0);
    let aBefore = myGraph.shapes().shape(anEdgeId).unwrap();
    assert!(!aBefore.is_null());

    myGraph
        .editor()
        .edges()
        .set_tolerance(EdgeId::start(), 0.123)
        .unwrap();
    let anAfter = myGraph.shapes().shape(anEdgeId).unwrap();
    assert!(!anAfter.is_null());

    // After mutation, Shape() reconstructs a new edge - different TShape.
    assert!(!aBefore.is_same(&anAfter));
}

#[test]
fn BRepGraphTest__DefaultBuild_AssignsValidUIDs() {
    let _myGraph = SetUp();
    let aGraph = graphOf(&boxShape(10.0, 20.0, 30.0));

    assert!(!aGraph.is_empty());
    assert!(aGraph.topo().faces().nb() > 0);

    let aFaceId = NodeId::new(NodeKind::FACE, 0);
    let aUID = aGraph.uids().of_node(aFaceId);

    assert!(aUID.is_valid());
    assert!(aGraph.uids().has_node(&aUID));
    assert_eq!(aGraph.uids().node_id_from(&aUID), aFaceId);
}

#[test]
fn BRepGraphTest__UIDsGeneration_IncrementsAcrossBuilds() {
    let _myGraph = SetUp();
    let mut aGraph = graphOf(&boxShape(10.0, 20.0, 30.0));
    let aGeneration1 = aGraph.uids().generation();

    aGraph.clear().unwrap();
    let _aBuildRes10 = aGraph
        .shapes_mut()
        .add(&boxShape(11.0, 21.0, 31.0))
        .unwrap();
    let aGeneration2 = aGraph.uids().generation();

    assert!(aGeneration1 > 0);
    assert_eq!(aGeneration2, aGeneration1 + 1);
}

#[test]
fn BRepGraphTest__StaleUID_HasReturnsFalseAfterRebuild() {
    let _myGraph = SetUp();
    let mut aGraph = graphOf(&boxShape(10.0, 20.0, 30.0));
    assert!(aGraph.topo().faces().nb() > 0);
    let anOldUID = aGraph.uids().of_node(NodeId::new(NodeKind::FACE, 0));
    assert!(anOldUID.is_valid());
    assert!(aGraph.uids().has_node(&anOldUID));

    aGraph.clear().unwrap();
    let _aBuildRes12 = aGraph
        .shapes_mut()
        .add(&boxShape(11.0, 21.0, 31.0))
        .unwrap();

    assert!(!aGraph.uids().has_node(&anOldUID));
    assert!(!aGraph.uids().node_id_from(&anOldUID).is_valid());
}

#[test]
fn BRepGraph_UIDsViewTest__ReverseLookupStaysCurrentAfterProgrammaticAdd() {
    let mut aGraph = BRepGraph::new();

    let aFirstVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 0.001);
    let aFirstUID = aGraph.uids().of_node(aFirstVertex.into());
    assert!(aFirstUID.is_valid());
    assert_eq!(
        aGraph.uids().node_id_from(&aFirstUID),
        NodeId::from(aFirstVertex)
    );

    let aSecondVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 0.001);
    let aSecondUID = aGraph.uids().of_node(aSecondVertex.into());
    assert!(aSecondUID.is_valid());

    assert_eq!(
        aGraph.uids().node_id_from(&aFirstUID),
        NodeId::from(aFirstVertex)
    );
    assert_eq!(
        aGraph.uids().node_id_from(&aSecondUID),
        NodeId::from(aSecondVertex)
    );
    assert!(aGraph.uids().has_node(&aFirstUID));
    assert!(aGraph.uids().has_node(&aSecondUID));
}
