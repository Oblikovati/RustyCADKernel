//! Generic scalar wrappers preserve primitive results and widen narrow integer products.
#[path = "vector_component_contract/custom_division.rs"]
mod custom_division;
use custom_division::DivisionElement;
use rustycad_tkernel::ncollection::{
    Mat3, Mat4, Vec2, Vec3, Vec4, VectorDumpComponent, VectorMagnitudeComponent,
};
use rustycad_tkernel::standard::dump::{DumpOutput, DumpStream};

macro_rules! primitive_component_contract {
    ($test:ident, $scalar:ty, $first:expr, $second:expr) => {
        #[test]
        fn $test() {
            assert_eq!(
                Vec2::new(3 as $scalar, 4 as $scalar).modulus(),
                5 as $scalar
            );
            let vector = Vec3::new(3 as $scalar, 4 as $scalar, 0 as $scalar);
            assert_eq!(vector.modulus(), 5 as $scalar);
            assert_eq!(vector.square_modulus(), 25 as $scalar);
            assert_eq!(
                vector.normalized().data(),
                [$first as $scalar, $second as $scalar, 0 as $scalar]
            );
            let mut zero = Vec3::from_value(0 as $scalar);
            zero.normalize();
            assert_eq!(zero.data(), [0 as $scalar; 3]);
            assert_eq!(
                Vec2::new(-3 as $scalar, 4 as $scalar).cwise_abs().data(),
                [3 as $scalar, 4 as $scalar]
            );
            assert_eq!(
                Vec3::new(-3 as $scalar, -4 as $scalar, -5 as $scalar)
                    .cwise_abs()
                    .data(),
                [3 as $scalar, 4 as $scalar, 5 as $scalar]
            );
            assert_eq!(
                Vec4::from_value(-3 as $scalar).cwise_abs().data(),
                [3 as $scalar; 4]
            );
            assert_eq!(<$scalar>::vector_component_modulus([]), 0 as $scalar);
        }
    };
}
primitive_component_contract!(vector_component_f32_contract, f32, 0.6, 0.8);
primitive_component_contract!(vector_component_f64_contract, f64, 0.6, 0.8);
primitive_component_contract!(vector_component_i8_contract, i8, 0, 0);
primitive_component_contract!(vector_component_i16_contract, i16, 0, 0);
primitive_component_contract!(vector_component_i32_contract, i32, 0, 0);
primitive_component_contract!(vector_component_i64_contract, i64, 0, 0);
primitive_component_contract!(vector_component_isize_contract, isize, 0, 0);

#[test]
fn narrow_i8_magnitude_promotes_products_before_sqrt() {
    assert_eq!(Vec2::new(100_i8, 0).modulus(), 100);
    assert_eq!(Vec2::new(100_i8, 0).square_modulus(), 16);
    assert_eq!(Vec3::new(100_i8, 0, 0).normalized().data(), [1, 0, 0]);
}
#[test]
fn narrow_i16_magnitude_promotes_products_before_sqrt() {
    assert_eq!(Vec2::new(200_i16, 200).modulus(), 282);
    assert_eq!(Vec2::new(200_i16, 200).square_modulus(), 14_464);
    assert_eq!(Vec3::new(200_i16, 200, 0).normalized().data(), [0, 0, 0]);
}
macro_rules! floating_absolute_bits {
    ($test:ident, $scalar:ty, $signed_nan:expr, $absolute_nan:expr) => {
        #[test]
        fn $test() {
            let nan: $scalar = <$scalar>::from_bits($signed_nan);
            let absolute = Vec2::new(-0.0 as $scalar, nan).cwise_abs();
            assert_eq!(absolute.x.to_bits(), 0);
            assert_eq!(absolute.y.to_bits(), $absolute_nan);
        }
    };
}
floating_absolute_bits!(
    vector_component_f32_abs_preserves_nan_payload,
    f32,
    0xffc1_2345,
    0x7fc1_2345
);
floating_absolute_bits!(
    vector_component_f64_abs_preserves_nan_payload,
    f64,
    0xfff8_0000_0000_1234,
    0x7ff8_0000_0000_1234
);

fn assert_dump_bytes(expected: &str, write: impl FnOnce(&mut DumpStream)) {
    let mut stream = DumpStream::default();
    write(&mut stream);
    assert_eq!(stream.text(), expected);
    assert_eq!(DumpOutput::position(&stream), expected.len() as i64);
}
macro_rules! floating_dump_contract {
    ($test:ident, $scalar:ty) => {
        #[test]
        fn $test() {
            assert_eq!(
                (0.1 as $scalar).promote_vector_dump(),
                f64::from(0.1 as $scalar)
            );
            assert_dump_bytes("\"Vec2\": [0.1, -2]", |out| {
                Vec2::new(0.1 as $scalar, -2.0 as $scalar)
                    .dump_json(out)
                    .unwrap()
            });
            assert_dump_bytes("\"Vec3\": [1, 0.5, -2]", |out| {
                Vec3::new(1.0 as $scalar, 0.5 as $scalar, -2.0 as $scalar)
                    .dump_json(out)
                    .unwrap()
            });
            assert_dump_bytes("\"Vec4\": [0.1, 0, 0, 1]", |out| {
                Vec4::new(
                    0.1 as $scalar,
                    0.0 as $scalar,
                    0.0 as $scalar,
                    1.0 as $scalar,
                )
                .dump_json(out)
                .unwrap()
            });
            assert_dump_bytes("\"NCollection_Mat3\": [1, 0, 0, 0, 1, 0, 0, 0, 1]", |out| {
                Mat3::<$scalar>::identity().dump_json(out).unwrap()
            });
            assert_dump_bytes(
                "\"NCollection_Mat4\": [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]",
                |out| Mat4::<$scalar>::identity().dump_json(out).unwrap(),
            );
        }
    };
}
floating_dump_contract!(vector_component_f32_dump_contract, f32);
floating_dump_contract!(vector_component_f64_dump_contract, f64);

#[test]
fn custom_normalization_requires_division_without_div_assign() {
    let vector = Vec3::new(
        DivisionElement::from(3),
        DivisionElement::from(4),
        DivisionElement::from(0),
    );
    let normalized = vector.normalized();
    assert_eq!(
        normalized.data().map(|component| component.value),
        [0.6, 0.8, 0.0]
    );
    assert_eq!(
        normalized.data().map(|component| component.divisions),
        [1, 1, 1]
    );
    assert_eq!(
        vector.data().map(|component| component.divisions),
        [0, 0, 0]
    );
}

macro_rules! square_only_contract {
    ($test:ident, $scalar:ty, $first:literal, $second:literal, $expected:literal) => {
        #[test]
        fn $test() {
            assert_eq!(
                Vec2::new($first as $scalar, $second as $scalar).square_modulus(),
                $expected as $scalar
            );
            assert_eq!(
                Vec3::new($first as $scalar, $second as $scalar, 0 as $scalar).square_modulus(),
                $expected as $scalar
            );
        }
    };
}
square_only_contract!(vector_component_u8_promoted_square, u8, 100, 0, 16);
square_only_contract!(vector_component_u16_promoted_square, u16, 200, 200, 14464);
square_only_contract!(vector_component_u32_wrapping_square, u32, 65536, 0, 0);
square_only_contract!(vector_component_u64_wrapping_square, u64, 4294967296, 0, 0);
square_only_contract!(vector_component_usize_square, usize, 3, 4, 25);
square_only_contract!(vector_component_i128_square, i128, 3, 4, 25);
square_only_contract!(vector_component_u128_square, u128, 3, 4, 25);
