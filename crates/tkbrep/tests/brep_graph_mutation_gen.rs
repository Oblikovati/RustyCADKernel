//! Direct translation of BRepGraph_MutationGen_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). Where the native test mutates the
//! graph inside a loop over one of its iterators and returns after the first step,
//! the first id is taken before the mutation.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::iterator::*;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkmath::toploc::Location;
use support::make_box;

/// The `BRepGraph_MutationGenTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!myGraph.is_empty());
    myGraph
}

fn edge_gens(aGraph: &BRepGraph, anEdge: EdgeId) -> (u32, u32) {
    let aBase = &aGraph.topo().edges().definition(anEdge).base;
    (aBase.own_gen, aBase.subtree_gen)
}

fn set_edge_tolerance(aGraph: &mut BRepGraph, aTolerance: f64) {
    aGraph
        .editor()
        .edges()
        .set_tolerance(EdgeId::start(), aTolerance)
        .unwrap();
}

#[test]
fn BRepGraph_MutationGenTest__OwnGen_IncrementedOnMutation() {
    let mut myGraph = SetUp();
    assert_eq!(edge_gens(&myGraph, EdgeId::start()), (0, 0));

    set_edge_tolerance(&mut myGraph, 0.5);

    assert_eq!(edge_gens(&myGraph, EdgeId::start()), (1, 1));
}

#[test]
fn BRepGraph_MutationGenEditorTest__EditorCreationRegistersOwnGeneration() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();

    let aV0 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    let aV1 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1.0e-7);
    let anEdge = aGraph
        .editor()
        .edges()
        .add(aV0, aV1, &Handle::default(), 0.0, 1.0, 1.0e-7);
    let aCoEdge = aGraph
        .editor()
        .co_edges()
        .add(anEdge, ParityOrientation::FORWARD);

    let aCoEdges = vec![aCoEdge];
    let aWire = aGraph.editor().wires().add(&aCoEdges);

    let anInnerWires: Vec<WireId> = Vec::new();
    let aPlane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).into_cast();
    let aFace = aGraph
        .editor()
        .faces()
        .add(&aPlane, aWire, &anInnerWires, 1.0e-7);
    let aShell = aGraph.editor().shells().add();
    let aSolid = aGraph.editor().solids().add();

    let aChildren: Vec<NodeId> = vec![aSolid.into()];
    let aCompound = aGraph.editor().compounds().add(&aChildren);

    let aSolids = vec![aSolid];
    let aCompSolid = aGraph.editor().comp_solids().add(&aSolids);

    let aTopo = aGraph.topo();
    assert_eq!(aTopo.vertices().definition(aV0).base.own_gen, 1);
    assert_eq!(aTopo.vertices().definition(aV1).base.own_gen, 1);
    assert_eq!(aTopo.edges().definition(anEdge).base.own_gen, 3);
    assert_eq!(aTopo.co_edges().definition(aCoEdge).base.own_gen, 2);
    assert_eq!(aTopo.wires().definition(aWire).base.own_gen, 1);
    assert_eq!(aTopo.faces().definition(aFace).base.own_gen, 1);
    assert_eq!(aTopo.shells().definition(aShell).base.own_gen, 1);
    assert_eq!(aTopo.solids().definition(aSolid).base.own_gen, 1);
    assert_eq!(aTopo.compounds().definition(aCompound).base.own_gen, 1);
    assert_eq!(aTopo.comp_solids().definition(aCompSolid).base.own_gen, 1);
    assert!(aGraph.validate_relations());
}

