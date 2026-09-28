//! Native scale validation and compound-power regressions beyond legacy declarations.
use rustycad_tkmath::{
    gp::{Axis1, Axis2, Circle3, Direction3, Point3, Transform3},
    toploc::Location,
};
#[test]
fn scale_checks_native_resolution_and_inverse_failure() {
    for factor in [0.0, -0.0, f64::MIN_POSITIVE, -f64::MIN_POSITIVE] {
        let failure = Transform3::scaling(Point3::default(), factor).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_ConstructionError");
        // The native text leads; the offending scale follows it.
        assert!(failure.message().starts_with("gp_Trsf::SetScaleFactor"));
    }
    let small = Transform3::scaling(Point3::default(), 1e-200).unwrap();
    let zero = small.multiplied(small);
    assert_eq!(zero.scale_factor(), 0.0);
    let failure = zero.inverted().unwrap_err();
    assert!(
        failure
            .message()
            .starts_with("gp_Trsf::Invert() - transformation has zero scale")
    );
    assert!(zero.powered(-1).is_err());
    assert!(Location::new(zero).inverted().is_err());
}
#[test]
fn compound_power_preserves_native_temporary_scale_order() {
    let scale = Transform3::scaling(Point3::new(1.0, 0.0, 0.0), 2.0).unwrap();
    let rotation = Transform3::rotation(Axis1::default(), 0.0).unwrap();
    let compound = rotation.multiplied(scale).powered(4).unwrap();
    assert_eq!(scale.powered(4).unwrap().translation_part().x(), -15.0);
    assert_eq!(compound.translation_part().x(), -23.0);
    assert_eq!(compound.scale_factor(), 16.0);
}
#[test]
fn circle_scaling_keeps_right_handed_normal_and_checks_radius() {
    let frame = Axis2::new(Point3::new(1.0, 0.0, 0.0), Direction3::Z).unwrap();
    let mut circle = Circle3::new(frame, 2.0).unwrap();
    assert!(circle.set_radius(-1.0).is_err());
    assert_eq!(circle.radius(), 2.0);
    let negative = circle
        .transformed(Transform3::scaling(Point3::default(), -3.0).unwrap())
        .unwrap();
    assert_eq!(negative.radius(), 6.0);
    assert!(
        negative
            .position()
            .direction()
            .is_equal(&Direction3::Z, 1e-12)
    );
    assert!(Circle3::new(frame, -1.0).is_err());
    assert_eq!(Circle3::default().radius(), f64::MAX);
    assert!(circle.contains(Point3::new(3.0, 0.0, 0.0), 1e-7));
    assert_eq!(circle.distance(Point3::new(3.0, 0.0, 0.0)), 0.0);
}
