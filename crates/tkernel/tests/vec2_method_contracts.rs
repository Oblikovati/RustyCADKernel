//! Constructor overloads, raw pointer identities and unsigned magnitude behavior from native probes.
use rustycad_tkernel::ncollection::{ElementCast, Vec2, Vec3, Vec4, VectorMagnitudeComponent};

#[test]
fn integer_converting_constructors_preserve_native_narrowing_bits() {
    let source = Vec2::new(-257_i64, 65537);
    assert_eq!(Vec2::<i8>::convert_from(source).data(), [-1, 1]);
    assert_eq!(Vec2::<u8>::convert_from(source).data(), [255, 1]);
    assert_eq!(Vec2::<i16>::convert_from(source).data(), [-257, 1]);
    assert_eq!(Vec2::<u16>::convert_from(source).data(), [65279, 1]);
}

#[test]
fn floating_converting_constructors_truncate_native_defined_integer_values() {
    assert_eq!(
        Vec2::<i8>::convert_from(Vec2::new(-12.75_f64, 127.75)).data(),
        [-12, 127]
    );
    assert_eq!(
        Vec2::<u8>::convert_from(Vec2::new(0.75_f32, 254.75)).data(),
        [0, 254]
    );
    assert_eq!(
        Vec2::<i64>::convert_from(Vec2::new(-123.5_f64, 456.75)).data(),
        [-123, 456]
    );
}

#[test]
fn boolean_converting_constructors_match_nonzero_and_nan_truth_values() {
    assert_eq!(
        Vec2::<bool>::convert_from(Vec2::new(f64::NAN, -0.0)).data(),
        [true, false]
    );
    assert_eq!(
        Vec2::<f32>::convert_from(Vec2::new(true, false)).data(),
        [1.0, 0.0]
    );
    assert_eq!(
        Vec2::<bool>::convert_from(Vec2::new(true, false)).data(),
        [true, false]
    );
}

