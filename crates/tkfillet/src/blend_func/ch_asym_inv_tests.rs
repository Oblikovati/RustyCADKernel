use super::*;
use crate::blend::{BlendFuncInv, BlendInverseBounds};
use crate::blend_func::test_fixtures::right_angle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::Direction2;
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives};

/// The inverse ending on the line `v = 1` of the plane z = 0, with choice 1 (the guide
/// plane reversed, so that the angle tangent is positive).
fn inverse(angle: f64) -> ChAsymInv {
    let (s1, s2, guide) = right_angle();
    let mut inverse = ChAsymInv::new(&s1, &s2, &guide).unwrap();
    inverse.set_dist_angle(1.0, angle, 1);
    let direction = Direction2::new(1., 0.).unwrap();
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0., 1.), direction)).cast();
    let curve: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(&line).unwrap()).cast();
    inverse.set_curve(true, &curve).unwrap();
    inverse
}

fn x4(values: [f64; 4]) -> Vector<'static> {
    let mut x = vector(4).unwrap();
    for (i, value) in (1..).zip(values) {
        vset(&mut x, i, value).unwrap();
    }
    x
}

#[test]
fn the_section_at_the_angle_is_a_solution() {
    // (0.5, 1, 0) on the curve and (0.5, 0, 2) on the plane y = 0.
    let x = x4([0.5, 0.5, 0.5, -2.0]);
    let mut f = vector(4).unwrap();
    assert!(inverse(0.0).value(&x, &mut f));
    assert_eq!([f[1], f[2], f[3]], [0.0, 0.0, 0.0]);
    let mut function = inverse((-f[4]).atan());
    assert!(function.is_solution(&x, 1e-7).unwrap());
    let (mut f, mut d) = (vector(4).unwrap(), matrix(4, 4).unwrap());
    assert!(function.values(&x, &mut f, &mut d));
    assert_eq!(*d.value(1, 1).unwrap(), -1.0);
    assert_eq!(*d.value(1, 2).unwrap(), 1.0);
    assert_eq!(*d.value(3, 2).unwrap(), 0.0);
}

#[test]
fn derivatives_keep_the_last_equations() {
    let mut function = inverse(0.2);
    let mut f = vector(4).unwrap();
    f.init(5.0);
    let mut d = matrix(4, 4).unwrap();
    // `Derivatives` does not compute the equations: `FX` keeps its last (zero) values.
    assert!(function.derivatives(&x4([0.5, 0.5, 0.5, -2.0]), &mut d));
    assert!(!function.compute_values(&x4([0.0; 4]), 1, 0).unwrap());
    assert!(function.value(&x4([0.5, 0.5, 0.5, -2.0]), &mut f));
    assert_eq!(f[1], 0.0);
}
