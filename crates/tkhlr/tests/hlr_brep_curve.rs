//! HLRBRep_Curve and HLRBRep_CLProps: the projected type, the parameters and the box.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine};
use rustycad_tkhlr::hlr_algo::{Projector, init_min_max};
use rustycad_tkhlr::hlr_brep::{CLProps, Curve};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Direction3, Point2, Point3};
use std::sync::Arc;

fn edge(curve: Handle<dyn GeomCurve>, u1: f64, u2: f64) -> Edge {
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder
        .make_edge_with_curve(&mut edge, &curve, 1e-7)
        .unwrap();
    builder.range(&edge, u1, u2, false).unwrap();
    edge
}

fn projected(curve: Handle<dyn GeomCurve>, view: Direction3) -> Curve {
    let frame = Axis2::new(Point3::new(0., 0., 0.), view).unwrap();
    let mut c = Curve::new();
    c.set_projector(Some(Arc::new(Projector::with_frame(frame).unwrap())));
    c.set_curve(&edge(curve, 0., 2.)).unwrap();
    let (mut mn, mut mx) = ([0.; 16], [0.; 16]);
    init_min_max(1e100, &mut mn, &mut mx);
    c.update(&mut mn, &mut mx).unwrap();
    c
}

fn circle(normal: Direction3) -> Handle<dyn GeomCurve> {
    let frame = Axis2::new(Point3::new(1., 2., 3.), normal).unwrap();
    Handle::new(GeomCircle::new(frame, 2.).unwrap()).cast()
}

#[test]
fn a_curve_without_projector_fails_with_null_object() {
    let c = Curve::new();
    let error = c.parameter_2d(1.).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
}

#[test]
fn a_circle_keeps_its_type_only_when_seen_along_its_axis() {
    let z = Direction3::new(0., 0., 1.).unwrap();
    let x = Direction3::new(1., 0., 0.).unwrap();
    let oblique = Direction3::new(1., 0., 1.).unwrap();
    assert_eq!(projected(circle(z), z).get_type(), CurveType::Circle);
    assert_eq!(projected(circle(x), z).get_type(), CurveType::OtherCurve);
    assert_eq!(projected(circle(oblique), z).get_type(), CurveType::Ellipse);
}

#[test]
fn a_parallel_line_scales_its_parameter_by_the_projected_speed() {
    let z = Direction3::new(0., 0., 1.).unwrap();
    let d = Direction3::new(1., 0., 1.).unwrap();
    let line = Handle::new(GeomLine::from_point(Point3::new(0., 0., 0.), d)).cast();
    let c = projected(line, z);
    assert_eq!(c.get_type(), CurveType::Line);
    let p2 = c.parameter_2d(2.).unwrap();
    assert!((p2 - 2. * std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-15);
    assert!((c.parameter_3d(p2).unwrap() - 2.).abs() < 1e-15);
}

#[test]
fn the_tangent_of_a_projected_circle_is_its_unit_derivative() {
    let z = Direction3::new(0., 0., 1.).unwrap();
    let c = projected(circle(z), z);
    let (p, d) = c.tangent(true).unwrap();
    assert!((p.x() - 3.).abs() < 1e-12 && (p.y() - 2.).abs() < 1e-12);
    assert!(d.x().abs() < 1e-12 && (d.y() - 1.).abs() < 1e-12);
    let mut props = CLProps::new(&c, 2, 1e-9).unwrap();
    props.set_parameter(0.).unwrap();
    assert!((props.curvature().unwrap() - 0.5).abs() < 1e-12);
}

#[test]
fn d3_and_dn_leave_their_outputs_as_natively_unimplemented() {
    let z = Direction3::new(0., 0., 1.).unwrap();
    let c = projected(circle(z), z);
    let mut p = Point2::new(7., 8.);
    let (mut v1, mut v2, mut v3) = Default::default();
    c.d3(0.5, &mut p, &mut v1, &mut v2, &mut v3);
    assert_eq!((p.x(), p.y()), (7., 8.));
    assert_eq!(c.dn(0.5, 1).magnitude(), 0.);
}
