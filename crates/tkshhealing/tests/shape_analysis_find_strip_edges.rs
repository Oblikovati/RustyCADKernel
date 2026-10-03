//! Differential coverage of ShapeAnalysis_CheckSmallFace::FindStripEdges.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

const ORACLE: &str = include_str!("../../../tools/oracle/shape_analysis_find_strip_edges.expected");

#[test]
fn find_strip_edges_matches_native_edge_selection_and_status() {
    let first = edge(0.0, 1.0);
    let close = edge(5e-4, 1.0);
    check_case(
        "two_strip",
        face(&[first.clone(), close.clone()]),
        first.clone(),
        close.clone(),
    );
    check_case(
        "seam_strip",
        face(&[first.clone(), first.clone(), close.clone()]),
        first.clone(),
        close.clone(),
    );
    let far = edge(2e-3, 1.0);
    check_case(
        "too_far",
        face(&[first.clone(), far.clone()]),
        first.clone(),
        far,
    );
    check_case(
        "third_edge",
        face(&[first.clone(), close.clone(), edge(0.0, 1.0)]),
        first.clone(),
        close.clone(),
    );
    check_case(
        "small_edge_skipped",
        face(&[edge(0.0, 1e-4), first.clone(), close.clone()]),
        first,
        close,
    );
    check_case(
        "one_edge",
        face(&[edge(0.0, 1.0)]),
        Edge::default(),
        Edge::default(),
    );
}

fn check_case(name: &str, face: Face, expected_first: Edge, expected_second: Edge) {
    let mut first = Edge::default();
    let mut second = Edge::default();
    let mut maximum = -1.0;
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let found = analyzer
        .find_strip_edges(&face, &mut first, &mut second, 1e-3, &mut maximum)
        .unwrap();
    let values = expected(name);
    assert_eq!(found as i32, values[0] as i32, "{name}");
    assert_eq!(
        analyzer.status_strip(ShapeExtendStatus::Done3) as i32,
        values[1] as i32,
        "{name}"
    );
    assert!((maximum - values[2]).abs() <= 1e-14, "{name}: {maximum}");
    assert_eq!(first.is_null() as i32, values[3] as i32, "{name} first");
    assert_eq!(second.is_null() as i32, values[4] as i32, "{name} second");
    if !expected_first.is_null() {
        assert!(first.is_same(&expected_first), "{name} first identity");
    }
    if !expected_second.is_null() {
        assert!(second.is_same(&expected_second), "{name} second identity");
    }
}

fn expected(name: &str) -> Vec<f64> {
    ORACLE
        .lines()
        .find(|row| row.starts_with(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|field| field.parse::<f64>().unwrap())
        .collect()
}

fn face(edges: &[Edge]) -> Face {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let mut wire_shape = wire.into_shape();
    for edge in edges {
        let oriented = edge.oriented(Orientation::Forward);
        Builder.add(&mut wire_shape, &oriented).unwrap();
    }
    let mut face = Face::default();
    builder.make_face(&mut face);
    let mut face_shape = face.into_shape();
    Builder.add(&mut face_shape, &wire_shape).unwrap();
    topods::face(&face_shape).unwrap()
}

fn edge(z: f64, last: f64) -> Edge {
    let curve = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(0.0, 0.0, z),
        Direction3::X,
    )))
    .cast::<dyn GeomCurve>();
    BRepBuilderAPIMakeEdge::from_curve_params(&curve, 0.0, last)
        .unwrap()
        .edge()
        .unwrap()
}
