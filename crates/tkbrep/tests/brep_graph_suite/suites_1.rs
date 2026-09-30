// BRepGraph_Test.cxx, from `Build_SimpleBox_IsNotEmpty` to
// `ParallelBuild_CompoundOfFaces`.

#[test]
fn BRepGraphTest__Build_SimpleBox_IsNotEmpty() {
    let myGraph = SetUp();
    assert!(!myGraph.is_empty());
}

#[test]
fn BRepGraphTest__Build_SimpleBox_CorrectCounts() {
    let myGraph = SetUp();
    assert_eq!(myGraph.topo().solids().nb(), 1);
    assert_eq!(myGraph.topo().shells().nb(), 1);
    assert_eq!(myGraph.topo().faces().nb(), 6);
    assert_eq!(myGraph.topo().wires().nb(), 6);
    assert_eq!(myGraph.topo().edges().nb(), 12);
    assert_eq!(myGraph.topo().vertices().nb(), 8);
}

#[test]
fn BRepGraphTest__Build_SimpleBox_SurfaceCount() {
    let myGraph = SetUp();
    assert_eq!(myGraph.topo().faces().nb(), 6);
}

#[test]
fn BRepGraphTest__Face_Surface_IsValid() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        assert!(
            tool::face::has_surface(&myGraph, aFaceId),
            "Face {} has no surface rep",
            aFaceId.index
        );
    }
}

#[test]
fn BRepGraphTest__Edge_CurveAndVertices_AreValid() {
    let myGraph = SetUp();
    for anEdgeId in edgeIds(&myGraph) {
        if !tool::edge::degenerated(&myGraph, anEdgeId) {
            assert!(
                tool::edge::has_curve(&myGraph, anEdgeId),
                "Edge {} has no Curve3D rep",
                anEdgeId.index
            );
        }
        assert!(
            tool::edge::start_vertex_id(&myGraph, anEdgeId).is_valid(),
            "Edge {} has invalid StartVertexId",
            anEdgeId.index
        );
        assert!(
            tool::edge::end_vertex_id(&myGraph, anEdgeId).is_valid(),
            "Edge {} has invalid EndVertexId",
            anEdgeId.index
        );
    }
}

#[test]
fn BRepGraphTest__Wire_OuterWire_Exists() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        let anOuterWire = BRepGraph_TestTools::outer_wire_of_face(&myGraph, aFaceId).unwrap();
        assert!(
            anOuterWire.is_valid(),
            "Face {} has no outer wire",
            aFaceId.index
        );
    }
}

#[test]
fn BRepGraphTest__FaceDef_HasValidSurface() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        assert!(
            tool::face::has_surface(&myGraph, aFaceId),
            "Face {} has no surface rep",
            aFaceId.index
        );
    }
}

#[test]
fn BRepGraphTest__FindPCurveCoEdgeId_ValidPair() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        let anOuterWire = BRepGraph_TestTools::outer_wire_of_face(&myGraph, aFaceId).unwrap();
        if !anOuterWire.is_valid() {
            continue;
        }
        for &aCoEdgeId in BRepGraph_TestTools::co_edges_of_wire(&myGraph, anOuterWire) {
            let anEdge = myGraph.topo().co_edges().definition(aCoEdgeId).child_edge_id;
            if tool::edge::degenerated(&myGraph, anEdge) {
                continue;
            }
            let aPCurveId = tool::edge::find_pcurve_co_edge_id(&myGraph, anEdge, aFaceId);
            assert!(
                aPCurveId.is_valid(),
                "Missing PCurve for edge {} on face {}",
                anEdge.index,
                aFaceId.index
            );
        }
    }
}

/// The ids a `BRepGraph_Iterator<Def>` visits.
fn activeIds<T: NodeTag>(theGraph: &BRepGraph) -> Vec<TypedId<T>> {
    let mut anIt = DefIterator::<T>::new(theGraph);
    let mut anIds = Vec::new();
    while anIt.more() {
        anIds.push(anIt.current_id());
        anIt.next();
    }
    anIds
}

/// The face ids of a `BRepGraph_FaceIterator` walk.
pub fn faceIds(theGraph: &BRepGraph) -> Vec<FaceId> {
    activeIds::<FaceTag>(theGraph)
}

