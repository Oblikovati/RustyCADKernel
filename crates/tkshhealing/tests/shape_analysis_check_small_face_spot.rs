//! Regression coverage for native ShapeAnalysis_CheckSmallFace::IsSpotFace behavior.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn expected(name: &str) -> (i32, Point3, f64) {
    let row = include_str!("../../../tools/oracle/shape_analysis_check_small_face_spot.expected")
        .lines()
        .find(|row| row.starts_with(name))
        .unwrap();
    let mut fields = row.split_whitespace().skip(1);
    let status = fields.next().unwrap().parse().unwrap();
    let point = Point3::new(
        fields.next().unwrap().parse().unwrap(),
        fields.next().unwrap().parse().unwrap(),
        fields.next().unwrap().parse().unwrap(),
    );
    (status, point, fields.next().unwrap().parse().unwrap())
}

fn expected_checker_state() -> ([i32; 7], f64) {
    let row = include_str!("../../../tools/oracle/shape_analysis_check_small_face_spot.expected")
        .lines()
        .find(|row| row.starts_with("check_spot "))
        .unwrap();
    let values = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    let states = std::array::from_fn(|index| values[index] as i32);
    (states, values[7])
}

#[test]
fn spot_face_results_match_pinned_oracle() {
    assert_probe("no_wire", no_wire_face(), Some(0.001));
    assert_probe("empty_wire", empty_wire_face(), Some(0.001));
    assert_probe("local_tolerance", circle_face(1e-8), None);
    assert_probe("shared_vertex", shared_vertex_face(), Some(0.001));
    assert_probe("wide_loop", circle_face(0.01), Some(0.001));
    assert_probe("tiny_box", box_face(0.0001), Some(0.001));
    assert_probe("regular_box", box_face(1.0), Some(0.001));
}

#[test]
fn check_spot_face_updates_only_its_status_and_preserves_unknown_native_fields() {
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    assert_eq!(analyzer.status(ShapeExtendStatus::Ok), None);
    assert!(analyzer.status_spot(ShapeExtendStatus::Ok));
    assert_eq!(analyzer.tolerance(), None);
    analyzer.set_tolerance(-0.25);
    let states = run_checker_state_cases(&mut analyzer);
    assert_eq!(states, expected_checker_state().0);
    assert_eq!(analyzer.tolerance(), Some(expected_checker_state().1));
    assert_eq!(analyzer.status(ShapeExtendStatus::Ok), None);
}

fn run_checker_state_cases(analyzer: &mut ShapeAnalysisCheckSmallFace) -> [i32; 7] {
    let check_tiny = analyzer.check_spot_face(&box_face(0.0001), 0.001).unwrap();
    let tiny_done = analyzer.status_spot(ShapeExtendStatus::Done1);
    let check_regular = analyzer.check_spot_face(&box_face(1.0), 0.001).unwrap();
    let still_done = analyzer.status_spot(ShapeExtendStatus::Done1);
    let check_shared = analyzer
        .check_spot_face(&shared_vertex_face(), 0.001)
        .unwrap();
    let shared_done = analyzer.status_spot(ShapeExtendStatus::Done2);
    let fail1 = analyzer.status_spot(ShapeExtendStatus::Fail1);
    [
        check_tiny as i32,
        tiny_done as i32,
        check_regular as i32,
        still_done as i32,
        check_shared as i32,
        shared_done as i32,
        fail1 as i32,
    ]
}

fn assert_probe(name: &str, face: Face, tolerance: Option<f64>) {
    let mut location = Point3::new(11.0, 12.0, 13.0);
    let mut spot_tolerance = 17.0;
    let analyzer = ShapeAnalysisCheckSmallFace::new();
    let status = match tolerance {
        Some(value) => {
            analyzer.is_spot_face_with_tolerance(&face, &mut location, &mut spot_tolerance, value)
        }
        None => analyzer.is_spot_face(&face, &mut location, &mut spot_tolerance),
    }
    .unwrap();
    let (expected_status, expected_location, expected_tolerance) = expected(name);
    assert_eq!(status, expected_status, "{name}");
    assert_eq!(location, expected_location, "{name}");
    assert!(
        (spot_tolerance - expected_tolerance).abs() < 1e-14,
        "{name}"
    );
}

fn no_wire_face() -> Face {
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    face
}

fn empty_wire_face() -> Face {
    let mut face = no_wire_face();
    let mut face_shape = face.clone().into_shape();
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut face_shape, &wire).unwrap();
    face = topods::face(&face_shape).unwrap();
    face
}

fn shared_vertex_face() -> Face {
    circle_face(0.0001)
}

fn circle_face(radius: f64) -> Face {
    let builder = BRepBuilder::default();
    let frame = Axis2::new(Point3::new(2.0, 3.0, 4.0), Direction3::Z).unwrap();
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, radius).unwrap()).cast();
    let edge = make_circle_edge(&builder, &curve);
    let edge_shape = circle_edge_with_vertex(&builder, edge, radius);
    circle_face_with_edge(&edge_shape)
}

fn make_circle_edge(builder: &BRepBuilder, curve: &Handle<dyn GeomCurve>) -> Edge {
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, curve, precision::confusion())
        .unwrap();
    builder
        .range(&edge, 0.0, std::f64::consts::TAU, true)
        .unwrap();
    edge
}

fn circle_edge_with_vertex(builder: &BRepBuilder, edge: Edge, radius: f64) -> Shape {
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(
            &mut vertex,
            Point3::new(2.0 + radius, 3.0, 4.0),
            precision::confusion(),
        )
        .unwrap();
    let mut edge_shape = edge.into_shape();
    let start = vertex.oriented(rustycad_tkg3d::top_abs::Orientation::Forward);
    let end = vertex.oriented(rustycad_tkg3d::top_abs::Orientation::Reversed);
    Builder.add(&mut edge_shape, &start).unwrap();
    Builder.add(&mut edge_shape, &end).unwrap();
    edge_shape
}

fn circle_face_with_edge(edge: &Shape) -> Face {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut wire_shape = wire.into_shape();
    Builder.add(&mut wire_shape, edge).unwrap();
    let mut face = no_wire_face().into_shape();
    Builder.add(&mut face, &wire_shape).unwrap();
    topods::face(&face).unwrap()
}

fn box_face(size: f64) -> Face {
    let shape = BRepPrimAPIMakeBox::at_point(Point3::new(2.0, 3.0, 4.0), size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let faces = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    topods::face(faces.current()).unwrap()
}
