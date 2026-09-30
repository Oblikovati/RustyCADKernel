//! Direct translation of BRepGraph_SparseModel_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871, split over the included
//! `brep_graph_sparse_model/suites_N.rs` files in the upstream order.
//! `BRepGraph_Compact`, `BRepGraph_Deduplicate` and `BRepGraph_Validate` are the
//! `compact`, `deduplicate` and `validate` modules.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::compact::{self, CompactResult};
use rustycad_tkbrep::brep_graph::deduplicate::{self, DeduplicateResult};
use rustycad_tkbrep::brep_graph::iterator::{CoEdgeIterator, EdgeIterator};
use rustycad_tkbrep::brep_graph::validate::{self, Options};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkmath::toploc::Location;

fn addVertex(aGraph: &mut BRepGraph, x: f64, y: f64, z: f64) -> VertexId {
    aGraph.editor().vertices().add(Point3::new(x, y, z), 1.0e-7)
}

/// `Edges().Add(theV0, theV1, theCurve, 0.0, 10.0, 1.e-7)`.
fn addEdge(
    aGraph: &mut BRepGraph,
    theV0: VertexId,
    theV1: VertexId,
    theCurve: &Handle<dyn GeomCurve>,
) -> EdgeId {
    aGraph
        .editor()
        .edges()
        .add(theV0, theV1, theCurve, 0.0, 10.0, 1.0e-7)
}

fn noCurve() -> Handle<dyn GeomCurve> {
    Handle::default()
}

/// `new Geom_Line(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0))`.
fn xLine() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::X,
    )))
    .into_cast()
}

/// `new Geom_Plane(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1))`.
fn xyPlane() -> Handle<dyn GeomSurface> {
    let aDir = Direction3::new(0.0, 0.0, 1.0).unwrap();
    Handle::new(GeomPlane::from_point(Point3::new(0.0, 0.0, 0.0), aDir).unwrap()).into_cast()
}

/// A one-coedge wire (FORWARD) over the edge.
fn addWire(aGraph: &mut BRepGraph, theEdge: EdgeId) -> WireId {
    let aCoEdges = [aGraph
        .editor()
        .co_edges()
        .add(theEdge, ParityOrientation::FORWARD)];
    aGraph.editor().wires().add(&aCoEdges)
}

/// `Faces().Add(aPlane, theWire, {}, 1.e-7)`.
fn addPlaneFace(aGraph: &mut BRepGraph, theWire: WireId) -> FaceId {
    aGraph
        .editor()
        .faces()
        .add(&xyPlane(), theWire, &[], 1.0e-7)
}

/// A free edge (no curve) between (0,0,0) and (10,0,0) in a one-coedge wire.
fn freeWire(aGraph: &mut BRepGraph) -> (EdgeId, WireId) {
    let aV0 = addVertex(aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(aGraph, 10.0, 0.0, 0.0);
    let anEdge = addEdge(aGraph, aV0, aV1, &noCurve());
    let aWire = addWire(aGraph, anEdge);
    (anEdge, aWire)
}

/// The face, shell, solid and part product of the canonical pattern over a wire.
fn canonicalSolid(aGraph: &mut BRepGraph, theWire: WireId) -> (FaceId, SolidId, ProductId) {
    let aFace = addPlaneFace(aGraph, theWire);
    let aShell = aGraph.editor().shells().add();
    aGraph
        .editor()
        .shells()
        .append(aShell, aFace, ParityOrientation::FORWARD)
        .unwrap();
    let aSolid = aGraph.editor().solids().add();
    aGraph
        .editor()
        .solids()
        .append(aSolid, aShell, ParityOrientation::FORWARD)
        .unwrap();
    let aProduct = aGraph
        .editor()
        .products()
        .add(aSolid.into(), &Location::default());
    (aFace, aSolid, aProduct)
}

fn removeNode(aGraph: &mut BRepGraph, theNode: impl Into<NodeId>) {
    aGraph
        .editor()
        .generic()
        .remove_node(theNode.into())
        .unwrap();
}

fn cleanup(aGraph: &mut BRepGraph) {
    aGraph.editor().generic().cleanup_removed_references();
}

fn compactGraph(aGraph: &mut BRepGraph) -> CompactResult {
    compact::perform(aGraph).unwrap()
}

fn dedupGraph(aGraph: &mut BRepGraph) -> DeduplicateResult {
    deduplicate::perform(aGraph).unwrap()
}

fn dedupMerging(aGraph: &mut BRepGraph) -> DeduplicateResult {
    let anOpts = deduplicate::Options {
        merge_entities_when_safe: true,
        ..deduplicate::Options::default()
    };
    deduplicate::perform_with(aGraph, &anOpts).unwrap()
}

fn isValid(aGraph: &BRepGraph) -> bool {
    validate::perform(aGraph).is_valid()
}

fn isAuditValid(aGraph: &BRepGraph) -> bool {
    validate::perform_with_options(aGraph, &Options::audit()).is_valid()
}

include!("brep_graph_sparse_model/suites_1.rs");
include!("brep_graph_sparse_model/suites_2.rs");
