//! Behavior tests of the BRepCheck package functions, the status enum, the base
//! members of BRepCheck_Result and BRepLib_ValidateEdge. BRepCheck has no upstream
//! suite for these classes; the probes compare them with OCCT (bck_* and ble_validate).
mod brep_check_support;

use brep_check_support::{TOL, square, vertex};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::real::{epsilon, real_epsilon};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomEllipse, GeomLine, GeomPlane, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tktopalgo::brep_check::{
    BRepCheck, BRepCheckResult, BRepCheckStatus, BRepCheckVertex, ListOfStatus,
};
use rustycad_tktopalgo::brep_lib::BRepLibValidateEdge;

fn statuses(list: &ListOfStatus) -> Vec<BRepCheckStatus> {
    list.iter().copied().collect()
}

#[test]
fn status_values_follow_the_native_enum() {
    assert_eq!(BRepCheckStatus::ALL.len(), 37);
    for (i, s) in BRepCheckStatus::ALL.iter().enumerate() {
        assert_eq!(*s as usize, i);
        assert_eq!(BRepCheckStatus::from_index(i as i32), Some(*s));
    }
    assert_eq!(BRepCheckStatus::from_index(-1), None);
    assert_eq!(BRepCheckStatus::from_index(37), None);
    assert_eq!(
        BRepCheckStatus::EnclosedRegion.name(),
        "BRepCheck_EnclosedRegion"
    );
}

#[test]
fn add_removes_no_error_before_a_defect_and_keeps_statuses_unique() {
    use BRepCheckStatus::*;
    let mut list = ListOfStatus::default();
    BRepCheck::add(&mut list, NoError);
    BRepCheck::add(&mut list, NoError);
    assert_eq!(statuses(&list), [NoError]);
    BRepCheck::add(&mut list, FreeEdge);
    BRepCheck::add(&mut list, FreeEdge);
    assert_eq!(statuses(&list), [FreeEdge]);
    // A NoError after a defect is appended (only defects remove NoError).
    BRepCheck::add(&mut list, NoError);
    BRepCheck::add(&mut list, NotClosed);
    assert_eq!(statuses(&list), [FreeEdge, NotClosed]);
}

#[test]
fn print_writes_the_enumerator_name_and_a_line_end() {
    let mut out: Vec<u8> = Vec::new();
    BRepCheck::print(BRepCheckStatus::InvalidWire, &mut out).unwrap();
    assert_eq!(out, b"BRepCheck_InvalidWire\n");
}

#[test]
fn precisions_are_real_epsilon_except_for_ellipses_and_cones() {
    let z = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::new(1.0, 2.0, 3.0), z)).cast();
    assert_eq!(
        BRepCheck::prec_curve(&GeomAdaptorCurve::with_curve(&line).unwrap()).unwrap(),
        real_epsilon()
    );
    let frame = Axis2::new(Point3::new(-1e6, 2.0, 3.0), z).unwrap();
    let ellipse: Handle<dyn GeomCurve> =
        Handle::new(GeomEllipse::new(frame, 5.0, 1.0).unwrap()).cast();
    let prec = BRepCheck::prec_curve(&GeomAdaptorCurve::with_curve(&ellipse).unwrap()).unwrap();
    assert_eq!(prec, epsilon(1e6));
    let cone = GeomConicalSurface::new(
        Axis3::from_normal(Point3::new(0.0, 0.0, 7.0), z).unwrap(),
        0.5,
        100.0,
    );
    let cone: Handle<dyn GeomSurface> = Handle::new(cone.unwrap()).cast();
    let adaptor: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&cone).unwrap()).cast();
    assert_eq!(BRepCheck::prec_surface(&adaptor).unwrap(), epsilon(100.0));
    let plane: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(Point3::new(0.0, 0.0, 0.0), z).unwrap()).cast();
    let adaptor: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()).cast();
    assert_eq!(BRepCheck::prec_surface(&adaptor).unwrap(), real_epsilon());
    assert!(BRepCheck::prec_surface(&Handle::default()).is_err());
}

#[test]
fn self_intersection_finds_the_crossing_edges_of_a_bow_tie() {
    let sq = square(1.0, &[], &[], 0.0);
    let (mut e1, mut e2) = (Edge::default(), Edge::default());
    let found =
        BRepCheck::self_intersection(&sq.wires[0][0], &sq.faces[0], &mut e1, &mut e2).unwrap();
    assert!(!found && e1.is_null() && e2.is_null());
    let mut bow = brep_check_support::Planar::new(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
    ]);
    bow.face(
        &[&[0, 1, 2, 3]],
        Some(rustycad_tkmath::gp::Vector3::new(0.0, 0.0, 1.0)),
        &[],
        0.0,
    );
    bow.finish();
    let found =
        BRepCheck::self_intersection(&bow.wires[0][0], &bow.faces[0], &mut e1, &mut e2).unwrap();
    assert!(found);
    assert!(e1.is_same(&bow.edges[&(0, 1)]) && e2.is_same(&bow.edges[&(2, 3)]));
}

