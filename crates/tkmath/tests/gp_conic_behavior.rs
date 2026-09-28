//! Supplemental checks for the gp conics that the ElCLib suite only constructs: gp_Elips,
//! gp_Hypr, gp_Parab and their 2D counterparts have no upstream suites in TKMath.
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis22d, Circle2, Circle3, Direction2, Direction3, Ellipse2, Ellipse3,
    Hyperbola2, Hyperbola3, Parabola2, Parabola3, Point2, Point3, Transform2, Vector3,
};
use std::f64::consts::PI;

fn frame() -> Axis2 {
    Axis2::with_x_direction(Point3::new(1.0, 2.0, 3.0), Direction3::Z, Direction3::X).unwrap()
}
fn plane_frame() -> Axis22d {
    Axis22d::new(
        Point2::new(1.0, -2.0),
        Direction2::new(0.6, 0.8).unwrap(),
        true,
    )
    .unwrap()
}
/// Evaluates `A·x² + B·y² + 2C·xy + 2D·x + 2E·y + F`.
fn conic([a, b, c, d, e, f]: [f64; 6], p: Point2) -> f64 {
    let (x, y) = (p.x(), p.y());
    a * x * x + b * y * y + 2.0 * c * x * y + 2.0 * d * x + 2.0 * e * y + f
}

#[test]
fn ellipse_focal_geometry() {
    let ellipse = Ellipse3::new(frame(), 5.0, 3.0).unwrap();
    assert_eq!(ellipse.focal(), 8.0);
    assert_eq!(ellipse.eccentricity(), 0.8);
    assert_eq!(ellipse.parameter(), 9.0 / 5.0);
    assert_eq!(ellipse.focus1(), Point3::new(5.0, 2.0, 3.0));
    assert_eq!(ellipse.focus2(), Point3::new(-3.0, 2.0, 3.0));
    let directrix = ellipse.directrix1().unwrap();
    assert!((directrix.location().x() - (1.0 + 5.0 / 0.8)).abs() < 1e-12);
    assert_eq!(directrix.direction(), Direction3::Y);
    let circle_like = Ellipse3::new(frame(), 2.0, 2.0).unwrap();
    assert!(circle_like.directrix2().is_err());
    assert!(Ellipse3::new(frame(), 1.0, 2.0).is_err());
    assert!(Ellipse3::new(frame(), 1.0, -0.5).is_err());
    let mut ellipse = ellipse;
    assert!(ellipse.set_major_radius(2.0).is_err());
    assert!(ellipse.set_minor_radius(6.0).is_err());
    ellipse.set_minor_radius(1.0).unwrap();
    assert_eq!(ellipse.area(), PI * 5.0 * 1.0);
    let default = Ellipse3::default();
    assert_eq!(
        (default.major_radius(), default.minor_radius()),
        (f64::MAX, f64::MIN_POSITIVE)
    );
}

