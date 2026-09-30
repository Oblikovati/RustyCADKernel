//! Direct translation of BRepGraph_RefId_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). The `GTEST_SKIP` branches are kept as
//! early returns; a box graph never takes them.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::editor::FaceOps;
use rustycad_tkbrep::brep_graph::iterator::*;
use rustycad_tkbrep::brep_graph::reverse_iterator::RefsShellsOfFace;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use support::make_box;
use support::ref_test_tools as BRepGraph_TestTools;

// The sum of a count over the active nodes of one kind.
fn sum_over<T: NodeTag>(theGraph: &BRepGraph, theCount: impl Fn(TypedId<T>) -> u32) -> u32 {
    let mut aNb = 0;
    let mut anIt = DefIterator::<T>::new(theGraph);
    while anIt.more() {
        aNb += theCount(anIt.current_id());
        anIt.next();
    }
    aNb
}

fn countInlineFaceRefs(theGraph: &BRepGraph) -> u32 {
    sum_over::<ShellTag>(theGraph, |aShell| {
        BRepGraph_TestTools::count_face_refs_of_shell(theGraph, aShell)
    })
}

fn countInlineWireRefs(theGraph: &BRepGraph) -> u32 {
    sum_over::<FaceTag>(theGraph, |aFace| {
        BRepGraph_TestTools::count_wire_refs_of_face(theGraph, aFace)
    })
}

fn countInlineCoEdges(theGraph: &BRepGraph) -> u32 {
    sum_over::<WireTag>(theGraph, |aWire| {
        BRepGraph_TestTools::count_co_edges_of_wire(theGraph, aWire)
    })
}

fn countInlineVertexRefs(theGraph: &BRepGraph) -> u32 {
    sum_over::<EdgeTag>(theGraph, |anEdge| {
        let aDef = theGraph.topo().edges().definition(anEdge);
        u32::from(aDef.start_vertex_ref_id.is_valid())
            + u32::from(aDef.end_vertex_ref_id.is_valid())
    })
}

fn countInlineShellRefs(theGraph: &BRepGraph) -> u32 {
    sum_over::<SolidTag>(theGraph, |aSolid| {
        BRepGraph_TestTools::count_shell_refs_of_solid(theGraph, aSolid)
    })
}

fn countInlineSolidRefs(theGraph: &BRepGraph) -> u32 {
    sum_over::<CompSolidTag>(theGraph, |aCS| {
        BRepGraph_TestTools::count_solid_refs_of_comp_solid(theGraph, aCS)
    })
}

fn countInlineChildRefs(theGraph: &BRepGraph) -> u32 {
    let aCount = |aNode: NodeId| BRepGraph_TestTools::count_child_refs_of_parent(theGraph, aNode);
    sum_over::<CompoundTag>(theGraph, |aComp| aCount(aComp.into()))
        + sum_over::<ShellTag>(theGraph, |aShell| aCount(aShell.into()))
        + sum_over::<SolidTag>(theGraph, |aSolid| aCount(aSolid.into()))
}

fn boxGraph(dx: f64, dy: f64, dz: f64) -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph.shapes_mut().add(&make_box(dx, dy, dz)).unwrap();
    assert!(!aGraph.is_empty());
    aGraph
}

#[test]
fn BRepGraph_RefIdTest__DefaultRefId_IsInvalid() {
    let aRefId = RefId::default();
    assert!(!aRefId.is_valid());
}

#[test]
fn BRepGraph_RefIdTest__TypedRefId_ConvertsToUntyped() {
    let aFaceRefId = FaceRefId::new(3);
    let anUntyped: RefId = aFaceRefId.into();
    assert_eq!(anUntyped.kind, RefKind::FACE);
    assert_eq!(anUntyped.index, 3);
}

