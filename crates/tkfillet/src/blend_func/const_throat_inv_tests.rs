use super::*;
use crate::blend::{BlendFuncInv, BlendInverseBounds};
use crate::blend_func::linalg::matrix;
use crate::blend_func::test_fixtures::right_angle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Direction2, Point2};
use rustycad_tkmath::math::FunctionSetWithDerivatives;

/// The line `v = 1` of the plane z = 0, parameterized by `u`.
fn iso_line() -> Handle<dyn Adaptor2dCurve2d> {
    let direction = Direction2::new(1., 0.).unwrap();
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0., 1.), direction)).cast();
    Handle::new(Geom2dAdaptorCurve::with_curve(&line).unwrap()).cast()
}

fn x4(values: [f64; 4]) -> Vector<'static> {
    let mut x = vector(4).unwrap();
    for (i, value) in (1..).zip(values) {
        vset(&mut x, i, value).unwrap();
    }
    x
}

#[test]
fn the_symmetric_section_on_the_iso_line_is_a_solution() {
    let (s1, s2, guide) = right_angle();
    let mut inverse = ConstThroatInv::new(&s1, &s2, &guide);
    inverse.set_distances(0.5f64.sqrt(), 0.0, 1);
    inverse.set_curve(true, &iso_line()).unwrap();
    // (0.5, 1, 0) on the curve and (0.5, 0, 1) on the plane y = 0.
    let x = x4([0.5, 0.5, 0.5, -1.0]);
    assert!(inverse.is_solution(&x, 1e-7).unwrap());
    let (mut f, mut d) = (vector(4).unwrap(), matrix(4, 4).unwrap());
    assert!(inverse.values(&x, &mut f, &mut d));
    // Along the curve the first point leaves the guide plane at unit speed.
    assert_eq!(*d.value(1, 1).unwrap(), 1.0);
    assert_eq!(*d.value(1, 2).unwrap(), -1.0);
    assert_eq!((inverse.core.sign1, inverse.core.sign2), (-1., -1.));
}

#[test]
fn derivatives_use_the_points_of_the_previous_evaluation() {
    let (s1, s2, guide) = right_angle();
    let mut inverse = ConstThroatWithPenetrationInv::new(&s1, &s2, &guide);
    inverse.set_distances(1.0, 0.0, 1);
    inverse.set_curve(false, &iso_line()).unwrap();
    let mut d = matrix(4, 4).unwrap();
    // No point was evaluated: the differences start from the origin.
    assert!(inverse.derivatives(&x4([0.0, 2.0, 2.0, 1.0]), &mut d));
    assert_eq!(*d.value(1, 2).unwrap(), -1.0);
    assert_eq!(*d.value(3, 2).unwrap(), 4.0);
}
