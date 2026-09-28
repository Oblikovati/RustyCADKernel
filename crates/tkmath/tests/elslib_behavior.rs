//! Supplemental ElSLib checks: there is no upstream ElSLib GoogleTest suite, so these tests pin
//! the analytic meaning of every evaluator and the native quirks the port preserves.
use rustycad_tkmath::elslib::{self, CurvedSurface, ElementarySurface};
use rustycad_tkmath::gp::{
    Axis3, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};
use std::f64::consts::{FRAC_PI_2, PI};

fn frame() -> Axis3 {
    let normal = Direction3::new(0.3, -0.4, 0.8).unwrap();
    let x = Direction3::new(1.0, 0.5, 0.1).unwrap();
    Axis3::new(Point3::new(1.0, -2.0, 0.5), normal, x).unwrap()
}
fn close_points(left: Point3, right: Point3, tolerance: f64) {
    assert!(left.distance(&right) <= tolerance, "{left:?} vs {right:?}");
}
fn close_vectors(left: Vector3, right: Vector3, tolerance: f64) {
    assert!(
        (left - right).magnitude() <= tolerance,
        "{left:?} vs {right:?}"
    );
}
fn surfaces() -> (Cone3, Cylinder3, Sphere3, Torus3) {
    (
        Cone3::new(frame(), 0.4, 2.0).unwrap(),
        Cylinder3::new(frame(), 1.5).unwrap(),
        Sphere3::new(frame(), 2.5).unwrap(),
        Torus3::new(frame(), 4.0, 1.25).unwrap(),
    )
}
/// Central differences of the value function approximate the first derivatives.
fn check_first_derivatives<S: ElementarySurface>(surface: &S, u: f64, v: f64) {
    let step = 1e-6;
    let (point, du, dv) = elslib::d1(u, v, surface);
    close_points(point, elslib::value(u, v, surface), 0.0);
    close_points(point, elslib::d0(u, v, surface), 0.0);
    let numeric_u =
        Vector3::between(surface.value(u - step, v), surface.value(u + step, v)) / (2.0 * step);
    let numeric_v =
        Vector3::between(surface.value(u, v - step), surface.value(u, v + step)) / (2.0 * step);
    close_vectors(du, numeric_u, 1e-6);
    close_vectors(dv, numeric_v, 1e-6);
    close_vectors(elslib::dn(u, v, surface, 1, 0), du, 1e-12);
    close_vectors(elslib::dn(u, v, surface, 0, 1), dv, 1e-12);
}
/// D2/D3 must agree with the DN evaluator for every mixed order they return.
fn check_higher_derivatives<S: CurvedSurface>(surface: &S, u: f64, v: f64) {
    let second = elslib::d2(u, v, surface);
    let third = elslib::d3(u, v, surface);
    assert_eq!(third.second.point, second.point);
    for (value, (nu, nv)) in [
        (second.duu, (2, 0)),
        (second.dvv, (0, 2)),
        (second.duv, (1, 1)),
    ] {
        close_vectors(value, elslib::dn(u, v, surface, nu, nv), 1e-12);
    }
    for (value, (nu, nv)) in [
        (third.duuu, (3, 0)),
        (third.dvvv, (0, 3)),
        (third.duuv, (2, 1)),
    ] {
        close_vectors(value, elslib::dn(u, v, surface, nu, nv), 1e-12);
    }
}

#[test]
fn evaluators_match_finite_differences_and_dn() {
    let (cone, cylinder, sphere, torus) = surfaces();
    let plane = Plane3::from_position(frame());
    for (u, v) in [(0.3, 0.7), (2.0, -0.4), (4.5, 1.1)] {
        check_first_derivatives(&plane, u, v);
        check_first_derivatives(&cone, u, v);
        check_first_derivatives(&cylinder, u, v);
        check_first_derivatives(&sphere, u, v);
        check_first_derivatives(&torus, u, v);
        check_higher_derivatives(&cone, u, v);
        check_higher_derivatives(&cylinder, u, v);
        check_higher_derivatives(&torus, u, v);
    }
}