#[test]
fn BRepGraph_RefIdTest__UntypedArithmetic_PreservesKindAndIndex() {
    let mut aRefId = RefId::new(RefKind::FACE, 1);

    let aPrev = aRefId.post_increment();
    assert_eq!(aPrev.kind, RefKind::FACE);
    assert_eq!(aPrev.index, 1);
    assert_eq!(aRefId.index, 2);

    aRefId.pre_increment();
    assert_eq!(aRefId.index, 3);

    let anAdvanced = aRefId.offset(4);
    assert_eq!(anAdvanced.kind, RefKind::FACE);
    assert_eq!(anAdvanced.index, 7);

    let aRetreated = anAdvanced.retreat(6);
    assert_eq!(aRetreated.kind, RefKind::FACE);
    assert_eq!(aRetreated.index, 1);
}

#[test]
fn BRepGraph_RefIdTest__TypedArithmetic_PreservesKindAndIndex() {
    let mut aFaceRefId = FaceRefId::new(1);

    let aPrev = aFaceRefId.post_increment();
    assert_eq!(aPrev.index, 1);
    assert_eq!(aFaceRefId.index, 2);

    aFaceRefId.pre_increment();
    assert_eq!(aFaceRefId.index, 3);

    let anAdvanced = aFaceRefId.offset(4);
    assert_eq!(anAdvanced.index, 7);

    let aRetreated = anAdvanced.retreat(6);
    assert_eq!(aRetreated.index, 1);

    // Verify kind is preserved through implicit conversion.
    let aRefId: RefId = anAdvanced.into();
    assert_eq!(aRefId.kind, RefKind::FACE);
    assert_eq!(aRefId.index, 7);
}

#[test]
fn BRepGraph_RefIdTest__TypedArithmetic_IndexZeroBoundary() {
    let mut aVertexRef = VertexRefId::new(0);
    assert!(aVertexRef.is_valid());

    // Increment from zero.
    aVertexRef.pre_increment();
    assert_eq!(aVertexRef.index, 1);

    // Retreat back to zero - still valid.
    let aZero = aVertexRef.retreat(1);
    assert_eq!(aZero.index, 0);
    assert!(aZero.is_valid());

    // Subtract to the invalid sentinel produces an invalid id.
    let anInvalid = aZero.retreat(1);
    assert_eq!(anInvalid.index, VertexRefId::INVALID_INDEX);
    assert!(!anInvalid.is_valid());
}

#[test]
fn BRepGraph_RefIdTest__InvalidKind_IsRejected() {
    const THE_RESERVED_KIND: i32 = 7;
    let aRef = RefId::new(RefKind(THE_RESERVED_KIND), 0);

    assert!(!RefKind::is_valid_kind(aRef.kind));
    assert!(!aRef.is_valid());
    assert!(!aRef.is_valid_for(10));
    assert!(!RefKind::is_topology_ref_kind(aRef.kind));
}

#[test]
fn BRepGraph_RefIdTest__FromRefId_WrongKindReturnsInvalid() {
    let aRef = RefId::new(RefKind::FACE, 3);
    let aWire = WireRefId::from_ref_id(aRef);

    assert!(!aWire.is_valid());
    assert_eq!(aWire.index, WireRefId::INVALID_INDEX);
}

#[test]
fn BRepGraph_RefIdTest__RefUID_Default_IsInvalid() {
    let aRefUID = RefUid::default();
    assert!(!aRefUID.is_valid());
}

#[test]
fn BRepGraph_RefIdTest__RefUID_Equality_IgnoresGeneration() {
    let aUIDV1 = RefUid::new(RefKind::FACE, 42);
    let aUIDV2 = RefUid::new(RefKind::FACE, 42);
    assert_eq!(aUIDV1, aUIDV2);
}

#[test]
fn BRepGraph_RefIdTest__RefsView_AfterBuild_HasFaceRefs() {
    let aGraph = boxGraph(10.0, 20.0, 30.0);
    let aFaceRefCount = aGraph.refs().faces().nb();

    if aFaceRefCount > 0 {
        let aFaceRefId = FaceRefId::new(0);
        let _anEntry = aGraph.refs().faces().entry(aFaceRefId);
        assert!(aFaceRefId.is_valid_for(aFaceRefCount));
    } else {
        assert!(!aGraph.uids().of_ref(FaceRefId::start().into()).is_valid());
    }
}

