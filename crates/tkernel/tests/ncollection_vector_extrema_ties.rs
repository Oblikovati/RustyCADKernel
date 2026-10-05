//! OCCT 3d097a03 chooses the right component when comparisons are unordered or tied.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};

fn assert_right_bits(actual: &[f64], expected: &[f64]) {
    let actual_bits: Vec<u64> = actual.iter().map(|value| value.to_bits()).collect();
    let expected_bits: Vec<u64> = expected.iter().map(|value| value.to_bits()).collect();
    assert_eq!(actual_bits, expected_bits);
}

#[test]
fn vec2_extrema_keep_right_signed_zero_and_nan_payload() {
    let nan = f64::from_bits(0x7ff8_0000_0000_1234);
    let left = Vec2::new(-0.0, 7.0);
    let right = Vec2::new(0.0, nan);
    assert_right_bits(&left.cwise_min(&right).data(), &right.data());
    assert_right_bits(&left.cwise_max(&right).data(), &right.data());
    assert_right_bits(&right.cwise_min(&left).data(), &left.data());
    assert_right_bits(&right.cwise_max(&left).data(), &left.data());
}

#[test]
fn vec3_extrema_keep_right_signed_zero_and_nan_payload() {
    let nan = f64::from_bits(0x7ff8_0000_0000_1234);
    let left = Vec3::new(-0.0, 7.0, nan);
    let right = Vec3::new(0.0, nan, 7.0);
    assert_right_bits(&left.cwise_min(&right).data(), &right.data());
    assert_right_bits(&left.cwise_max(&right).data(), &right.data());
    assert_right_bits(&right.cwise_min(&left).data(), &left.data());
    assert_right_bits(&right.cwise_max(&left).data(), &left.data());
}

#[test]
fn vec4_extrema_keep_right_signed_zero_and_nan_payload() {
    let nan = f64::from_bits(0x7ff8_0000_0000_1234);
    let left = Vec4::new(-0.0, 7.0, nan, 0.0);
    let right = Vec4::new(0.0, nan, 7.0, -0.0);
    assert_right_bits(&left.cwise_min(&right).data(), &right.data());
    assert_right_bits(&left.cwise_max(&right).data(), &right.data());
    assert_right_bits(&right.cwise_min(&left).data(), &left.data());
    assert_right_bits(&right.cwise_max(&left).data(), &left.data());
}
