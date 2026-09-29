//! Supplemental MathPoly::Utils and LongDouble behavior pinned against native OCCT.
use rustycad_tkmath::math_poly::utils::{self, LongCoefficients};
use rustycad_tkmath::math_poly::{LongDouble, PolyResult, cubic};
use rustycad_tkmath::math_utils::Status;

fn long(values: &[f64]) -> LongCoefficients {
    let mut c: LongCoefficients = utils::long_coefficients();
    for (slot, &v) in c.iter_mut().zip(values) {
        *slot = LongDouble::new(v);
    }
    c
}

#[test]
fn long_double_follows_the_x87_format() {
    let one: LongDouble = LongDouble::from_bits(0x3fff_8000_0000_0000_0000);
    assert_eq!(one, LongDouble::new(1.0));
    assert_eq!(one.to_bits(), 0x3fff_8000_0000_0000_0000);
    assert_eq!(LongDouble::epsilon().to_bits(), 0x3fc0_8000_0000_0000_0000);
    // 0.1 keeps 11 more significand bits than the double, and narrowing rounds back.
    let tenth: LongDouble = LongDouble::new(1.0) / LongDouble::new(10.0);
    assert_ne!(tenth, LongDouble::new(0.1));
    assert_eq!(tenth.narrow(), 0.1);
    assert!(LongDouble::largest().narrow().is_infinite());
    let nan: LongDouble = LongDouble::ZERO / LongDouble::ZERO;
    assert!(nan.is_nan() && !nan.is_finite());
    assert!(nan.max(one).is_nan(), "std::max keeps a NaN first argument");
    assert_eq!(one.max(nan), one);
    assert_eq!(LongDouble::new(-1.5).floor(), LongDouble::new(-2.0));
    assert_eq!(LongDouble::new(-1.5).ceil_integer(), -1);
    assert_eq!(LongDouble::from_usize(7).to_usize(), 7);
    assert_eq!(LongDouble::from(2.0).scale(3), LongDouble::new(16.0));
    assert!(LongDouble::new(1.0).copysign(LongDouble::new(-0.0)) < LongDouble::ZERO);
    assert!(LongDouble::infinity().is_infinite());
    assert!(LongDouble::smallest_normal() > LongDouble::ZERO);
}

#[test]
fn validate_and_to_double_checks() {
    assert!(utils::validate(Some(&[1.0, 2.0]), 1, 1.0));
    assert!(!utils::validate(None, 1, 1.0));
    assert!(
        !utils::validate(Some(&[1.0]), 1, 1.0),
        "slice shorter than degree + 1"
    );
    assert!(!utils::validate(Some(&[1.0, 2.0]), -1, 1.0));
    assert!(!utils::validate(Some(&[1.0; 22]), 21, 1.0));
    assert!(!utils::validate(Some(&[1.0, 2.0]), 1, f64::NAN));
    assert!(!utils::validate(Some(&[1.0, 2.0]), 1, 0.0));
    assert!(!utils::validate(Some(&[1.0, f64::INFINITY]), 1, 1.0));
    assert_eq!(utils::to_double(LongDouble::new(0.5)), Some(0.5));
    assert_eq!(utils::to_double(LongDouble::largest()), None);
    assert_eq!(utils::to_double(LongDouble::infinity()), None);
}

#[test]
fn scaling_evaluation_and_bounds() {
    assert!(
        utils::scale_polynomial(&[1.0, 2.0, 0.0], 2)
            .unwrap()
            .is_none()
    );
    let (scaled, power) = utils::scale_polynomial(&[4.0, 0.0, 1.0], 2)
        .unwrap()
        .unwrap();
    // x = 2y turns 4 + x^2 into 4 + 4y^2, normalized by 2^-2 to 1 + y^2.
    assert_eq!(power, 1);
    assert_eq!(scaled[0], LongDouble::new(1.0));
    assert_eq!(scaled[2], LongDouble::new(1.0));
    let c: LongCoefficients = long(&[-2.0, 0.0, 1.0]);
    let (value, derivative, magnitude) = utils::evaluate(&c, 2, LongDouble::new(3.0)).unwrap();
    assert_eq!(
        (value, derivative, magnitude),
        (
            LongDouble::new(7.0),
            LongDouble::new(6.0),
            LongDouble::new(11.0)
        )
    );
    // At x = 1 the value is -1 and the magnitude bound is 3.
    assert!(!utils::has_small_residual(&c, 2, LongDouble::new(1.0), LongDouble::new(0.3)).unwrap());
    assert!(utils::has_small_residual(&c, 2, LongDouble::new(1.0), LongDouble::new(0.34)).unwrap());
    assert_eq!(utils::root_bound(&c, 2).unwrap(), LongDouble::new(3.0));
    assert_eq!(
        utils::distinct_root_residual_tolerance(),
        LongDouble::new(64.0) * LongDouble::epsilon()
    );
    assert_eq!(
        utils::root_multiplicity_tolerance(),
        LongDouble::new(32768.0) * LongDouble::epsilon()
    );
}