#[test]
fn BRepGraph_RefIdTest__RefDomain_StampGUIDGeneration_IfSupported() {
    let aGraph = boxGraph(10.0, 20.0, 30.0);

    if aGraph.refs().faces().nb() == 0 {
        return; // GTEST_SKIP: Ref-domain entries are not populated at this phase
    }

    let aStamp = aGraph.uids().stamp_of_ref(FaceRefId::start().into());
    if !aStamp.is_valid() {
        return; // GTEST_SKIP: Ref-domain stamping/GUID generation not wired yet
    }

    assert!(aStamp.is_ref_stamp());

    let aGraphGUID = aGraph.uids().graph_guid();
    let aGUID1 = aStamp.to_guid(aGraphGUID);
    let aGUID2 = aStamp.to_guid(aGraphGUID);
    assert_eq!(aGUID1, aGUID2);
}

#[test]
fn BRepGraph_RefIdTest__RefsView_AfterBuild_CountsDoNotExceedInlineStorage() {
    let aGraph = boxGraph(10.0, 20.0, 30.0);

    assert!(aGraph.refs().faces().nb() <= countInlineFaceRefs(&aGraph));
    assert!(aGraph.refs().wires().nb() <= countInlineWireRefs(&aGraph));
    assert!(countInlineCoEdges(&aGraph) > 0);
    assert_eq!(aGraph.topo().co_edges().nb(), countInlineCoEdges(&aGraph));
    assert!(aGraph.refs().vertices().nb() <= countInlineVertexRefs(&aGraph));
    assert!(aGraph.refs().shells().nb() <= countInlineShellRefs(&aGraph));
    assert!(aGraph.refs().solids().nb() <= countInlineSolidRefs(&aGraph));
    assert!(aGraph.refs().children().nb() <= countInlineChildRefs(&aGraph));
}

// The UID round trip of every reference of one kind, then the child check of its entry.
fn check_ref_uids<T: RefTag>(aGraph: &BRepGraph, aChildValid: impl Fn(TypedId<T>) -> bool)
where
    TypedId<T>: Into<RefId>,
{
    let mut aRefId = TypedId::<T>::start();
    while aRefId.is_valid_for(aGraph.refs().generic().nb(RefId::from(aRefId).kind)) {
        let anUntyped: RefId = aRefId.into();
        let aUID = aGraph.uids().of_ref(anUntyped);
        assert!(aUID.is_valid());
        assert_eq!(aGraph.uids().ref_id_from(&aUID), anUntyped);
        assert!(aChildValid(aRefId));
        aRefId.pre_increment();
    }
}

