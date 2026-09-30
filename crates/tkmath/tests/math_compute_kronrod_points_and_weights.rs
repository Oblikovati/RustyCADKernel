//! OCC33048: port of the original order-125 memory regression.
#![allow(non_snake_case)]
use rustycad_tkmath::math::KronrodPointsAndWeights;
#[test]
fn math_ComputeKronrodPointsAndWeights_Test__OCC33048_ComputeWithOrder125() {
    // Rust panics and constructor failures fail this test, matching the native catch-all FAIL.
    let calculation: KronrodPointsAndWeights = KronrodPointsAndWeights::new(125).unwrap();
    assert!(
        calculation.is_done(),
        "Kronrod points and weights calculation should succeed"
    );
    let is_ok: bool = calculation.is_done();
    assert!(is_ok);
}
