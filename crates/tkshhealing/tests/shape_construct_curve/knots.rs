#[test]
fn construct_fix_knots_uses_occ90_epsilon_spacing() {
    let mut knots = Array1::from_shared(&[1., 1., 0., 2.], 1);
    assert!(ShapeConstructCurve::fix_knots(&mut knots).unwrap());
    assert_eq!(knots[2], 1. + 2. * epsilon(1.));
    assert_eq!(knots[3], knots[2] + 2. * epsilon(knots[2]));
    assert_eq!(knots[4], 2.);
    assert!(!ShapeConstructCurve::fix_knots(&mut knots).unwrap());
}

#[test]
fn construct_fix_knots_handle_mutates_shared_array() {
    let array = HArray1::from_array(&Array1::from_shared(&[0., 0., 1.], 1));
    let knots = Handle::new(array);
    let alias = knots.clone();
    assert!(ShapeConstructCurve::fix_knots_handle(&knots).unwrap());
    assert_eq!(alias.array1()[2], 2. * epsilon(0.));
}

#[test]
fn construct_fix_knots_preserves_partial_mutation_before_bounds_error() {
    let mut knots = Array1::from_shared(&[7., 1., 1.], 0);
    let failure = ShapeConstructCurve::fix_knots(&mut knots).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!(knots[0], 7.);
    assert_eq!(knots[2], 1. + 2. * epsilon(1.));
}

#[test]
fn construct_fix_knots_rejects_empty_and_non_one_lower_bounds() {
    let mut empty = Array1::<f64>::default();
    let mut shifted = Array1::from_shared(&[1., 1.], 2);
    assert_eq!(
        ShapeConstructCurve::fix_knots(&mut empty)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        ShapeConstructCurve::fix_knots(&mut shifted)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(shifted[2], 1.);
}

#[test]
fn construct_fix_knots_null_handle_reports_its_expected_array_shape() {
    let knots: Handle<HArray1<f64>> = Handle::default();
    let failure = ShapeConstructCurve::fix_knots_handle(&knots).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
}
