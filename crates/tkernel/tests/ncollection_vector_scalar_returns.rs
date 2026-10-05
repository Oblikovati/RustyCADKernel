//! Scalar assignment references preserve the direct and reciprocal division paths.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};

macro_rules! scalar_destination_identity {
    ($test:ident, $vector:ident, $count:literal) => {
        #[test]
        fn $test() {
            let mut destination = $vector::from_value(12_i64);
            let original: *const $vector<i64> = &destination;
            let returned = destination.multiply_scalar_returning(3);
            assert!(std::ptr::eq(returned, original));
            assert_eq!(returned.data(), [36; $count]);
            let returned = returned.divide_scalar_returning(3);
            assert!(std::ptr::eq(returned, original));
            assert_eq!(returned.data(), [12; $count]);
        }
    };
}

scalar_destination_identity!(vec2_scalar_returns_destination, Vec2, 2);
scalar_destination_identity!(vec3_scalar_returns_destination, Vec3, 3);
scalar_destination_identity!(vec4_scalar_returns_destination, Vec4, 4);

macro_rules! reciprocal_destination_identity {
    ($test:ident, $vector:ident, $element:ty, $count:literal) => {
        #[test]
        fn $test() {
            let mut destination = $vector::from_value(5.0 as $element);
            let original: *const $vector<$element> = &destination;
            let returned = destination.divide_scalar_returning(3.0);
            assert!(std::ptr::eq(returned, original));
            let expected: $element = 5.0 * (1.0 / 3.0 as $element);
            for component in returned.data() {
                assert_eq!(component.to_bits(), expected.to_bits());
                assert_ne!(component.to_bits(), (5.0 / 3.0 as $element).to_bits());
            }
            assert_eq!(returned.data().len(), $count);
        }
    };
}

reciprocal_destination_identity!(vec3_f32_return_keeps_reciprocal, Vec3, f32, 3);
reciprocal_destination_identity!(vec3_f64_return_keeps_reciprocal, Vec3, f64, 3);
reciprocal_destination_identity!(vec4_f32_return_keeps_reciprocal, Vec4, f32, 4);
reciprocal_destination_identity!(vec4_f64_return_keeps_reciprocal, Vec4, f64, 4);

#[test]
fn vec2_floating_return_keeps_direct_division() {
    let mut destination = Vec2::new(5.0_f64, 10.0);
    let returned = destination.divide_scalar_returning(3.0);
    assert_eq!(returned.x.to_bits(), (5.0 / 3.0_f64).to_bits());
    assert_eq!(returned.y.to_bits(), (10.0 / 3.0_f64).to_bits());
}
