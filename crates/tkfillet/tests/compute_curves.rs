//! ChFi3d_ComputeCurves on a plane with a plane, a plane with a cylinder (analytic), and
//! a sphere with a cylinder (walked and approximated).
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi3d::{IntersectionCurves, compute_curves};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn adapt(s: Handle<dyn GeomSurface>) -> Handle<dyn Adaptor3dSurface> {
    Handle::new(GeomAdaptorSurface::with_surface(&s).unwrap()).cast()
}

fn plane(p: Point3, n: (f64, f64, f64), x: (f64, f64, f64)) -> Handle<dyn Adaptor3dSurface> {
    let frame = Axis3::new(
        p,
        Direction3::new(n.0, n.1, n.2).unwrap(),
        Direction3::new(x.0, x.1, x.2).unwrap(),
    )
    .unwrap();
    adapt(Handle::new(GeomPlane::new(frame)).cast())
}

fn assert_ends(out: &IntersectionCurves, start: Point3, end: Point3) {
    let c = out.c3d.get().expect("a 3d curve");
    assert!(c.value(c.first_parameter()).unwrap().distance(&start) < 1e-6);
    assert!(c.value(c.last_parameter()).unwrap().distance(&end) < 1e-6);
    assert!(!out.pc1.is_null() && !out.pc2.is_null());
}

#[test]
fn two_planes_meet_along_their_common_line() {
    let (s1, s2) = (
        plane(Point3::default(), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0)),
        plane(Point3::default(), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)),
    );
    let mut out = IntersectionCurves::default();
    assert!(
        compute_curves(
            (&s1, &s2),
            ([0.0, 0.0, 0.0, 0.0], [0.0, 5.0, 5.0, 0.0]),
            (1e-4, 1e-5),
            false,
            &mut out
        )
        .unwrap()
    );
    assert_ends(&out, Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 5.0, 0.0));
    assert_eq!(
        out.tolreached, 1e-4,
        "the analytic line keeps the 3d tolerance"
    );
}

#[test]
fn a_plane_cuts_a_cylinder_in_a_circle_arc() {
    let s1 = plane(Point3::default(), (0.0, 0.0, 1.0), (1.0, 0.0, 0.0));
    let s2 = adapt(Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap()).cast());
    let mut out = IntersectionCurves::default();
    let fin = [0.0, 2.0, std::f64::consts::FRAC_PI_2, 0.0];
    assert!(
        compute_curves(
            (&s1, &s2),
            ([2.0, 0.0, 0.0, 0.0], fin),
            (1e-4, 1e-5),
            false,
            &mut out
        )
        .unwrap()
    );
    assert_ends(&out, Point3::new(2.0, 0.0, 0.0), Point3::new(0.0, 2.0, 0.0));
    assert!(out.tolreached <= 1.5e-4);
}

#[test]
fn a_sphere_and_a_cylinder_are_walked_and_approximated() {
    let s1 = adapt(Handle::new(GeomSphericalSurface::new(Axis3::default(), 3.0).unwrap()).cast());
    let s2 = adapt(Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap()).cast());
    let (lat, h) = ((5.0_f64.sqrt() / 3.0).asin(), 5.0_f64.sqrt());
    let mut out = IntersectionCurves::default();
    assert!(
        compute_curves(
            (&s1, &s2),
            ([0.3, lat, 0.3, h], [1.5, lat, 1.5, h]),
            (1e-4, 1e-5),
            false,
            &mut out
        )
        .unwrap()
    );
    let at = |a: f64| Point3::new(2.0 * a.cos(), 2.0 * a.sin(), h);
    assert_ends(&out, at(0.3), at(1.5));
    assert!(out.tolreached < 1e-4);
}
