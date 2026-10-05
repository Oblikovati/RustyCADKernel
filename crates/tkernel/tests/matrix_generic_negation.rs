//! Generic negation requires only the native element unary operation.
use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::ops::Neg;

#[derive(Clone, Copy, Debug, PartialEq)]
struct NegationElement(i32);

impl Neg for NegationElement {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

macro_rules! negation_case {
    ($name:ident, $matrix:ident, $count:literal) => {
        #[test]
        fn $name() {
            let entries = std::array::from_fn(|index| NegationElement(index as i32 - 4));
            let matrix = $matrix::from_data(entries);
            let expected = entries.map(|entry| NegationElement(-entry.0));
            assert_eq!(matrix.negated().data(), expected);
            assert_eq!((-matrix).data(), expected);
            assert_eq!(matrix.data(), entries);
            assert_eq!((-$matrix::from_data([7_i64; $count])).data(), [-7; $count]);
        }
    };
}
negation_case!(mat3_generic_unary_negation, Mat3, 9);
negation_case!(mat4_generic_unary_negation, Mat4, 16);

#[test]
fn floating_negation_preserves_native_sign_bits() {
    let entries = [
        0.0_f64,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(0x7ff8_0000_0000_0042),
        1.0,
        -1.0,
        2.0,
        -2.0,
    ];
    let result = Mat3::from_data(entries).negated().data();
    for (original, negated) in entries.into_iter().zip(result) {
        assert_eq!(negated.to_bits(), original.to_bits() ^ (1_u64 << 63));
    }
}
