// BRepGraph_Test.cxx, the rest of Group 3 (Mutation APIs): from
// `RemoveFace_PrunesOrphanAndKeepsRelationsConsistent` to
// `MutableVertex_ChangePoint_Verified`.

#[test]
fn BRepGraphTest__RemoveFace_PrunesOrphanAndKeepsRelationsConsistent() {
    let mut myGraph = SetUp();
    let aFaceRefsBefore = BRepGraph_TestTools::face_refs_of_shell(&myGraph, ShellId::start());
    assert!(!aFaceRefsBefore.is_empty());
    let aNbFaceRefsBefore = aFaceRefsBefore.len() as u32;

    let aFaceRefId = aFaceRefsBefore[0];
    let aFaceId = myGraph.refs().faces().entry(aFaceRefId).child_face_id;

    assert!(
        myGraph
            .editor()
            .shells()
            .remove_face(ShellId::start(), aFaceRefId)
            .unwrap()
    );

    assert_eq!(
        BRepGraph_TestTools::count_face_refs_of_shell(&myGraph, ShellId::start()),
        aNbFaceRefsBefore - 1
    );
    assert!(aFaceId.is_removed(&myGraph));
    assert!(isBoundaryValid(&mut myGraph));

    let aShellParents = &myGraph
        .topo()
        .shells()
        .relations(ShellId::start())
        .parent_shell_ref_ids;
    for aSolidId in SolidsOfShell::new(&myGraph, aShellParents) {
        assert_ne!(aSolidId, SolidId::default());
    }

    let aFaceParents = &myGraph.topo().faces().relations(aFaceId).parent_face_ref_ids;
    for aShellId in ShellsOfFace::new(&myGraph, aFaceParents) {
        assert_ne!(aShellId, ShellId::start());
    }
}

#[test]
fn BRepGraphTest__RemoveShell_PrunesOrphanAndKeepsRelationsConsistent() {
    let mut myGraph = SetUp();
    let aShellRefsBefore = BRepGraph_TestTools::shell_refs_of_solid(&myGraph, SolidId::start());
    assert!(!aShellRefsBefore.is_empty());
    let aNbShellRefsBefore = aShellRefsBefore.len() as u32;

    let aShellRefId = aShellRefsBefore[0];
    let aShellId = myGraph.refs().shells().entry(aShellRefId).child_shell_id;

    assert!(
        myGraph
            .editor()
            .solids()
            .remove_shell(SolidId::start(), aShellRefId)
            .unwrap()
    );

    assert_eq!(
        BRepGraph_TestTools::count_shell_refs_of_solid(&myGraph, SolidId::start()),
        aNbShellRefsBefore - 1
    );
    assert!(aShellId.is_removed(&myGraph));
    assert!(isBoundaryValid(&mut myGraph));

    let aSolidParents = &myGraph
        .topo()
        .solids()
        .relations(SolidId::start())
        .parent_solid_ref_ids;
    for aCompSolidId in CompSolidsOfSolid::new(&myGraph, aSolidParents) {
        assert_ne!(aCompSolidId, CompSolidId::default());
    }

    let aShellParents = &myGraph.topo().shells().relations(aShellId).parent_shell_ref_ids;
    for aSolidId in SolidsOfShell::new(&myGraph, aShellParents) {
        assert_ne!(aSolidId, SolidId::start());
    }
}

/// The first face (and its wire reference) that uses the first wire.
fn faceUsingFirstWire(theGraph: &BRepGraph) -> Option<(FaceId, WireRefId)> {
    for aCandidateFaceId in faceIds(theGraph) {
        let aWireRefs = BRepGraph_TestTools::wire_refs_of_face(theGraph, aCandidateFaceId);
        for &aCandidateWireRefId in aWireRefs {
            let aChild = theGraph.refs().wires().entry(aCandidateWireRefId).child_wire_id;
            if aChild == WireId::start() {
                return Some((aCandidateFaceId, aCandidateWireRefId));
            }
        }
    }
    None
}

#[test]
fn BRepGraphTest__RemoveWire_PrunesOrphanAndKeepsRelationsConsistent() {
    let mut myGraph = SetUp();
    let aFound = faceUsingFirstWire(&myGraph);

    assert!(aFound.is_some());
    let (aFaceId, aWireRefId) = aFound.unwrap();
    let aNbWireRefsBefore = BRepGraph_TestTools::count_wire_refs_of_face(&myGraph, aFaceId);

    assert!(
        myGraph
            .editor()
            .faces()
            .remove_wire(aFaceId, aWireRefId)
            .unwrap()
    );

    assert_eq!(
        BRepGraph_TestTools::count_wire_refs_of_face(&myGraph, aFaceId),
        aNbWireRefsBefore - 1
    );
    assert!(!BRepGraph_TestTools::face_uses_wire(
        &myGraph,
        aFaceId,
        WireId::start()
    ));
    assert!(WireId::start().is_removed(&myGraph));
    assert!(isBoundaryValid(&mut myGraph));

    let aWireParents = &myGraph
        .topo()
        .wires()
        .relations(WireId::start())
        .parent_wire_ref_ids;
    for aWireFaceId in FacesOfWire::new(&myGraph, aWireParents) {
        assert_ne!(aWireFaceId, aFaceId);
    }
}