#[test]
fn sphere_d3_keeps_native_duvv_equal_to_duuu() {
    // OCCT 3d097a ElSLib::SphereD3 assigns Dif1 to Vuvv, the same value as Vuuu.
    let (_, _, sphere, _) = surfaces();
    let third = elslib::d3(0.4, 0.3, &sphere);
    assert_eq!(third.duvv, third.duuu);
    check_higher_derivatives(&sphere, 0.4, 0.3);
}

#[test]
fn parameters_invert_values() {
    let (cone, cylinder, sphere, torus) = surfaces();
    let plane = Plane3::from_position(frame());
    for (u, v) in [(0.3, 0.7), (2.0, -0.4), (4.5, 1.1)] {
        let (pu, pv) = elslib::parameters(&plane, plane.value(u, v));
        assert!((pu - u).abs() < 1e-12 && (pv - v).abs() < 1e-12);
        for (found, expected) in [
            (elslib::parameters(&cone, cone.value(u, v)), (u, v)),
            (elslib::parameters(&cylinder, cylinder.value(u, v)), (u, v)),
            (elslib::parameters(&sphere, sphere.value(u, v)), (u, v)),
            (
                elslib::parameters(&torus, torus.value(u, v)),
                (u, v.rem_euclid(2.0 * PI)),
            ),
        ] {
            assert!(
                (found.0 - expected.0).abs() < 1e-9,
                "{found:?} vs {expected:?}"
            );
            assert!(
                (found.1 - expected.1).abs() < 1e-9,
                "{found:?} vs {expected:?}"
            );
        }
    }
}

#[test]
fn parameters_handle_axis_points() {
    let (cone, _, _, _) = surfaces();
    // Only an exactly zero planar distance (below DBL_MIN) selects the pole branch, so the
    // canonical frame is used to keep the local X/Y coordinates exact.
    let sphere = Sphere3::new(Axis3::default(), 2.0).unwrap();
    assert_eq!(
        elslib::parameters(&sphere, Point3::new(0.0, 0.0, 3.0)),
        (0.0, FRAC_PI_2)
    );
    assert_eq!(
        elslib::parameters(&sphere, Point3::new(0.0, 0.0, -3.0)),
        (0.0, -FRAC_PI_2)
    );
    assert_eq!(
        elslib::parameters(&sphere, Point3::new(0.0, 0.0, 0.0)),
        (0.0, -FRAC_PI_2)
    );
    let (u, _) = elslib::parameters(&cone, cone.location());
    assert_eq!(u, 0.0);
    // Past the apex the cone longitude points the other way.
    let apex_side = cone.value(0.5, -20.0);
    let (u, v) = elslib::parameters(&cone, apex_side);
    assert!((u - 0.5).abs() < 1e-9 && (v + 20.0).abs() < 1e-9);
}

#[test]
fn iso_curves_lie_on_their_surfaces() {
    let (cone, cylinder, sphere, torus) = surfaces();
    let position = frame();
    let u_line = elslib::cylinder_u_iso(position, cylinder.radius(), 1.2);
    close_points(u_line.location(), cylinder.value(1.2, 0.0), 1e-12);
    let v_circle = elslib::cylinder_v_iso(position, cylinder.radius(), 0.8);
    close_points(
        v_circle.location(),
        elslib::plane_value(0.0, 0.0, position)
            .translated(Vector3::from(position.direction()) * 0.8),
        1e-12,
    );
    let cone_line = elslib::cone_u_iso(position, 2.0, 0.4, 1.2);
    close_points(cone_line.location(), cone.value(1.2, 0.0), 1e-12);
    assert!(elslib::cone_v_iso(position, 2.0, 0.4, 0.8).contains(cone.value(2.0, 0.8), 1e-9));
    assert!(elslib::sphere_u_iso(position, 2.5, 1.2).contains(sphere.value(1.2, 0.4), 1e-9));
    assert!(elslib::sphere_v_iso(position, 2.5, 0.4).contains(sphere.value(3.0, 0.4), 1e-9));
    assert!(elslib::torus_u_iso(position, 4.0, 1.25, 1.2).contains(torus.value(1.2, 2.0), 1e-9));
    assert!(elslib::torus_v_iso(position, 4.0, 1.25, 2.0).contains(torus.value(0.3, 2.0), 1e-9));
    assert!(
        elslib::plane_u_iso(position, 1.5)
            .contains(elslib::plane_value(1.5, -3.0, position), 1e-12)
    );
    assert!(
        elslib::plane_v_iso(position, 1.5)
            .contains(elslib::plane_value(-3.0, 1.5, position), 1e-12)
    );
}

