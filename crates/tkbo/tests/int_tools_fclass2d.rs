//! Classification regressions use actual BRepBuilderAPI faces shared with the native probe.
#[path = "../examples/bo_probe/fclass_faces.rs"]
mod faces;
use rustycad_tkbo::int_tools::FClass2d;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point2;
#[test]
fn rectangle_classifies_in_out_and_exact_boundary() {
    let face = faces::face(0, 2.).unwrap();
    let classifier = FClass2d::new(&face, 1e-7).unwrap();
    assert!(!classifier.is_hole());
    assert_eq!(classifier.perform_infinite_point().unwrap(), State::Out);
    for (p, state) in [
        (Point2::new(1., 1.), State::In),
        (Point2::new(3., 1.), State::Out),
        (Point2::new(0., 1.), State::On),
    ] {
        assert_eq!(classifier.perform(p, true).unwrap(), state);
        assert_eq!(
            classifier.test_on_restriction(p, 1e-7, false).unwrap(),
            state
        );
    }
}
#[test]
fn repeated_init_appends_old_polygons() {
    let mut classifier = FClass2d::default();
    let p = Point2::new(1.5, 1.5);
    assert_eq!(classifier.perform(p, true).unwrap(), State::In);
    classifier.init(&faces::face(0, 2.).unwrap(), 1e-7).unwrap();
    assert_eq!(classifier.perform(p, true).unwrap(), State::In);
    classifier.init(&faces::face(0, 1.).unwrap(), 1e-7).unwrap();
    assert_eq!(classifier.perform(p, true).unwrap(), State::Out);
}
#[test]
fn circular_hole_and_empty_surface_preserve_native_roles() {
    let classifier = FClass2d::new(&faces::face(6, 2.).unwrap(), 1e-7).unwrap();
    assert!(classifier.is_hole());
    assert_eq!(
        classifier.perform(Point2::default(), false).unwrap(),
        State::Out
    );
    assert_eq!(
        classifier.perform(Point2::new(1.5, 0.), false).unwrap(),
        State::In
    );
    let empty = FClass2d::new(&faces::face(7, 2.).unwrap(), 1e-7).unwrap();
    assert!(empty.is_hole());
    assert_eq!(empty.perform_infinite_point().unwrap(), State::In);
}
#[test]
fn periodic_cylinder_reframes_only_when_requested() {
    let classifier = FClass2d::new(&faces::face(1, 2.).unwrap(), 1e-7).unwrap();
    let p = Point2::new(3. + std::f64::consts::TAU, 0.);
    assert_eq!(classifier.perform(p, false).unwrap(), State::Out);
    assert_eq!(classifier.perform(p, true).unwrap(), State::In);
    assert_eq!(
        classifier.test_on_restriction(p, 1e-7, true).unwrap(),
        State::In
    );
}
#[test]
fn all_real_fixture_kinds_initialize_independently() {
    for kind in 0..14 {
        let face = faces::face(kind, 2.).unwrap();
        let mut classifier = FClass2d::new(&face, 1e-7).unwrap();
        let before = classifier.perform(Point2::new(10., 10.), false).unwrap();
        classifier.init(&face, 1e-7).unwrap();
        assert_eq!(
            classifier.perform(Point2::new(10., 10.), false).unwrap(),
            before
        );
    }
}