#[test]
fn distinct_real_isolation() {
    let mut roots: [LongDouble; 4] = [LongDouble::ZERO; 4];
    let mut count: usize = 0;
    utils::append_unique(&mut roots, &mut count, LongDouble::new(1.0)).unwrap();
    utils::append_unique(
        &mut roots,
        &mut count,
        LongDouble::new(1.0) + LongDouble::epsilon(),
    )
    .unwrap();
    assert_eq!(count, 1, "within 128 LDBL_EPSILON");
    let c: LongCoefficients = long(&[-6.0, 11.0, -6.0, 1.0]);
    let (mut found, mut nb) = ([LongDouble::ZERO; 4], 0_usize);
    assert!(utils::solve_distinct_real(&c, 3, &mut found, &mut nb).unwrap());
    let mut values: Vec<f64> = found[..nb].iter().map(|r| r.narrow()).collect();
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![1.0, 2.0, 3.0]);
    assert!(
        !utils::solve_distinct_real(&c, 0, &mut found, &mut nb).unwrap(),
        "degree 0 never terminates natively"
    );
    assert!(!utils::solve_distinct_real(&long(&[1.0, 0.0]), 1, &mut found, &mut nb).unwrap());
}

#[test]
fn a_fifth_distinct_root_is_refused_instead_of_overrunning_the_buffer() {
    let mut roots: [LongDouble; 4] = [1.0, 2.0, 3.0, 4.0].map(LongDouble::new);
    let mut count: usize = 4;
    // A root already stored needs no slot, as natively.
    utils::append_unique(&mut roots, &mut count, LongDouble::new(2.0)).unwrap();
    let failure = utils::append_unique(&mut roots, &mut count, LongDouble::new(9.0))
        .err()
        .unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(
        failure
            .message()
            .contains("distinct roots 5 of a 4-slot buffer"),
        "{}",
        failure.message()
    );
    assert_eq!(count, 4);
    let mut too_many: usize = 5;
    assert!(utils::append_unique(&mut roots, &mut too_many, LongDouble::new(9.0)).is_err());
}

#[test]
fn solve_direct_refuses_five_distinct_real_roots() {
    // (x-1)(x-2)(x-3)(x-4)(x-5): natively the fifth root overruns the four-root buffer.
    let c: [f64; 6] = [-120.0, 274.0, -225.0, 85.0, -15.0, 1.0];
    let failure = utils::solve_direct(&c, 5).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    // Fewer distinct real roots fit: x^5 - 1 has one.
    let one: PolyResult = utils::solve_direct(&[-1.0, 0.0, 0.0, 0.0, 0.0, 1.0], 5).unwrap();
    assert_eq!(
        (one.status, one.nb_roots, one.roots[0]),
        (Status::Ok, 1, 1.0)
    );
    let mut roots = [LongDouble::ZERO; 4];
    let mut count: usize = 0;
    let beyond = utils::long_coefficients();
    assert!(utils::solve_distinct_real(&beyond, 21, &mut roots, &mut count).is_err());
}

#[test]
fn long_coefficient_helpers_refuse_degrees_outside_the_array() {
    let c: LongCoefficients = long(&[1.0, 2.0]);
    for degree in [-1, 21] {
        let failure = utils::evaluate(&c, degree, LongDouble::new(1.0))
            .err()
            .unwrap();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        assert!(
            failure.message().contains(&format!("degree {degree}")),
            "{}",
            failure.message()
        );
        assert!(utils::root_bound(&c, degree).is_err());
        assert!(
            utils::has_small_residual(&c, degree, LongDouble::ZERO, LongDouble::new(1.0)).is_err()
        );
    }
    let failure = utils::scale_polynomial(&[1.0, 2.0], 3).err().unwrap();
    assert!(
        failure.message().contains("degree 3 with 2 coefficients"),
        "{}",
        failure.message()
    );
}

#[test]
fn solve_direct_is_the_direct_solvers_engine() {
    let direct: PolyResult = utils::solve_direct(&[-6.0, 11.0, -6.0, 1.0], 3).unwrap();
    let wrapped: PolyResult = cubic(1.0, -6.0, 11.0, -6.0);
    assert_eq!(direct.roots, wrapped.roots);
    assert_eq!(direct.status, Status::Ok);
    assert_eq!(
        utils::solve_direct(&[1.0, 0.0, 0.0], 2).unwrap().status,
        Status::NoSolution
    );
    assert_eq!(
        utils::solve_direct(&[0.0, 0.0], 1).unwrap().status,
        Status::InfiniteSolutions
    );
    assert_eq!(
        utils::solve_direct(&[1.0, 2.0], 7).unwrap().status,
        Status::InvalidInput
    );
    let double: PolyResult = utils::solve_direct(&[1.0, -2.0, 1.0, 0.0, 0.0], 4).unwrap();
    assert_eq!((double.nb_roots, double.multiplicities[0]), (1, 2));
}
