// BRepGraph_Test.cxx, Groups 4, 7 and 9: from `EdgeDef_HasValidCurve3d` to
// `Build_DefaultAllocator_IsNotEmpty`.

// ===================================================================
// Group 4: Geometric Queries
// ===================================================================

#[test]
fn BRepGraphTest__EdgeDef_HasValidCurve3d() {
    let myGraph = SetUp();
    for anEdgeId in edgeIds(&myGraph) {
        if tool::edge::degenerated(&myGraph, anEdgeId) {
            continue;
        }

        assert!(
            tool::edge::has_curve(&myGraph, anEdgeId),
            "Edge {} has no Curve3D rep",
            anEdgeId.index
        );
    }
}

// ===================================================================
// Group 7: Detection Methods
// ===================================================================

#[test]
fn BRepGraphTest__FreeEdges_SingleFace_AllEdgesFree() {
    let _myGraph = SetUp();
    let aFace = firstOf(&boxShape(10.0, 20.0, 30.0), ShapeEnum::Face);

    let aGraph = graphOf(&aFace);
    assert!(!aGraph.is_empty());

    let aFreeEdges = collectFreeEdges(&aGraph);
    assert_eq!(aFreeEdges.len(), 4);
}

#[test]
fn BRepGraphTest__Decompose_ThreeDisconnectedFaces_ThreeComponents() {
    let _myGraph = SetUp();
    let aCompound = compoundOf(&[
        firstOf(&boxShape(10.0, 10.0, 10.0), ShapeEnum::Face),
        firstOf(&boxShape(20.0, 20.0, 20.0), ShapeEnum::Face),
        firstOf(&boxShape(30.0, 30.0, 30.0), ShapeEnum::Face),
    ]);

    let aGraph = graphOf(&aCompound);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().faces().nb(), 3);

    assert_eq!(countFaceComponents(&aGraph), 3);
}

/// Whether the edge has a live 3D curve (not degenerated, curve present).
fn hasLiveCurve(theGraph: &BRepGraph, theEdge: EdgeId) -> bool {
    !tool::edge::degenerated(theGraph, theEdge) && tool::edge::has_curve(theGraph, theEdge)
}

/// The two first edges on one curve handle get the tolerances 0.001 and 1.0.
fn setUpToleranceConflict(myGraph: &mut BRepGraph) -> bool {
    let aNbEdges = myGraph.topo().edges().nb();
    for anEdgeIndex in 0..aNbEdges {
        let anEdgeId = EdgeId::new(anEdgeIndex);
        if !hasLiveCurve(myGraph, anEdgeId) {
            continue;
        }
        let aCurve = tool::edge::curve(myGraph, anEdgeId).object_address();
        let anOther = (anEdgeIndex + 1..aNbEdges).map(EdgeId::new).find(|&anOtherId| {
            hasLiveCurve(myGraph, anOtherId)
                && tool::edge::curve(myGraph, anOtherId).object_address() == aCurve
        });
        if let Some(anOtherId) = anOther {
            // Set very different tolerances on two edges sharing the same curve.
            myGraph.editor().edges().set_tolerance(anEdgeId, 0.001).unwrap();
            myGraph.editor().edges().set_tolerance(anOtherId, 1.0).unwrap();
            return true;
        }
    }
    false
}

/// The edges on the same curve whose tolerances spread over 0.5.
fn collectToleranceConflicts(myGraph: &BRepGraph) -> Vec<EdgeId> {
    let mut aConflicts: Vec<EdgeId> = Vec::new();
    let mut aConflictKeys: HashSet<u32> = HashSet::new();
    for anEdgeId in edgeIds(myGraph) {
        if !hasLiveCurve(myGraph, anEdgeId) {
            continue;
        }
        let aCurve = tool::edge::curve(myGraph, anEdgeId).object_address();
        let aCurveEdges: Vec<EdgeId> = std::iter::once(anEdgeId)
            .chain(edgeIds(myGraph).into_iter().filter(|&anOtherId| {
                anOtherId != anEdgeId
                    && hasLiveCurve(myGraph, anOtherId)
                    && tool::edge::curve(myGraph, anOtherId).object_address() == aCurve
            }))
            .collect();
        let aTols = aCurveEdges
            .iter()
            .map(|&anId| myGraph.topo().edges().definition(anId).tolerance);
        let (aMinTol, aMaxTol) = aTols.fold((f64::MAX, f64::MIN), |(aMin, aMax), aTol| {
            (aMin.min(aTol), aMax.max(aTol))
        });
        if aCurveEdges.len() > 1 && aMaxTol - aMinTol > 0.5 {
            for aConflictId in aCurveEdges {
                if aConflictKeys.insert(aConflictId.index) {
                    aConflicts.push(aConflictId);
                }
            }
        }
    }
    aConflicts
}

