//! Supplemental ElCLib checks beyond the upstream suite: planar evaluators, N-th derivative
//! cycles, To3d embeddings, error paths and the negative-order parity regression.
use rustycad_tkmath::elclib::{self, SecondDerivativeCurve, ThirdDerivativeCurve};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Axis22d, Circle2, Direction2, Direction3, Ellipse2, Hyperbola2, Hyperbola3,
    Line2, Parabola2, Point2, Point3, Vector2, Vector3,
};

fn plane_frame(sense: bool) -> Axis22d {
    Axis22d::new(
        Point2::new(1.0, -1.0),
        Direction2::new(0.8, 0.6).unwrap(),
        sense,
    )
}
fn space_frame() -> Axis2 {
    Axis2::with_x_direction(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(0.0, 1.0, 1.0).unwrap(),
        Direction3::X,
    )
    .unwrap()
}
fn close2(left: Vector2, right: Vector2) {
    assert!((left - right).magnitude() < 1e-12, "{left:?} vs {right:?}");
}

#[test]
fn planar_parameters_invert_values_in_both_senses() {
    for sense in [true, false] {
        let frame = plane_frame(sense);
        let circle = Circle2::new(frame, 2.0).unwrap();
        let ellipse = Ellipse2::new(frame, 3.0, 1.0).unwrap();
        let hyperbola = Hyperbola2::new(frame, 2.0, 0.5).unwrap();
        let parabola = Parabola2::new(frame, 0.5).unwrap();
        for u in [0.25, 2.0, 5.5] {
            let circle_u = elclib::parameter(&circle, elclib::value(u, &circle)).unwrap();
            assert!((circle_u - u).abs() < 1e-12);
            let ellipse_u = elclib::parameter(&ellipse, elclib::value(u, &ellipse)).unwrap();
            assert!((ellipse_u - u).abs() < 1e-12);
            let hyperbola_u = elclib::parameter(&hyperbola, elclib::value(u - 3.0, &hyperbola));
            assert!((hyperbola_u.unwrap() - (u - 3.0)).abs() < 1e-12);
            let parabola_u = elclib::parameter(&parabola, elclib::value(u, &parabola)).unwrap();
            assert!((parabola_u - u).abs() < 1e-12);
        }
    }
    let line = Line2::new(Point2::new(1.0, 1.0), Direction2::Y);
    assert_eq!(
        elclib::parameter(&line, Point2::new(7.0, 4.0)).unwrap(),
        3.0
    );
}

