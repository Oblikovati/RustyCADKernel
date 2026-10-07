use super::*;
use crate::blend::{BlendAppFunction, BlendFunction};
use rustycad_tkmath::math::FunctionSet;

/// The right-angled planes of [`super::super::test_fixtures::right_angle`].
fn right_angle() -> ConstRad {
    let (s1, s2, guide) = super::super::test_fixtures::right_angle();
    ConstRad::new(&s1, &s2, &guide).unwrap()
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
