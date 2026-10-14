// SPDX-License-Identifier: GPL-2.0-only
//! Refused policy bits remain original input; no normalization or budget retry is permitted.
use super::super::{request_replay::encode_owned_prism_request, test_fixture};
use super::{operation_tests::CompleteObserver, *};
#[test]
fn refused_nan_policy_retains_payload_bits_and_actual_local_failure() {
    let mut request = test_fixture::request();
    let nan = f64::from_bits(0x7ff8_0000_0000_0042);
    let total = &mut request.policy.publication.measures.scalars.total_area;
    total.absolute_error_limit = nan;
    let expected = encode_owned_prism_request(&request);
    with_owned_prism_operation(request, &mut CompleteObserver::default(), |result| {
        assert_nan_refusal(result.err().unwrap(), &expected, nan.to_bits());
    });
}
#[test]
fn signed_zero_input_is_kept_even_when_accepted_geometry_is_equivalent() {
    let mut request = test_fixture::request();
    request.extrusion[0] = -0.0;
    let expected = encode_owned_prism_request(&request);
    with_owned_prism_operation(request, &mut CompleteObserver::default(), |result| {
        let accepted = result.unwrap_or_else(|_| panic!("signed-zero extrusion is admitted"));
        assert_eq!(
            accepted.request().extrusion[0].to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(encode_owned_prism_request(accepted.request()), expected);
        assert_eq!(accepted.names().plan().version(), 2);
    });
}
#[test]
fn numerical_exhaustion_cannot_become_a_partial_success_or_widened_budget() {
    let mut request = test_fixture::request();
    let side = &mut request.policy.publication.measures.scalars.side_area;
    side.max_subdivisions = 1;
    side.absolute_error_limit = 1e-15;
    let expected = encode_owned_prism_request(&request);
    with_owned_prism_operation(request, &mut CompleteObserver::default(), |result| {
        let failure = result.err().unwrap();
        assert_eq!(encode_owned_prism_request(failure.request()), expected);
        assert!(matches!(failure.cause(), ScopedPrismCause::Publication(_)));
        assert!(failure.callback().is_none());
    });
}

fn assert_nan_refusal(failure: ScopedPrismFailure<'_>, expected: &[u8], nan_bits: u64) {
    assert!(failure.callback().is_none());
    assert_eq!(encode_owned_prism_request(failure.request()), expected);
    let ScopedPrismCause::Publication(packet) = failure.into_parts().1 else {
        panic!("actual measures refusal")
    };
    use crate::sketch_region_prism::publication::PrismPublicationPhase;
    assert_eq!(packet.failure().phase, PrismPublicationPhase::Measures);
    let options = packet.failure().requested.measures.scalars;
    assert_eq!(options.total_area.absolute_error_limit.to_bits(), nan_bits);
}
