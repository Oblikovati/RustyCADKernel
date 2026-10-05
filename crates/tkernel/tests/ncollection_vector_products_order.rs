//! Supplemental checks of the pinned OCCT product and interpolation formulas.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};

#[test]
fn dot_two_retains_negative_zero() {
    let left: Vec2<f64> = Vec2::from_value(-0.0);
    let right: Vec2<f64> = Vec2::from_value(1.0);
    assert_eq!(left.dot(&right).to_bits(), (-0.0_f64).to_bits());
}

macro_rules! cancellation_products {
    ($test:ident, $element:ty, $large:expr) => {
        #[test]
        fn $test() {
            let triple: Vec3<$element> = Vec3::new($large, -$large, 1.0);
            let quadruple: Vec4<$element> = Vec4::new($large, -$large, 1.0, 2.0);
            assert_eq!(
                triple.dot(&Vec3::from_value(1.0)).to_bits(),
                (1.0 as $element).to_bits()
            );
            assert_eq!(
                quadruple.dot(&Vec4::from_value(1.0)).to_bits(),
                (3.0 as $element).to_bits()
            );
        }
    };
}

cancellation_products!(dot_float_cancels_before_small_terms, f32, 1.0e20_f32);
cancellation_products!(dot_double_cancels_before_small_terms, f64, 1.0e20_f64);

#[test]
fn cross_product_retains_orientation_and_input_values() {
    let first: Vec3<i32> = Vec3::new(1, 2, 3);
    let second: Vec3<i32> = Vec3::new(4, 5, 6);
    assert_eq!(Vec3::cross(&first, &second).data(), [-3, 6, -3]);
    assert_eq!(Vec3::cross(&second, &first).data(), [3, -6, 3]);
    assert_eq!(first.data(), [1, 2, 3]);
    assert_eq!(second.data(), [4, 5, 6]);
}

#[test]
fn interpolation_extrapolates_without_clamping() {
    let first: Vec3<i32> = Vec3::new(1, 2, 3);
    let second: Vec3<i32> = Vec3::new(4, 5, 6);
    assert_eq!(Vec3::lerp(&first, &second, 2).data(), [7, 8, 9]);
    assert_eq!(Vec3::lerp(&first, &second, -1).data(), [-2, -1, 0]);
}

#[test]
fn interpolation_endpoint_keeps_native_infinite_product() {
    let first: Vec3<f64> = Vec3::from_value(2.0);
    let second: Vec3<f64> = Vec3::from_value(f64::INFINITY);
    let result: Vec3<f64> = Vec3::lerp(&first, &second, 0.0);
    assert!(
        result
            .data()
            .iter()
            .all(|component: &f64| component.is_nan())
    );
}
