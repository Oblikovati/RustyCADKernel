//! 1:1 translation of src/ModelingAlgorithms/TKTopAlgo/GTests/BRepBuilderAPI_MakeWire_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

/// `aBB.MakeVertex(V, P, Tol)` on a new vertex.
fn make_vertex(a_bb: &BRepBuilder, point: Point3, tolerance: f64) -> Vertex {
    let mut vertex = Vertex::default();
    a_bb.make_vertex_at(&mut vertex, point, tolerance).unwrap();
    vertex
}

/// `BRepBuilderAPI_MakeEdge(V1, V2).Edge()`.
fn make_edge(v1: &Vertex, v2: &Vertex) -> Edge {
    BRepBuilderAPIMakeEdge::from_vertices(v1, v2)
        .unwrap()
        .edge()
        .unwrap()
}

#[test]
fn BRepBuilderAPI_MakeWire_Test__OCC27552_AddEdgesAndListOfEdges() {
    // Bug OCC27552: Wire creation fails depending on the order of edges
    // This test verifies that BRepBuilderAPI_MakeWire can successfully add
    // edges individually and as a list

    let a_bb = BRepBuilder::default();

    // Create vertices
    let a_v1 = make_vertex(&a_bb, Point3::new(0.0, 0.0, 0.0), 0.1);
    let a_v2 = make_vertex(&a_bb, Point3::new(5.0, 0.0, 0.0), 0.1);
    let a_v3 = make_vertex(&a_bb, Point3::new(10.0, 0.0, 0.0), 0.1);

    // Create edges
    let an_e1 = make_edge(&a_v1, &a_v2);
    let an_e2 = make_edge(&a_v2, &a_v3);

    // Build wire with individually added edges
    let mut a_mw = BRepBuilderAPIMakeWire::new();
    assert!(a_mw.add_edge(&an_e1).is_ok());
    assert!(a_mw.add_edge(&an_e2).is_ok());

    // Create additional vertices and edges for list test
    let a_v4 = make_vertex(&a_bb, Point3::new(10.0, 0.05, 0.0), 0.07);
    let a_v5 = make_vertex(&a_bb, Point3::new(10.0, -0.05, 0.0), 0.07);
    let a_v6 = make_vertex(&a_bb, Point3::new(10.0, 2.0, 0.0), 0.07);
    let a_v7 = make_vertex(&a_bb, Point3::new(10.0, -2.0, 0.0), 0.07);

    let an_e3 = make_edge(&a_v4, &a_v6);
    let an_e4 = make_edge(&a_v5, &a_v7);

    // Add edges as a list
    let mut a_list_of_edges: List<Shape> = List::default();
    a_list_of_edges.append(an_e3.into_shape());
    a_list_of_edges.append(an_e4.into_shape());

    assert!(a_mw.add_list(&a_list_of_edges).is_ok());

    // Verify wire was created successfully
    assert!(a_mw.is_done(), "Wire builder should complete successfully");
    let a_wire = a_mw.wire().unwrap();
    assert!(!a_wire.is_null(), "Resulting wire should not be null");
}
