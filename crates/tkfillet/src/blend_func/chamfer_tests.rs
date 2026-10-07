use super::*;
use crate::blend::BlendAppFunction;
use crate::blend_func::test_fixtures::right_angle;
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives};

/// The chamfer of distances 1 and 2 between the right-angled planes, at `t = 0.5`.
fn chamfer() -> Chamfer {
    let (s1, s2, guide) = right_angle();
    let mut chamfer = Chamfer::new(&s1, &s2, &guide);
    chamfer.set_distances(1.0, 2.0, 1);
    chamfer.set_param(0.5).unwrap();
    chamfer
}

fn x4(values: [f64; 4]) -> Vector<'static> {
    let mut x = vector(4).unwrap();
    for (i, value) in (1..).zip(values) {
        vset(&mut x, i, value).unwrap();
    }
    x
}

#[test]
fn the_exact_section_is_a_solution() {
    let mut chamfer = chamfer();
    let sol = x4([0.5, 1.0, 0.5, -2.0]);
    let mut f = vector(4).unwrap();
    assert!(chamfer.value(&sol, &mut f));
    assert_eq!((0..4).map(|i| f[i + 1]).collect::<Vec<_>>(), vec![0.0; 4]);
    assert!(chamfer.is_solution(&sol, 1e-9).unwrap());
    assert_eq!(chamfer.point_on_s1(), Point3::new(0.5, 1.0, 0.0));
    assert_eq!(chamfer.point_on_s2(), Point3::new(0.5, 0.0, 2.0));
    assert_eq!(chamfer.get_minimal_distance().unwrap(), 5f64.sqrt());
    assert_eq!(chamfer.tangent2d_on_s1().unwrap(), Vector2::new(1.0, 0.0));
}

#[test]
fn the_jacobian_is_block_diagonal() {
    let mut chamfer = chamfer();
    let mut d = matrix(4, 4).unwrap();
    d.init(7.0);
    assert!(chamfer.derivatives(&x4([0.5, 1.0, 0.5, -2.0]), &mut d));
    for (i, j) in [
        (1, 3),
        (1, 4),
        (2, 3),
        (2, 4),
        (3, 1),
        (3, 2),
        (4, 1),
        (4, 2),
    ] {
        assert_eq!(*d.value(i, j).unwrap(), 0.0, "D({i}, {j})");
    }
    assert_eq!(*d.value(2, 2).unwrap(), 2.0);
}

#[test]
fn the_section_is_the_line_between_the_points() {
    let chamfer = chamfer();
    let (pdeb, pfin, line) = chamfer.section_line([0.5, 1.0, 0.5, -2.0]).unwrap();
    assert_eq!((pdeb, pfin), (0.0, 5f64.sqrt()));
    assert_eq!(line.location(), Point3::new(0.5, 1.0, 0.0));
    let error = chamfer.get_section_size().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert_eq!(error.message(), "BlendFunc_Chamfer::GetSectionSize()");
}

#[test]
fn the_choice_reverses_the_tangents() {
    let mut chamfer = chamfer();
    let plain = chamfer.tangent(0.5, 1.0, 0.5, -2.0).unwrap();
    chamfer.set_distances(1.0, 2.0, 2);
    let reversed = chamfer.tangent(0.5, 1.0, 0.5, -2.0).unwrap();
    assert_eq!(reversed.tg_first, plain.tg_first.reversed());
    assert_eq!(reversed.tg_last, plain.tg_last.reversed());
    assert_eq!(reversed.norm_first, plain.norm_first);
}
