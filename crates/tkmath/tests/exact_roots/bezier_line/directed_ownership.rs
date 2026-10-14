// SPDX-License-Identifier: GPL-2.0-only
use super::incidences::request;
use super::*;

#[test]
fn directed_public_types_are_send_and_sync() {
    fn assert_threads<T: Send + Sync>() {}
    assert_threads::<DirectedBezierIncidenceQuery<'static>>();
    assert_threads::<DirectedBezierPosition>();
    assert_threads::<DirectedBezierIncidenceReport>();
    assert_threads::<DirectedBezierIncidenceFailure>();
    assert_threads::<DirectedBezierIncidenceCause>();
}
#[test]
fn source_mutation_cannot_change_success_failure_or_cancellation() {
    let mut controls = vec![[0., -1.], [1., 2.]];
    let mut weights = vec![2., 1.];
    let mut query = request(&controls);
    query.curve.weights = Some(&weights);
    let success = super::directed_incidences::directed(query);
    query.max_world_error = 1e-20;
    let failure =
        certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || false)
            .unwrap_err();
    let cancelled =
        certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || true)
            .unwrap_err();
    let before = format!("{success:?}{failure:?}{cancelled:?}");
    assert_eq!(controls, [[0., -1.], [1., 2.]]);
    assert_eq!(weights, [2., 1.]);
    controls.fill([99., -99.]);
    weights.clear();
    assert_eq!(format!("{success:?}{failure:?}{cancelled:?}"), before);
    assert_eq!(success.incidence().source().weights(), Some(&[2., 1.][..]));
}
pub(super) fn replay_bytes() -> Vec<u8> {
    let controls = [[0., -9.], [1., 13.], [2., -13.], [3., 9.]];
    let mut query = request(&controls);
    query.curve.line = [[1., 0.], [2., 0.]];
    let success = super::directed_incidences::directed(query);
    let cancelled =
        certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || true)
            .unwrap_err();
    format!("{success:?}\n{cancelled:?}\n").into_bytes()
}
#[test]
fn serial_and_concurrent_queries_replay_identical_bytes_after_cancellation() {
    let expected = replay_bytes();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    for _ in 0..16 {
                        assert_eq!(replay_bytes(), expected);
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    });
}
#[test]
fn cancellation_at_each_observed_checkpoint_never_publishes_partial_success() {
    let mut incidence = request(&[[0., -1.], [1., 2.]]);
    incidence.curve.weights = Some(&[2., 1.]);
    let query = DirectedBezierIncidenceQuery::v1(incidence);
    let mut calls = 0;
    let expected = certify_directed_bezier_incidences(query, || {
        calls += 1;
        false
    })
    .unwrap();
    for cutoff in 1..=calls {
        verify_cancel_checkpoint(query, cutoff);
    }
    assert_eq!(
        certify_directed_bezier_incidences(query, || false).unwrap(),
        expected
    );
}
fn verify_cancel_checkpoint(query: DirectedBezierIncidenceQuery<'_>, cutoff: usize) {
    let mut visited = 0;
    let failure = certify_directed_bezier_incidences(query, || {
        visited += 1;
        visited >= cutoff
    })
    .unwrap_err();
    assert_eq!(
        failure.completion(),
        BezierIncidenceCompletion::Cancelled,
        "checkpoint {cutoff}"
    );
}

#[test]
fn irrational_refinement_cancels_early_midway_and_at_final_checkpoint() {
    let query = DirectedBezierIncidenceQuery::v1(request(&[[0., 1.], [1., -2.], [2., 1.]]));
    let mut calls = 0;
    let expected = certify_directed_bezier_incidences(query, || {
        calls += 1;
        false
    })
    .unwrap();
    for cutoff in [1, calls / 2, calls] {
        verify_cancel_checkpoint(query, cutoff);
    }
    assert_eq!(
        certify_directed_bezier_incidences(query, || false).unwrap(),
        expected
    );
}
