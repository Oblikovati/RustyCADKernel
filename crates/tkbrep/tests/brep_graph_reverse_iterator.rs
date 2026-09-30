//! Direct translation of BRepGraph_ReverseIterator_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). A relation list the native test keeps
//! by reference across an edit is read again after it, which is what the live
//! reference sees.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::reverse_iterator::*;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use support::make_box;

/// `countIterator(theIterator)` over the native `More`/`Next` protocol.
macro_rules! countIterator {
    ($iterator:expr) => {{
        let mut theIterator = $iterator;
        let mut aCount: u32 = 0;
        while theIterator.more() {
            aCount += 1;
            theIterator.next();
        }
        aCount
    }};
}

/// The `BRepGraph_ReverseIteratorTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    myGraph
}

fn remove(aGraph: &mut BRepGraph, aNode: impl Into<NodeId>) {
    aGraph.editor().generic().remove_node(aNode.into()).unwrap();
}

#[test]
fn BRepGraph_ReverseIteratorTest__FacesOfEdge_BoxEdgeSharedByTwoFaces() {
    let myGraph = SetUp();
    // Every edge of a box is shared by exactly 2 faces.
    let anEdgeId = EdgeId::new(0);
    let aCount = countIterator!(myGraph.topo().edges().faces_of(anEdgeId));
    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_ReverseIteratorTest__EdgesOfVertex_BoxVertexSharedByThreeEdges() {
    let myGraph = SetUp();
    // Every vertex of a box is shared by exactly 3 edges.
    let aVertexId = VertexId::new(0);
    let aCount = countIterator!(EdgesOfVertex::new(
        &myGraph,
        myGraph.topo().vertices().edges(aVertexId)
    ));
    assert_eq!(aCount, 3);
}

#[test]
fn BRepGraph_ReverseIteratorTest__SolidsOfShell_BoxShellHasOneSolid() {
    let myGraph = SetUp();
    let aShellId = ShellId::new(0);
    let aCount = countIterator!(SolidsOfShell::new(
        &myGraph,
        &myGraph
            .topo()
            .shells()
            .relations(aShellId)
            .parent_shell_ref_ids
    ));
    assert_eq!(aCount, 1);
}

#[test]
fn BRepGraph_ReverseIteratorTest__ShellsOfFace_BoxFaceHasOneShell() {
    let myGraph = SetUp();
    let aFaceId = FaceId::new(0);
    let aCount = countIterator!(ShellsOfFace::new(
        &myGraph,
        &myGraph
            .topo()
            .faces()
            .relations(aFaceId)
            .parent_face_ref_ids
    ));
    assert_eq!(aCount, 1);
}

#[test]
fn BRepGraph_ReverseIteratorTest__FacesOfWire_BoxWireHasOneFace() {
    let myGraph = SetUp();
    let aWireId = WireId::new(0);
    let aCount = countIterator!(FacesOfWire::new(
        &myGraph,
        &myGraph
            .topo()
            .wires()
            .relations(aWireId)
            .parent_wire_ref_ids
    ));
    assert_eq!(aCount, 1);
}

#[test]
fn BRepGraph_ReverseIteratorTest__WiresOfEdge_BoxEdgeBelongsToTwoWires() {
    let myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let aCount = countIterator!(myGraph.topo().edges().wires_of(anEdgeId));
    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_ReverseIteratorTest__SequentialIteration_SkipsRemovedParent() {
    let mut myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let aFaces = myGraph.topo().edges().faces_of(anEdgeId);
    assert!(aFaces.more());
    let aFirstFace = aFaces.current_id();
    assert_eq!(countIterator!(myGraph.topo().edges().faces_of(anEdgeId)), 2);

    // Remove one parent face.
    remove(&mut myGraph, aFirstFace);

    let aCount = countIterator!(myGraph.topo().edges().faces_of(anEdgeId));
    assert_eq!(aCount, 1);
}

#[test]
fn BRepGraph_ReverseIteratorTest__IndexedAccess_TracksActiveReverseBucket() {
    let mut myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let aFaces = myGraph.topo().edges().faces_of(anEdgeId);
    assert!(aFaces.more());
    let aFirstFace = aFaces.current_id();
    assert_eq!(countIterator!(myGraph.topo().edges().faces_of(anEdgeId)), 2);

    remove(&mut myGraph, aFirstFace);

    let mut anIt = myGraph.topo().edges().faces_of(anEdgeId);
    // Reverse buckets are maintained as active relationships after removals.
    assert!(anIt.more());
    assert!(anIt.current_id().is_valid_for(myGraph.topo().faces().nb()));
    anIt.next();
    assert!(!anIt.more());
}

#[test]
fn BRepGraph_ReverseIteratorTest__RangeFor_WorksCorrectly() {
    let myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let mut aCount: u32 = 0;
    for aFaceId in myGraph.topo().edges().faces_of(anEdgeId) {
        assert!(aFaceId.is_valid_for(myGraph.topo().faces().nb()));
        aCount += 1;
    }
    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_ReverseIteratorTest__RefsShellsOfFace_ReturnsValidRefId() {
    let myGraph = SetUp();
    let aFaceId = FaceId::new(0);
    let aShellId = ShellId::new(0);
    let anIt = RefsShellsOfFace::new(
        &myGraph,
        &myGraph
            .topo()
            .faces()
            .relations(aFaceId)
            .parent_face_ref_ids,
    );
    assert!(anIt.more());
    assert_eq!(anIt.current_parent_id(), aShellId);
    assert!(anIt.current_ref_id().is_valid());
}

#[test]
fn BRepGraph_ReverseIteratorTest__RefsEdgesOfVertex_ReturnsValidRefId() {
    let myGraph = SetUp();
    let aVertexId = VertexId::new(0);
    let mut aCount: u32 = 0;
    let mut anIt = RefsEdgesOfVertex::new(
        &myGraph,
        myGraph.topo().vertices().edges(aVertexId),
        aVertexId,
    );
    while anIt.more() {
        assert!(
            anIt.current_parent_id()
                .is_valid_for(myGraph.topo().edges().nb())
        );
        assert!(anIt.current_ref_id().is_valid());
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 3);
}

#[test]
fn BRepGraph_ReverseIteratorTest__CoEdgesOfEdge_BoxEdgeHasCoEdges() {
    let myGraph = SetUp();
    // Each edge of a box is used in 2 coedges (one per adjacent face).
    let anEdgeId = EdgeId::new(0);
    let aCount = countIterator!(CoEdgesOfEdge::new(
        &myGraph,
        myGraph.topo().edges().co_edges(anEdgeId)
    ));
    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_ReverseIteratorTest__WiresOfCoEdge_BoxCoEdgeBelongsToOneWire() {
    let myGraph = SetUp();
    let aCoEdgeId = CoEdgeId::new(0);
    assert!(myGraph.topo().co_edges().wire(aCoEdgeId).is_valid());
}

#[test]
fn BRepGraph_ReverseIteratorTest__Definition_ReturnsFaceDefinition() {
    let myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let anIt = myGraph.topo().edges().faces_of(anEdgeId);
    assert!(anIt.more());

    let aFaceId = anIt.current_id();
    assert!(!anIt.current_id().is_removed(&myGraph));
    // Face must have at least one wire.
    assert!(
        !myGraph
            .topo()
            .faces()
            .relations(aFaceId)
            .wire_ref_ids
            .is_empty()
    );
}

#[test]
fn BRepGraph_ReverseIteratorTest__Definition_ReturnsEdgeDefinition() {
    let myGraph = SetUp();
    let aVertexId = VertexId::new(0);
    let anIt = EdgesOfVertex::new(&myGraph, myGraph.topo().vertices().edges(aVertexId));
    assert!(anIt.more());

    let anEdgeDef = anIt.definition();
    assert!(!anIt.current_id().is_removed(&myGraph));
    assert!(anEdgeDef.start_vertex_ref_id.is_valid());
    assert!(anEdgeDef.end_vertex_ref_id.is_valid());
}

#[test]
fn BRepGraph_ReverseIteratorTest__SkipsRemovedParent_EdgesOfVertex() {
    let mut myGraph = SetUp();
    let aVertexId = VertexId::new(0);
    let anEdges = myGraph.topo().vertices().edges(aVertexId);
    // Box vertex touches 3 edges.
    assert_eq!(anEdges.len(), 3);
    let aFirstEdge = anEdges[0];

    remove(&mut myGraph, aFirstEdge);

    let anEdges = myGraph.topo().vertices().edges(aVertexId);
    let aCount = countIterator!(EdgesOfVertex::new(&myGraph, anEdges));
    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_ReverseIteratorTest__SkipsRemovedParent_AllRemoved() {
    let mut myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let mut aFaces = myGraph.topo().edges().faces_of(anEdgeId);
    assert!(aFaces.more());
    let aFirstFace = aFaces.current_id();
    aFaces.next();
    assert!(aFaces.more());
    let aSecondFace = aFaces.current_id();
    assert_eq!(countIterator!(myGraph.topo().edges().faces_of(anEdgeId)), 2);

    remove(&mut myGraph, aFirstFace);
    remove(&mut myGraph, aSecondFace);

    let aCount = countIterator!(myGraph.topo().edges().faces_of(anEdgeId));
    assert_eq!(aCount, 0);
}

#[test]
fn BRepGraph_ReverseIteratorTest__StartingIndex_SkipsToPosition() {
    let myGraph = SetUp();
    let aVertexId = VertexId::new(0);
    let anEdges = myGraph.topo().vertices().edges(aVertexId);
    assert_eq!(anEdges.len(), 3);

    // Start at index 1 - should yield only edges at index >= 1.
    let anIt = EdgesOfVertex::starting_at(&myGraph, anEdges, 1);
    let aCount = countIterator!(anIt);
    assert_eq!(aCount, 2);
}

// The graph of `CompoundsOfChild_AllTopologyKinds`: one node of every topology kind,
// each a child of one compound.
struct AllKinds {
    aV0: VertexId,
    aCoEdge: CoEdgeId,
    anEdge: EdgeId,
    aWire: WireId,
    aFace: FaceId,
    aShell: ShellId,
    aSolid: SolidId,
    aCompSolid: CompSolidId,
    aNestedCompound: CompoundId,
    aCompound: CompoundId,
}

fn build_all_kinds(aGraph: &mut BRepGraph) -> AllKinds {
    let aV0 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    let aV1 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1.0e-7);
    assert!(aV0.is_valid());
    assert!(aV1.is_valid());

    let anEdge = aGraph
        .editor()
        .edges()
        .add(aV0, aV1, &Handle::default(), 0.0, 1.0, 1.0e-7);
    assert!(anEdge.is_valid());

    let aCoEdgeIds = vec![
        aGraph
            .editor()
            .co_edges()
            .add(anEdge, ParityOrientation::FORWARD),
    ];
    let aWire = aGraph.editor().wires().add(&aCoEdgeIds);
    assert!(aWire.is_valid());
    assert_eq!(aGraph.topo().edges().co_edges(anEdge).len(), 1);
    let aCoEdge = aGraph.topo().edges().co_edges(anEdge)[0];

    let aFace = aGraph
        .editor()
        .faces()
        .add(&Handle::default(), aWire, &[], 1.0e-7);
    assert!(aFace.is_valid());
    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());
    let aFaceRef = aGraph
        .editor()
        .shells()
        .append(aShell, aFace, ParityOrientation::FORWARD);
    assert!(aFaceRef.unwrap().is_valid());
    let aSolid = aGraph.editor().solids().add();
    assert!(aSolid.is_valid());
    let aShellRef = aGraph
        .editor()
        .solids()
        .append(aSolid, aShell, ParityOrientation::FORWARD);
    assert!(aShellRef.unwrap().is_valid());

    let aSolids = vec![aSolid];
    let aCompSolid = aGraph.editor().comp_solids().add(&aSolids);
    assert!(aCompSolid.is_valid());

    let aNestedChildren: Vec<NodeId> = vec![aSolid.into()];
    let aNestedCompound = aGraph.editor().compounds().add(&aNestedChildren);
    assert!(aNestedCompound.is_valid());

    let aChildren: Vec<NodeId> = vec![
        aV0.into(),
        aCoEdge.into(),
        anEdge.into(),
        aWire.into(),
        aFace.into(),
        aShell.into(),
        aSolid.into(),
        aCompSolid.into(),
        aNestedCompound.into(),
    ];
    let aCompound = aGraph.editor().compounds().add(&aChildren);
    assert!(aCompound.is_valid());
    AllKinds {
        aV0,
        aCoEdge,
        anEdge,
        aWire,
        aFace,
        aShell,
        aSolid,
        aCompSolid,
        aNestedCompound,
        aCompound,
    }
}

// One parent iteration: the expected compounds in order, then the end.
fn expect_compound_parents(aGraph: &BRepGraph, aChild: NodeId, anExpected: &[CompoundId]) {
    let mut aParents =
        CompoundsOfChild::new(aGraph, aGraph.topo().generic().compound_ref_ids(aChild));
    for anId in anExpected {
        assert!(aParents.more());
        assert_eq!(aParents.current_id(), *anId);
        aParents.next();
    }
    assert!(!aParents.more());
}

#[test]
fn BRepGraph_ReverseIteratorTest__CompoundsOfChild_AllTopologyKinds() {
    let _myGraph = SetUp();
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let k = build_all_kinds(&mut aGraph);

    // `BRepGraph_CompoundsOfVertex`, `...OfCoEdge`, `...OfEdge`, `...OfWire`,
    // `...OfFace`, `...OfShell`, `...OfSolid`, `...OfCompSolid` and `...OfCompound` are
    // one Rust type (`CompoundsOfChild`), as their traits are one native trait.
    expect_compound_parents(&aGraph, k.aV0.into(), &[k.aCompound]);
    expect_compound_parents(&aGraph, k.aCoEdge.into(), &[k.aCompound]);
    expect_compound_parents(&aGraph, k.anEdge.into(), &[k.aCompound]);
    expect_compound_parents(&aGraph, k.aWire.into(), &[k.aCompound]);
    expect_compound_parents(&aGraph, k.aFace.into(), &[k.aCompound]);
    expect_compound_parents(&aGraph, k.aShell.into(), &[k.aCompound]);
    expect_compound_parents(&aGraph, k.aSolid.into(), &[k.aNestedCompound, k.aCompound]);
    expect_compound_parents(&aGraph, k.aCompSolid.into(), &[k.aCompound]);
    expect_compound_parents(&aGraph, k.aNestedCompound.into(), &[k.aCompound]);

    let mut aGenericParents = CompoundsOfChild::new(
        &aGraph,
        aGraph.topo().generic().compound_ref_ids(k.aCoEdge.into()),
    );
    assert!(aGenericParents.more());
    assert_eq!(aGenericParents.current_id(), k.aCompound);
    aGenericParents.next();
    assert!(!aGenericParents.more());

    let mut aFaceParentsOfWire = FacesOfWire::new(
        &aGraph,
        &aGraph.topo().wires().relations(k.aWire).parent_wire_ref_ids,
    );
    assert!(aFaceParentsOfWire.more());
    assert_eq!(aFaceParentsOfWire.current_id(), k.aFace);
    aFaceParentsOfWire.next();
    assert!(!aFaceParentsOfWire.more());
}

#[test]
fn BRepGraph_ReverseIteratorTest__OccurrencesOfChild_ProductAndTopologyChildren() {
    let _myGraph = SetUp();
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph.shapes_mut().add(&make_box(1.0, 1.0, 1.0)).unwrap();
    assert!(!aGraph.is_empty());

    let aChildren: Vec<NodeId> = vec![SolidId::start().into()];
    let aCompound = aGraph.editor().compounds().add(&aChildren);
    assert!(aCompound.is_valid());

    let aPart = aGraph
        .editor()
        .products()
        .add(aCompound.into(), &Location::default());
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
    let (aPartOccurrence, _) = aGraph
        .editor()
        .products()
        .append(
            anAssembly,
            aPart,
            &Location::default(),
            OccurrenceId::default(),
        )
        .unwrap();
    assert!(aPartOccurrence.is_valid());

    let mut aTopologyOccurrences = OccurrencesOfChild::new(
        &aGraph,
        aGraph.topo().generic().occurrence_ref_ids(aCompound.into()),
    );
    assert!(aTopologyOccurrences.more());
    let aCompoundOccurrence = aTopologyOccurrences.current_id();
    assert!(aCompoundOccurrence.is_valid());
    assert_eq!(
        aGraph
            .topo()
            .occurrences()
            .definition(aCompoundOccurrence)
            .child_node_id,
        NodeId::from(aCompound)
    );
    aTopologyOccurrences.next();
    assert!(!aTopologyOccurrences.more());

    let mut aProductOccurrences = OccurrencesOfChild::new(
        &aGraph,
        aGraph.topo().generic().occurrence_ref_ids(aPart.into()),
    );
    assert!(aProductOccurrences.more());
    assert_eq!(aProductOccurrences.current_id(), aPartOccurrence);
    aProductOccurrences.next();
    assert!(!aProductOccurrences.more());
}
