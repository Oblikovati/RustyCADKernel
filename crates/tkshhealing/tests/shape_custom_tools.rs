//! Behavior tests of ShapeCustom_Curve2d and ShapeCustom_RestrictionParameters.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2};
use rustycad_tkshhealing::shape_custom::{ShapeCustomCurve2d, ShapeCustomRestrictionParameters};

fn poles(points: &[(f64, f64)]) -> Vec<Point2> {
    points.iter().map(|&(x, y)| Point2::new(x, y)).collect()
}

fn bezier(points: &[(f64, f64)]) -> Handle<dyn Geom2dCurve> {
    let poles = poles(points);
    Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}

fn cubic(points: &[(f64, f64)], knots: &[f64], mults: &[i32]) -> Geom2dBSplineCurve {
    let poles = poles(points);
    Geom2dBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(knots, 1),
        &Array1::from_shared(mults, 1),
        3,
        false,
    )
    .unwrap()
}

#[test]
fn is_linear_measures_the_deviation_from_the_farthest_poles() {
    let straight = poles(&[(0.0, 0.0), (1.0, 1.0), (3.0, 3.0)]);
    let bent = poles(&[(0.0, 0.0), (1.0, 1e-4), (2.0, 0.0)]);
    let one = poles(&[(1.0, 2.0)]);
    let close = poles(&[(0.0, 0.0), (1e-10, 0.0)]);
    let linear =
        |p: &[Point2], tol| ShapeCustomCurve2d::is_linear(&Array1::from_shared(p, 1), tol).unwrap();
    assert_eq!(linear(&straight, 0.0), Some(0.0));
    assert!(linear(&bent, 1e-3).is_some_and(|d| (d - 1e-4).abs() < 1e-12));
    assert_eq!(linear(&bent, 1e-5), None);
    assert_eq!(linear(&one, 1.0), None);
    assert_eq!(linear(&close, 1.0), None);
}

#[test]
fn convert_to_line2d_moves_the_line_origin_to_the_first_parameter() {
    let curve = bezier(&[(0.0, 0.0), (2.0, 1.0), (4.0, 2.0)]);
    let line = ShapeCustomCurve2d::convert_to_line2d(&curve, 0.25, 1.0, 1e-7)
        .unwrap()
        .expect("linear poles");
    // Quirk kept: the line is located at its own point of parameter c1, so the first
    // point gets the parameter -c1.
    assert!((line.first + 0.25).abs() < 1e-15);
    assert!(line.last > line.first);
    let bent = bezier(&[(0.0, 0.0), (2.0, 3.0), (4.0, 2.0)]);
    assert!(
        ShapeCustomCurve2d::convert_to_line2d(&bent, 0.0, 1.0, 1e-7)
            .unwrap()
            .is_none()
    );
    let axis = Axis22d::new(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        true,
    )
    .unwrap();
    let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::new(axis, 1.0).unwrap()).cast();
    assert!(
        ShapeCustomCurve2d::convert_to_line2d(&circle, 0.0, 1.0, 1e-7)
            .unwrap()
            .is_none()
    );
    assert!(ShapeCustomCurve2d::convert_to_line2d(&Handle::default(), 0.0, 1.0, 1e-7).is_err());
}

#[test]
fn simplify_bspline2d_removes_knots_within_the_tolerance() {
    let points = [
        (0.0, 0.0),
        (1.0, 2.0),
        (2.0, -1.0),
        (3.0, 1.0),
        (4.0, 0.5),
        (5.0, 3.0),
    ];
    let tight = cubic(&points, &[0.0, 1.0, 2.0, 3.0], &[4, 1, 1, 4]);
    assert!(!ShapeCustomCurve2d::simplify_bspline2d(&tight, 1e-7).unwrap());
    assert_eq!(tight.nb_knots(), 4);
    let loose = cubic(&points, &[0.0, 1.0, 2.0, 3.0], &[4, 1, 1, 4]);
    assert!(ShapeCustomCurve2d::simplify_bspline2d(&loose, 10.0).unwrap());
    assert_eq!(loose.nb_knots(), 3);
    let bare = cubic(
        &[(0.0, 0.0), (1.0, 1.0), (2.0, -1.0), (3.0, 0.0)],
        &[0.0, 1.0],
        &[4, 4],
    );
    assert!(!ShapeCustomCurve2d::simplify_bspline2d(&bare, 1e-7).unwrap());
}

#[test]
fn restriction_parameters_keep_the_native_defaults_and_cross_two_flags() {
    let parameters = ShapeCustomRestrictionParameters::new();
    assert_eq!(
        (parameters.g_max_degree(), parameters.g_max_seg()),
        (15, 10000)
    );
    assert!(!parameters.convert_plane() && parameters.convert_revolution_surf());
    parameters.set_convert_toroidal_surf(true);
    assert!(parameters.convert_toroidal_surf());
    assert!(!parameters.convert_cylindrical_surf());
    parameters.set_g_max_seg(3);
    assert_eq!(parameters.g_max_seg(), 3);
}