#[test]
fn BRepGraph_RefIdTest__RefsView_AfterBuild_UIDRoundtripAndParentKinds() {
    let aGraph = boxGraph(10.0, 20.0, 30.0);
    let aTopo = aGraph.topo();
    let aRefs = aGraph.refs();

    let mut aFaceRefId = FaceRefId::start();
    while aFaceRefId.is_valid_for(aRefs.faces().nb()) {
        let aRefId: RefId = aFaceRefId.into();
        let aUID = aGraph.uids().of_ref(aRefId);
        let aStamp = aGraph.uids().stamp_of_ref(aRefId);
        assert!(aUID.is_valid());
        assert_eq!(aGraph.uids().ref_id_from(&aUID), aRefId);
        assert!(aGraph.uids().has_ref(&aUID));
        assert!(aStamp.is_valid());
        assert!(aStamp.is_ref_stamp());
        assert!(!aGraph.uids().is_stale(&aStamp));

        let anEntry = aRefs.faces().entry(aFaceRefId);
        assert!(anEntry.child_face_id.is_valid_for(aTopo.faces().nb()));
        aFaceRefId.pre_increment();
    }

    check_ref_uids::<WireRefTag>(&aGraph, |r| {
        aRefs
            .wires()
            .entry(r)
            .child_wire_id
            .is_valid_for(aTopo.wires().nb())
    });

    let mut aCoEdgeId = CoEdgeId::start();
    while aCoEdgeId.is_valid_for(aTopo.co_edges().nb()) {
        let aUID = aGraph.uids().of_node(aCoEdgeId.into());
        let anEntry = aTopo.co_edges().definition(aCoEdgeId);
        assert!(aUID.is_valid());
        assert_eq!(aGraph.uids().node_id_from(&aUID), NodeId::from(aCoEdgeId));
        assert!(anEntry.child_edge_id.is_valid_for(aTopo.edges().nb()));
        aCoEdgeId.pre_increment();
    }

    check_ref_uids::<ShellRefTag>(&aGraph, |r| {
        aRefs
            .shells()
            .entry(r)
            .child_shell_id
            .is_valid_for(aTopo.shells().nb())
    });
    check_ref_uids::<VertexRefTag>(&aGraph, |r| {
        aRefs
            .vertices()
            .entry(r)
            .child_vertex_id
            .is_valid_for(aTopo.vertices().nb())
    });
    check_ref_uids::<SolidRefTag>(&aGraph, |r| {
        aRefs
            .solids()
            .entry(r)
            .child_solid_id
            .is_valid_for(aTopo.solids().nb())
    });
    check_ref_uids::<ChildRefTag>(&aGraph, |r| {
        aRefs.children().entry(r).child_node_id.is_valid()
    });
}

#[test]
fn BRepGraph_RefIdTest__RefUIDReverseLookupStaysCurrentAfterProgrammaticAdd() {
    let mut aGraph = BRepGraph::new();

    let aV0 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 0.001);
    let aV1 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 0.001);
    let _ = aGraph
        .editor()
        .edges()
        .add(aV0, aV1, &Handle::default(), 0.0, 1.0, 0.001);

    let aFirstRefId: RefId = VertexRefId::start().into();
    let aFirstUID = aGraph.uids().of_ref(aFirstRefId);
    assert!(aFirstUID.is_valid());
    assert_eq!(aGraph.uids().ref_id_from(&aFirstUID), aFirstRefId);

    let aV2 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(2.0, 0.0, 0.0), 0.001);
    let aV3 = aGraph
        .editor()
        .vertices()
        .add(Point3::new(3.0, 0.0, 0.0), 0.001);
    let _ = aGraph
        .editor()
        .edges()
        .add(aV2, aV3, &Handle::default(), 0.0, 1.0, 0.001);

    let aSecondRefId: RefId = VertexRefId::new(2).into();
    let aSecondUID = aGraph.uids().of_ref(aSecondRefId);
    assert!(aSecondUID.is_valid());

    assert_eq!(aGraph.uids().ref_id_from(&aFirstUID), aFirstRefId);
    assert_eq!(aGraph.uids().ref_id_from(&aSecondUID), aSecondRefId);
    assert!(aGraph.uids().has_ref(&aFirstUID));
    assert!(aGraph.uids().has_ref(&aSecondUID));
}

#[test]
fn BRepGraph_RefIdTest__StaleRefUID_HasReturnsFalseAndLookupBecomesInvalidAfterRebuild() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes5 = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!aGraph.is_empty());
    assert!(aGraph.refs().faces().nb() > 0);

    let anOldUID = aGraph.uids().of_ref(FaceRefId::start().into());
    assert!(anOldUID.is_valid());
    assert!(aGraph.uids().has_ref(&anOldUID));

    aGraph.clear().unwrap();
    let _aBuildRes6 = aGraph
        .shapes_mut()
        .add(&make_box(11.0, 21.0, 31.0))
        .unwrap();

    assert!(!aGraph.uids().has_ref(&anOldUID));
    assert!(!aGraph.uids().ref_id_from(&anOldUID).is_valid());
}

