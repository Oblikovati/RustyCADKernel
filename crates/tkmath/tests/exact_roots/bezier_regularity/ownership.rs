// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn public_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<BezierRegularityQuery<'static>>();
    assert_send_sync::<BezierRegularityOptions>();
    assert_send_sync::<BezierRegularitySource>();
    assert_send_sync::<BezierRegularityDecision>();
    assert_send_sync::<BezierRegularityReport>();
    assert_send_sync::<BezierRegularityCompletion>();
    assert_send_sync::<BezierRegularityStage>();
    assert_send_sync::<BezierRegularityCause>();
    assert_send_sync::<BezierRegularityFailure>();
}
#[test]
fn success_owns_all_inputs_and_placement_without_mutating_them() {
    let mut poles = CUBIC;
    let mut weights = [1.; 4];
    let mut request = query(&poles);
    request.weights = Some(&weights);
    request.placement.origin = [3., 4., 5.];
    request.placement.scale = 2.;
    let result = certify_bezier_regularity(request, || false).unwrap();
    assert_eq!(poles, CUBIC);
    assert_eq!(weights, [1.; 4]);
    assert_eq!(result.source().placement().origin, [3., 4., 5.]);
    assert_eq!(result.source().placement().scale, 2.);
    poles.fill([99.; 3]);
    weights.fill(0.);
    assert_eq!(result.source().poles(), CUBIC);
    assert_eq!(result.source().weights(), Some([1.; 4].as_slice()));
    assert_eq!(result.source().interval(), [0., 1.]);
    assert_eq!(result.clone(), result);
}
#[test]
fn decline_and_cancellation_own_sources_after_mutation() {
    let mut poles = [[0.; 3], [1., 0., 0.]];
    let mut weights = [1., -1.];
    let mut request = query(&poles);
    request.weights = Some(&weights);
    let declined = certify_bezier_regularity(request, || false).unwrap_err();
    let cancelled = certify_bezier_regularity(request, || true).unwrap_err();
    assert_eq!(poles, [[0.; 3], [1., 0., 0.]]);
    assert_eq!(weights, [1., -1.]);
    poles.fill([99.; 3]);
    weights.fill(5.);
    for error in [declined, cancelled] {
        assert_eq!(error.source().poles(), [[0.; 3], [1., 0., 0.]]);
        assert_eq!(error.source().weights(), Some([1., -1.].as_slice()));
    }
}
pub(super) fn replay_bytes() -> Vec<u8> {
    let mut text = String::new();
    for poles in [&CUBIC[..], &IRRATIONAL[..], &[[0.; 3]][..]] {
        text.push_str(&format!(
            "{:?}\n",
            certify_bezier_regularity(query(poles), || false)
        ));
    }
    let mut request = query(&IRRATIONAL);
    request.options.roots.max_bisections = 1;
    text.push_str(&format!(
        "{:?}\n",
        certify_bezier_regularity(request, || false)
    ));
    text.push_str(&format!(
        "{:?}\n",
        certify_bezier_regularity(request, || true)
    ));
    text.into_bytes()
}
#[test]
fn threaded_replay_matches_serial_bytes_without_prior_operation_dependence() {
    let expected = replay_bytes();
    std::thread::scope(|scope| {
        let joins: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| (0..8).map(|_| replay_bytes()).collect::<Vec<_>>()))
            .collect();
        for join in joins {
            for bytes in join.join().unwrap() {
                assert_eq!(bytes, expected);
            }
        }
    });
}
