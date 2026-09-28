//! Behaviour of the gce port beyond the upstream suites: the status enumeration, the
//! StdFail_NotDone results, the conversion operators and the native paths that leave
//! the status unassigned (tools/verify_tkgeombase_numeric.py compares them with OCCT).
use rustycad_tkgeombase::gce::{
    ErrorType, MakeCirc, MakeCirc2d, MakeDir, MakeMirror, MakeScale, Root,
};
use rustycad_tkmath::gp::{Circle2, Direction3, Plane3, Point2, Point3, Transform3, TransformForm};

#[test]
fn error_type_values_and_names_follow_the_native_enumeration() {
    for (index, error) in ErrorType::ALL.iter().enumerate() {
        assert_eq!(*error as usize, index);
        assert!(error.name().starts_with("gce_"));
    }
    assert_eq!(ErrorType::BadEquation.name(), "gce_BadEquation");
    assert_eq!(ErrorType::default(), ErrorType::Done);
}

#[test]
fn value_of_a_failed_construction_raises_not_done_with_the_status() {
    let maker = MakeDir::from_coordinates(0.0, 0.0, 0.0).unwrap();
    assert!(maker.is_error());
    assert_eq!(maker.status(), ErrorType::NullVector);
    let failure = maker.value().unwrap_err();
    assert_eq!(failure.exception_type(), "StdFail_NotDone");
    assert!(failure.message().contains("gce_MakeDir::Value()"));
    assert!(failure.message().contains("gce_NullVector"));
    assert_eq!(
        Direction3::try_from(&maker).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn conversion_operators_return_the_value() {
    let maker = MakeDir::from_coordinates(0.0, 3.0, 4.0).unwrap();
    assert_eq!(
        Direction3::try_from(&maker).unwrap(),
        maker.value().unwrap()
    );
    let mirror = MakeMirror::from_point(Point3::new(1.0, 2.0, 3.0));
    assert_eq!(Transform3::from(&mirror).form(), TransformForm::PointMirror);
}

#[test]
fn center_plane_circle_raises_not_done_for_a_negative_radius() {
    let plane = Plane3::new(Point3::default(), Direction3::default()).unwrap();
    let failure = MakeCirc::from_center_plane(Point3::default(), plane, -1.0).unwrap_err();
    assert_eq!(failure.exception_type(), "StdFail_NotDone");
}

#[test]
fn coincident_points_give_a_null_circle_with_the_zeroed_status() {
    let p = Point3::new(1.0, 1.0, 1.0);
    let maker = MakeCirc::through_points(p, p, p).unwrap();
    assert!(maker.is_done());
    let circle = maker.value().unwrap();
    assert_eq!(circle.radius(), 0.0);
    assert_eq!(circle.position().direction().x(), 1.0);
}

#[test]
fn collinear_points_in_2d_keep_the_default_circle() {
    let (a, b, c) = (
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(3.0, 0.0),
    );
    let maker = MakeCirc2d::through_points(a, b, c).unwrap();
    assert!(maker.is_done());
    assert_eq!(maker.value().unwrap(), Circle2::default());
}

#[test]
fn null_scale_factor_raises_through_gp() {
    let failure = MakeScale::new(Point3::default(), 0.0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}
