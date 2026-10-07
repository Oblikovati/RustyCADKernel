use super::*;
use crate::blend::BlendAppFunction;
use crate::blend_func::test_fixtures::right_angle;
use rustycad_tkmath::math::FunctionSet;

fn x4(values: [f64; 4]) -> Vector<'static> {
    let mut x = vector(4).unwrap();
    for (i, value) in (1..).zip(values) {
        vset(&mut x, i, value).unwrap();
    }
    x
}

fn chamfer(angle: f64) -> ChAsym {
    let (s1, s2, guide) = right_angle();
    let mut chamfer = ChAsym::new(&s1, &s2, &guide).unwrap();
    chamfer.set_dist_angle(-1.0, angle, 1);
    chamfer.set_param(0.5).unwrap();
    chamfer
}

#[test]
fn the_angle_equation_fixes_the_second_point() {
    // The points (t, 1, 0) and (t, 0, 2); the angle with no tangent gives the residual.
    let sol = x4([0.5, 1.0, 0.5, -2.0]);
    let mut f = vector(4).unwrap();
    let mut flat = chamfer(0.0);
    assert!(flat.value(&sol, &mut f));
    assert_eq!([f[1], f[2], f[3]], [0.0, 0.0, 0.0]);
    let mut function = chamfer((-f[4]).atan());
    assert!(function.is_solution(&sol, 1e-7).unwrap());
    assert!(!function.is_tangency_point());
    assert_eq!(function.get_minimal_distance().unwrap(), 5f64.sqrt());
    assert_eq!(function.tangent2d_on_s2().unwrap(), Vector2::new(1.0, 0.0));
}

#[test]
fn a_refused_point_is_a_tangency_point() {
    let mut function = chamfer(0.3);
    assert!(
        !function
            .is_solution(&x4([0.5, 1.5, 0.5, -2.0]), 1e-7)
            .unwrap()
    );
    assert!(function.is_tangency_point());
    let error = function.twist_on_s1().unwrap_err();
    assert_eq!(error.message(), "BlendFunc_ChAsym::TwistOnS1");
    assert!(!function.compute_values(&x4([0.; 4]), 1, 0).unwrap());
}

#[test]
fn the_distance_is_unsigned_and_the_angle_tangent_kept() {
    let function = chamfer(0.25);
    assert_eq!(
        (function.dist1, function.angle, function.tgang),
        (1.0, 0.25, 0.25f64.tan())
    );
    let error = function.get_section_size().unwrap_err();
    assert_eq!(error.message(), "BlendFunc_ChAsym::GetSectionSize()");
}
