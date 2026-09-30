//! Analytic contour contracts; every overload also has exact native comparisons.
use rustycad_tkhlr::contap::ContapContAna;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3, Direction3, Point3, Sphere3};

#[test]
fn analytic_accessors_require_perform() {
    let contour = ContapContAna::default();
    assert!(!contour.is_done());
    assert_eq!(
        contour.nb_contours().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        contour.type_contour().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        contour.circle().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        contour.line(1).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}
#[test]
fn sphere_direction_and_draft_keep_circle_frame() {
    let sphere = Sphere3::new(Axis3::default(), 2.0).unwrap();
    let mut contour = ContapContAna::new();
    contour.perform_sphere(&sphere, Direction3::X).unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 1);
    assert_eq!(contour.type_contour().unwrap(), CurveType::Circle);
    assert_eq!(contour.circle().unwrap().radius(), 2.0);
    assert!(contour.line(1).is_err());
    contour
        .perform_sphere_draft(&sphere, Direction3::Z, 0.0)
        .unwrap();
    assert_eq!(contour.circle().unwrap().radius(), 2.0);
}
#[test]
fn sphere_eye_removes_prior_contour_inside_sphere() {
    let sphere = Sphere3::new(Axis3::default(), 2.0).unwrap();
    let mut contour = ContapContAna::new();
    contour
        .perform_sphere_eye(&sphere, Point3::new(0.0, 0.0, 4.0))
        .unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 1);
    contour
        .perform_sphere_eye(&sphere, Point3::default())
        .unwrap();
    assert!(contour.is_done());
    assert_eq!(contour.nb_contours().unwrap(), 0);
    assert!(contour.type_contour().is_err() && contour.circle().is_err());
}
#[test]
fn cylinder_silhouette_parallel_limit_and_draft() {
    let cylinder = Cylinder3::new(Axis3::default(), 2.0).unwrap();
    let mut contour = ContapContAna::new();
    contour.perform_cylinder(&cylinder, Direction3::X).unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 2);
    assert_eq!(contour.type_contour().unwrap(), CurveType::Line);
    assert!(contour.circle().is_err());
    assert_eq!(contour.line(1).unwrap().direction(), Direction3::Z);
    assert_eq!(
        contour.line(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        contour.line(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    contour
        .perform_cylinder_draft(&cylinder, Direction3::X, 0.0)
        .unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 2);
    contour.perform_cylinder(&cylinder, Direction3::Z).unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 0);
}
#[test]
fn cylinder_eye_contours_depend_on_radial_distance() {
    let cylinder = Cylinder3::new(Axis3::default(), 2.0).unwrap();
    let mut contour = ContapContAna::new();
    contour
        .perform_cylinder_eye(&cylinder, Point3::new(4.0, 0.0, 100.0))
        .unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 2);
    contour
        .perform_cylinder_eye(&cylinder, Point3::new(0.0, 0.0, 100.0))
        .unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 0);
}
#[test]
fn cone_overloads_preserve_two_sided_draft_results() {
    let cone = Cone3::new(Axis3::default(), 0.4, 2.0).unwrap();
    let mut contour = ContapContAna::new();
    contour.perform_cone(&cone, Direction3::X).unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 2);
    contour
        .perform_cone_draft(&cone, Direction3::X, 0.0)
        .unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 4);
    assert!(contour.line(4).is_ok());
    assert!(contour.line(5).is_err());
    contour
        .perform_cone_eye(&cone, Point3::new(10.0, 0.0, 0.0))
        .unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 2);
    contour.perform_cone(&cone, Direction3::Z).unwrap();
    assert_eq!(contour.nb_contours().unwrap(), 0);
}
