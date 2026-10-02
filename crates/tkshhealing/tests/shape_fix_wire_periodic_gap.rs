//! Native parity probes for the periodic-cone `ShapeFix_Wire::FixGaps2d` path.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomCurve, GeomSurface};
use rustycad_tkgeomalgo::geom_api::Interpolate;
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisWire;
use rustycad_tkshhealing::shape_build::ShapeBuildEdge;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixFace, ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
};
use std::f64::consts::PI;

#[test]
fn periodic_cone_gap_curve_copy_moves_both_endpoint_poles() {
    let (face, _) = fixed_cone_ring_edge();
    let edge = ring_edge(&face);
    let pcurve = BRepTool::curve_on_face(&edge, &face).unwrap();
    let midpoint = Point2::new(0.0, 0.5);
    let copy = ShapeBuildEdge
        .copy_bspline_pcurve_with_endpoints(&pcurve.pcurve, midpoint, midpoint)
        .unwrap();
    assert_native_ring_endpoints(&pcurve, midpoint);
    assert!(copy.value(pcurve.first).unwrap().distance(midpoint) < 1e-9);
    assert!(copy.value(pcurve.last).unwrap().distance(midpoint) < 1e-9);
}

#[test]
fn periodic_cone_wire_gap_fix_matches_native_context_replacement() {
    let (face, edge) = fixed_cone_ring_edge();
    let maker = BRepBuilderAPIMakeWire::from_edge(&edge).unwrap();
    let wire = maker.wire().unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire(&wire, &face, 1e-7).unwrap());
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_gaps_2d().unwrap());
    assert_native_gap_result(&fixer, &analyzer, &face, &wire, &edge);
}

fn fixed_cone_ring_edge() -> (Face, rustycad_tkbrep::topods::Edge) {
    let face = periodic_cone_face();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.perform().unwrap();
    let fixed_face = fixer.face().unwrap();
    (fixed_face.clone(), ring_edge(&fixed_face))
}

fn periodic_cone_face() -> Face {
    let (cone, wire) = periodic_cone_fixture();
    let surface: Handle<dyn GeomSurface> = cone.cast();
    BRepBuilderAPIMakeFace::from_surface_wire(&surface, &wire, true)
        .unwrap()
        .face()
        .unwrap()
}

fn periodic_cone_fixture() -> (Handle<GeomConicalSurface>, Wire) {
    let axis = Axis3::new(
        Point3::new(0.0, 4.0, -9.1),
        Direction3::new(0.0, 0.09, -1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let cone = Handle::new(GeomConicalSurface::new(axis, 0.35, 0.0).unwrap());
    let mut points = Array1::with_bounds(1, 8).unwrap();
    for index in 1..=8 {
        points[index] = cone
            .value(2.0 * PI * (index - 1) as f64 / 8.0, 0.5)
            .unwrap();
    }
    let curve = interpolated_cone_curve(points);
    (cone, wire_from_curve(&curve))
}

fn interpolated_cone_curve(points: Array1<'static, Point3>) -> Handle<dyn GeomCurve> {
    let points = Handle::new(HArray1::from_owned(points));
    let mut interpolation = Interpolate::new(&points, true, 1e-6).unwrap();
    interpolation.perform().unwrap();
    interpolation.curve().unwrap().cast()
}

fn wire_from_curve(curve: &Handle<dyn GeomCurve>) -> Wire {
    let edge = BRepBuilderAPIMakeEdge::from_curve(curve).unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge.edge().unwrap()).unwrap();
    wire.wire().unwrap()
}

fn ring_edge(face: &Face) -> rustycad_tkbrep::topods::Edge {
    let wires = ShapeIterator::new(&face.clone().into_shape(), false, true);
    let wire = rustycad_tkbrep::topods::wire(wires.value().unwrap()).unwrap();
    for child in ShapeIterator::new(&wire.clone().into_shape(), false, true) {
        let edge = rustycad_tkbrep::topods::edge(&child).unwrap();
        let pcurve = BRepTool::curve_on_face(&edge, face).unwrap();
        let start = pcurve.pcurve.value(pcurve.first).unwrap();
        let end = pcurve.pcurve.value(pcurve.last).unwrap();
        if (start.y() - 0.5).abs() < 1e-6 && (end.y() - 0.5).abs() < 1e-6 {
            return edge;
        }
    }
    panic!("periodic cone ring p-curve was not found; expected v=0.5 endpoints");
}

fn assert_native_ring_endpoints(pcurve: &rustycad_tkbrep::brep::EdgePCurve, midpoint: Point2) {
    let start = pcurve.pcurve.value(pcurve.first).unwrap();
    let end = pcurve.pcurve.value(pcurve.last).unwrap();
    assert!(start.distance(midpoint) < 1e-9);
    assert!((end.x() - 2.0 * PI).abs() < 1e-9);
    assert!((end.y() - 0.5).abs() < 1e-9);
    assert!((pcurve.last - 1.0497704792601261).abs() < 1e-9);
}

fn assert_native_gap_result(
    fixer: &ShapeFixWire,
    analyzer: &ShapeAnalysisWire,
    face: &Face,
    wire: &Wire,
    old_edge: &rustycad_tkbrep::topods::Edge,
) {
    assert!(fixer.status_gaps_2d(ShapeExtendStatus::Done1));
    assert!(!fixer.status_gaps_2d(ShapeExtendStatus::Fail));
    let fixed = analyzer.wire_data().edge(1).unwrap();
    assert_closed_native_pcurve(&fixed, face);
    assert_eq!(BRepTool::edge_tolerance(&fixed).unwrap(), 1e-7);
    assert!(!fixed.is_same(old_edge));
    assert!(!BRepTool::same_range(&fixed).unwrap());
    assert_context_replacement(&fixer.context(), wire, old_edge, &fixed);
}

fn assert_closed_native_pcurve(edge: &rustycad_tkbrep::topods::Edge, face: &Face) {
    let pcurve = BRepTool::curve_on_face(edge, face).unwrap();
    let expected = Point2::new(0.0, 0.5);
    assert!(
        pcurve
            .pcurve
            .value(pcurve.first)
            .unwrap()
            .distance(expected)
            < 1e-9
    );
    assert!(pcurve.pcurve.value(pcurve.last).unwrap().distance(expected) < 1e-9);
    assert_eq!(
        BRepTool::range_on_face(edge, face).unwrap(),
        (0.0, 1.0497704792601261)
    );
    let (first, last) = rustycad_tkbrep::topexp::vertices(edge, false);
    assert!(first.is_same(&last));
    assert_eq!(BRepTool::vertex_tolerance(&last).unwrap(), 1e-7);
}

fn assert_context_replacement(
    context: &Handle<rustycad_tkshhealing::shape_build::ShapeBuildReShape>,
    wire: &Wire,
    old_edge: &rustycad_tkbrep::topods::Edge,
    expected: &rustycad_tkbrep::topods::Edge,
) {
    let result: Shape = ReShape::apply(
        context.get().unwrap(),
        &wire.clone().into_shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap();
    let fixed_wire = rustycad_tkbrep::topods::wire(&result).unwrap();
    let actual = rustycad_tkbrep::topods::edge(
        ShapeIterator::new(&fixed_wire.clone().into_shape(), false, true)
            .value()
            .unwrap(),
    )
    .unwrap();
    assert!(actual.is_same(expected));
    assert!(!actual.is_same(old_edge));
}