#[test]
fn BRepGraphTest__RemoveOccurrence_PrunesOccurrenceSubtreeAndKeepsRelationsConsistent() {
    let mut myGraph = SetUp();
    let aPartId = ProductId::start();
    let aAssemblyId = myGraph.editor().products().add_empty();
    myGraph
        .editor()
        .products()
        .append_document_root(aAssemblyId)
        .unwrap();
    assert!(aAssemblyId.is_valid());
    let (anOccId, _) = myGraph
        .editor()
        .products()
        .append(
            aAssemblyId,
            aPartId,
            &Location::default(),
            OccurrenceId::default(),
        )
        .unwrap();
    assert!(anOccId.is_valid());

    let aOccurrenceRefsBefore = myGraph.refs().occurrences().ids_of(aAssemblyId);
    assert_eq!(aOccurrenceRefsBefore.len(), 1);
    let anOccurrenceRefId = aOccurrenceRefsBefore[0];

    assert!(
        myGraph
            .editor()
            .products()
            .remove_occurrence(aAssemblyId, anOccurrenceRefId)
            .unwrap()
    );

    assert_eq!(myGraph.refs().occurrences().ids_of(aAssemblyId).len(), 0);
    assert!(myGraph.topo().generic().is_removed(anOccId.into()));
    assert_eq!(myGraph.topo().products().nb_components(aAssemblyId), 0);
    assert!(isBoundaryValid(&mut myGraph));
}

#[test]
fn BRepGraphTest__RemoveShapeRoot_PrunesUniqueTopologyRoot() {
    let mut myGraph = SetUp();
    let aPartId = ProductId::start();
    let aShapeRootBefore = myGraph.topo().products().shape_root(aPartId);
    assert!(aShapeRootBefore.is_valid());

    assert!(
        myGraph
            .editor()
            .products()
            .remove_shape_root(aPartId)
            .unwrap()
    );

    let aProducts = myGraph.topo().products();
    assert!(!aProducts.shape_root(aPartId).is_valid());
    assert!(myGraph.topo().generic().is_removed(aShapeRootBefore));
    assert!(!aProducts.is_part(aPartId));
    assert!(!aProducts.is_assembly(aPartId));
    assert_eq!(aProducts.nb_components(aPartId), 0);
    assert!(isBoundaryValid(&mut myGraph));
}

#[test]
fn BRepGraphTest__RemoveVertex_ClearsBoundarySlotAndPrunesUniqueVertex() {
    let mut myGraph = SetUp();
    let aTol = precision::confusion();
    let aStartVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), aTol);
    let anEndVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(10.0, 0.0, 0.0), aTol);
    assert!(aStartVertex.is_valid());
    assert!(anEndVertex.is_valid());

    let aNoCurve: Handle<dyn GeomCurve> = Handle::default();
    let anEdge = myGraph
        .editor()
        .edges()
        .add(aStartVertex, anEndVertex, &aNoCurve, 0.0, 1.0, aTol);
    assert!(anEdge.is_valid());

    let aStartRefId = myGraph.topo().edges().definition(anEdge).start_vertex_ref_id;
    assert!(aStartRefId.is_valid());

    assert!(
        myGraph
            .editor()
            .edges()
            .remove_vertex(anEdge, aStartRefId)
            .unwrap()
    );

    let anEdgeDef = myGraph.topo().edges().definition(anEdge);
    assert!(!anEdgeDef.start_vertex_ref_id.is_valid());
    assert!(myGraph.topo().generic().is_removed(aStartVertex.into()));
    assert!(!tool::edge::start_vertex_id(&myGraph, anEdge).is_valid());
    assert!(tool::edge::end_vertex_id(&myGraph, anEdge).is_valid());
    assert!(isBoundaryValid(&mut myGraph));
}

#[test]
fn BRepGraphTest__RemoveNode_EdgeWithReplacement_ReparentsAllCoEdges() {
    let mut myGraph = SetUp();
    let aCoEdgeIds = BRepGraph_TestTools::co_edges_of_wire(&myGraph, WireId::start());
    assert!(!aCoEdgeIds.is_empty());

    let anOldEdgeId = myGraph.topo().co_edges().definition(aCoEdgeIds[0]).child_edge_id;
    let aNewEdgeId = EdgeId::new((anOldEdgeId.index + 1) % myGraph.topo().edges().nb());

    myGraph
        .editor()
        .generic()
        .replace_node(anOldEdgeId.into(), aNewEdgeId.into())
        .unwrap();

    let anOldCoEdges = myGraph.topo().edges().co_edges(anOldEdgeId);
    assert_eq!(anOldCoEdges.len(), 0);

    let aNewCoEdges = myGraph.topo().edges().co_edges(aNewEdgeId);
    assert!(!aNewCoEdges.is_empty());
    for &aNewCoEdgeId in aNewCoEdges {
        let aCoEdge = myGraph.topo().co_edges().definition(aNewCoEdgeId);
        assert_eq!(aCoEdge.child_edge_id, aNewEdgeId);
    }
}

#[test]
fn BRepGraphTest__MutableVertex_ChangePoint_Verified() {
    let mut myGraph = SetUp();
    let mut aMutVert = myGraph
        .editor()
        .vertices()
        .mutate(VertexId::start())
        .unwrap();
    VertexOps::set_point_guarded(&mut aMutVert, Point3::new(99.0, 99.0, 99.0));

    let aVert = aMutVert.graph().topo().vertices().definition(VertexId::start());
    let aTol = precision::confusion();
    assert!((aVert.point.x() - 99.0).abs() <= aTol);
    assert!((aVert.point.y() - 99.0).abs() <= aTol);
    assert!((aVert.point.z() - 99.0).abs() <= aTol);
}
