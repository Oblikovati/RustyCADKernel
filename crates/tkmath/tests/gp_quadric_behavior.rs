//! Supplemental gp_Sphere/Cylinder/Cone/Torus checks: gp_Torus_Test depends on TKG3d, and
//! the other quadrics have no upstream suite, so these tests pin their behaviour directly.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::elslib::{self, ElementarySurface};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Cone3, Cylinder3, Direction3, Point3, Sphere3, Torus3, Transform3, Vector3,
};
use std::f64::consts::PI;

fn frame() -> Axis3 {
    let normal = Direction3::new(-0.2, 0.6, 0.7).unwrap();
    Axis3::new(Point3::new(0.5, 1.0, -1.5), normal, Direction3::X).unwrap()
}
/// Evaluates the ten-coefficient quadric equation at a point.
fn quadric(q: [f64; 10], p: Point3) -> f64 {
    let (x, y, z) = (p.x(), p.y(), p.z());
    q[0] * x * x
        + q[1] * y * y
        + q[2] * z * z
        + 2.0 * (q[3] * x * y + q[4] * x * z + q[5] * y * z)
        + 2.0 * (q[6] * x + q[7] * y + q[8] * z)
        + q[9]
}
/// Evaluates the 35-coefficient torus quartic in the documented monomial order.
fn quartic(c: &[f64], p: Point3) -> f64 {
    let (x, y, z) = (p.x(), p.y(), p.z());
    let monomials = [
        x.powi(4),
        y.powi(4),
        z.powi(4),
        x.powi(3) * y,
        x.powi(3) * z,
        y.powi(3) * x,
        y.powi(3) * z,
        z.powi(3) * x,
        z.powi(3) * y,
        x * x * y * y,
        x * x * z * z,
        y * y * z * z,
        x * x * y * z,
        x * y * y * z,
        x * y * z * z,
        x.powi(3),
        y.powi(3),
        z.powi(3),
        x * x * y,
        x * x * z,
        y * y * x,
        y * y * z,
        z * z * x,
        z * z * y,
        x * y * z,
        x * x,
        y * y,
        z * z,
        x * y,
        x * z,
        y * z,
        x,
        y,
        z,
        1.0,
    ];
    monomials.iter().zip(c).map(|(m, k)| m * k).sum()
}

#[test]
fn implicit_equations_vanish_on_the_surfaces() {
    let sphere = Sphere3::new(frame(), 2.0).unwrap();
    let cylinder = Cylinder3::new(frame(), 1.5).unwrap();
    let cone = Cone3::new(frame(), 0.35, 1.0).unwrap();
    let torus = Torus3::new(frame(), 3.0, 0.75).unwrap();
    let mut coefficients = Array1::with_bounds(1, 35).unwrap();
    torus.coefficients(&mut coefficients).unwrap();
    for (u, v) in [(0.2, 0.3), (2.5, -1.0), (4.0, 1.2)] {
        assert!(quadric(sphere.coefficients(), sphere.value(u, v)).abs() < 1e-12);
        assert!(quadric(cylinder.coefficients(), cylinder.value(u, v)).abs() < 1e-12);
        assert!(quadric(cone.coefficients(), cone.value(u, v)).abs() < 1e-12);
        assert!(quartic(coefficients.as_slice(), torus.value(u, v)).abs() < 1e-9);
    }
    let outside = Point3::new(10.0, 10.0, 10.0);
    assert!(quadric(sphere.coefficients(), outside) > 1.0);
}

#[test]
fn torus_coefficients_respect_the_array_bounds() {
    let torus = Torus3::new(frame(), 3.0, 0.75).unwrap();
    let mut shifted = Array1::with_bounds(-4, 40).unwrap();
    torus.coefficients(&mut shifted).unwrap();
    let mut exact = Array1::with_bounds(1, 35).unwrap();
    torus.coefficients(&mut exact).unwrap();
    assert_eq!(&shifted.as_slice()[..35], exact.as_slice());
    assert!(shifted.as_slice()[35..].iter().all(|value| *value == 0.0));
    let mut short = Array1::with_bounds(1, 34).unwrap();
    let error = torus.coefficients(&mut short).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DimensionError");
}

#[test]
fn constructors_and_setters_reject_invalid_parameters() {
    assert!(Sphere3::new(frame(), -1.0).is_err());
    assert!(Cylinder3::new(frame(), -1.0).is_err());
    assert!(Cone3::new(frame(), 0.0, 1.0).is_err());
    assert!(Cone3::new(frame(), PI / 2.0, 1.0).is_err());
    assert!(Cone3::new(frame(), -0.3, 1.0).is_ok());
    assert!(Torus3::new(frame(), 1.0, -0.1).is_err());
    // A torus whose tube is wider than its ring is accepted by the constructor...
    let mut torus = Torus3::new(frame(), 1.0, 2.0).unwrap();
    // ...but the setters require major - minor > resolution.
    assert!(torus.set_major_radius(2.0).is_err());
    assert!(torus.set_minor_radius(0.5).is_ok());
    let mut cone = Cone3::default();
    assert_eq!(cone.semi_angle(), PI * 0.25);
    assert!(cone.set_semi_angle(PI).is_err());
    assert!(cone.set_radius(-2.0).is_err());
}