#[test]
fn BRepGraph_MutationGenEditorTest__ProductAppendRegistersBothProductsAndOccurrence() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();

    let aParent = aGraph.editor().products().add_empty();
    let aChild = aGraph.editor().products().add_empty();
    aGraph
        .editor()
        .products()
        .append_document_root(aChild)
        .unwrap();

    let aParentStamp = aGraph.uids().stamp_of_node(aParent.into());
    let aChildStamp = aGraph.uids().stamp_of_node(aChild.into());
    let aParentOwnGen = aGraph.topo().products().definition(aParent).base.own_gen;
    let aChildOwnGen = aGraph.topo().products().definition(aChild).base.own_gen;

    let (anOccurrence, _) = aGraph
        .editor()
        .products()
        .append(
            aParent,
            aChild,
            &Location::default(),
            OccurrenceId::default(),
        )
        .unwrap();

    assert!(anOccurrence.is_valid());
    assert!(aGraph.uids().is_stale(&aParentStamp));
    assert!(aGraph.uids().is_stale(&aChildStamp));
    assert_eq!(
        aGraph
            .topo()
            .occurrences()
            .definition(anOccurrence)
            .base
            .own_gen,
        1
    );
    assert_eq!(
        aGraph.topo().products().definition(aParent).base.own_gen,
        aParentOwnGen + 1
    );
    assert_eq!(
        aGraph.topo().products().definition(aChild).base.own_gen,
        aChildOwnGen + 1
    );
    assert!(aGraph.validate_relations());
}

#[test]
fn BRepGraph_MutationGenTest__OwnGen_MultipleIncrements() {
    let mut myGraph = SetUp();
    set_edge_tolerance(&mut myGraph, 0.1);
    set_edge_tolerance(&mut myGraph, 0.2);

    assert_eq!(edge_gens(&myGraph, EdgeId::start()).0, 2);
}

#[test]
fn BRepGraph_MutationGenTest__OwnGen_DeferredMode() {
    let mut myGraph = SetUp();
    myGraph.editor().begin_deferred_invalidation();
    set_edge_tolerance(&mut myGraph, 0.5);

    // OwnGen is incremented even in deferred mode.
    assert_eq!(edge_gens(&myGraph, EdgeId::start()).0, 1);

    myGraph.editor().end_deferred_invalidation();

    // Still 1 after flush - flush doesn't re-increment.
    assert_eq!(edge_gens(&myGraph, EdgeId::start()).0, 1);
}

/// `(any SubtreeGen > 0, every OwnGen == 0)` over the active nodes of one kind.
fn subtree_only<T: rustycad_tkbrep::brep_graph::NodeTag>(aGraph: &BRepGraph) -> (bool, bool)
where
    T::Def: rustycad_tkbrep::brep_graph_inc::HasBaseDef,
{
    use rustycad_tkbrep::brep_graph_inc::HasBaseDef;
    let mut anyIncremented = false;
    let mut allOwnZero = true;
    let mut anIt = DefIterator::<T>::new(aGraph);
    while anIt.more() {
        let aBase = anIt.current().base();
        anyIncremented |= aBase.subtree_gen > 0;
        allOwnZero &= aBase.own_gen == 0;
        anIt.next();
    }
    (anyIncremented, allOwnZero)
}

#[test]
fn BRepGraph_MutationGenTest__SubtreeGen_PropagatedParent_Incremented() {
    let mut myGraph = SetUp();
    // Mutate an edge - parent wire/face/shell/solid get SubtreeGen incremented
    // via propagation, enabling generation-based cache freshness on parents.
    // Parent OwnGen must NOT change (only the edge itself was directly mutated).
    set_edge_tolerance(&mut myGraph, 0.5);

    assert_eq!(edge_gens(&myGraph, EdgeId::start()), (1, 1));

    // At least one parent in each level must have SubtreeGen incremented,
    // but OwnGen must remain 0 (parent's own data was not touched).
    assert_eq!(subtree_only::<WireTag>(&myGraph), (true, true));
    assert_eq!(subtree_only::<FaceTag>(&myGraph), (true, true));
    assert_eq!(subtree_only::<ShellTag>(&myGraph), (true, true));
    assert_eq!(subtree_only::<SolidTag>(&myGraph), (true, true));
}

/// The SubtreeGen of every active node of one kind, in index order.
fn subtree_gens<T: rustycad_tkbrep::brep_graph::NodeTag>(aGraph: &BRepGraph) -> Vec<(u32, u32)> {
    use rustycad_tkbrep::brep_graph_inc::HasBaseDef;
    let mut aGens = Vec::new();
    let mut anIt = DefIterator::<T>::new(aGraph);
    while anIt.more() {
        aGens.push((anIt.current_id().index, anIt.current().base().subtree_gen));
        anIt.next();
    }
    aGens
}

