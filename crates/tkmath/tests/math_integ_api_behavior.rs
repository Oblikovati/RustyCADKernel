//! Supplemental checks for the public MathInteg constants and TanhSinhSingular (MathInteg_*.hxx
//! at 3d097a0); bit-level parity lives in the `integ_lin_singular` numeric probe.
use rustycad_tkmath::math::Function;
use rustycad_tkmath::math_integ::*;
use rustycad_tkmath::math_utils::Status;

struct InverseSqrt;
impl Function for InverseSqrt {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = 1.0 / x.sqrt();
        true
    }
}

#[test]
fn constants_match_native_values() {
    assert_eq!(GAUSS_MAX_ORDER, 61);
    assert_eq!(SET_GAUSS_MAX_ORDER, 61);
    assert_eq!(MULTIPLE_GAUSS_MAX_ORDER, 61);
    assert_eq!(KRONROD_MAX_GAUSS_ORDER, 30);
    assert_eq!(KRONROD_AUTO_INITIAL_ORDER, 7);
    assert_eq!(KRONROD_AUTO_ORDER_STEP, 4);
    assert_eq!(KRONROD_AUTO_MAX_ITERATIONS, 50);
    assert_eq!(DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL, 10000);
    assert_eq!(DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT, 700.0);
    assert_eq!(EXP_SINH_MIN_TRANSFORM_ARGUMENT, -30.0);
}

#[test]
fn config_defaults_use_the_constants() {
    assert_eq!(
        KronrodConfig::default().nb_gauss_points,
        KRONROD_AUTO_INITIAL_ORDER
    );
    assert_eq!(
        DoubleExpConfig::default().max_points_per_level,
        DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL
    );
    assert_eq!(
        MultipleConfig::default().max_order,
        MULTIPLE_GAUSS_MAX_ORDER
    );
}

#[test]
fn tanh_sinh_singular_equals_tanh_sinh_with_default_config() {
    let singular = tanh_sinh_singular(&mut InverseSqrt, 0.0, 1.0, 1e-8);
    let config: DoubleExpConfig = DoubleExpConfig::new(1e-8, 100);
    let direct = tanh_sinh(&mut InverseSqrt, 0.0, 1.0, &config);
    assert_eq!(singular.status, direct.status);
    assert_eq!(
        singular.value.map(f64::to_bits),
        direct.value.map(f64::to_bits)
    );
    assert!((singular.value.unwrap() - 2.0).abs() <= 1e-6);
}

#[test]
fn tanh_sinh_singular_rejects_non_positive_tolerance() {
    assert_eq!(
        tanh_sinh_singular(&mut InverseSqrt, 0.0, 1.0, 0.0).status,
        Status::InvalidInput
    );
}

#[test]
fn clamped_gauss_order_matches_the_maximum_order() {
    let mut cubic = |x: f64, value: &mut f64| -> bool {
        *value = x * x * x;
        true
    };
    struct Callback<'a>(&'a mut dyn FnMut(f64, &mut f64) -> bool);
    impl Function for Callback<'_> {
        fn value(&mut self, x: f64, value: &mut f64) -> bool {
            (self.0)(x, value)
        }
    }
    let clamped = gauss(&mut Callback(&mut cubic), 0.0, 1.0, 500);
    let maximum = gauss(&mut Callback(&mut cubic), 0.0, 1.0, GAUSS_MAX_ORDER);
    assert_eq!(
        clamped.value.map(f64::to_bits),
        maximum.value.map(f64::to_bits)
    );
    assert_eq!(clamped.number_of_points, GAUSS_MAX_ORDER);
}