#[test]
fn planar_parameters_raise_at_the_centre() {
    let frame = plane_frame(true);
    let circle = Circle2::new(frame, 2.0).unwrap();
    let error = elclib::parameter(&circle, frame.location()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let ellipse = Ellipse2::new(frame, 3.0, 1.0).unwrap();
    let error = elclib::parameter(&ellipse, frame.location()).unwrap_err();
    assert_eq!(error.exception_type(), "gp_VectorWithNullMagnitude");
    // The 3D circle returns zero for a coinciding point instead.
    assert_eq!(
        elclib::circle_parameter(space_frame(), space_frame().location()),
        0.0
    );
}

#[test]
fn derivative_orders_cycle_consistently() {
    let frame = plane_frame(false);
    let ellipse = Ellipse2::new(frame, 3.0, 1.0).unwrap();
    let (_, first, second, third) = ellipse.d3(0.7);
    close2(elclib::dn(0.7, &ellipse, 1), first);
    close2(elclib::dn(0.7, &ellipse, 2), second);
    close2(elclib::dn(0.7, &ellipse, 3), third);
    close2(elclib::dn(0.7, &ellipse, 5), first);
    close2(
        elclib::dn(0.7, &ellipse, 4),
        elclib::ellipse_dn_2d(0.7, frame, 3.0, 1.0, 0),
    );
    let parabola = Parabola2::new(frame, 0.5).unwrap();
    let (_, _, curvature) = parabola.d2(1.3);
    close2(elclib::dn(1.3, &parabola, 2), curvature);
    assert_eq!(elclib::dn(1.3, &parabola, 3), Vector2::new(0.0, 0.0));
    assert_eq!(
        elclib::line_dn_2d(0.0, Axis2d::default(), 2),
        Vector2::new(0.0, 0.0)
    );
}

#[test]
fn hyperbola_dn_is_zero_for_negative_odd_orders() {
    // Regression: Standard_Integer's IsOdd is `n % 2 == 1`, so -3 is neither odd nor even and
    // native HyperbolaDN leaves both coefficients at zero.
    let hyperbola = Hyperbola3::new(space_frame(), 2.0, 1.0).unwrap();
    assert_eq!(elclib::dn(0.5, &hyperbola, -3).magnitude(), 0.0);
    assert!(elclib::dn(0.5, &hyperbola, -2).magnitude() > 0.0);
    let planar = Hyperbola2::new(plane_frame(true), 2.0, 1.0).unwrap();
    assert_eq!(elclib::dn(0.5, &planar, -1).magnitude(), 0.0);
    assert_eq!(
        elclib::dn(0.5, &hyperbola, 3),
        elclib::d3(0.5, &hyperbola).3
    );
}

#[test]
fn embeddings_map_planar_geometry_into_the_frame() {
    let frame = space_frame();
    let planar = plane_frame(true);
    let point = Point2::new(2.0, 3.0);
    let embedded = elclib::to3d_point(frame, point);
    let expected = frame
        .location()
        .translated(Vector3::from(frame.x_direction()) * 2.0)
        .translated(Vector3::from(frame.y_direction()) * 3.0);
    assert!(embedded.distance(&expected) < 1e-15);
    let direction = elclib::to3d_direction(frame, Direction2::Y).unwrap();
    assert_eq!(direction, frame.y_direction());
    let vector = elclib::to3d_vector(frame, Vector2::new(2.0, 0.0));
    assert_eq!(vector, Vector3::from(frame.x_direction()) * 2.0);
    let axis = elclib::to3d_axis(frame, planar.x_axis()).unwrap();
    assert!(
        axis.location()
            .distance(&elclib::to3d_point(frame, planar.location()))
            < 1e-15
    );
    let direct = elclib::to3d_frame(frame, planar).unwrap();
    assert!(direct.direction().is_equal(&frame.direction(), 1e-12));
    // An indirect planar frame yields the reversed normal.
    let indirect = elclib::to3d_frame(frame, plane_frame(false)).unwrap();
    assert!(indirect.direction().is_opposite(frame.direction(), 1e-12));
    let parabola = elclib::to3d_parabola(frame, Parabola2::new(planar, 0.5).unwrap()).unwrap();
    assert_eq!(parabola.focal(), 0.5);
    let line = elclib::to3d_line(frame, Line2::new(point, Direction2::X)).unwrap();
    assert!(line.contains(elclib::to3d_point(frame, Point2::new(9.0, 3.0)), 1e-12));
    let ellipse = elclib::to3d_ellipse(frame, Ellipse2::new(planar, 2.0, 1.0).unwrap()).unwrap();
    assert_eq!((ellipse.major_radius(), ellipse.minor_radius()), (2.0, 1.0));
    let hyperbola =
        elclib::to3d_hyperbola(frame, Hyperbola2::new(planar, 2.0, 1.0).unwrap()).unwrap();
    assert_eq!(hyperbola.minor_radius(), 1.0);
}

#[test]
fn periodic_helpers_ignore_infinite_or_empty_periods() {
    assert_eq!(elclib::in_period(5.0, 0.0, 2e100), 5.0);
    assert_eq!(elclib::in_period(5.0, 1.0, 1.0), 5.0);
    assert_eq!(
        elclib::adjust_periodic(1.0, 1.0, 1e-7, 3.0, 4.0),
        (1.0, 1.0)
    );
    assert_eq!(
        elclib::adjust_periodic(-1e101, 1.0, 1e-7, 3.0, 4.0),
        (-1e101, 1.0)
    );
    // First ends within precision of the last bound move back one period.
    let (u1, _) = elclib::adjust_periodic(0.0, 1.0, 0.1, 0.95, 2.0);
    assert!((u1 + 0.05).abs() < 1e-15);
}