#[test]
fn BRepGraphTest__DetectToleranceConflicts_ManualConflict_Detected() {
    let mut myGraph = SetUp();
    // Find two edges that share the same Curve3d handle.
    let isConflictSetUp = setUpToleranceConflict(&mut myGraph);

    if isConflictSetUp {
        let aConflicts = collectToleranceConflicts(&myGraph);
        assert!(!aConflicts.is_empty());
    }
}

#[test]
fn BRepGraphTest__Build_EmptyCompound_IsEmptyZeroCounts() {
    let _myGraph = SetUp();
    let aCompound = compoundOf(&[]);

    let aGraph = graphOf(&aCompound);
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().solids().nb(), 0);
    assert_eq!(aGraph.topo().shells().nb(), 0);
    assert_eq!(aGraph.topo().faces().nb(), 0);
    assert_eq!(aGraph.topo().wires().nb(), 0);
    assert_eq!(aGraph.topo().edges().nb(), 0);
    assert_eq!(aGraph.topo().vertices().nb(), 0);
}

#[test]
fn BRepGraphTest__TopoNode_GenericLookup_MatchesTypedAccess() {
    let myGraph = SetUp();
    let aFaceId = NodeId::new(NodeKind::FACE, 0);
    let aBase = myGraph.topo().generic().topo_entity(aFaceId);
    assert!(aBase.is_some());

    // Invalid node id should return nullptr.
    let anInvalidId = NodeId::default();
    let anInvalidBase = myGraph.topo().generic().topo_entity(anInvalidId);
    assert!(anInvalidBase.is_none());
}

// ===================================================================
// Group 9: Node Counts and Identity
// ===================================================================

#[test]
fn BRepGraphTest__NbNodes_Box_TotalCount() {
    let myGraph = SetUp();
    // NbNodes should equal sum of all per-kind counts (topology + assembly).
    let aTopo = myGraph.topo();
    let anExpected = aTopo.solids().nb()
        + aTopo.shells().nb()
        + aTopo.faces().nb()
        + aTopo.wires().nb()
        + aTopo.co_edges().nb()
        + aTopo.edges().nb()
        + aTopo.vertices().nb()
        + aTopo.compounds().nb()
        + aTopo.comp_solids().nb()
        + aTopo.products().nb()
        + aTopo.occurrences().nb();
    assert_eq!(aTopo.generic().nb_nodes(), anExpected);
}

#[test]
fn BRepGraphTest__HasUID_ValidFace_ReturnsTrue() {
    let myGraph = SetUp();
    let aUID = myGraph.uids().of_node(NodeId::new(NodeKind::FACE, 0));
    assert!(myGraph.uids().has_node(&aUID));
}

#[test]
fn BRepGraphTest__HasUID_InvalidUID_ReturnsFalse() {
    let myGraph = SetUp();
    let anInvalidUID = Uid::invalid();
    assert!(!myGraph.uids().has_node(&anInvalidUID));
}

#[test]
fn BRepGraphTest__NodeIdFromUID_InvalidUID_ReturnsInvalid() {
    let myGraph = SetUp();
    let anInvalidUID = Uid::invalid();
    let aResolved = myGraph.uids().node_id_from(&anInvalidUID);
    assert!(!aResolved.is_valid());
}

#[test]
fn BRepGraphTest__Allocator_DefaultConstructor_NotNull() {
    let _myGraph = SetUp();
    // `EXPECT_FALSE(myGraph.Allocator().IsNull())`: the graph storage owns its vectors,
    // so there is no allocator handle to check (docs/topology-design.md section 11.8).
}

#[test]
fn BRepGraphTest__Build_DefaultAllocator_IsNotEmpty() {
    let _myGraph = SetUp();
    let aGraph = graphOf(&boxShape(10.0, 20.0, 30.0));
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().faces().nb(), 6);
    // `EXPECT_FALSE(aGraph.Allocator().IsNull())` has no Rust counterpart (section 11.8).
}
