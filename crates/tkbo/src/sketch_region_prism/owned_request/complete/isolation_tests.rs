// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    CallerSourceRevision, OwnedPolynomialPrismRequest, names::*,
    request_replay::encode_owned_prism_request, test_fixture,
};
use super::{operation_tests::*, *};
fn run(request: OwnedPolynomialPrismRequest) -> OwnedCompletePrism {
    with_owned_prism_operation(request, &mut CompleteObserver::default(), |result| {
        result.unwrap_or_else(|_| panic!("fixture must complete through actual production path"))
    })
}
#[test]
fn native_poles_traversal_and_extrusion_reversals_keep_distinct_directed_names() {
    for bits in 0..8 {
        let mut request = test_fixture::request();
        if bits & 1 != 0 {
            request.sources[0].world_poles.reverse();
        }
        request.sources[0].reversed_traversal = bits & 2 != 0;
        if bits & 4 != 0 {
            request.extrusion[2] = -2.;
        }
        assert_reversal(request);
    }
}
fn assert_reversal(request: OwnedPolynomialPrismRequest) {
    let expected = encode_owned_prism_request(&request);
    let result = run(request);
    assert_eq!(encode_owned_prism_request(result.request()), expected);
    assert_eq!(result.names().member_bindings().len(), 13);
    assert_eq!(result.names().use_bindings().len(), 26);
    assert_eq!(
        result.names().plan().identities().start,
        CallerDirectedUseId(901)
    );
    assert_eq!(
        result.names().plan().identities().end,
        CallerDirectedUseId(19)
    );
    let [lo, hi] = result.volume().enclosure();
    assert!(lo <= 24. / 5. && hi >= 24. / 5.);
}
#[test]
fn caller_revision_changes_whole_envelope_without_rewriting_semantic_child_names() {
    let request = test_fixture::request();
    let first = run(request.clone());
    let mut revised = request;
    revised.sources[0].revision = CallerSourceRevision(72);
    let second = run(revised);
    assert_eq!(first.names().replay_bytes(), second.names().replay_bytes());
    assert_eq!(
        first.publication().replay_bytes(),
        second.publication().replay_bytes()
    );
    assert_ne!(first.replay_bytes(), second.replay_bytes());
}
#[test]
fn same_revision_with_changed_coefficients_cannot_reuse_accepted_evidence() {
    let request = test_fixture::request();
    let first = run(request.clone());
    let mut edited = request;
    for pole in &mut edited.sources[0].world_poles {
        pole[0] *= 2.;
        pole[1] *= 2.;
    }
    let second = run(edited);
    assert_eq!(
        first.request().sources[0].revision,
        second.request().sources[0].revision
    );
    assert_ne!(first.names().replay_bytes(), second.names().replay_bytes());
    assert_ne!(first.body().replay_bytes(), second.body().replay_bytes());
    let [lo, hi] = second.volume().enclosure();
    assert!(lo <= 96. / 5. && hi >= 96. / 5.);
}
#[test]
fn caller_namespace_is_not_confused_by_identical_internal_dictionary_tokens() {
    let request = test_fixture::request();
    let first = run(request.clone());
    let mut other = request;
    other.operation = CallerOperationId(19);
    other.sources[0].source = CallerBoundaryId(257);
    let second = run(other);
    assert_eq!(
        first.publication().replay_bytes(),
        second.publication().replay_bytes()
    );
    assert_ne!(first.names().replay_bytes(), second.names().replay_bytes());
    assert_ne!(first.body().replay_bytes(), second.body().replay_bytes());
    assert_ne!(first.replay_bytes(), second.replay_bytes());
}
#[test]
fn owned_success_survives_original_request_mutation_and_fresh_reads() {
    let mut request = test_fixture::request();
    let result = run(request.clone());
    let before = result.replay_bytes();
    request.sources[0].world_poles[1][0] = 999.;
    request.sources.clear();
    assert_eq!(result.request().sources[0].world_poles[1][0], 4.);
    assert_eq!(result.body().members().len(), 13);
    assert_eq!(result.replay_bytes(), before);
    assert!(result.body().binds(result.publication(), result.names()));
}