#[test]
fn BRepGraph_MutationGenTest__SubtreeGen_DeferredPropagatedParent_Incremented() {
    let mut myGraph = SetUp();
    // Store baselines before mutation.
    let aEdgeOwnGenBefore = edge_gens(&myGraph, EdgeId::start()).0;
    let aWireSubtreeGensBefore = subtree_gens::<WireTag>(&myGraph);
    let aFaceSubtreeGensBefore = subtree_gens::<FaceTag>(&myGraph);

    // Deferred mutation + flush.
    myGraph.editor().begin_deferred_invalidation();
    set_edge_tolerance(&mut myGraph, 0.5);
    myGraph.editor().end_deferred_invalidation();

    // Directly mutated edge: OwnGen incremented by exactly 1.
    assert_eq!(
        edge_gens(&myGraph, EdgeId::start()).0,
        aEdgeOwnGenBefore + 1
    );

    // At least one parent wire must have SubtreeGen incremented vs its baseline.
    let anyAbove = |aNow: Vec<(u32, u32)>, aBefore: &[(u32, u32)]| {
        aNow.iter()
            .any(|(anIndex, aGen)| *aGen > aBefore[*anIndex as usize].1)
    };
    assert!(anyAbove(
        subtree_gens::<WireTag>(&myGraph),
        &aWireSubtreeGensBefore
    ));

    // At least one parent face must have SubtreeGen incremented vs its baseline.
    assert!(anyAbove(
        subtree_gens::<FaceTag>(&myGraph),
        &aFaceSubtreeGensBefore
    ));
}

// The root product and its first occurrence, with their SubtreeGens.
fn root_context(myGraph: &BRepGraph) -> (ProductId, OccurrenceId, u32, u32) {
    assert_eq!(myGraph.root_product_ids().len(), 1);
    let aRootProduct = myGraph.root_product_ids()[0];
    let aRootOccurrence = myGraph.topo().products().component(aRootProduct, 0);
    assert!(aRootOccurrence.is_valid());
    let anOccurrenceGen = myGraph
        .topo()
        .occurrences()
        .definition(aRootOccurrence)
        .base
        .subtree_gen;
    let aProductGen = myGraph
        .topo()
        .products()
        .definition(aRootProduct)
        .base
        .subtree_gen;
    (aRootProduct, aRootOccurrence, anOccurrenceGen, aProductGen)
}

fn append_new_shell(myGraph: &mut BRepGraph) {
    let aShell = myGraph.editor().shells().add();
    assert!(aShell.is_valid());
    let aShellRef = myGraph
        .editor()
        .solids()
        .append(SolidId::start(), aShell, ParityOrientation::FORWARD)
        .unwrap();
    assert!(aShellRef.is_valid());
}

#[test]
fn BRepGraph_MutationGenTest__SubtreeGen_SolidMutation_PropagatesThroughRootOccurrence() {
    let mut myGraph = SetUp();
    let (aRootProduct, aRootOccurrence, anOccurrenceSubtreeGenBefore, aProductSubtreeGenBefore) =
        root_context(&myGraph);

    append_new_shell(&mut myGraph);

    assert!(
        myGraph
            .topo()
            .occurrences()
            .definition(aRootOccurrence)
            .base
            .subtree_gen
            > anOccurrenceSubtreeGenBefore
    );
    assert!(
        myGraph
            .topo()
            .products()
            .definition(aRootProduct)
            .base
            .subtree_gen
            > aProductSubtreeGenBefore
    );
}

#[test]
fn BRepGraph_MutationGenTest__SubtreeGen_DeferredSolidMutation_PropagatesThroughRootOccurrence() {
    let mut myGraph = SetUp();
    let (aRootProduct, aRootOccurrence, anOccurrenceSubtreeGenBefore, aProductSubtreeGenBefore) =
        root_context(&myGraph);

    myGraph.editor().begin_deferred_invalidation();
    append_new_shell(&mut myGraph);

    assert_eq!(
        myGraph
            .topo()
            .occurrences()
            .definition(aRootOccurrence)
            .base
            .subtree_gen,
        anOccurrenceSubtreeGenBefore
    );
    assert_eq!(
        myGraph
            .topo()
            .products()
            .definition(aRootProduct)
            .base
            .subtree_gen,
        aProductSubtreeGenBefore
    );

    myGraph.editor().end_deferred_invalidation();

    assert!(
        myGraph
            .topo()
            .occurrences()
            .definition(aRootOccurrence)
            .base
            .subtree_gen
            > anOccurrenceSubtreeGenBefore
    );
    assert!(
        myGraph
            .topo()
            .products()
            .definition(aRootProduct)
            .base
            .subtree_gen
            > aProductSubtreeGenBefore
    );
}

