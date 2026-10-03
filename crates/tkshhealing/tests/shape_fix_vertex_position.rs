//! Native ShapeFix::FixVertexPosition endpoint observations.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::{self, Explorer},
    topods::{self, Compound, Edge, Shape, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkshhealing::{shape_build::ShapeBuildReShape, shape_fix::ShapeFix};
#[derive(Clone, Copy)]
struct EdgeSpec {
    x: f64,
    y: f64,
    kind: i32,
}
fn vertex(point: Point3, tolerance: f64) -> Vertex {
    let mut result = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut result, point, tolerance)
        .unwrap();
    result
}
fn edge(spec: EdgeSpec, shared: &Vertex) -> Edge {
    let builder = BRepBuilder::default();
    let mut result = Edge::default();
    if spec.kind == 3 {
        builder.make_edge(&mut result).unwrap();
    } else {
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
            Point3::new(spec.x, spec.y, 0.),
            Direction3::new(1., 0., 0.).unwrap(),
        ))
        .cast();
        builder
            .make_edge_with_curve(&mut result, &curve, 1e-7)
            .unwrap();
        builder.range(&result, 0., 1., false).unwrap();
    }
    let (mut first, mut last) = edge_vertices(spec, shared);
    first.set_orientation(Orientation::Forward);
    last.set_orientation(Orientation::Reversed);
    builder.add(&mut result, &first).unwrap();
    builder.add(&mut result, &last).unwrap();
    result
}
fn edge_vertices(spec: EdgeSpec, shared: &Vertex) -> (Vertex, Vertex) {
    let first = if spec.kind == 1 {
        vertex(Point3::new(spec.x, spec.y, 0.), 1e-7)
    } else {
        shared.clone()
    };
    let last = if spec.kind == 1 || spec.kind == 2 {
        shared.clone()
    } else {
        vertex(Point3::new(spec.x + 1., spec.y, 0.), 1e-7)
    };
    (first, last)
}
fn fixture(specs: &[(f64, f64, i32)], tolerance: f64) -> (Shape, Vertex) {
    let shared = vertex(Point3::new(10., 10., 0.), tolerance);
    let mut compound = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut compound);
    for &(x, y, kind) in specs {
        builder
            .add(&mut compound, &edge(EdgeSpec { x, y, kind }, &shared))
            .unwrap();
    }
    (compound.into_shape(), shared)
}
fn observations(shape: &Shape) -> Vec<f64> {
    let mut result = Vec::new();
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        let (first, last) = topexp::vertices(&topods::edge(explorer.current()).unwrap(), false);
        for endpoint in [first, last] {
            let point = BRepTool::pnt(&endpoint).unwrap();
            result.extend([
                point.x(),
                point.y(),
                BRepTool::vertex_tolerance(&endpoint).unwrap(),
            ]);
        }
        explorer.next().unwrap();
    }
    result
}
fn assert_case(name: &str, specs: &[(f64, f64, i32)], tolerance: f64) {
    let (mut shape, shared) = fixture(specs, tolerance);
    let context = Handle::new(ShapeBuildReShape::new());
    let changed = ShapeFix::fix_vertex_position(&mut shape, 0.1, &context).unwrap();
    let mut actual = vec![
        f64::from(changed),
        BRepTool::vertex_tolerance(&shared).unwrap(),
    ];
    actual.extend(observations(&shape));
    let expected = expected_row(name);
    assert_eq!(actual.len(), expected.len(), "{name}");
    for (a, e) in actual.iter().zip(&expected) {
        assert!((a - e).abs() < 1e-14, "{name}: {a} != {e}");
    }
}
fn expected_row(name: &str) -> Vec<f64> {
    let row = include_str!("../../../tools/oracle/tkshhealing_vertex_position.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    expected
}
#[test]
fn no_repair_for_empty_single_low_tolerance_or_missing_curve() {
    assert_case("empty", &[], 20.);
    assert_case("single", &[(0., 0., 0)], 20.);
    assert_case("low", &[(0., 0., 0), (0., 0.02, 0)], 0.01);
    assert_case("no_curve", &[(0., 0., 3), (0., 0.02, 0)], 20.);
}
#[test]
fn shared_first_last_and_incremental_centers_match_native() {
    assert_case("pair", &[(0., 0., 0), (0., 0.02, 0)], 20.);
    assert_case("last", &[(-1., 0., 1), (0., 0.02, 0)], 20.);
    assert_case("triple", &[(0., 0., 0), (0., 0.02, 0), (0., 0.04, 0)], 20.);
}
#[test]
fn rejected_edges_and_recursive_nearest_group_match_native() {
    assert_case("reject", &[(0., 0., 0), (0., 2., 0)], 20.);
    assert_case("recursive", &[(0., 2., 0), (0., 0., 0), (0., 0.02, 0)], 20.);
}
#[test]
fn closed_edges_split_the_endpoint_away_from_selected_center() {
    assert_case("closed", &[(0., 0., 2), (0., 0.02, 0)], 20.);
    assert_case("closed_last", &[(0., 0., 2), (0., 0.02, 1)], 20.);
    assert_case("closed_reject", &[(0., 0., 2), (0., 2., 0)], 20.);
}

#[test]
fn loop_reordering_and_separate_endpoint_groups_match_native() {
    assert_case("closed_pair", &[(0., 0., 2), (0., 0.02, 2)], 20.);
    assert_case(
        "closed_groups",
        &[(0., 0., 2), (0., 0.02, 0), (0., 0.02, 1)],
        20.,
    );
}
#[test]
fn null_context_is_only_required_when_replacement_is_needed() {
    let (mut empty, _) = fixture(&[], 20.);
    assert!(!ShapeFix::fix_vertex_position(&mut empty, 0.1, &Handle::default()).unwrap());
    let (mut pair, _) = fixture(&[(0., 0., 0), (0., 0.02, 0)], 20.);
    assert!(ShapeFix::fix_vertex_position(&mut pair, 0.1, &Handle::default()).is_err());
}
