//! GoogleTest assertion semantics shared by the translated TKG2d suites.
#![allow(non_snake_case, dead_code)]

/// GoogleTest `EXPECT_DOUBLE_EQ`: at most four ULPs apart, never NaN.
pub fn EXPECT_DOUBLE_EQ(a: f64, b: f64) {
    assert!(
        !a.is_nan() && !b.is_nan(),
        "EXPECT_DOUBLE_EQ on NaN: {a} vs {b}"
    );
    assert!(
        ordered_bits(a).abs_diff(ordered_bits(b)) <= 4,
        "EXPECT_DOUBLE_EQ: {a} != {b}"
    );
}

/// GoogleTest `EXPECT_NEAR`: `|a - b| <= tolerance`.
pub fn EXPECT_NEAR(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() <= tolerance,
        "EXPECT_NEAR: |{a} - {b}| > {tolerance}"
    );
}

// Maps doubles to integers whose order matches the numeric order (GoogleTest's
// sign-and-magnitude to biased conversion).
fn ordered_bits(value: f64) -> u64 {
    let bits: u64 = value.to_bits();
    if bits >> 63 != 0 {
        (!bits).wrapping_add(1)
    } else {
        bits | (1 << 63)
    }
}

/// GoogleTest `EXPECT_THROW(statement, Type)`: the statement fails with the native
/// exception `exception`.
pub fn EXPECT_THROW<T>(result: Result<T, rustycad_tkernel::standard::Failure>, exception: &str) {
    match result {
        Ok(_) => panic!("EXPECT_THROW: no exception; expected {exception}"),
        Err(error) => assert_eq!(
            error.exception_type(),
            exception,
            "EXPECT_THROW: wrong exception; expected {exception}"
        ),
    }
}

/// The `CreateUniformParams` helper of the Geom2dGridEval suites: `theNbPoints`
/// parameters from `theFirst` to `theLast`, indexed from 1.
pub fn CreateUniformParams(
    theFirst: f64,
    theLast: f64,
    theNbPoints: i32,
) -> rustycad_tkernel::ncollection::Array1<'static, f64> {
    let mut aParams = rustycad_tkernel::ncollection::Array1::with_bounds(1, theNbPoints).unwrap();
    let aStep: f64 = (theLast - theFirst) / f64::from(theNbPoints - 1);
    for i in 1..=theNbPoints {
        aParams[i] = theFirst + f64::from(i - 1) * aStep;
    }
    aParams
}
