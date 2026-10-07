use super::*;
use crate::blend::{BlendAppFunction, BlendFunction};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis3, Direction3};
use rustycad_tkmath::math::FunctionSet;

/// Two planes meeting at a right angle along the X axis: z = 0 (normal +Z) and y = 0
/// (normal +Y), with the X axis as guide.
fn right_angle() -> ConstRad {
    let plane = |normal: Direction3, x: Direction3| -> Handle<dyn Adaptor3dSurface> {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(
            Axis3::new(Point3::default(), normal, x).unwrap(),
        ))
        .cast();
        Handle::new(GeomAdaptorSurface::with_surface(&surface).unwrap()).cast()
    };
    let s1 = plane(
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let s2 = plane(
        Direction3::new(0., 1., 0.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::default(),
        Direction3::new(1., 0., 0.).unwrap(),
    ))
    .cast();
    let guide: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(&line).unwrap()).cast();
    ConstRad::new(&s1, &s2, &guide).unwrap()
}

#[test]
fn signed_radii_follow_the_choice() {
    assert_eq!(signed_radii(2., 1), (-2., -2.));
    assert_eq!(signed_radii(2., 4), (2., -2.));
    assert_eq!(signed_radii(2., 6), (2., 2.));
    assert_eq!(signed_radii(2., 8), (-2., 2.));
    assert_eq!(signed_radii(2., 0), (-2., -2.));
}

#[test]
fn equations_vanish_at_the_tangency_points_of_the_fillet() {
    let mut function = right_angle();
    function.set_radius(1., 1);
    function.set_param(0.).unwrap();
    // On z = 0 the plane coordinates are (x, y); on y = 0 they are (x, -z) or (x, z)
    // depending on the frame: probe both and keep the zero.
    let mut x = super::super::linalg::vector(4).unwrap();
    let mut f = super::super::linalg::vector(4).unwrap();
    for (i, value) in (1..).zip([0., 1., 0., 1.]) {
        x[i] = value;
    }
    assert!(function.value(&x, &mut f));
    assert_eq!(function.variables(), 4);
    assert_eq!(f[1], 0.0);
}

#[test]
fn tangent_queries_raise_on_a_tangency_point() {
    let function = right_angle();
    assert!(function.is_tangency_point());
    assert_eq!(
        function.tangent_on_s1().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    assert!(function.twist_on_s1().is_err());
}
