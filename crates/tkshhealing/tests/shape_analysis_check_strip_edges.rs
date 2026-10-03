//! Differential coverage of ShapeAnalysis_CheckSmallFace::CheckStripEdges.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;

const ORACLE: &str =
    include_str!("../../../tools/oracle/shape_analysis_check_strip_edges.expected");

#[test]
fn check_strip_edges_matches_native_sampling_and_output_mutation() {
    check_case("coincident", edge(0.0, 1e-7), edge(0.0, 1e-7), 1e-3);
    check_case("within", edge(0.0, 1e-7), edge(5e-4, 1e-7), 1e-3);
    check_case("outside", edge(0.0, 1e-7), edge(2e-3, 1e-7), 1e-3);
    check_case("exact_limit", edge(0.0, 1e-7), edge(1e-3, 1e-7), 1e-3);
    check_case(
        "native_tolerance",
        edge(0.0, 2e-3),
        edge(1.5e-3, 2e-3),
        -1.0,
    );
}

fn check_case(name: &str, first: Edge, second: Edge, tolerance: f64) {
    let expected = ORACLE.lines().find(|row| row.starts_with(name)).unwrap();
    let mut fields = expected.split_whitespace().skip(1);
    let expected_result = fields.next().unwrap().parse::<i32>().unwrap();
    let expected_distance = fields.next().unwrap().parse::<f64>().unwrap();
    let mut distance = -1.0;
    let result = ShapeAnalysisCheckSmallFace::new()
        .check_strip_edges(&first, &second, tolerance, &mut distance)
        .unwrap();
    assert_eq!(result as i32, expected_result, "{name}");
    assert!(
        (distance - expected_distance).abs() <= 1e-14,
        "{name}: {distance}"
    );
}

fn edge(z: f64, tolerance: f64) -> Edge {
    let line = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(0.0, 0.0, z),
        Direction3::X,
    )))
    .cast::<dyn GeomCurve>();
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &line, tolerance)
        .unwrap();
    builder.range(&edge, 0.0, 1.0, true).unwrap();
    assert!(BRepTool::edge_tolerance(&edge).unwrap() >= tolerance);
    edge
}
