//! Direct translation of BRepGraph_Iterator_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::iterator::*;
use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId};
use support::make_box;

/// The `BRepGraph_IteratorTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    myGraph
}

#[test]
fn BRepGraph_IteratorTest__FaceIterator_CountMatchesTopology() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = FaceIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 6);
}

#[test]
fn BRepGraph_IteratorTest__EdgeIterator_CountMatchesTopology() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = EdgeIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 12);
}

#[test]
fn BRepGraph_IteratorTest__VertexIterator_CountMatchesTopology() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = VertexIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 8);
}

#[test]
fn BRepGraph_IteratorTest__SolidIterator_BoxHasOneSolid() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = SolidIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 1);
}

#[test]
fn BRepGraph_IteratorTest__ShellIterator_BoxHasOneShell() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = ShellIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 1);
}

#[test]
fn BRepGraph_IteratorTest__WireIterator_BoxHasSixWires() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = WireIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 6);
}

#[test]
fn BRepGraph_IteratorTest__RootProductIterator_MatchesStoredRoots() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = RootProductIterator::new(&myGraph);
    while anIt.more() {
        assert!(aCount < myGraph.root_product_ids().len());
        assert_eq!(anIt.current(), myGraph.root_product_ids()[aCount]);
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, myGraph.root_product_ids().len());
}

#[test]
fn BRepGraph_IteratorTest__CurrentId_ReturnsValidTypedIds() {
    let myGraph = SetUp();
    let anIt = FaceIterator::new(&myGraph);
    assert!(anIt.more());
    let aFaceId: FaceId = anIt.current_id();
    assert!(aFaceId.is_valid_for(myGraph.topo().faces().nb()));
}

#[test]
fn BRepGraph_IteratorTest__Current_ReturnsDefinition() {
    let myGraph = SetUp();
    let anIt = VertexIterator::new(&myGraph);
    assert!(anIt.more());
    let aVtx = anIt.current();
    // Vertex should have a valid point (box vertex coordinates are finite).
    assert!(aVtx.point.x().is_finite());
    assert!(aVtx.point.y().is_finite());
    assert!(aVtx.point.z().is_finite());
}

#[test]
fn BRepGraph_IteratorTest__RemovedFace_SkippedByDefaultIterator() {
    let mut myGraph = SetUp();
    let aNbBefore = myGraph.topo().faces().nb();
    myGraph
        .editor()
        .generic()
        .remove_node(FaceId::start().into())
        .unwrap();

    let mut aCount = 0;
    let mut anIt = FaceIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, aNbBefore - 1);
}

#[test]
fn BRepGraph_IteratorTest__FullTraverse_IncludesRemovedFace() {
    let mut myGraph = SetUp();
    let aNbBefore = myGraph.topo().faces().nb();
    myGraph
        .editor()
        .generic()
        .remove_node(FaceId::start().into())
        .unwrap();

    let mut aCount = 0;
    let mut anIt = FullFaceIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, aNbBefore);
}

#[test]
fn BRepGraph_IteratorTest__RangeFor_WorksCorrectly() {
    let myGraph = SetUp();
    let mut aCount = 0;
    for aFace in FaceIterator::new(&myGraph) {
        let _ = aFace;
        aCount += 1;
    }
    assert_eq!(aCount, 6);
}

#[test]
fn BRepGraph_IteratorStandalone__EmptyGraph_IteratorIsEmpty() {
    let aGraph = BRepGraph::new();
    assert!(!FaceIterator::new(&aGraph).more());
    assert!(!EdgeIterator::new(&aGraph).more());
    assert!(!VertexIterator::new(&aGraph).more());
    assert!(!SolidIterator::new(&aGraph).more());
}

#[test]
fn BRepGraph_IteratorTest__CoEdgeIterator_CountMatchesTopology() {
    let myGraph = SetUp();
    let mut aCount = 0;
    let mut anIt = CoEdgeIterator::new(&myGraph);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    // A box has 6 faces x 4 edges/face = 24 coedges.
    assert_eq!(aCount, 24);
}