/// The edge ids of a `BRepGraph_EdgeIterator` walk.
pub fn edgeIds(theGraph: &BRepGraph) -> Vec<EdgeId> {
    activeIds::<EdgeTag>(theGraph)
}

/// The wire ids of a `BRepGraph_WireIterator` walk.
pub fn wireIds(theGraph: &BRepGraph) -> Vec<WireId> {
    let mut anIt = WireIterator::new(theGraph);
    let mut anIds = Vec::new();
    while anIt.more() {
        anIds.push(anIt.current_id());
        anIt.next();
    }
    anIds
}

#[test]
fn BRepGraphTest__UID_Unique() {
    let myGraph = SetUp();
    let mut aUIDSet: HashSet<Uid> = HashSet::new();
    let mut aNodes: Vec<NodeId> = Vec::new();
    aNodes.extend(SolidIterator::new(&myGraph).map_ids());
    aNodes.extend(ShellIterator::new(&myGraph).map_ids());
    aNodes.extend(faceIds(&myGraph).into_iter().map(NodeId::from));
    aNodes.extend(wireIds(&myGraph).into_iter().map(NodeId::from));
    aNodes.extend(edgeIds(&myGraph).into_iter().map(NodeId::from));
    aNodes.extend(VertexIterator::new(&myGraph).map_ids());
    for aNode in aNodes {
        assert!(aUIDSet.insert(myGraph.uids().of_node(aNode)));
    }
    // Surface/Curve geometry UIDs are no longer separate nodes; geometry is stored inline on defs.
}

/// The node ids of a definition iterator's walk.
trait MapIds {
    fn map_ids(self) -> Vec<NodeId>;
}

impl<T: NodeTag> MapIds for DefIterator<'_, T> {
    fn map_ids(mut self) -> Vec<NodeId> {
        let mut anIds = Vec::new();
        while self.more() {
            anIds.push(self.current_id().into());
            DefIterator::next(&mut self);
        }
        anIds
    }
}

#[test]
fn BRepGraphTest__UID_NodeIdRoundTrip() {
    let myGraph = SetUp();
    for aFace in faceIds(&myGraph) {
        let aFaceId = NodeId::from(aFace);
        let aUID = myGraph.uids().of_node(aFaceId);
        let aResolved = myGraph.uids().node_id_from(&aUID);
        assert_eq!(
            aResolved, aFaceId,
            "Round trip failed for face {}",
            aFace.index
        );
    }
}

#[test]
fn BRepGraphTest__SameDomainFaces_Box_Empty() {
    let myGraph = SetUp();
    for aFaceId in faceIds(&myGraph) {
        assert_eq!(
            countSameDomainFaces(&myGraph, aFaceId),
            0,
            "Box face {} should have no same-domain faces",
            aFaceId.index
        );
    }
}

