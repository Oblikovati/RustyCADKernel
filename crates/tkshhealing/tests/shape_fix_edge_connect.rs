//! Translation of ShapeFix_EdgeConnect_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871; original geometry and tolerances.
#![allow(non_snake_case)]
mod healing_support;
use healing_support::compound;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_fix::ShapeFixEdgeConnect;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn connect_segment(start: Point3, end: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(start, end)
        .unwrap()
        .edge()
        .unwrap()
}

fn x_segment(first: f64, last: f64) -> Edge {
    connect_segment(Point3::new(first, 0.0, 0.0), Point3::new(last, 0.0, 0.0))
}

fn first_vertex(edge: &Edge) -> Vertex {
    topexp::first_vertex(edge, true)
}
fn last_vertex(edge: &Edge) -> Vertex {
    topexp::last_vertex(edge, true)
}
fn vertex_point(vertex: &Vertex) -> Point3 {
    BRepTool::pnt(vertex).unwrap()
}
fn reversed_edge(edge: &Edge) -> Edge {
    topods::edge(&edge.reversed()).unwrap()
}

fn assert_joint(first: &Edge, last: &Edge) {
    assert!(last_vertex(first).is_same(&first_vertex(last)));
}

fn build_connected_pair(first: &Edge, last: &Edge) -> ShapeFixEdgeConnect {
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_edges(first, last);
    connector.build().unwrap();
    connector
}

fn connect_wire(edges: &[&Edge], closed: bool) -> Wire {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, edge).unwrap();
    }
    if closed {
        wire.set_closed(true).unwrap();
    }
    wire
}

fn total_length(shape: &Shape) -> f64 {
    let mut properties = GProps::new();
    BRepGProp::linear_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

#[test]
fn ShapeFix_EdgeConnectTest__ConnectsPairAtBoundingBoxMidpoint() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.2, 2.0);
    build_connected_pair(&first, &second);
    let joint = last_vertex(&first);
    assert!(joint.is_same(&first_vertex(&second)));
    assert!((vertex_point(&joint).x() - 1.1).abs() <= precision::confusion());
    assert!(
        (BRepTool::vertex_tolerance(&joint).unwrap() - 0.10001).abs() <= precision::confusion()
    );
    assert!(vertex_point(&first_vertex(&first)).x().abs() <= precision::confusion());
    assert!((vertex_point(&last_vertex(&second)).x() - 2.0).abs() <= precision::confusion());
}

#[test]
fn ShapeFix_EdgeConnectTest__HonorsEdgeOrientation() {
    let mut first = x_segment(0.0, 1.0);
    let mut second = x_segment(2.0, 3.0);
    first.reverse();
    second.reverse();
    let expected = Point3::from(
        (vertex_point(&last_vertex(&first)).xyz() + vertex_point(&first_vertex(&second)).xyz())
            * 0.5,
    );
    build_connected_pair(&first, &second);
    let joint = last_vertex(&first);
    assert!(joint.is_same(&first_vertex(&second)));
    assert!(vertex_point(&joint).is_equal(&expected, precision::confusion()));
}

#[test]
fn ShapeFix_EdgeConnectTest__MergesExistingConnectivityGroups() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.1, 2.0);
    let third = x_segment(3.0, 4.0);
    let fourth = x_segment(4.1, 5.0);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_edges(&first, &second);
    connector.add_edges(&third, &fourth);
    connector.add_edges(&reversed_edge(&second), &reversed_edge(&third));
    connector.build().unwrap();
    let joint = last_vertex(&first);
    assert!(joint.is_same(&first_vertex(&second)));
    assert!(joint.is_same(&last_vertex(&third)));
    assert!(joint.is_same(&first_vertex(&fourth)));
    assert!((vertex_point(&joint).x() - 2.55).abs() <= precision::confusion());
}

#[test]
fn ShapeFix_EdgeConnectTest__AddsAllConsecutiveEdgesOfOpenWire() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.1, 2.0);
    let third = x_segment(2.1, 3.0);
    let wire = connect_wire(&[&first, &second, &third], false);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_shape(&wire).unwrap();
    connector.build().unwrap();
    assert_joint(&first, &second);
    assert_joint(&second, &third);
    assert!(!first_vertex(&first).is_same(&last_vertex(&third)));
}

#[test]
fn ShapeFix_EdgeConnectTest__ConnectsLastAndFirstEdgesOfClosedWire() {
    let first = x_segment(0.0, 1.0);
    let second = connect_segment(Point3::new(1.1, 0.0, 0.0), Point3::new(0.5, 1.0, 0.0));
    let third = connect_segment(Point3::new(0.4, 1.0, 0.0), Point3::new(0.0, 0.1, 0.0));
    let wire = connect_wire(&[&first, &second, &third], true);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_shape(&wire).unwrap();
    connector.build().unwrap();
    assert_joint(&first, &second);
    assert_joint(&second, &third);
    assert_joint(&third, &first);
}

#[test]
fn ShapeFix_EdgeConnectTest__RepeatedAddIsIdempotent() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.2, 2.0);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_edges(&first, &second);
    connector.add_edges(&first, &second);
    connector.build().unwrap();
    assert_joint(&first, &second);
    assert!((vertex_point(&last_vertex(&first)).x() - 1.1).abs() <= precision::confusion());
}

#[test]
fn ShapeFix_EdgeConnectTest__ClearDiscardsPendingConnections() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.2, 2.0);
    let original1 = last_vertex(&first);
    let original2 = first_vertex(&second);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_edges(&first, &second);
    connector.clear();
    connector.build().unwrap();
    assert!(original1.is_same(&last_vertex(&first)));
    assert!(original2.is_same(&first_vertex(&second)));
    assert!(!last_vertex(&first).is_same(&first_vertex(&second)));
}

