// SPDX-License-Identifier: GPL-2.0-only
//! Degenerate or unsupported geometry never escapes as a partial named body.
use super::super::{
    OwnedPolynomialPrismRequest, request_replay::encode_owned_prism_request, test_fixture,
};
use super::{operation_tests::CompleteObserver, *};
#[test]
fn collapsed_collinear_tangent_and_nonplanar_inputs_retain_precise_preparation_failure() {
    for kind in 0..4 {
        let request = degenerate(kind);
        let expected = encode_owned_prism_request(&request);
        with_owned_prism_operation(request, &mut CompleteObserver::default(), |result| {
            let failure = result
                .err()
                .unwrap_or_else(|| panic!("degenerate kind {kind} must refuse"));
            assert_eq!(encode_owned_prism_request(failure.request()), expected);
            assert!(failure.callback().is_none());
            assert!(matches!(failure.cause(), ScopedPrismCause::Preparation(_)));
        });
    }
}
fn degenerate(kind: u8) -> OwnedPolynomialPrismRequest {
    let mut request = test_fixture::request();
    match kind {
        0 => request.sources[0].world_poles.fill([0.; 3]),
        1 => request.sources[0].world_poles[2] = [2., 0., 0.],
        2 => request.extrusion = [2., 0., 0.],
        3 => request.sources[0].world_poles[1][2] = 1.,
        _ => panic!("fixture kind {kind} outside 0..4"),
    }
    request
}
#[test]
fn owned_callback_observation_can_outlive_scope_without_borrowed_originals() {
    let request = test_fixture::request();
    let expected = encode_owned_prism_request(&request);
    let mut progress = stopped_progress();
    let (input, stopped) = with_owned_prism_operation(request, &mut progress, |result| {
        let failure = result.err().unwrap();
        (
            encode_owned_prism_request(failure.request()),
            failure.callback().unwrap().clone(),
        )
    });
    assert_eq!(input, expected);
    assert_eq!(stopped.checkpoint.ordinal, 1);
    assert!(matches!(
        stopped.cause,
        CompletePrismCallbackCause::Cancelled
    ));
}

fn stopped_progress() -> CompleteObserver {
    CompleteObserver {
        stop: Some(1),
        ..Default::default()
    }
}
