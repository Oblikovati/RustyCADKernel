//! Direct translation of BRepGraph_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871,
//! split over the included `brep_graph_suite/suites_N.rs` files in the upstream order. `BRepPrimAPI_MakeBox`
//! is called by the shared production-builder fixture, the `BRepGraphTest`
//! fixture is [`SetUp`], and `BRepGraph_TestTools` is `support::ref_test_tools`.
//!
//! The native `static_assert`s on id conversions become the compile-time functions
//! below for the conversions that exist; the refused cross-kind conversions have no
//! `From` impl, which a negative bound cannot state. `BRepGraph::Allocator()` has no
//! Rust counterpart (docs/topology-design.md section 11.8): the two declarations that
//! read it keep the rest of their body.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_graph::child_explorer::ChildExplorer;
use rustycad_tkbrep::brep_graph::editor::{EdgeOps, VertexOps};
use rustycad_tkbrep::brep_graph::iterator::{
    DefIterator, ShellIterator, SolidIterator, VertexIterator, WireIterator,
};
use rustycad_tkbrep::brep_graph::iterator::{EdgeIterator, FaceIterator};
use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
use rustycad_tkbrep::brep_graph::parent_explorer::ParentExplorer;
use rustycad_tkbrep::brep_graph::reverse_iterator::{
    CompSolidsOfSolid, CompoundsOfChild, FacesOfWire, ShellsOfFace, SolidsOfShell,
};
use rustycad_tkbrep::brep_graph::shapes_view;
use rustycad_tkbrep::brep_graph::tool;
use rustycad_tkbrep::brep_graph::validate;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::{NodeTag, ParityOrientation};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::topods::{self, Compound, Edge};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Direction3, Vector3};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use std::collections::HashSet;
use support::make_box;
use support::ref_test_tools as BRepGraph_TestTools;

// `static_assert(std::is_convertible_v<BRepGraph_FaceId, BRepGraph_NodeId>)` and the
// representation twin; the explorers take any id convertible into a node id.
const _: fn(FaceId) -> NodeId = NodeId::from;
const _: fn(FaceSurfaceRepId) -> RepId = RepId::from;

/// `containsId(theIds, theId)`.
pub fn containsId<T: PartialEq>(theIds: impl IntoIterator<Item = T>, theId: T) -> bool {
    theIds.into_iter().any(|anId| anId == theId)
}

/// `componentKey(theNode)`.
pub fn componentKey(theNode: NodeId) -> i32 {
    theNode.index as i32 * NodeId::KIND_COUNT + theNode.kind.0
}

/// `collectFreeEdges(theGraph)`: the non-degenerated edges owned by one face.
pub fn collectFreeEdges(theGraph: &BRepGraph) -> Vec<EdgeId> {
    let mut aResult = Vec::new();
    let mut anEdgeIt = EdgeIterator::new(theGraph);
    while anEdgeIt.more() {
        let anEdgeId = anEdgeIt.current_id();
        anEdgeIt.next();
        if tool::edge::degenerated(theGraph, anEdgeId) {
            continue;
        }
        if owningFaceCount(theGraph, anEdgeId) == 1 {
            aResult.push(anEdgeId);
        }
    }
    aResult
}

// The distinct faces above an edge, the walk stopping at the second one.
fn owningFaceCount(theGraph: &BRepGraph, theEdge: EdgeId) -> usize {
    let mut aOwningFaces: HashSet<u32> = HashSet::new();
    let mut aFaceExp =
        ParentExplorer::with_target(theGraph, theEdge.into(), NodeKind::FACE).unwrap();
    while aFaceExp.more() {
        aOwningFaces.insert(aFaceExp.current().def_id.index);
        if aOwningFaces.len() > 1 {
            break;
        }
        aFaceExp.next().unwrap();
    }
    aOwningFaces.len()
}

/// `addCompatibleReplacementEdge(theGraph, theOldEdge, theReversed)`: a line edge on the
/// old edge's vertices, reversed on request.
pub fn addCompatibleReplacementEdge(
    theGraph: &mut BRepGraph,
    theOldEdge: EdgeId,
    theReversed: bool,
) -> EdgeId {
    let aStartRef = tool::edge::start_vertex_id(theGraph, theOldEdge);
    let anEndRef = tool::edge::end_vertex_id(theGraph, theOldEdge);
    let aStartVertex = theGraph.refs().vertices().entry(aStartRef).child_vertex_id;
    let anEndVertex = theGraph.refs().vertices().entry(anEndRef).child_vertex_id;
    let (aNewStart, aNewEnd) = if theReversed {
        (anEndVertex, aStartVertex)
    } else {
        (aStartVertex, anEndVertex)
    };
    let aP0 = tool::vertex::pnt(theGraph, aNewStart);
    let aP1 = tool::vertex::pnt(theGraph, aNewEnd);
    let aLength = aP0.distance(&aP1);
    let aDirection = Direction3::from_vector(Vector3::between(aP0, aP1)).unwrap();
    let aCurve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(aP0, aDirection))).into_cast();
    theGraph
        .editor()
        .edges()
        .add(aNewStart, aNewEnd, &aCurve, 0.0, aLength, 1.0e-7)
}

