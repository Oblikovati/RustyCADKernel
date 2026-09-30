//! Original BRepGraph_EdgeCases_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{BRepGraph, NodeId, NodeKind, shapes_view::Options},
    topods::{Compound, Shape},
};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn edge_case_box(dx: f64, dy: f64, dz: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(dx, dy, dz)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
fn reset_graph_shape(graph: &mut BRepGraph, shape: &Shape) {
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    assert!(!graph.is_empty());
}
fn edge_case_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    reset_graph_shape(&mut graph, &edge_case_box(10.0, 20.0, 30.0));
    graph
}
fn edge_case_counts(graph: &BRepGraph) -> [u32; 8] {
    let topo = graph.topo();
    [
        topo.solids().nb(),
        topo.shells().nb(),
        topo.faces().nb(),
        topo.wires().nb(),
        topo.edges().nb(),
        topo.vertices().nb(),
        topo.faces().nb(),
        topo.edges().nb(),
    ]
}
#[test]
fn BRepGraph_EdgeCasesTest__Build_NullShape_IsEmpty() {
    let mut graph = BRepGraph::new();
    let shape = Shape::default();
    graph.clear().unwrap();
    graph.shapes_mut().add(&shape).unwrap();
    assert!(graph.is_empty());
}
#[test]
fn BRepGraph_EdgeCasesTest__Build_EmptyCompound_IsEmptyZeroCounts() {
    let mut graph = BRepGraph::new();
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    graph.clear().unwrap();
    graph.shapes_mut().add(&compound).unwrap();
    // The upstream invariant concerns definitions, not success of the empty import.
    assert_eq!(edge_case_counts(&graph), [0; 8]);
}
#[test]
fn BRepGraph_EdgeCasesTest__Shape_InvalidNodeId_ReturnsNull() {
    let graph = edge_case_graph();
    assert!(graph.shapes().shape(NodeId::default()).unwrap().is_null());
}
#[test]
fn BRepGraph_EdgeCasesTest__ReconstructShape_InvalidNodeId_ReturnsNull() {
    let graph = edge_case_graph();
    assert!(
        graph
            .shapes()
            .reconstruct(NodeId::default())
            .unwrap()
            .is_null()
    );
}
#[test]
fn BRepGraph_EdgeCasesTest__TopoEntity_InvalidNodeId_ReturnsNull() {
    let graph = edge_case_graph();
    assert!(
        graph
            .topo()
            .generic()
            .topo_entity(NodeId::default())
            .is_none()
    );
}
#[test]
fn BRepGraph_EdgeCasesTest__NbNodes_BeforeBuild_ReturnsZero() {
    let graph = BRepGraph::new();
    assert_eq!(graph.topo().generic().nb_nodes(), 0);
}
#[test]
fn BRepGraph_EdgeCasesTest__Build_TwiceOnSameGraph_GenerationIncrements() {
    let mut graph = BRepGraph::new();
    let shape = edge_case_box(10.0, 20.0, 30.0);
    reset_graph_shape(&mut graph, &shape);
    let first = graph.uids().generation();
    reset_graph_shape(&mut graph, &shape);
    let second = graph.uids().generation();
    assert!(second > first);
}
#[test]
fn BRepGraph_EdgeCasesTest__Build_TwiceOnSameGraph_OldUIDsInvalidated() {
    let mut graph = BRepGraph::new();
    let shape = edge_case_box(10.0, 20.0, 30.0);
    reset_graph_shape(&mut graph, &shape);
    let first = graph.uids().generation();
    assert!(graph.topo().generic().nb_nodes() > 0);
    reset_graph_shape(&mut graph, &shape);
    let second = graph.uids().generation();
    assert_ne!(first, second);
    assert!(
        graph
            .uids()
            .of_node(NodeId::new(NodeKind::SOLID, 0))
            .is_valid()
    );
}
#[test]
fn BRepGraph_EdgeCasesTest__Build_TwiceOnSameGraph_CountsResetCorrectly() {
    let mut graph = BRepGraph::new();
    let shape = edge_case_box(10.0, 20.0, 30.0);
    reset_graph_shape(&mut graph, &shape);
    let first = edge_case_counts(&graph);
    reset_graph_shape(&mut graph, &shape);
    assert_eq!(edge_case_counts(&graph), first);
}
#[test]
fn BRepGraph_EdgeCasesTest__UID_AlwaysEnabled_AfterBuild() {
    let graph = edge_case_graph();
    assert!(
        graph
            .uids()
            .of_node(NodeId::new(NodeKind::SOLID, 0))
            .is_valid()
    );
}
fn graph_with_parallel_option(shape: &Shape, parallel: bool) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let options = Options {
        parallel,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(shape, &options)
        .unwrap();
    assert!(!graph.is_empty());
    graph
}
fn compare_parallel_population(shape: &Shape) {
    let sequential = graph_with_parallel_option(shape, false);
    let parallel = graph_with_parallel_option(shape, true);
    assert_eq!(edge_case_counts(&parallel), edge_case_counts(&sequential));
    assert_eq!(
        parallel.topo().generic().nb_nodes(),
        sequential.topo().generic().nb_nodes()
    );
}
#[test]
fn BRepGraph_EdgeCasesTest__ParallelBuild_Sphere_SameAsSequential() {
    let shape = BRepPrimAPIMakeSphere::new(50.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    compare_parallel_population(&shape);
}
#[test]
fn BRepGraph_EdgeCasesTest__ParallelBuild_Compound_SameAsSequential() {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for dimensions in [(10.0, 10.0, 10.0), (20.0, 30.0, 40.0), (5.0, 15.0, 25.0)] {
        builder
            .add(
                &mut compound,
                &edge_case_box(dimensions.0, dimensions.1, dimensions.2),
            )
            .unwrap();
    }
    compare_parallel_population(&compound);
}