#[test]
fn result_contexts_skip_the_shape_itself() {
    let sq = square(1.0, &[], &[], 0.0);
    let v = &sq.vertices[0];
    let checker = BRepCheckVertex::new(v).unwrap();
    assert!(checker.is_minimum() && !checker.is_blind() && !checker.is_parallel());
    let edge: Shape = sq.edges[&(0, 1)].clone().into_shape();
    let face: Shape = sq.faces[0].clone().into_shape();
    checker.in_context(&edge).unwrap();
    checker.in_context(&face).unwrap();
    checker.set_fail_status(v);
    assert_eq!(
        statuses(&checker.status().unwrap()),
        [BRepCheckStatus::CheckFail]
    );
    let mut seen = Vec::new();
    checker.init_context_iterator().unwrap();
    while checker.more_shape_in_context() {
        seen.push(checker.contextual_shape().unwrap());
        assert_eq!(
            statuses(&checker.status_on_shape().unwrap()),
            [BRepCheckStatus::NoError]
        );
        checker.next_shape_in_context();
    }
    assert_eq!(seen.len(), 2);
    assert!(checker.contextual_shape().is_err() && checker.status_on_shape().is_err());
    let other: Shape = vertex(Point3::new(5.0, 5.0, 5.0), TOL).into_shape();
    assert!(!checker.is_status_on_shape(&other));
    let missing = checker
        .status_on_shape_of(&other)
        .err()
        .expect("no statuses");
    assert_eq!(missing.exception_type(), "Standard_NoSuchObject");
    checker.blind();
    checker.set_parallel(true);
    assert!(checker.is_blind() && checker.is_parallel());
    checker.init(&sq.vertices[1]).unwrap();
    assert!(!checker.is_blind() && !checker.is_status_on_shape(&edge));
}

/// The adaptors of a unit segment on the x axis and of a line pcurve on z = 0.
fn segment_and_pcurve(
    shift: f64,
    range: (f64, f64),
) -> (Handle<dyn Adaptor3dCurve>, Handle<Adaptor3dCurveOnSurface>) {
    let x = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::new(0.0, 0.0, 0.0), x)).cast();
    let reference: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(&line, 0.0, 1.0).unwrap()).cast();
    let dir = Direction2::new(1.0, 0.0).unwrap();
    let pc: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0.0, shift), dir)).cast();
    let pc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&pc, range.0, range.1).unwrap()).cast();
    let z = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let plane: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(Point3::new(0.0, 0.0, 0.0), z).unwrap()).cast();
    let plane: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()).cast();
    (
        reference,
        Handle::new(Adaptor3dCurveOnSurface::with_curve(&pc, &plane).unwrap()),
    )
}

#[test]
fn validate_edge_measures_the_distance_of_the_curve_on_surface() {
    let (reference, other) = segment_and_pcurve(1e-3, (0.0, 1.0));
    let mut validate = BRepLibValidateEdge::new(&reference, &other, true);
    assert_eq!(validate.get_max_distance(), 0.0);
    validate.process().unwrap();
    assert!(validate.is_done());
    assert!((validate.get_max_distance() - 1e-3 * 1.00001).abs() < 1e-15);
    assert!(!validate.check_tolerance(TOL).unwrap() && validate.check_tolerance(1e-2).unwrap());
    let mut tol = TOL;
    validate.update_tolerance(&mut tol);
    assert_eq!(tol, validate.get_max_distance());
    let mut large = 1.0;
    validate.update_tolerance(&mut large);
    assert_eq!(large, 1.0);
}

#[test]
fn validate_edge_options_change_the_method_not_the_result_on_a_line() {
    for (exact, points, parallel) in [
        (false, 0, false),
        (false, 1, true),
        (true, 22, false),
        (true, -4, true),
    ] {
        let (reference, other) = segment_and_pcurve(0.0, (0.0, 1.0));
        let mut validate = BRepLibValidateEdge::new(&reference, &other, true);
        validate.set_exact_method(exact);
        validate.set_control_points_number(points);
        validate.set_parallel(parallel);
        validate.process().unwrap();
        assert!(validate.is_done());
        assert_eq!(validate.is_exact_method(), exact);
        assert_eq!(validate.is_parallel(), parallel);
        assert!(validate.get_max_distance() <= precision::confusion());
    }
}

#[test]
fn validate_edge_projects_another_parametrization_and_stops_early() {
    let (reference, other) = segment_and_pcurve(0.5, (0.0, 2.0));
    let mut validate = BRepLibValidateEdge::new(&reference, &other, true);
    validate.set_exit_if_tolerance_exceeded(TOL).unwrap();
    validate.process().unwrap();
    // The ends are compared first: the distance at the start already exceeds TOL.
    assert!(validate.is_done());
    assert!((validate.get_max_distance() - 0.5 * 1.00001).abs() < 1e-12);
    let mut null = BRepLibValidateEdge::new(&Handle::default(), &other, true);
    assert_eq!(
        null.process().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}