#[test]
fn hyperbola_branches_and_asymptotes() {
    let hyperbola = Hyperbola3::new(frame(), 4.0, 3.0).unwrap();
    assert_eq!(hyperbola.focal(), 10.0);
    assert_eq!(hyperbola.eccentricity().unwrap(), 1.25);
    assert_eq!(hyperbola.parameter().unwrap(), 9.0 / 4.0);
    let asymptote = hyperbola.asymptote1().unwrap();
    let slope = asymptote.direction().y() / asymptote.direction().x();
    assert!((slope - 0.75).abs() < 1e-15);
    let other = hyperbola.other_branch().unwrap();
    assert_eq!(other.position().x_direction(), Direction3::X.reversed());
    let conjugate = hyperbola.conjugate_branch1().unwrap();
    assert_eq!(
        (conjugate.major_radius(), conjugate.minor_radius()),
        (3.0, 4.0)
    );
    assert_eq!(conjugate.position().x_direction(), Direction3::Y);
    let directrix = hyperbola.directrix2().unwrap();
    assert!((directrix.location().x() - (1.0 - 4.0 / 1.25)).abs() < 1e-12);
    let zero = Hyperbola3::new(frame(), 0.0, 1.0).unwrap();
    assert_eq!(
        zero.eccentricity().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    assert_eq!(
        zero.asymptote1().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    // The default hyperbola stores RealFirst as minor radius, so its conjugate is invalid.
    assert!(Hyperbola3::default().conjugate_branch1().is_err());
    let mut moved = hyperbola;
    moved.set_location(Point3::new(0.0, 0.0, 0.0)).unwrap();
    assert_eq!(moved.location(), Point3::new(0.0, 0.0, 0.0));
}

#[test]
fn parabola_from_directrix_round_trips() {
    let directrix = Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::Y);
    let focus = Point3::new(4.0, 1.0, 0.0);
    let parabola = Parabola3::from_directrix(directrix, focus).unwrap();
    assert_eq!(parabola.focal(), 2.0);
    assert!(parabola.focus().distance(&focus) < 1e-12);
    assert_eq!(parabola.parameter(), 4.0);
    assert!(parabola.directrix().location().x().abs() < 1e-12);
    assert!(Parabola3::from_directrix(directrix, Point3::new(0.0, 5.0, 0.0)).is_err());
    assert!(Parabola3::new(frame(), -1.0).is_err());
}

#[test]
fn space_conics_share_placement_members() {
    let circle = Circle3::new(frame(), 2.0).unwrap();
    let axis = Axis1::new(Point3::default(), Direction3::Z);
    let rotated = circle.rotated(axis, PI / 2.0).unwrap();
    assert!(rotated.location().distance(&Point3::new(-2.0, 1.0, 3.0)) < 1e-12);
    let plane = Axis2::new(Point3::default(), Direction3::Z).unwrap();
    let mirrored = circle.mirrored_plane(plane).unwrap();
    assert_eq!(mirrored.location(), Point3::new(1.0, 2.0, -3.0));
    let mirrored = circle.mirrored_axis(axis).unwrap();
    assert_eq!(mirrored.location(), Point3::new(-1.0, -2.0, 3.0));
    let moved = circle.translated_between(Point3::default(), Point3::new(1.0, 1.0, 1.0));
    assert_eq!(moved.location(), Point3::new(2.0, 3.0, 4.0));
    let mut scaled = circle;
    scaled.scale(Point3::default(), -1.0);
    assert_eq!(scaled.radius(), 2.0);
    assert_eq!(scaled.location(), Point3::new(-1.0, -2.0, -3.0));
    let mut located = circle;
    located.set_location(Point3::default());
    located
        .set_axis(Axis1::new(Point3::default(), Direction3::X))
        .unwrap();
    assert_eq!(located.axis().direction(), Direction3::X);
    assert_eq!(circle.x_axis().direction(), Direction3::X);
    assert_eq!(
        circle
            .translated(Vector3::new(0.0, 0.0, 1.0))
            .location()
            .z(),
        4.0
    );
}

#[test]
fn planar_implicit_equations_vanish_on_curves() {
    let frame = plane_frame();
    let circle = Circle2::new(frame, 2.0).unwrap();
    let ellipse = Ellipse2::new(frame, 3.0, 1.5).unwrap();
    let hyperbola = Hyperbola2::new(frame, 2.0, 1.0).unwrap();
    let parabola = Parabola2::new(frame, 0.75).unwrap();
    for u in [0.3, 1.7, -2.4] {
        assert!(conic(circle.coefficients(), elclib::value(u, &circle)).abs() < 1e-12);
        assert!(conic(ellipse.coefficients(), elclib::value(u, &ellipse)).abs() < 1e-12);
        assert!(conic(hyperbola.coefficients(), elclib::value(u, &hyperbola)).abs() < 1e-11);
        assert!(conic(parabola.coefficients(), elclib::value(u, &parabola)).abs() < 1e-12);
    }
    assert_eq!(
        Ellipse2::new(frame, 0.0, 0.0).unwrap().coefficients(),
        [0.0; 6]
    );
}

#[test]
fn planar_hyperbola_keeps_native_asymptote_and_directrix_quirks() {
    let hyperbola = Hyperbola2::new(Axis22d::default(), 4.0, 3.0).unwrap();
    // OCCT 3d097a builds Asymptote1 from Y + Y·(b/a), so it points along Y.
    assert_eq!(hyperbola.asymptote1().unwrap().direction(), Direction2::Y);
    // Asymptote2 uses Y·(1 - b/a) and fails for equal radii.
    let equal = Hyperbola2::new(Axis22d::default(), 2.0, 2.0).unwrap();
    assert_eq!(
        equal.asymptote2().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    // Directrix2 is Focus1 + X·(p/e), not the mirror image of Directrix1.
    let second = hyperbola.directrix2().unwrap();
    assert!((second.location().x() - (5.0 + (9.0 / 4.0) / 1.25)).abs() < 1e-12);
    assert!((hyperbola.directrix1().unwrap().location().x() - 3.2).abs() < 1e-12);
    let conjugate = hyperbola.conjugate_branch2();
    assert_eq!(conjugate.axis().x_direction(), Direction2::NEG_Y);
    assert_eq!(hyperbola.other_branch().axis().x_direction().x(), -1.0);
}

#[test]
fn planar_ellipse_transform_variants_differ_in_zero_sign() {
    let ellipse = Ellipse2::new(Axis22d::default(), 0.0, 0.0).unwrap();
    let mirror = Transform2::mirror_axis(Axis2d::new(Point2::default(), Direction2::X));
    assert!(
        ellipse
            .transformed(mirror)
            .major_radius()
            .is_sign_negative()
    );
    let mut in_place = ellipse;
    in_place.transform(mirror);
    assert!(in_place.major_radius().is_sign_positive());
}

#[test]
fn planar_parabola_from_directrix_handles_focus_on_directrix() {
    let directrix = Axis2d::new(Point2::new(0.0, 0.0), Direction2::Y);
    let parabola = Parabola2::from_directrix(directrix, Point2::new(2.0, 1.0), true).unwrap();
    assert_eq!(parabola.focal(), 1.0);
    assert_eq!(parabola.location(), Point2::new(1.0, 1.0));
    assert!(parabola.focus().distance(Point2::new(2.0, 1.0)) < 1e-15);
    let flat = Parabola2::from_directrix(directrix, Point2::new(0.0, 3.0), true).unwrap();
    assert_eq!(flat.focal(), 0.0);
    // The mirror axis is the directrix turned by -PI/2 for a direct sense.
    assert!((flat.axis_of_symmetry().direction().x() - 1.0).abs() < 1e-15);
    let mut moved = parabola;
    moved
        .set_mirror_axis(Axis2d::new(Point2::new(5.0, 5.0), Direction2::Y))
        .unwrap();
    assert_eq!(moved.axis_of_symmetry().direction(), Direction2::Y);
    assert_eq!(moved.directrix().location(), Point2::new(5.0, 4.0));
}

#[test]
fn planar_frames_keep_their_sense() {
    let direct = Axis22d::new(Point2::default(), Direction2::X, true).unwrap();
    assert_eq!(direct.y_direction(), Direction2::Y);
    let indirect = Axis22d::from_axis(Axis2d::default(), false).unwrap();
    assert_eq!(indirect.y_direction(), Direction2::NEG_Y);
    let mut frame = indirect;
    frame.set_x_direction(Direction2::Y).unwrap();
    assert_eq!(frame.y_direction().x(), 1.0);
    frame.set_y_direction(Direction2::X).unwrap();
    assert_eq!(frame.x_direction().y(), 1.0);
    let built =
        Axis22d::with_directions(Point2::default(), Direction2::X, Direction2::NEG_Y).unwrap();
    assert_eq!(built, indirect);
    let mut circle = Circle2::new(direct, 1.0).unwrap();
    assert!(circle.is_direct());
    circle.reverse();
    assert!(!circle.is_direct());
    assert!(circle.reversed().is_direct());
    let mirrored = direct.mirrored_axis(Axis2d::new(Point2::default(), Direction2::X));
    assert_eq!(mirrored.y_direction(), Direction2::NEG_Y);
}

#[test]
fn planar_axes_report_coaxiality_and_dump_nested_fields() {
    let axis = Axis2d::new(Point2::new(1.0, 1.0), Direction2::X);
    assert!(axis.is_coaxial(axis.reversed(), 1e-12, 1e-12));
    assert!(!axis.is_coaxial(
        axis.translated_between(Point2::default(), Point2::new(0.0, 1.0)),
        1e-12,
        1e-12
    ));
    let mut stream = rustycad_tkernel::standard::dump::DumpStream::default();
    axis.dump_json(&mut stream, -1).unwrap();
    assert_eq!(
        stream.text(),
        "\"className\": \"gp_Ax2d\", \"loc\": {\"gp_Pnt2d\": [1, 1]}, \"vdir\": {\"gp_Dir2d\": [1, 0]}"
    );
    let mut shallow = rustycad_tkernel::standard::dump::DumpStream::default();
    axis.dump_json(&mut shallow, 0).unwrap();
    assert_eq!(shallow.text(), "\"className\": \"gp_Ax2d\"");
}
