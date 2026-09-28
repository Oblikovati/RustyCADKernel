//! Native FunctionSample disables index checks and evaluates its weighted endpoint formula.
use rustycad_tkmath::math::{FunctionSample, FunctionSampling};
#[test]
fn samples_keep_bounds_count_and_unchecked_extrapolation() {
    let sample: FunctionSample = FunctionSample::new(2.0, 6.0, 3);
    assert_eq!(sample.bounds(), [2.0, 6.0]);
    assert_eq!(sample.number_of_points(), 3);
    for (index, expected) in [(0, 0.0), (1, 2.0), (2, 4.0), (3, 6.0), (4, 8.0)] {
        assert_eq!(sample.parameter(index), expected);
    }
    let protocol: &dyn FunctionSampling = &sample;
    assert_eq!(protocol.bounds(), [2.0, 6.0]);
    assert_eq!(protocol.number_of_points(), 3);
    assert_eq!(protocol.parameter(2), 4.0);
}
#[test]
fn singleton_and_reversed_samples_keep_native_arithmetic() {
    let singleton: FunctionSample = FunctionSample::new(2.0, 6.0, 1);
    assert!(singleton.parameter(1).is_nan());
    assert_eq!(singleton.parameter(2), f64::INFINITY);
    let reversed: FunctionSample = FunctionSample::new(6.0, 2.0, 3);
    assert_eq!(reversed.bounds(), [6.0, 2.0]);
    assert_eq!(reversed.parameter(1), 6.0);
    assert_eq!(reversed.parameter(3), 2.0);
}
