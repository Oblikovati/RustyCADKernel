//! Native-backed FixSmall and FixConnected behavior for ShapeFix_Wire.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Builder, Edge, Vertex, Wire};
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixWire;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn small_distinct_vertex_edge_requires_topology_mode() {
    let wire = short_edge_wire();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();

    assert!(!fixer.fix_small_at(1, false, 2e-5).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Fail2));
    assert_eq!(fixer.wire_data().unwrap().nb_edges(), 3);
}

#[test]
fn topology_mode_removes_small_edge_and_reconnects_neighbors() {
    let wire = short_edge_wire();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();
    fixer.set_topology_mode(true);

    assert!(fixer.fix_small_at(1, false, 2e-5).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail3));
    assert_eq!(fixer.wire_data().unwrap().nb_edges(), 2);
}

#[test]
fn connected_two_edge_gap_rebuilds_both_edges_with_a_shared_vertex() {
    let wire = disconnected_wire();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();

    assert!(fixer.fix_connected_at(2, 1e-3, true).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done3));
    let wire_data = fixer.wire_data().unwrap();
    let first = wire_data.edge(1).unwrap();
    let second = wire_data.edge(2).unwrap();
    let analysis = ShapeAnalysisEdge::new();
    assert!(
        analysis
            .last_vertex(&first)
            .is_same(&analysis.first_vertex(&second))
    );
}

fn disconnected_wire() -> Wire {
    let edges = [
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
            .unwrap()
            .edge()
            .unwrap(),
        BRepBuilderAPIMakeEdge::from_points(
            Point3::new(1.0001, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        )
        .unwrap()
        .edge()
        .unwrap(),
    ];
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for edge in &edges {
        Builder.add(&mut wire, &edge.clone().into_shape()).unwrap();
    }
    wire
}

#[test]
fn degenerated_small_edge_keeps_analyzer_failure_when_removal_succeeds() {
    let wire = degenerated_edge_wire();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();

    assert!(fixer.fix_small_at(1, false, 1e-7).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Fail1));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert_eq!(fixer.wire_data().unwrap().nb_edges(), 1);
}

#[test]
fn connected_single_edge_replaces_both_endpoints_in_either_mode() {
    for topology_mode in [false, true] {
        let fixer = ShapeFixWire::new();
        fixer.load(&one_edge_wire()).unwrap();
        fixer.set_topology_mode(topology_mode);

        assert!(fixer.fix_connected_at(1, 1e-3, true).unwrap());
        assert!(fixer.last_fix_status(ShapeExtendStatus::Done3));
        let edge = fixer.wire_data().unwrap().edge(1).unwrap();
        let analysis = ShapeAnalysisEdge::new();
        assert!(
            analysis
                .first_vertex(&edge)
                .is_same(&analysis.last_vertex(&edge))
        );
    }
}

fn one_edge_wire() -> Wire {
    let edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1e-4, 0.0, 0.0),
    )
    .unwrap()
    .edge()
    .unwrap();
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &edge.into_shape()).unwrap();
    wire
}

fn degenerated_edge_wire() -> Wire {
    let mut edge = Edge::default();
    let mut vertex = Vertex::default();
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    builder.degenerated(&edge, true).unwrap();
    builder
        .make_vertex_at(&mut vertex, Point3::new(0.0, 0.0, 0.0), 1e-7)
        .unwrap();
    let mut edge_shape = edge.clone().into_shape();
    Builder
        .add(&mut edge_shape, &vertex.clone().into_shape())
        .unwrap();
    Builder
        .add(
            &mut edge_shape,
            &vertex
                .clone()
                .into_shape()
                .oriented(rustycad_tkg3d::top_abs::Orientation::Reversed),
        )
        .unwrap();
    edge = rustycad_tkbrep::topods::edge(&edge_shape).unwrap();
    let long_edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0))
            .unwrap()
            .edge()
            .unwrap();
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &edge.into_shape()).unwrap();
    Builder.add(&mut wire, &long_edge.into_shape()).unwrap();
    wire
}

fn short_edge_wire() -> rustycad_tkbrep::topods::Wire {
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1e-5, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let edges = [
        BRepBuilderAPIMakeEdge::from_points(points[0], points[1])
            .unwrap()
            .edge()
            .unwrap(),
        BRepBuilderAPIMakeEdge::from_points(points[1], points[2])
            .unwrap()
            .edge()
            .unwrap(),
        BRepBuilderAPIMakeEdge::from_points(points[2], points[0])
            .unwrap()
            .edge()
            .unwrap(),
    ];
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for edge in &edges {
        Builder.add(&mut wire, &edge.clone().into_shape()).unwrap();
    }
    wire
}
