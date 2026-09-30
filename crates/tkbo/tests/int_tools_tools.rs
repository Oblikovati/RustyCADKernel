//! Regressions for intentional differences between native source and header descriptions.
#[path = "../examples/bo_probe/tools_geometry.rs"]
mod geometry;
use rustycad_tkbo::int_tools::{CommonPrt, Curve, Range, Tools};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Direction3, Line3, Plane3, Point3},
};
#[test]
fn interval_tests_preserve_native_strictness_and_asymmetry() {
    let range = Range::new(0., 1.);
    assert!(!Tools::is_on_pave(0., &range, 0.));
    assert!(Tools::is_on_pave1(0., &range, 0.));
    assert!(!Tools::is_on_pave(0.5, &range, 0.5));
    assert!(!Tools::is_in_range(&range, &Range::new(-1., 2.), 0.));
    assert!(Tools::is_in_range(&Range::new(-1., 2.), &range, 0.));
}
#[test]
fn vertex_parameters_write_first_before_second_range_error() {
    let mut part = CommonPrt::default();
    part.set_range1_bounds(2., 4.);
    part.set_vertex_parameter1(10.);
    let (mut first, mut last) = (31., 32.);
    let failure = Tools::vertex_parameters(&part, &mut first, &mut last).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!((first, last), (3., 32.));
    part.append_range2_bounds(10., 20.);
    part.set_vertex_parameter2(12.);
    Tools::vertex_parameters(&part, &mut first, &mut last).unwrap();
    assert_eq!((first, last), (3., 12.));
}
#[test]
fn opposite_directions_and_right_angle_use_native_special_cases() {
    let first = Direction3::new(1., 0., 0.).unwrap();
    let last = Direction3::new(-1., 0., 0.).unwrap();
    assert!(Tools::is_dirs_coinside(first, last));
    assert!(!Tools::is_dirs_coinside_tolerance(first, last, 0.));
    assert_eq!(
        Tools::compute_int_range(2., 3., std::f64::consts::FRAC_PI_2),
        3.
    );
}
#[test]
fn negative_plane_side_is_common_and_retains_outputs() {
    let line = Line3::new(
        Point3::new(0., 0., -10.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let mut point = Point3::new(31., 32., 33.);
    let (mut t, mut tol, mut lo, mut hi) = (34., 35., 36., 37.);
    let status = segment_status(&line, &mut point, &mut t, &mut tol, &mut lo, &mut hi);
    assert_eq!(status, 1);
    assert_eq!(point, Point3::new(31., 32., 33.));
    assert_eq!((t, tol, lo, hi), (34., 35., 36., 37.));
}
#[test]
fn closed_bezier_splits_at_asymmetric_parameter_and_appends() {
    let geom = geometry::geometry(3, 0., 1.).unwrap();
    let curve = Curve::new(&geom, &Handle::default(), &Handle::default(), 0.1, 0.2);
    let mut result = vec![curve.clone()];
    assert_eq!(Tools::split_curve(&curve, &mut result).unwrap(), 2);
    assert_eq!(result.len(), 3);
    assert_eq!(result[1].curve().last_parameter(), 0.43213918);
    assert_eq!(result[2].curve().first_parameter(), 0.43213918);
    assert_eq!(result[1].tolerance(), 0.1);
    assert_eq!(result[2].tangential_tolerance(), 0.2);
}
#[test]
fn reject_lines_mutates_input_and_stops_at_second_direction() {
    let first = geometry::geometry(0, 2., 4.).unwrap();
    let other = geometry::geometry(6, 2., 4.).unwrap();
    let a = Curve::new(&first, &Handle::default(), &Handle::default(), 0., 0.);
    let b = Curve::new(&other, &Handle::default(), &Handle::default(), 0., 0.);
    let mut input = vec![a.clone(), b, a];
    let mut output = Vec::new();
    Tools::reject_lines(&mut input, &mut output).unwrap();
    assert_eq!(output.len(), 2);
    assert!(!input[0].has_bounds());
    assert!(!input[1].has_bounds());
    assert!(input[2].has_bounds());
}
#[test]
fn nonline_rejection_restores_full_mutated_input() {
    let first = geometry::geometry(0, 2., 4.).unwrap();
    let other = geometry::geometry(2, 2., 4.).unwrap();
    let mut input = vec![
        Curve::new(&first, &Handle::default(), &Handle::default(), 0., 0.),
        Curve::new(&other, &Handle::default(), &Handle::default(), 0., 0.),
    ];
    let mut output = vec![Curve::default()];
    Tools::reject_lines(&mut input, &mut output).unwrap();
    assert_eq!(output.len(), 2);
    assert!(!input[0].has_bounds());
    assert!(!input[1].has_bounds());
    assert!(input[1].curve() == output[1].curve());
}
#[test]
fn closure_requires_bounded_type_and_null_check_preserves_box() {
    let circle = geometry::geometry(1, 0., std::f64::consts::TAU).unwrap();
    assert!(!Tools::is_closed(&circle).unwrap());
    let trimmed = geometry::geometry(2, 0., std::f64::consts::TAU).unwrap();
    assert!(Tools::is_closed(&trimmed).unwrap());
    let mut bounds = BoundingBox3::default();
    assert!(!Tools::check_curve(&Curve::default(), &mut bounds).unwrap());
    assert!(bounds.is_void());
    assert_eq!(
        Tools::curve_tolerance(&Handle::default(), 0.1).unwrap(),
        0.1
    );
    assert_eq!(
        Tools::curve_tolerance(&geometry::geometry(4, -2., 3.).unwrap(), 0.1).unwrap(),
        0.1 * 4.5_f64.sqrt()
    );
}

fn segment_status(
    line: &Line3,
    point: &mut Point3,
    t: &mut f64,
    tol: &mut f64,
    lo: &mut f64,
    hi: &mut f64,
) -> i32 {
    Tools::seg_pln(
        line,
        0.,
        1.,
        1e-7,
        &Plane3::default(),
        1e-7,
        point,
        t,
        tol,
        lo,
        hi,
    )
    .unwrap()
}
