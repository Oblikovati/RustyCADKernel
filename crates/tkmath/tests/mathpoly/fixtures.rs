const THE_TOLERANCE: f64 = 1.0e-10;

/// `EXPECT_NEAR(actual, expected, tolerance)`.
fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} within {tolerance}"
    );
}

/// Helper to verify a root satisfies the polynomial equation.
fn EvalCubic(theA: f64, theB: f64, theC: f64, theD: f64, theX: f64) -> f64 {
    theA * theX * theX * theX + theB * theX * theX + theC * theX + theD
}

fn EvalQuartic(theA: f64, theB: f64, theC: f64, theD: f64, theE: f64, theX: f64) -> f64 {
    let aX2: f64 = theX * theX;
    theA * aX2 * aX2 + theB * aX2 * theX + theC * aX2 + theD * theX + theE
}