#[test]
fn ShapeFix_EdgeConnectTest__BuildClearsCompletedConnections() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.2, 2.0);
    let mut connector = build_connected_pair(&first, &second);
    let third = x_segment(3.0, 4.0);
    let fourth = x_segment(4.2, 5.0);
    connector.build().unwrap();
    assert!(!last_vertex(&third).is_same(&first_vertex(&fourth)));
}

#[test]
fn ShapeFix_EdgeConnectTest__UsesConfusionAsMinimumTolerance() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.0, 2.0);
    build_connected_pair(&first, &second);
    let joint = last_vertex(&first);
    assert!(joint.is_same(&first_vertex(&second)));
    assert!(
        (BRepTool::vertex_tolerance(&joint).unwrap() - precision::confusion()).abs()
            <= precision::p_confusion()
    );
}

#[test]
fn ShapeFix_EdgeConnectTest__ComputesThreeDimensionalBoundingBoxCenter() {
    let first = connect_segment(Point3::new(-1.0, -2.0, -3.0), Point3::new(1.0, 2.0, 3.0));
    let second = connect_segment(Point3::new(5.0, 8.0, 11.0), Point3::new(6.0, 9.0, 12.0));
    build_connected_pair(&first, &second);
    let joint = last_vertex(&first);
    assert!(vertex_point(&joint).is_equal(&Point3::new(3.0, 5.0, 7.0), precision::confusion()));
    let distance = Point3::new(1.0, 2.0, 3.0).distance(&Point3::new(3.0, 5.0, 7.0));
    assert!(
        (BRepTool::vertex_tolerance(&joint).unwrap() - distance * 1.0001).abs()
            <= precision::confusion()
    );
}

#[test]
fn ShapeFix_EdgeConnectTest__AppendsUnboundVertexToExistingGroup() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.1, 2.0);
    let third = x_segment(1.2, 3.0);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_edges(&first, &second);
    connector.add_edges(&reversed_edge(&second), &third);
    connector.build().unwrap();
    let joint = last_vertex(&first);
    assert!(joint.is_same(&first_vertex(&second)));
    assert!(joint.is_same(&first_vertex(&third)));
}

#[test]
fn ShapeFix_EdgeConnectTest__PrependsUnboundVertexToExistingGroup() {
    let first = x_segment(0.0, 0.9);
    let second = x_segment(1.0, 2.0);
    let third = x_segment(2.1, 3.0);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_edges(&second, &third);
    connector.add_edges(&first, &second);
    connector.build().unwrap();
    let joint = last_vertex(&first);
    assert!(joint.is_same(&first_vertex(&second)));
    assert!(!joint.is_same(&last_vertex(&second)));
    assert_joint(&second, &third);
}

#[test]
fn ShapeFix_EdgeConnectTest__ProcessesMultipleWiresIndependently() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.1, 2.0);
    let third = x_segment(10.0, 11.0);
    let fourth = x_segment(11.1, 12.0);
    let wire1 = connect_wire(&[&first, &second], false);
    let wire2 = connect_wire(&[&third, &fourth], false);
    let shape = compound(&[wire1.into_shape(), wire2.into_shape()]);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_shape(&shape).unwrap();
    connector.build().unwrap();
    assert_joint(&first, &second);
    assert_joint(&third, &fourth);
    assert!(!last_vertex(&first).is_same(&first_vertex(&fourth)));
}

#[test]
fn ShapeFix_EdgeConnectTest__IgnoresEdgesOutsideWiresInShapeOverload() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.1, 2.0);
    let shape = compound(&[first.clone().into_shape(), second.clone().into_shape()]);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_shape(&shape).unwrap();
    connector.build().unwrap();
    assert!(!last_vertex(&first).is_same(&first_vertex(&second)));
}

#[test]
fn ShapeFix_EdgeConnectTest__AcceptsEmptyWire() {
    let wire = connect_wire(&[], false);
    let mut connector = ShapeFixEdgeConnect::new();
    connector.add_shape(&wire).unwrap();
    connector.build().unwrap();
}

#[test]
fn ShapeFix_EdgeConnectTest__CanBeReusedAfterBuild() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.1, 2.0);
    let third = x_segment(3.0, 4.0);
    let fourth = x_segment(4.1, 5.0);
    let mut connector = build_connected_pair(&first, &second);
    connector.add_edges(&third, &fourth);
    connector.build().unwrap();
    assert_joint(&first, &second);
    assert_joint(&third, &fourth);
}

#[test]
fn ShapeFix_EdgeConnectTest__PreservesUnderlyingCurves() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.2, 2.0);
    let original1 = BRepTool::curve(&first).unwrap();
    let original2 = BRepTool::curve(&second).unwrap();
    build_connected_pair(&first, &second);
    let updated1 = BRepTool::curve(&first).unwrap();
    assert!(updated1.0 == original1.0);
    assert_eq!(updated1.1, original1.1);
    assert_eq!(updated1.2, original1.2);
    let updated2 = BRepTool::curve(&second).unwrap();
    assert!(updated2.0 == original2.0);
    assert_eq!(updated2.1, original2.1);
    assert_eq!(updated2.2, original2.2);
}

#[test]
fn ShapeFix_EdgeConnectTest__PreservesTotalLengthAgainstReference() {
    let first = x_segment(0.0, 1.0);
    let second = x_segment(1.2, 2.0);
    let shape = compound(&[first.clone().into_shape(), second.clone().into_shape()]);
    let input_length = total_length(&shape);
    build_connected_pair(&first, &second);
    let result_length = total_length(&shape);
    assert!((input_length - 1.8).abs() <= precision::confusion());
    assert!((result_length - input_length).abs() <= precision::confusion());
    assert!((result_length - 1.8).abs() <= precision::confusion());
}
