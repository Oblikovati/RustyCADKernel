#[test]
fn construct_line_uses_both_points_even_when_only_one_is_taken() {
    let tool = ShapeConstructCurve;
    let curve = construct_line_3d();
    let first = Point3::new(0., 1., 0.);
    let last = Point3::new(1., 1., 0.);
    assert!(tool.adjust_curve(&curve, first, last, false, true).unwrap());
    let line = curve.cast::<GeomLine>();
    assert_eq!(line.lin().location(), Point3::new(2., 1., 0.));
    assert_eq!(line.lin().direction(), Direction3::X);
}

#[test]
fn construct_line_segment_ignores_reversed_parameter_bounds() {
    let tool = ShapeConstructCurve;
    let curve = construct_line_3d();
    let first = Point3::new(0., 1., 0.);
    assert!(
        tool.adjust_curve_segment(&curve, first, Point3::new(1., 1., 0.), 7., -2.)
            .unwrap()
    );
    assert_eq!(
        curve.cast::<GeomLine>().lin().location(),
        Point3::new(2., 1., 0.)
    );
}

#[test]
fn construct_line_2d_keeps_projected_origin_and_reports_equal_points() {
    let tool = ShapeConstructCurve;
    let curve = construct_line_2d();
    let first = Point2::new(0., 1.);
    assert!(
        tool.adjust_curve_2d(&curve, first, Point2::new(1., 1.), true, false)
            .unwrap()
    );
    assert_eq!(curve.cast::<Geom2dLine>().lin2d().location(), first);
    let failure = tool
        .adjust_curve_2d(&curve, first, first, true, true)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert_eq!(curve.cast::<Geom2dLine>().lin2d().location(), first);
}

#[test]
fn construct_unsupported_3d_curve_is_unchanged() {
    let tool = ShapeConstructCurve;
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 2.).unwrap()).cast();
    let origin = Point3::default();
    assert!(
        !tool
            .adjust_curve(&circle, origin, origin, true, true)
            .unwrap()
    );
    assert!(
        !tool
            .adjust_curve_segment(&circle, origin, origin, 0., 1.)
            .unwrap()
    );
}

#[test]
fn construct_unsupported_2d_curve_is_unchanged() {
    let tool = ShapeConstructCurve;
    let circle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::new(Axis22d::default(), 2.).unwrap()).cast();
    let origin = Point2::default();
    assert!(
        !tool
            .adjust_curve_2d(&circle, origin, origin, true, true)
            .unwrap()
    );
}

#[test]
fn construct_no_selected_end_returns_before_null_handle_check() {
    let tool = ShapeConstructCurve;
    let p = Point3::default();
    let p2 = Point2::default();
    assert!(
        tool.adjust_curve(&Handle::default(), p, p, false, false)
            .unwrap()
    );
    assert!(
        tool.adjust_curve_2d(&Handle::default(), p2, p2, false, false)
            .unwrap()
    );
    assert_eq!(
        tool.adjust_curve(&Handle::default(), p, p, true, false)
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn construct_bspline_adjustment_changes_only_selected_poles() {
    let tool = ShapeConstructCurve;
    let spline = construct_spline_3d();
    let old_last = spline.pole(spline.nb_poles()).unwrap();
    let first = Point3::new(-1., 4., 7.);
    assert!(
        tool.adjust_curve(&spline.cast(), first, Point3::default(), true, false)
            .unwrap()
    );
    assert_eq!(spline.pole(1).unwrap(), first);
    assert_eq!(spline.pole(spline.nb_poles()).unwrap(), old_last);
}

#[test]
fn construct_bspline_2d_changes_both_selected_ends() {
    let tool = ShapeConstructCurve;
    let spline = construct_spline_2d();
    let (first, last) = (Point2::new(2., 3.), Point2::new(5., 8.));
    assert!(
        tool.adjust_curve_2d(&spline.cast(), first, last, true, true)
            .unwrap()
    );
    assert_eq!(spline.pole(1).unwrap(), first);
    assert_eq!(spline.pole(spline.nb_poles()).unwrap(), last);
}

#[test]
fn construct_bspline_segment_clamps_then_changes_endpoints() {
    let tool = ShapeConstructCurve;
    let spline = construct_spline_3d();
    let (first, last) = (Point3::new(2., 3., 4.), Point3::new(5., 8., 9.));
    assert!(
        !tool
            .adjust_curve_segment(&spline.cast(), first, last, 1., 0.)
            .unwrap()
    );
    assert!(
        tool.adjust_curve_segment(&spline.cast(), first, last, -1., 0.75)
            .unwrap()
    );
    assert_eq!(
        (spline.first_parameter(), spline.last_parameter()),
        (0., 0.75)
    );
    assert_eq!(spline.pole(1).unwrap(), first);
    assert_eq!(spline.pole(spline.nb_poles()).unwrap(), last);
}