fn face_gens(aGraph: &BRepGraph, aFace: FaceId) -> (u32, u32) {
    let aBase = &aGraph.topo().faces().definition(aFace).base;
    (aBase.own_gen, aBase.subtree_gen)
}

#[test]
fn BRepGraph_MutationGenTest__RepMutation_SurfacePropagatesSubtreeGenToFace() {
    let mut myGraph = SetUp();
    let aFaceId = FaceId::new(0);
    assert_eq!(face_gens(&myGraph, aFaceId), (0, 0));

    myGraph.editor().faces().clear_surface(aFaceId).unwrap();

    assert!(face_gens(&myGraph, aFaceId).0 > 0);
    assert!(face_gens(&myGraph, aFaceId).1 > 0);
}

#[test]
fn BRepGraph_MutationGenTest__RepMutation_Curve3DPropagatesSubtreeGenToEdge() {
    let mut myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    assert_eq!(edge_gens(&myGraph, anEdgeId), (0, 0));

    myGraph.editor().edges().clear_curve(anEdgeId).unwrap();

    assert!(edge_gens(&myGraph, anEdgeId).0 > 0);
    assert!(edge_gens(&myGraph, anEdgeId).1 > 0);
}

#[test]
fn BRepGraph_MutationGenTest__RepMutation_Curve2DPropagatesSubtreeGenToCoEdge() {
    let mut myGraph = SetUp();
    let aCoEdgeIt = CoEdgeIterator::new(&myGraph);
    if !aCoEdgeIt.more() {
        return;
    }
    let aCoEdgeId = aCoEdgeIt.current_id();
    assert_eq!(aCoEdgeIt.current().base.own_gen, 0);
    assert_eq!(aCoEdgeIt.current().base.subtree_gen, 0);

    myGraph.editor().co_edges().clear_pcurve(aCoEdgeId).unwrap();

    assert!(myGraph.topo().co_edges().definition(aCoEdgeId).base.own_gen > 0);
    assert!(
        myGraph
            .topo()
            .co_edges()
            .definition(aCoEdgeId)
            .base
            .subtree_gen
            > 0
    );
}

#[test]
fn BRepGraph_MutationGenTest__RepMutation_TriangulationPropagatesSubtreeGenToFace() {
    let mut myGraph = SetUp();
    let aFaceIt = FaceIterator::new(&myGraph);
    if !aFaceIt.more() {
        return;
    }
    let aFaceId = aFaceIt.current_id();
    assert_eq!(aFaceIt.current().base.own_gen, 0);
    assert_eq!(aFaceIt.current().base.subtree_gen, 0);

    myGraph
        .editor()
        .faces()
        .clear_persistent_triangulation(aFaceId)
        .unwrap();

    assert!(face_gens(&myGraph, aFaceId).0 > 0);
    assert!(face_gens(&myGraph, aFaceId).1 > 0);
}

#[test]
fn BRepGraph_MutationGenTest__RepMutation_Polygon3DPropagatesSubtreeGenToEdge() {
    let mut myGraph = SetUp();
    let anEdgeIt = EdgeIterator::new(&myGraph);
    if !anEdgeIt.more() {
        return;
    }
    let anEdgeId = anEdgeIt.current_id();
    assert_eq!(anEdgeIt.current().base.own_gen, 0);
    assert_eq!(anEdgeIt.current().base.subtree_gen, 0);

    myGraph
        .editor()
        .edges()
        .clear_persistent_polygon3d(anEdgeId)
        .unwrap();

    assert!(edge_gens(&myGraph, anEdgeId).0 > 0);
    assert!(edge_gens(&myGraph, anEdgeId).1 > 0);
}
