//! Vec3/4 preserve native assignment dispatch and floating-point reciprocal specialization.
use rustycad_tkernel::ncollection::{Vec3, Vec4, VectorScalarDivision};
use std::ops::DivAssign;

#[derive(Clone, Copy, Debug, PartialEq)]
struct DivisionTagged(i32);
impl DivAssign for DivisionTagged {
    fn div_assign(&mut self, right: Self) {
        self.0 = 2000 + self.0 * 10 + right.0;
    }
}
impl VectorScalarDivision for DivisionTagged {}

#[test]
fn vec3_copy_and_assignment_use_generic_element_div_assign() {
    let original = Vec3::new(DivisionTagged(2), DivisionTagged(4), DivisionTagged(6));
    let expected = Vec3::new(
        DivisionTagged(2023),
        DivisionTagged(2043),
        DivisionTagged(2063),
    );
    assert_eq!(original / DivisionTagged(3), expected);
    let mut assigned = original;
    assigned /= DivisionTagged(3);
    assert_eq!(assigned, expected);
}

#[test]
fn vec4_copy_and_assignment_use_generic_element_div_assign() {
    let original = Vec4::from_array([DivisionTagged(2); 4]);
    let expected = Vec4::from_array([DivisionTagged(2023); 4]);
    assert_eq!(original / DivisionTagged(3), expected);
    let mut assigned = original;
    assigned /= DivisionTagged(3);
    assert_eq!(assigned, expected);
}

#[test]
fn wider_integer_vectors_divide_without_a_reciprocal() {
    assert_eq!(Vec3::new(15_i64, -9, 6) / 3, Vec3::new(5, -3, 2));
    assert_eq!(Vec4::new(15_u64, 9, 6, 3) / 3, Vec4::new(5, 3, 2, 1));
}

#[test]
fn float_vectors_keep_reciprocal_rounding() {
    let source = Vec3::new(5.0_f32, -5.0, 0.0);
    let result = source / 3.0;
    let expected = 5.0_f32 * (1.0 / 3.0);
    assert_eq!(result.x.to_bits(), expected.to_bits());
    assert_ne!(result.x.to_bits(), (5.0_f32 / 3.0).to_bits());
    let result = Vec4::new(5.0_f64, -5.0, 0.0, 1.0) / 3.0;
    assert_eq!(result.x.to_bits(), (5.0_f64 * (1.0 / 3.0)).to_bits());
    assert_ne!(result.x.to_bits(), (5.0_f64 / 3.0).to_bits());
}
