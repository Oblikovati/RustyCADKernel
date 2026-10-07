use super::*;
use crate::blend::BlendAppFunction;
use crate::blend_func::GenChamferFunction;
use crate::blend_func::test_fixtures::right_angle;
use rustycad_tkmath::math::FunctionSet;

fn x4(values: [f64; 4]) -> Vector<'static> {
    let mut x = vector(4).unwrap();
    for (i, value) in (1..).zip(values) {
        vset(&mut x, i, value).unwrap();
    }
    x
}

#[test]
fn the_symmetric_throat_section_is_a_solution() {
    let (s1, s2, guide) = right_angle();
    let mut chamfer = ConstThroat::new(&s1, &s2, &guide);
    chamfer.set_distances(0.5f64.sqrt(), 7.0, 1);
    chamfer.set_param(0.25).unwrap();
    // The points (t, 1, 0) and (t, 0, 1): their middle is at sqrt(1/2) from the guide.
    let sol = x4([0.25, 1.0, 0.25, -1.0]);
    // F(3) carries the rounding of sqrt(1/2)^2, within tol^2 = 1e-14.
    assert!(chamfer.is_solution(&sol, 1e-7).unwrap());
    assert!(!chamfer.is_tangency_point());
    assert_eq!(chamfer.point_on_s2(), Point3::new(0.25, 0.0, 1.0));
    assert_eq!(chamfer.get_minimal_distance().unwrap(), 2f64.sqrt());
    // Both points follow the guide.
    assert_eq!(chamfer.tangent2d_on_s1().unwrap(), Vector2::new(1.0, 0.0));
}

#[test]
fn penetration_measures_the_first_point() {
    let (s1, s2, guide) = right_angle();
    let mut chamfer = ConstThroatWithPenetration::new(&s1, &s2, &guide);
    chamfer.set_distances(1.0, 0.0, 1);
    chamfer.set_param(0.0).unwrap();
    let mut f = vector(4).unwrap();
    assert!(chamfer.value(&x4([0.0, 1.0, 0.0, -3.0]), &mut f));
    // (0, 1, 0) is at the throat; the chord to (0, 0, 3) is not normal to it.
    assert_eq!([f[1], f[2], f[3], f[4]], [0.0, 0.0, 0.0, -1.0]);
    assert!(
        !chamfer
            .is_solution(&x4([0.0, 1.0, 0.0, -3.0]), 1e-3)
            .unwrap()
    );
}

#[test]
fn tangents_fail_on_a_tangency_point() {
    let (s1, s2, guide) = right_angle();
    let mut chamfer = ConstThroatWithPenetration::new(&s1, &s2, &guide);
    chamfer.state.istangent = true;
    let error = chamfer.tangent_on_s2().unwrap_err();
    assert_eq!(
        error.message(),
        "BlendFunc_ConstThroatWithPenetration::TangentOnS2"
    );
    let section = ConstThroat::new(&s1, &s2, &guide)
        .get_section_size()
        .unwrap_err();
    assert_eq!(section.message(), "BlendFunc_ConstThroat::GetSectionSize()");
}