#[test]
fn iso_circles_normalize_negative_rings() {
    let position = frame();
    // Latitude beyond the pole: ring radius cos(v) < 0 reverses the circle normal.
    let circle = elslib::sphere_v_iso(position, 2.0, 2.5);
    assert!(circle.radius() > 0.0);
    assert!(
        circle
            .position()
            .direction()
            .is_opposite(position.direction(), 1e-12)
    );
    // Cone beyond its apex: X and Y are reversed instead.
    let cone = elslib::cone_v_iso(position, 1.0, 0.5, -5.0);
    assert!(cone.radius() > 0.0);
    assert!(
        cone.position()
            .x_direction()
            .is_opposite(position.x_direction(), 1e-12)
    );
}

#[test]
fn iso_circles_accept_negative_radii_like_the_no_exception_build() {
    // ElSLib.cxx defines No_Exception, so gp_Circ's radius check is compiled out.
    let position = frame();
    assert_eq!(elslib::torus_u_iso(position, 4.0, -1.0, 0.2).radius(), -1.0);
    assert_eq!(elslib::cylinder_v_iso(position, -2.0, 0.2).radius(), -2.0);
}

#[test]
fn dn_rejects_negative_orders_and_vanishes_beyond_support() {
    let (cone, cylinder, sphere, torus) = surfaces();
    assert_eq!(elslib::dn(0.3, 0.2, &sphere, -1, 2), Vector3::default());
    assert_eq!(elslib::dn(0.3, 0.2, &torus, 0, 0), Vector3::default());
    assert_eq!(
        elslib::dn(0.3, 0.2, &cylinder, 1, 1),
        Vector3::new(0.0, 0.0, 0.0)
    );
    assert_eq!(
        elslib::dn(0.3, 0.2, &cone, 1, 2),
        Vector3::new(0.0, 0.0, 0.0)
    );
    let plane = Plane3::from_position(frame());
    assert_eq!(
        elslib::dn(0.3, 0.2, &plane, 2, 0),
        Vector3::new(0.0, 0.0, 0.0)
    );
    // Native ConeDN(0,0) adds the location to the ring term but omits the axial v·cos(a)·Z
    // term, so it is not the surface point; the port keeps that result.
    let partial = elslib::cone_dn(0.3, 0.2, frame(), 2.0, 0.4, 0, 0);
    let axial = Vector3::from(frame().direction()) * (0.2 * 0.4_f64.cos());
    close_points(
        Point3::from(*(partial + axial).xyz()),
        cone.value(0.3, 0.2),
        1e-12,
    );
}

#[test]
fn torus_flushes_terms_below_its_tolerance() {
    let position = Axis3::default();
    let point = elslib::torus_value(FRAC_PI_2, FRAC_PI_2, position, 4.0, 1.0);
    // cos(PI/2)*R is about 3e-16, below 10*(R+r)*eps, so X is exactly zero.
    assert_eq!(point.x(), 0.0);
    assert!(elslib::cylinder_value(FRAC_PI_2, 0.0, position, 4.0).x() != 0.0);
}
