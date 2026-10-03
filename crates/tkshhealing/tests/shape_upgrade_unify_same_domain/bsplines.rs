use super::*;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::geom_abs::CurveType;

#[test]
fn smooth_bezier_chain_concatenates_when_requested() {
    assert_native_merge(
        &bezier_chain_wire(),
        "bezier_chain",
        2.,
        Point3::new(1., 1., 0.),
    );
}

#[test]
fn three_bezier_edges_join_as_one_native_spline() {
    assert_native_merge(
        &bezier_wire(&parabola_segments(), false, false),
        "bezier_three",
        3.,
        Point3::new(1.5, 2.25, 0.),
    );
}

#[test]
fn reverse_insertion_order_keeps_the_native_bezier_chain() {
    assert_native_merge(
        &bezier_wire(&parabola_segments()[..2], false, true),
        "bezier_reverse_order",
        2.,
        Point3::new(1., 1., 0.),
    );
}

#[test]
fn reversed_bezier_occurrences_keep_the_native_chain_direction() {
    assert_native_merge(
        &bezier_wire(&parabola_segments()[..2], true, true),
        "bezier_reversed_occurrences",
        2.,
        Point3::new(1., 1., 0.),
    );
}

#[test]
fn bezier_chain_stays_split_when_concatenation_is_disabled() {
    let input = bezier_chain_wire();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let edges = edge_shapes(&unifier.shape());
    assert_eq!(edges.len(), native_edge_count("bezier_disabled"));
    let curve = BRepAdaptorCurve::with_edge(&edges[0]).unwrap();
    assert_eq!(curve.get_type().unwrap(), CurveType::BezierCurve);
    let middle = curve.value(0.5).unwrap();
    assert!((middle.x() - 0.5).abs() < 1e-12);
    assert!((middle.y() - 0.25).abs() < 1e-12);
}

#[test]
fn spline_boundary_candidate_on_a_face_is_reported_unimplemented() {
    let input = bezier_face();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, true);
    assert_eq!(
        unifier.build().unwrap_err().exception_type(),
        "Standard_NotImplemented"
    );
}

fn assert_native_merge(input: &Shape, case: &str, last: f64, middle: Point3) {
    let unifier = ShapeUpgradeUnifySameDomain::new(input, true, false, true);
    unifier.build().unwrap();
    let edges = edge_shapes(&unifier.shape());
    assert_eq!(edges.len(), native_edge_count(case));
    let (start, end) = edge_endpoints(&edges[0]);
    assert_eq!(endpoint_coordinates(start, end), native_endpoints(case));
    assert_native_bspline_sample(&edges[0], last, middle);
}

fn assert_native_bspline_sample(edge: &rustycad_tkbrep::topods::Edge, last: f64, expected: Point3) {
    let curve = BRepAdaptorCurve::with_edge(edge).unwrap();
    assert_eq!(curve.get_type().unwrap(), CurveType::BSplineCurve);
    assert_eq!(curve.first_parameter().unwrap(), 0.);
    assert_eq!(curve.last_parameter().unwrap(), last);
    let middle = curve.value(last * 0.5).unwrap();
    assert!((middle.x() - expected.x()).abs() < 1e-12);
    assert!((middle.y() - expected.y()).abs() < 1e-12);
    assert!(middle.z().abs() < 1e-12);
}

fn bezier_chain_wire() -> Shape {
    bezier_wire(&parabola_segments()[..2], false, false)
}

fn parabola_segments() -> [[Point3; 3]; 3] {
    [
        [
            Point3::new(0., 0., 0.),
            Point3::new(0.5, 0., 0.),
            Point3::new(1., 1., 0.),
        ],
        [
            Point3::new(1., 1., 0.),
            Point3::new(1.5, 2., 0.),
            Point3::new(2., 4., 0.),
        ],
        [
            Point3::new(2., 4., 0.),
            Point3::new(2.5, 6., 0.),
            Point3::new(3., 9., 0.),
        ],
    ]
}

fn bezier_wire(segments: &[[Point3; 3]], reverse: bool, reverse_order: bool) -> Shape {
    let mut edges = segments
        .iter()
        .copied()
        .map(bezier_edge)
        .collect::<Vec<_>>();
    if reverse {
        edges.iter_mut().for_each(|edge| edge.reverse());
    }
    if reverse_order {
        edges.reverse();
    }
    let mut wire = BRepBuilderAPIMakeWire::new();
    edges.iter().for_each(|edge| wire.add_edge(edge).unwrap());
    wire.wire().unwrap().into_shape()
}

fn bezier_face() -> Shape {
    let curved = bezier_chain_wire();
    let mut wire = BRepBuilderAPIMakeWire::new();
    for edge in edge_shapes(&curved) {
        wire.add_edge(&edge).unwrap();
    }
    for (first, last) in [
        (Point3::new(2., 4., 0.), Point3::new(0., 4., 0.)),
        (Point3::new(0., 4., 0.), Point3::new(0., 0., 0.)),
    ] {
        let edge = BRepBuilderAPIMakeEdge::from_points(first, last)
            .unwrap()
            .edge()
            .unwrap();
        wire.add_edge(&edge).unwrap();
    }
    let wire = wire.wire().unwrap();
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn bezier_edge(points: [Point3; 3]) -> rustycad_tkbrep::topods::Edge {
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
}
