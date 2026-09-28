// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Supplemental tests for the MathUtils core members made public by the API completion
//! (the `utils` helpers, the iteration-count BracketMinimum overload, the Kronrod and
//! ordered Gauss wrappers, and the glibc-faithful CubeRoot). Not an upstream suite.
use rustycad_tkernel::precision;
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::math_utils::{utils, *};

fn parabola(x: f64, value: &mut f64) -> bool {
    *value = (x - 2.0) * (x - 2.0);
    true
}

#[test]
fn evaluate_finite_classifies_callback_and_value_failures() {
    let (mut value, mut status) = (0.0, Status::Ok);
    assert!(utils::evaluate_finite(
        &mut parabola,
        3.0,
        &mut value,
        &mut status
    ));
    assert_eq!((value, status), (1.0, Status::Ok));
    let mut failing = |_: f64, _: &mut f64| false;
    assert!(!utils::evaluate_finite(
        &mut failing,
        1.0,
        &mut value,
        &mut status
    ));
    assert_eq!(status, Status::CallbackError);
    let mut infinite = |_: f64, v: &mut f64| {
        *v = f64::INFINITY;
        true
    };
    assert!(!utils::evaluate_finite(
        &mut infinite,
        1.0,
        &mut value,
        &mut status
    ));
    assert_eq!((value, status), (f64::INFINITY, Status::NumericalError));
}

#[test]
fn opposite_signs_reads_sign_bits_and_accepts_zero() {
    assert!(utils::opposite_signs(-1.0, 1.0));
    assert!(utils::opposite_signs(0.0, 7.0));
    assert!(utils::opposite_signs(3.0, -0.0));
    assert!(!utils::opposite_signs(2.0, 5.0));
    assert!(!utils::opposite_signs(-2.0, -5.0));
}

#[test]
fn limited_and_is_valid_follow_the_bracket_options() {
    let mut options: MinBracketOptions = MinBracketOptions {
        use_limits: true,
        left_limit: -1.0,
        right_limit: 1.0,
        ..Default::default()
    };
    assert_eq!(utils::limited(5.0, &options), 1.0);
    assert_eq!(utils::limited(-5.0, &options), -1.0);
    assert_eq!(utils::limited(f64::NAN, &options), 1.0);
    assert!(utils::is_valid(&options));
    options.has_fa = true;
    options.fa = f64::NAN;
    assert!(!utils::is_valid(&options));
    options.use_limits = false;
    assert_eq!(utils::limited(5.0, &options), 5.0);
}

#[test]
fn perturbations_widen_to_adjacent_doubles() {
    let (mut minus, mut plus) = (0.0, 0.0);
    assert!(utils::perturbations(1e16, 1e-8, &mut minus, &mut plus));
    assert_eq!((minus, plus), (1e16_f64.next_down(), 1e16_f64.next_up()));
    assert!(!utils::perturbations(1.0, 0.0, &mut minus, &mut plus));
    assert!(!utils::perturbations(
        f64::MAX,
        1e300,
        &mut minus,
        &mut plus
    ));
    assert_eq!(plus, f64::INFINITY);
}

#[test]
fn is_finite_vector_rejects_any_non_finite_component() {
    let mut vector: Vector<'_> = Vector::with_size(3);
    assert!(utils::is_finite_vector(&vector));
    *vector.change_at(1).unwrap() = f64::NAN;
    assert!(!utils::is_finite_vector(&vector));
}

#[test]
fn bracket_minimum_iterations_matches_the_options_overload() {
    let by_count: MinBracketResult = bracket_minimum_iterations(&mut parabola, -3.0, -2.0, 50);
    let options: MinBracketOptions = MinBracketOptions::default();
    let by_options: MinBracketResult = bracket_minimum(&mut parabola, -3.0, -2.0, &options);
    assert_eq!(by_count.status, Status::Ok);
    assert_eq!(
        (by_count.a, by_count.b, by_count.c),
        (by_options.a, by_options.b, by_options.c)
    );
    assert!(by_count.a < 2.0 && 2.0 < by_count.c);
    assert_eq!(
        bracket_minimum_iterations(&mut parabola, -3.0, -2.0, 0).status,
        Status::InvalidInput
    );
}

#[test]
fn kronrod_and_ordered_gauss_wrappers_validate_sizes() {
    let (mut points, mut weights) = (Vector::with_size(15), Vector::with_size(15));
    assert!(get_kronrod_points_and_weights(
        15,
        &mut points,
        &mut weights
    ));
    let sum: f64 = (0..15).map(|i| *weights.at(i).unwrap()).sum();
    assert!((sum - 2.0).abs() <= 1e-14);
    assert!(!get_kronrod_points_and_weights(
        14,
        &mut points,
        &mut weights
    ));
    let (mut gauss, mut gauss_weights) = (Vector::with_size(7), Vector::with_size(7));
    assert!(get_ordered_gauss_points_and_weights(
        7,
        &mut gauss,
        &mut gauss_weights
    ));
    assert!((0..6).all(|i| gauss.at(i).unwrap() < gauss.at(i + 1).unwrap()));
    assert!(!get_ordered_gauss_points_and_weights(
        8,
        &mut gauss,
        &mut gauss_weights
    ));
}

#[test]
fn cube_root_reproduces_glibc_bits() {
    // Regression: Rust's correctly rounded cbrt returns ...164 here; glibc 2.39 returns ...165.
    assert_eq!(cube_root(2e100).to_bits(), 5107287141773884165);
    assert_eq!(cube_root(-8.0), -2.0);
    // glibc's kernel is not exact on perfect cubes: cbrt(27) and cbrt(0.125) are one ulp off.
    assert_eq!(cube_root(27.0), 3.0000000000000004);
    assert_eq!(cube_root(0.125), 0.49999999999999994);
    assert!(cube_root(f64::NAN).is_nan());
    assert_eq!(cube_root(f64::INFINITY), f64::INFINITY);
    assert_eq!(cube_root(-0.0).to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn config_defaults_read_precision() {
    assert_eq!(Config::default().step_min, precision::computational());
    let bounded: BoundedConfig = BoundedConfig::default();
    assert_eq!(
        (bounded.lower_bound, bounded.upper_bound),
        (-precision::infinite(), precision::infinite())
    );
}
