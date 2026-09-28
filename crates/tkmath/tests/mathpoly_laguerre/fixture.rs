/// Test fixture for MathPoly Laguerre solver tests (`MathPoly_LaguerreTest`).
const THE_TOL: f64 = 1.0e-10;

/// `EXPECT_NEAR(actual, expected, tolerance)`.
fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} within {tolerance}"
    );
}

/// `MathPoly::Laguerre` with the native default tolerance and iteration limit.
fn solve(theCoeffs: &[f64], theDegree: i32) -> PolyResult {
    laguerre(theCoeffs, theDegree, LAGUERRE_TOLERANCE, LAGUERRE_MAX_ITERATIONS)
}

/// Helper to verify a root.
fn EvaluatePoly(theCoeffs: &[f64], theDegree: usize, theX: f64) -> f64 {
    let mut aResult: f64 = theCoeffs[theDegree];
    for i in (0..theDegree).rev() {
        aResult = aResult * theX + theCoeffs[i];
    }
    aResult
}

fn EvaluateComplexPoly(theCoeffs: &[f64], theDegree: usize, theX: Complex) -> Complex {
    let mut aResult: Complex = Complex::new(theCoeffs[theDegree], 0.0);
    for anIndex in (0..theDegree).rev() {
        aResult = aResult * theX + theCoeffs[anIndex];
    }
    aResult
}

fn ComplexEvaluationRoundoff(theCoeffs: &[f64], theDegree: usize, theX: Complex) -> f64 {
    let mut aMagnitude: f64 = theCoeffs[theDegree].abs();
    let anAbsX: f64 = theX.abs();
    for anIndex in (0..theDegree).rev() {
        aMagnitude = aMagnitude * anAbsX + theCoeffs[anIndex].abs();
    }
    let anOperationRoundoff: f64 = theDegree as f64 * f64::EPSILON;
    anOperationRoundoff / (1.0 - anOperationRoundoff) * aMagnitude
}

/// `EXPECT_TRUE(|v-1| < tol || ... || |v-count| < tol)` over the integer roots 1..=count.
fn is_expected_integer_root(aVal: f64, count: usize) -> bool {
    (1..=count).any(|k| (aVal - k as f64).abs() < THE_TOL)
}