#[test]
fn measures_follow_native_formulas() {
    let sphere = Sphere3::new(frame(), 2.0).unwrap();
    assert_eq!(sphere.area(), 4.0 * PI * 2.0 * 2.0);
    assert_eq!(sphere.volume(), (4.0 * PI * 8.0) / 3.0);
    let torus = Torus3::new(frame(), 3.0, 0.5).unwrap();
    assert_eq!(torus.area(), 4.0 * PI * PI * 0.5 * 3.0);
    assert_eq!(torus.volume(), (PI * 0.25) * (2.0 * PI * 3.0));
    let cone = Cone3::new(frame(), 0.25, 2.0).unwrap();
    let apex = cone.apex();
    // The apex is where the ring radius R + v·sin(a) vanishes.
    let v = -2.0 / 0.25_f64.sin();
    assert!(apex.distance(&cone.value(1.0, v)) < 1e-12);
}

#[test]
fn reversals_change_the_parametrization_only() {
    let mut cone = Cone3::new(frame(), 0.3, 1.0).unwrap();
    let reference = cone.value(0.4, 0.8);
    cone.v_reverse();
    assert_eq!(cone.semi_angle(), -0.3);
    assert!(cone.value(0.4, -0.8).distance(&reference) < 1e-12);
    let mut sphere = Sphere3::new(frame(), 1.0).unwrap();
    assert!(sphere.direct().unwrap());
    sphere.u_reverse();
    assert!(!sphere.direct().unwrap());
    sphere.v_reverse();
    assert!(sphere.direct().unwrap());
}

#[test]
fn torus_in_place_scaling_keeps_positive_zero_radii() {
    // gp_Torus::Scale multiplies by |s| while Scaled multiplies then negates, so a zero minor
    // radius becomes -0.0 only through the copying variant.
    let torus = Torus3::new(Axis3::default(), 2.0, 0.0).unwrap();
    let copy = torus.scaled(Point3::default(), -2.0);
    assert!(copy.minor_radius().is_sign_negative());
    let mut in_place = torus;
    in_place.scale(Point3::default(), -2.0);
    assert!(in_place.minor_radius().is_sign_positive());
    assert_eq!(in_place.major_radius(), 4.0);
    let mirror = Transform3::scaling(Point3::default(), -1.0).unwrap();
    assert!(torus.transformed(mirror).minor_radius().is_sign_negative());
    let mut transformed = torus;
    transformed.transform(mirror);
    assert!(transformed.minor_radius().is_sign_positive());
}

#[test]
fn placement_members_move_the_frame() {
    let cylinder = Cylinder3::new(frame(), 1.0).unwrap();
    let axis = Axis1::new(Point3::default(), Direction3::Z);
    let rotated = cylinder.rotated(axis, PI).unwrap();
    assert!(rotated.location().distance(&Point3::new(-0.5, -1.0, -1.5)) < 1e-12);
    let shifted = cylinder.translated(Vector3::new(1.0, 0.0, 0.0));
    assert_eq!(shifted.location(), Point3::new(1.5, 1.0, -1.5));
    let between = cylinder.translated_between(Point3::default(), Point3::new(0.0, 2.0, 0.0));
    assert_eq!(between.location(), Point3::new(0.5, 3.0, -1.5));
    let mirrored = cylinder.mirrored_point(Point3::default());
    assert_eq!(mirrored.location(), Point3::new(-0.5, -1.0, 1.5));
    let plane = Axis2::new(Point3::default(), Direction3::Z).unwrap();
    assert_eq!(cylinder.mirrored_plane(plane).location().z(), 1.5);
    let mut moved = cylinder;
    moved.set_axis(axis).unwrap();
    assert_eq!(moved.axis(), axis);
    assert_eq!(cylinder.x_axis().direction(), frame().x_direction());
    assert_eq!(cylinder.y_axis().direction(), frame().y_direction());
    let mut located = cylinder;
    located.set_location(Point3::new(9.0, 9.0, 9.0));
    assert_eq!(located.location(), Point3::new(9.0, 9.0, 9.0));
    assert_eq!(cylinder.scaled(Point3::default(), -2.0).radius(), 2.0);
}

#[test]
fn parameters_match_the_generic_overloads() {
    let torus = Torus3::new(frame(), 3.0, 1.0).unwrap();
    let point = torus.value(1.0, 2.0);
    assert_eq!(
        elslib::parameters(&torus, point),
        elslib::torus_parameters(frame(), 3.0, 1.0, point)
    );
}