/// `componentRootOfFace(theGraph, theFaceId)`: the first solid, else the first shell,
/// else the face.
pub fn componentRootOfFace(theGraph: &BRepGraph, theFaceId: FaceId) -> NodeId {
    for aKind in [NodeKind::SOLID, NodeKind::SHELL] {
        let aExp = ParentExplorer::with_target(theGraph, theFaceId.into(), aKind).unwrap();
        if aExp.more() {
            return aExp.current().def_id;
        }
    }
    theFaceId.into()
}

/// `countFaceComponents(theGraph)`.
pub fn countFaceComponents(theGraph: &BRepGraph) -> usize {
    let mut aRoots: HashSet<i32> = HashSet::new();
    let mut aFaceIt = FaceIterator::new(theGraph);
    while aFaceIt.more() {
        aRoots.insert(componentKey(componentRootOfFace(
            theGraph,
            aFaceIt.current_id(),
        )));
        aFaceIt.next();
    }
    aRoots.len()
}

/// `countSameDomainFaces(theGraph, theFace)`: the other faces on the same surface
/// handle.
pub fn countSameDomainFaces(theGraph: &BRepGraph, theFace: FaceId) -> u32 {
    let aSurface = tool::face::surface(theGraph, theFace);
    if aSurface.is_null() {
        return 0;
    }
    let mut aCount = 0;
    let mut aFaceIt = FaceIterator::new(theGraph);
    while aFaceIt.more() {
        let anOtherFace = aFaceIt.current_id();
        if anOtherFace != theFace && tool::face::surface(theGraph, anOtherFace) == aSurface {
            aCount += 1;
        }
        aFaceIt.next();
    }
    aCount
}

/// The `BRepGraphTest` fixture: a graph of a 10x20x30 box.
pub fn SetUp() -> BRepGraph {
    let aBox: Shape = make_box(10.0, 20.0, 30.0).into();
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph.shapes_mut().add(&aBox).unwrap();
    myGraph
}

/// `BRepGraph aGraph; aGraph.Clear(); aGraph.Shapes().Add(theShape)`.
pub fn graphOf(theShape: &Shape) -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph.shapes_mut().add(theShape).unwrap();
    aGraph
}

/// `TopExp_Explorer(theShape, theKind).Current()`: the first sub-shape of the kind.
pub fn firstOf(theShape: &Shape, theKind: ShapeEnum) -> Shape {
    let anExp = Explorer::new(theShape, theKind, ShapeEnum::Shape).unwrap();
    assert!(anExp.more());
    anExp.current().clone()
}

/// `myGraph.LayerRegistry().Ensure<BRepGraph_LayerHistory>()`.
pub fn history(theGraph: &BRepGraph) -> Handle<LayerHistory> {
    theGraph.layer_registry().ensure::<LayerHistory>().unwrap()
}

/// `->NbRecords()` of the history layer.
pub fn nbRecords(theGraph: &BRepGraph) -> usize {
    history(theGraph).get().unwrap().nb_records()
}

/// `->Record(theLabel, theOriginal, theReplacements)` (the default Modified kind).
pub fn recordHistory(
    theGraph: &BRepGraph,
    theLabel: &str,
    theOriginal: NodeId,
    theRepl: &[NodeId],
) {
    let aLabel = AsciiString::new(theLabel);
    let aHistory = history(theGraph);
    let aLayer = aHistory.get().unwrap();
    aLayer.record(
        theGraph,
        &aLabel,
        theOriginal,
        theRepl,
        HistoryKind::Modified,
    );
}

/// `->Record(theIndex).OperationName.IsEqual(theName)`.
pub fn recordNameIs(theGraph: &BRepGraph, theIndex: usize, theName: &str) -> bool {
    let aHistory = history(theGraph);
    let aLayer = aHistory.get().unwrap();
    aLayer
        .record_at(theIndex, |anEvent| anEvent.operation_name.is_equal(theName))
        .unwrap()
}

/// `->Record(theIndex).SequenceNumber`.
pub fn recordSequence(theGraph: &BRepGraph, theIndex: usize) -> usize {
    let aHistory = history(theGraph);
    let aLayer = aHistory.get().unwrap();
    aLayer
        .record_at(theIndex, |anEvent| anEvent.sequence_number)
        .unwrap()
}

/// `Editor().Gen().ApplyModification(theTarget, modifier returning theResult, theLabel)`.
pub fn applyModification(
    aGraph: &mut BRepGraph,
    theTarget: NodeId,
    theResult: NodeId,
    theLabel: &str,
) {
    aGraph
        .editor()
        .generic()
        .apply_modification(
            theTarget,
            |_, _| Ok(vec![theResult]),
            &AsciiString::new(theLabel),
        )
        .unwrap();
}

/// `NodeId(Kind::Edge, theIndex)`.
pub fn edgeNode(theIndex: u32) -> NodeId {
    NodeId::new(NodeKind::EDGE, theIndex)
}

include!("brep_graph_suite/suites_1.rs");
include!("brep_graph_suite/suites_2.rs");
include!("brep_graph_suite/suites_3.rs");
include!("brep_graph_suite/suites_4.rs");
include!("brep_graph_suite/suites_5.rs");
include!("brep_graph_suite/suites_6.rs");