/// `BRep_Builder::MakeCompound` then `Add` of each shape.
pub fn compoundOf(theShapes: &[Shape]) -> Shape {
    let aBuilder = BRepBuilder::default();
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    for aShape in theShapes {
        aBuilder.add(&mut aCompound, aShape).unwrap();
    }
    aCompound.into()
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`.
pub fn boxShape(dx: f64, dy: f64, dz: f64) -> Shape {
    make_box(dx, dy, dz).into()
}

#[test]
fn BRepGraphTest__Decompose_TwoSeparateFaces() {
    let _myGraph = SetUp();
    let aFace1 = firstOf(&boxShape(10.0, 10.0, 10.0), ShapeEnum::Face);
    let aFace2 = firstOf(&boxShape(20.0, 20.0, 20.0), ShapeEnum::Face);

    let aCompound = compoundOf(&[aFace1, aFace2]);

    let aGraph = graphOf(&aCompound);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().faces().nb(), 2);

    assert_eq!(countFaceComponents(&aGraph), 2);
}

#[test]
fn BRepGraphTest__ReBuild_UIDMonotonic() {
    let _myGraph = SetUp();
    let aBox = boxShape(10.0, 20.0, 30.0);

    let mut aGraph = graphOf(&aBox);
    let aGen1 = aGraph.uids().generation();

    // Access a UID from the first build to verify it works.
    assert!(aGraph.topo().faces().nb() > 0);
    let aFirstFace = NodeId::new(NodeKind::FACE, 0);
    let aFirstUID = aGraph.uids().of_node(aFirstFace);
    assert!(aFirstUID.is_valid());

    aGraph.clear().unwrap();
    let _aBuildRes4 = aGraph.shapes_mut().add(&aBox).unwrap();
    let aGen2 = aGraph.uids().generation();

    assert!(aGen2 > aGen1);

    // Verify all face UIDs in second build are valid and have new generation.
    for aFace in faceIds(&aGraph) {
        let aUID = aGraph.uids().of_node(aFace.into());
        assert!(
            aUID.is_valid(),
            "Face {} should have a valid UID",
            aFace.index
        );
    }

    // First build's UID should no longer be valid in the new generation.
    assert!(!aGraph.uids().has_node(&aFirstUID));
}

#[test]
fn BRepGraphTest__DetectMissingPCurves_ValidBox_Empty() {
    let myGraph = SetUp();
    let mut aMissing: Vec<(EdgeId, FaceId)> = Vec::new();
    for aFaceId in faceIds(&myGraph) {
        let mut anEdgeExp =
            ChildExplorer::with_target(&myGraph, aFaceId, NodeKind::EDGE).unwrap();
        while anEdgeExp.more() {
            let anEdgeId = EdgeId::from_node_id(anEdgeExp.current().def_id);
            anEdgeExp.next().unwrap();
            if tool::edge::degenerated(&myGraph, anEdgeId) {
                continue;
            }
            if !tool::edge::find_pcurve_co_edge_id(&myGraph, anEdgeId, aFaceId).is_valid() {
                aMissing.push((anEdgeId, aFaceId));
            }
        }
    }
    assert_eq!(aMissing.len(), 0);
}

#[test]
fn BRepGraphTest__DetectDegenerateWires_ValidBox_Empty() {
    let myGraph = SetUp();
    let mut aDegenerate: Vec<WireId> = Vec::new();
    for aWireId in wireIds(&myGraph) {
        let aNbEdges = ChildExplorer::with_target(&myGraph, aWireId, NodeKind::EDGE)
            .unwrap()
            .count();
        if aNbEdges < 2 {
            aDegenerate.push(aWireId);
            continue;
        }
        let isOuterWire = isOuterWireOfSomeFace(&myGraph, aWireId);
        if isOuterWire && !tool::wire::is_closed(&myGraph, aWireId) {
            aDegenerate.push(aWireId);
        }
    }
    assert_eq!(aDegenerate.len(), 0);
}

// The face walk of `DetectDegenerateWires`: some parent face has the wire as its outer
// wire.
fn isOuterWireOfSomeFace(theGraph: &BRepGraph, theWireId: WireId) -> bool {
    let mut aFaceExp =
        ParentExplorer::with_target(theGraph, theWireId.into(), NodeKind::FACE).unwrap();
    while aFaceExp.more() {
        let aFaceId = FaceId::from_node_id(aFaceExp.current().def_id);
        if tool::face::outer_wire(theGraph, aFaceId).unwrap() == theWireId {
            return true;
        }
        aFaceExp.next().unwrap();
    }
    false
}

#[test]
fn BRepGraphTest__MutableEdge_ModifyTolerance() {
    let mut myGraph = SetUp();
    let anOrigTol = tool::edge::tolerance(&myGraph, EdgeId::start());
    let mut anEdge = myGraph.editor().edges().mutate(EdgeId::start()).unwrap();
    EdgeOps::set_tolerance_guarded(&mut anEdge, anOrigTol * 2.0);
    let aTolerance = tool::edge::tolerance(anEdge.graph(), EdgeId::start());
    assert!((aTolerance - anOrigTol * 2.0).abs() <= 1.0e-15);
}

#[test]
fn BRepGraphTest__NbFacesOfEdge_SharedEdge() {
    let myGraph = SetUp();
    // In a box, each non-degenerate edge is shared by exactly 2 faces.
    for anEdgeId in edgeIds(&myGraph) {
        if !tool::edge::degenerated(&myGraph, anEdgeId) {
            let aCount = myGraph.topo().edges().nb_faces(anEdgeId);
            assert_eq!(
                aCount, 2,
                "Edge {} should be shared by 2 faces",
                anEdgeId.index
            );
        }
    }
}

#[test]
fn BRepGraphTest__FreeEdges_ClosedBox_Empty() {
    let myGraph = SetUp();
    let aFree = collectFreeEdges(&myGraph);
    assert_eq!(aFree.len(), 0);
}

#[test]
fn BRepGraphTest__RecordHistory_BasicEntry() {
    let myGraph = SetUp();
    let aBefore = nbRecords(&myGraph);
    let anEdge0 = edgeNode(0);
    let anEdge1 = edgeNode(1);
    let aRepl = [anEdge1];
    recordHistory(&myGraph, "TestOp", anEdge0, &aRepl);
    assert_eq!(nbRecords(&myGraph), aBefore + 1);
    assert!(recordNameIs(&myGraph, aBefore, "TestOp"));
}

#[test]
fn BRepGraphTest__ReplaceEdge_Substitution() {
    let mut myGraph = SetUp();
    // Get the first wire and its first edge via incidence refs.
    let aCoEdgeIds = BRepGraph_TestTools::co_edges_of_wire(&myGraph, WireId::start());
    assert!(!aCoEdgeIds.is_empty());
    let anOldEdgeId = myGraph.topo().co_edges().definition(aCoEdgeIds[0]).child_edge_id;

    let aNewEdgeId = addCompatibleReplacementEdge(&mut myGraph, anOldEdgeId, false);
    assert!(aNewEdgeId.is_valid());

    myGraph
        .editor()
        .wires()
        .replace_edge(WireId::start(), anOldEdgeId, aNewEdgeId, false)
        .unwrap();

    // Verify the substitution via the updated incidence refs.
    let aCoEdgeIdsAfter = BRepGraph_TestTools::co_edges_of_wire(&myGraph, WireId::start());
    assert!(!aCoEdgeIdsAfter.is_empty());
    let aNewCoEdge = myGraph.topo().co_edges().definition(aCoEdgeIdsAfter[0]);
    assert_eq!(aNewCoEdge.child_edge_id.index, aNewEdgeId.index);
}

/// `BRepGraph::ShapesView::Options{{}, true, false, theParallel}`.
fn buildOptions(theParallel: bool) -> shapes_view::Options {
    shapes_view::Options {
        create_auto_product: true,
        flatten: false,
        parallel: theParallel,
        ..shapes_view::Options::default()
    }
}

fn graphWithOptions(theShape: &Shape, theParallel: bool) -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph
        .shapes_mut()
        .add_with_options(theShape, &buildOptions(theParallel))
        .unwrap();
    aGraph
}

#[test]
fn BRepGraphTest__ParallelBuild_SameAsSequential() {
    let _myGraph = SetUp();
    let aBox = boxShape(10.0, 20.0, 30.0);

    let aSeqGraph = graphWithOptions(&aBox, false);
    assert!(!aSeqGraph.is_empty());

    let aParGraph = graphWithOptions(&aBox, true);
    assert!(!aParGraph.is_empty());

    let (aPar, aSeq) = (aParGraph.topo(), aSeqGraph.topo());
    assert_eq!(aPar.solids().nb(), aSeq.solids().nb());
    assert_eq!(aPar.shells().nb(), aSeq.shells().nb());
    assert_eq!(aPar.faces().nb(), aSeq.faces().nb());
    assert_eq!(aPar.wires().nb(), aSeq.wires().nb());
    assert_eq!(aPar.edges().nb(), aSeq.edges().nb());
    assert_eq!(aPar.vertices().nb(), aSeq.vertices().nb());
    assert_eq!(aPar.faces().nb(), aSeq.faces().nb());
    assert_eq!(aPar.edges().nb(), aSeq.edges().nb());
}

/// Every face of the shape, in explorer order.
pub fn facesOf(theShape: &Shape) -> Vec<Shape> {
    let mut anExp = Explorer::new(theShape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut aFaces = Vec::new();
    while anExp.more() {
        aFaces.push(anExp.current().clone());
        anExp.next().unwrap();
    }
    aFaces
}

#[test]
fn BRepGraphTest__ParallelBuild_CompoundOfFaces() {
    let _myGraph = SetUp();
    let mut aFaces = facesOf(&boxShape(10.0, 10.0, 10.0));
    aFaces.extend(facesOf(&boxShape(20.0, 20.0, 20.0)));
    let aCompound = compoundOf(&aFaces);

    let aGraph = graphWithOptions(&aCompound, true);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().faces().nb(), 12);
}
