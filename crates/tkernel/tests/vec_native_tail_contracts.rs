//! Native boolean promotions, unsigned unary minus and explicit zero constructors.
use rustycad_tkernel::ncollection::{ElementCast, Vec2, Vec3, Vec4, VectorNegationComponent};
mod vec_native_tail_contracts {
    pub mod oracle;
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct CustomZero(i32);
impl Default for CustomZero {
    fn default() -> Self {
        Self(7)
    }
}
impl ElementCast<CustomZero> for i32 {
    fn cast(self) -> CustomZero {
        CustomZero(self)
    }
}
#[test]
fn vector_defaults_invoke_integer_zero_conversion() {
    assert_eq!(CustomZero::default().0, 7);
    assert_eq!(Vec2::<CustomZero>::default().data(), [CustomZero(0); 2]);
    assert_eq!(Vec3::<CustomZero>::default().data(), [CustomZero(0); 3]);
    assert_eq!(Vec4::<CustomZero>::default().data(), [CustomZero(0); 4]);
}
#[test]
fn omitted_components_use_native_zero() {
    let planar = Vec2::new(CustomZero(2), CustomZero(3));
    assert_eq!(Vec3::from_vec2_zero(planar).z, CustomZero(0));
    assert_eq!(
        Vec4::from_vec3_zero(Vec3::from_value(CustomZero(2))).w,
        CustomZero(0)
    );
    assert_eq!(
        Vec4::from_vec2(planar).data(),
        [CustomZero(2), CustomZero(3), CustomZero(0), CustomZero(0)]
    );
}
#[test]
fn unit_vectors_use_native_constants() {
    assert_eq!(Vec2::<i8>::dx().data(), [1, 0]);
    assert_eq!(Vec2::<i8>::dy().data(), [0, 1]);
    assert_eq!(Vec3::<CustomZero>::dx().data().map(|v| v.0), [1, 0, 0]);
    assert_eq!(Vec3::<CustomZero>::dy().data().map(|v| v.0), [0, 1, 0]);
    assert_eq!(Vec3::<CustomZero>::dz().data().map(|v| v.0), [0, 0, 1]);
}
#[test]
fn unsigned_negation_wraps_after_native_promotion() {
    assert_eq!((-Vec2::new(0_u8, 1)).data(), [0, 255]);
    assert_eq!((-Vec3::new(1_u16, 32768, 0)).data(), [65535, 32768, 0]);
    assert_eq!(
        (-Vec4::new(1_u32, 2147483648, 0, 2)).data(),
        [u32::MAX, 2147483648, 0, u32::MAX - 1]
    );
    assert_eq!(
        (-Vec2::new(1_u64, 9223372036854775808)).data(),
        [u64::MAX, 9223372036854775808]
    );
    assert_eq!((-Vec2::new(1_usize, 0)).data(), [usize::MAX, 0]);
    assert_eq!((-Vec2::new(1_u128, 0)).data(), [u128::MAX, 0]);
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct CustomNegation(i32);
impl std::ops::Neg for CustomNegation {
    type Output = Self;
    fn neg(self) -> Self {
        Self(100 - self.0)
    }
}
impl VectorNegationComponent for CustomNegation {
    fn vector_negated_component(self) -> Self {
        -self
    }
}
#[test]
fn custom_negation_preserves_scalar_unary_dispatch() {
    assert_eq!(
        (-Vec2::new(CustomNegation(1), CustomNegation(2))).data(),
        [CustomNegation(99), CustomNegation(98)]
    );
    assert_eq!(
        (-Vec3::from_value(CustomNegation(3))).data(),
        [CustomNegation(97); 3]
    );
    assert_eq!(
        (-Vec4::from_value(CustomNegation(4))).data(),
        [CustomNegation(96); 4]
    );
}
#[test]
fn signed_and_floating_negation_keep_existing_policy() {
    assert_eq!((-Vec2::new(i8::MIN, 1)).data(), [i8::MIN, -1]);
    assert_eq!((-Vec2::new(i16::MIN, 1)).data(), [i16::MIN, -1]);
    assert_eq!(
        (-Vec2::new(0.0_f32, -0.0)).x.to_bits(),
        (-0.0_f32).to_bits()
    );
    assert_eq!((-Vec2::new(2.0_f64, -3.0)).data(), [-2.0, 3.0]);
}
#[test]
fn boolean_normalization_preserves_zero_and_nonzero_vectors() {
    for mask in 0..8 {
        let mut vector = Vec3::new(mask & 1 != 0, mask & 2 != 0, mask & 4 != 0);
        let previous = vector;
        vector.normalize();
        assert_eq!(vector, previous);
        assert_eq!(vector.normalized(), previous);
        assert_eq!(vector.modulus(), mask != 0);
        assert_eq!(vector.square_modulus(), mask != 0);
    }
}
#[test]
#[should_panic(expected = "boolean vector divisor false: expected true")]
fn boolean_zero_division_reports_invalid_divisor() {
    let _ = Vec2::new(true, false) / false;
}
#[test]
fn native_fixture_rows_are_reconstructed_from_rust_operations() {
    assert_eq!(
        vec_native_tail_contracts::oracle::rows(),
        include_str!("../../../tools/oracle/tkernel_vec_native_tail_contracts.expected")
    );
}
macro_rules! boolean_assignment_contract {
    ($name:ident, $vector:ident) => {
        #[test]
        fn $name() {
            let mut value = $vector::from_value(true);
            value += $vector::from_value(false);
            value -= $vector::from_value(true);
            assert_eq!(value, $vector::from_value(false));
            value += $vector::from_value(true);
            value *= $vector::from_value(true);
            value /= $vector::from_value(true);
            value *= true;
            value /= true;
            assert_eq!(value, $vector::from_value(true));
            let pointer = &raw mut value;
            assert!(std::ptr::eq(
                value.multiply_scalar_returning(false),
                pointer
            ));
            assert_eq!(value, $vector::from_value(false));
            assert!(std::ptr::eq(value.divide_scalar_returning(true), pointer));
        }
    };
}
boolean_assignment_contract!(boolean_planar_assignment_operators, Vec2);
boolean_assignment_contract!(boolean_spatial_assignment_operators, Vec3);
boolean_assignment_contract!(boolean_homogeneous_assignment_operators, Vec4);
