use super::*;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::geom_abs::CurveType;

#[test]
fn smooth_bezier_chain_concatenates_when_requested() {
    let input = bezier_chain_wire();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, true);
    unifier.build().unwrap();
    let result = unifier.shape();
    let edges = edge_shapes(&result);
    assert_eq!(edges.len(), native_edge_count("bezier_chain"));
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let (start, end) = edge_endpoints(&edges[0]);
    assert_eq!(
        endpoint_coordinates(start, end),
        native_endpoints("bezier_chain")
    );
    assert_native_bspline_sample(&edges[0]);
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

fn assert_native_bspline_sample(edge: &rustycad_tkbrep::topods::Edge) {
    let curve = BRepAdaptorCurve::with_edge(edge).unwrap();
    assert_eq!(curve.get_type().unwrap(), CurveType::BSplineCurve);
    assert_eq!(curve.first_parameter().unwrap(), 0.);
    assert_eq!(curve.last_parameter().unwrap(), 2.);
    let middle = curve.value(1.).unwrap();
    assert!((middle.x() - 1.).abs() < 1e-12);
    assert!((middle.y() - 1.).abs() < 1e-12);
    assert!(middle.z().abs() < 1e-12);
}

fn bezier_chain_wire() -> Shape {
    let first = bezier_edge([
        Point3::new(0., 0., 0.),
        Point3::new(0.5, 0., 0.),
        Point3::new(1., 1., 0.),
    ]);
    let second = bezier_edge([
        Point3::new(1., 1., 0.),
        Point3::new(1.5, 2., 0.),
        Point3::new(2., 4., 0.),
    ]);
    let mut wire = BRepBuilderAPIMakeWire::new();
    wire.add_edge(&first).unwrap();
    wire.add_edge(&second).unwrap();
    wire.wire().unwrap().into_shape()
}

fn bezier_edge(points: [Point3; 3]) -> rustycad_tkbrep::topods::Edge {
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
}