#[test]
fn BRepGraph_RefIdTest__MutFaceRef_UpdatesRefStampAndParentModifiedFlag() {
    let mut aGraph = boxGraph(10.0, 20.0, 30.0);

    if aGraph.refs().faces().nb() == 0 {
        return; // GTEST_SKIP: No face references available
    }

    let aFaceRefId = FaceRefId::new(0);
    let aBeforeStamp = aGraph.uids().stamp_of_ref(aFaceRefId.into());
    assert!(aBeforeStamp.is_valid());
    assert!(aBeforeStamp.is_ref_stamp());
    let anBeforeOri = aGraph.refs().faces().entry(aFaceRefId).orientation;

    {
        let mut aMut = aGraph.editor().faces().mutate_ref(aFaceRefId).unwrap();
        let aNewOri = if anBeforeOri.orientation() == Orientation::Forward {
            ParityOrientation::REVERSED
        } else {
            ParityOrientation::FORWARD
        };
        FaceOps::set_ref_orientation_guarded(&mut aMut, aNewOri);
    }

    let aAfterEntry = aGraph.refs().faces().entry(aFaceRefId);
    assert_ne!(aAfterEntry.orientation, anBeforeOri);
    assert!(aGraph.uids().is_stale(&aBeforeStamp));

    let mut hasUpdatedParent = false;
    let mut anIt = RefsShellsOfFace::new(
        &aGraph,
        &aGraph
            .topo()
            .faces()
            .relations(aAfterEntry.child_face_id)
            .parent_face_ref_ids,
    );
    while anIt.more() {
        if anIt.current_ref_id() == aFaceRefId {
            let aParent = aGraph.topo().shells().definition(anIt.current_parent_id());
            hasUpdatedParent = aParent.base.subtree_gen > 0;
            break;
        }
        anIt.next();
    }
    assert!(hasUpdatedParent);
}

#[test]
fn BRepGraph_RefIdTest__MutFaceRef_MarkRemoved_PersistsAndInvalidatesStamp() {
    let mut aGraph = boxGraph(10.0, 20.0, 30.0);

    if aGraph.refs().faces().nb() == 0 {
        return; // GTEST_SKIP: No face references available
    }

    let aFaceRefId = FaceRefId::new(0);
    let aBeforeStamp = aGraph.uids().stamp_of_ref(aFaceRefId.into());
    assert!(aBeforeStamp.is_valid());
    assert!(aBeforeStamp.is_ref_stamp());

    aGraph
        .editor()
        .generic()
        .remove_ref(aFaceRefId.into())
        .unwrap();

    assert!(aFaceRefId.is_removed(&aGraph));
    assert!(aGraph.uids().is_stale(&aBeforeStamp));
    assert!(!aGraph.uids().stamp_of_ref(aFaceRefId.into()).is_valid());
}

#[test]
fn BRepGraph_RefIdTest__OutOfRangeMetadataQueriesReturnFalse() {
    let aGraph = boxGraph(10.0, 20.0, 30.0);

    let anOutOfRangeFaceRef = FaceRefId::new(aGraph.refs().faces().nb());
    assert!(!anOutOfRangeFaceRef.is_removed(&aGraph));
    assert!(!anOutOfRangeFaceRef.is_owned(&aGraph));

    let anOutOfRangeWireRef = RefId::new(RefKind::WIRE, aGraph.refs().wires().nb());
    assert!(!anOutOfRangeWireRef.is_removed(&aGraph));
    assert!(!anOutOfRangeWireRef.is_owned(&aGraph));
}

#[test]
fn BRepGraph_RefIdTest__ChildRefs_CompoundEntriesAreValid() {
    let aGraph = boxGraph(10.0, 20.0, 30.0);

    let aRefs = aGraph.refs();

    let mut aCompIt = CompoundIterator::new(&aGraph);
    while aCompIt.more() {
        let aChildRefs =
            BRepGraph_TestTools::child_refs_of_parent(&aGraph, aCompIt.current_id().into());
        for aChildRefId in aChildRefs {
            let aRef = aRefs.children().entry(*aChildRefId);
            assert!(aRef.child_node_id.is_valid());
            assert!(!aChildRefId.is_removed(&aGraph));
        }
        aCompIt.next();
    }
}