macro_rules! check_cast_destinations {
    ($source:expr; $($target:ty),+) => {$(
        let converted = Vec2::<$target>::convert_from(Vec2::from_value($source));
        assert_eq!(converted.x, 7 as $target);
    )+};
}
macro_rules! check_cast_sources {
    ($($source:ty),+) => {$(
        check_cast_destinations!(7 as $source; i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
        assert!(ElementCast::<bool>::cast(7 as $source));
        assert_eq!(ElementCast::<$source>::cast(true), 1 as $source);
    )+};
}

#[test]
fn all_supported_primitive_converting_constructor_pairs_compile_and_preserve_values() {
    check_cast_sources!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
}

#[test]
#[allow(unsafe_code)] // The native ChangeData adapter lends exclusive provenance for both components.
fn raw_pointer_conversions_expose_native_storage_without_copying() {
    let mut vector = Vec2::new(2_i32, 4);
    assert_eq!(vector.as_ptr(), std::ptr::from_ref(&vector.x));
    let pointer = vector.as_mut_ptr();
    // The vector remains in place and both elements are initialized throughout this write.
    unsafe { *pointer.add(1) = 9 };
    assert_eq!(vector.y, 9);
    assert_eq!(std::ptr::from_mut(vector.y_mut()), pointer.wrapping_add(1));
    *vector.x_mut() = 7;
    assert_eq!(vector.data(), [7, 9]);
}

#[test]
fn shared_coordinate_adapters_preserve_vec3_vec4_storage_and_mutations() {
    let mut triple = Vec3::new(1, 2, 3);
    *triple.x_mut() = 4;
    *triple.y_mut() = 5;
    *triple.z_mut() = 6;
    assert_eq!(triple.as_ptr(), triple.as_mut_ptr());
    assert_eq!(triple.data(), [4, 5, 6]);
    let mut quadruple = Vec4::new(1, 2, 3, 4);
    *quadruple.x_mut() = 5;
    *quadruple.y_mut() = 6;
    *quadruple.z_mut() = 7;
    *quadruple.w_mut() = 8;
    assert_eq!(quadruple.as_ptr(), quadruple.as_mut_ptr());
    assert_eq!(quadruple.data(), [5, 6, 7, 8]);
}

#[test]
fn vec2_compound_return_references_allow_native_chained_self_value_operations() {
    let mut vector = Vec2::new(2_i32, 9);
    let address = std::ptr::from_ref(&vector);
    let duplicate = vector;
    assert_eq!(
        std::ptr::from_mut(vector.add_assign_returning(&duplicate)),
        address.cast_mut()
    );
    let duplicate = vector;
    assert_eq!(
        std::ptr::from_mut(vector.sub_assign_returning(&duplicate)),
        address.cast_mut()
    );
    assert_eq!(vector.data(), [0, 0]);
}

macro_rules! unsigned_modulus_case {
    ($name:ident, $scalar:ty) => {
        #[test]
        fn $name() {
            assert_eq!(Vec2::new(3 as $scalar, 4).modulus(), 5);
            let mut triple = Vec3::new(3 as $scalar, 4, 0);
            assert_eq!(triple.modulus(), 5);
            triple.normalize();
            assert_eq!(triple.data(), [0, 0, 0]);
            let mut zero = Vec3::from_value(0 as $scalar);
            zero.normalize();
            assert_eq!(zero.data(), [0, 0, 0]);
            assert!((0 as $scalar).vector_component_is_zero());
            assert!(!(1 as $scalar).vector_component_is_zero());
        }
    };
}
unsigned_modulus_case!(unsigned_byte_modulus_and_normalization_match_native, u8);
unsigned_modulus_case!(unsigned_short_modulus_and_normalization_match_native, u16);
unsigned_modulus_case!(unsigned_int_modulus_and_normalization_match_native, u32);
unsigned_modulus_case!(unsigned_long_modulus_and_normalization_match_native, u64);
unsigned_modulus_case!(
    unsigned_pointer_modulus_and_normalization_match_native,
    usize
);

#[test]
fn vec3_planar_constructor_supplies_native_zero_default_argument() {
    assert_eq!(Vec3::from_vec2_zero(Vec2::new(1_u16, 2)).data(), [1, 2, 0]);
    assert_eq!(
        Vec3::from_vec2_zero(Vec2::new(true, true)).data(),
        [true, true, false]
    );
}

#[test]
fn narrow_integer_products_and_dot_keep_native_promotions_and_final_narrowing() {
    let signed = Vec2::new(100_i8, 2);
    let signed_other = Vec2::new(100_i8, 3);
    assert_eq!((signed * signed_other).data(), [16, 6]);
    assert_eq!((signed + signed_other).data(), [-56, 5]);
    assert_eq!(signed.dot(&signed_other), 22);
    let unsigned = Vec2::new(200_u8, 2);
    let unsigned_other = Vec2::new(2_u8, 3);
    assert_eq!((unsigned * unsigned_other).data(), [144, 6]);
    assert_eq!(unsigned.dot(&unsigned_other), 150);
    assert_eq!(
        (Vec2::new(120_i8, -120) - Vec2::new(-120, 120)).data(),
        [-16, 16]
    );
}

#[test]
fn narrow_signed_negation_and_division_use_promoted_intermediates() {
    assert_eq!((-Vec2::new(i8::MIN, 9)).data(), [i8::MIN, -9]);
    assert_eq!((Vec2::new(i8::MIN, 9) / -1).data(), [i8::MIN, -9]);
    assert_eq!((Vec3::new(i8::MIN, 9, 0) / -1).data(), [i8::MIN, -9, 0]);
    assert_eq!(
        (Vec4::new(i8::MIN, 9, 0, 1) / -1).data(),
        [i8::MIN, -9, 0, -1]
    );
}

#[test]
fn compound_integer_operations_narrow_after_each_native_assignment() {
    let mut vector = Vec2::new(100_i8, 2);
    vector *= Vec2::new(100, 3);
    assert_eq!(vector.data(), [16, 6]);
    vector += Vec2::new(120, 3);
    assert_eq!(vector.data(), [-120, 9]);
    vector -= Vec2::new(120, 3);
    assert_eq!(vector.data(), [16, 6]);
    vector /= Vec2::new(2, 3);
    assert_eq!(vector.data(), [8, 2]);
    assert_eq!(Vec3::new(65536_u32, 1, 0).modulus(), 1);
}
