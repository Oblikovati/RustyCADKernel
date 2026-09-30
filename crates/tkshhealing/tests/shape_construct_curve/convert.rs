#[test]
fn construct_bspline_identity_is_preserved_without_narrowing() {
    let tool = ShapeConstructCurve;
    let spline = construct_spline_3d();
    let same = tool
        .convert_to_bspline(&spline.cast(), -1., 2., 1e-7)
        .unwrap();
    assert_eq!(same.object_address(), spline.object_address());
    let copy = tool
        .convert_to_bspline(&spline.cast(), 0.25, 0.75, 1e-7)
        .unwrap();
    assert_ne!(copy.object_address(), spline.object_address());
    assert_eq!(
        (copy.first_parameter(), copy.last_parameter()),
        (0.25, 0.75)
    );
    assert_eq!(
        (spline.first_parameter(), spline.last_parameter()),
        (0., 1.)
    );
}

#[test]
fn construct_bspline_2d_identity_and_segmentation_match_3d() {
    let tool = ShapeConstructCurve;
    let spline = construct_spline_2d();
    let same = tool
        .convert_to_bspline_2d(&spline.cast(), 0., 1., 1e-7)
        .unwrap();
    assert_eq!(same.object_address(), spline.object_address());
    let copy = tool
        .convert_to_bspline_2d(&spline.cast(), 0.2, 0.8, 1e-7)
        .unwrap();
    assert_ne!(copy.object_address(), spline.object_address());
    assert_eq!((copy.first_parameter(), copy.last_parameter()), (0.2, 0.8));
    assert_eq!(
        (spline.first_parameter(), spline.last_parameter()),
        (0., 1.)
    );
}

#[test]
fn construct_equal_line_bounds_propagate_trimmed_constructor_errors() {
    let tool = ShapeConstructCurve;
    let first = tool.convert_to_bspline(&construct_line_3d(), 0.5, 0.5, 1e-7);
    let second = tool.convert_to_bspline_2d(&construct_line_2d(), 0.5, 0.5, 1e-7);
    assert_eq!(
        first.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        second.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn construct_failed_spline_segment_falls_back_without_mutating_input() {
    let tool = ShapeConstructCurve;
    let spline = construct_spline_3d();
    let result = tool
        .convert_to_bspline(&spline.cast(), 0.75, 0.25, 1e-7)
        .unwrap();
    assert!(result.is_null());
    assert_eq!(
        (spline.first_parameter(), spline.last_parameter()),
        (0., 1.)
    );
}
