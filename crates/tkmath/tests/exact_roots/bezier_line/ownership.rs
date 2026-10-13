// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn bezier_cancellation_at_every_checkpoint_discards_the_entire_query() {
    let controls = [[0., 1.], [1., -2.], [2., 1.]];
    let mut request = query(&controls);
    request.weights = Some(&[1., 2., 1.]);
    check_cancellation(request);
    request.controls = &[[0., 0.], [1., 0.], [2., 0.]];
    check_cancellation(request);
    request.weights = Some(&[1., -1., 1.]);
    check_cancellation(request);
}
fn check_cancellation(request: BezierLineQuery<'_>) {
    let mut calls = 0;
    let expected = intersect_bezier_line(request, || {
        calls += 1;
        false
    });
    for checkpoint in 1..=calls {
        cancel_at(request, checkpoint);
    }
    assert_eq!(intersect_bezier_line(request, || false), expected);
}
fn cancel_at(request: BezierLineQuery<'_>, checkpoint: usize) {
    let mut calls = 0;
    let result = intersect_bezier_line(request, || {
        calls += 1;
        calls == checkpoint
    });
    assert!(
        matches!(
            result,
            Err(BezierLineError::Root {
                cause: RootIsolationError::Cancelled,
                ..
            })
        ),
        "checkpoint {checkpoint}: {result:?}"
    );
}
#[test]
fn bezier_source_and_result_ownership_survive_success_failure_cancel_and_mutation() {
    let mut controls = vec![[-0., 1.], [1., -2.], [2., 1.]];
    let mut weights = vec![1., 2., 1.];
    let before = source_bits(&controls, &weights);
    let mut request = query(&controls);
    request.weights = Some(&weights);
    let completed = contacts(request);
    let snapshot = completed.clone();
    request.options.max_bisections = 0;
    assert!(intersect_bezier_line(request, || false).is_err());
    assert!(intersect_bezier_line(request, || true).is_err());
    assert_eq!(source_bits(&controls, &weights), before);
    controls.fill([1., 5.]);
    weights.fill(1.);
    assert!(contacts(query(&controls)).roots().is_empty());
    controls.clear();
    assert_eq!(completed, snapshot);
}

fn source_bits(controls: &[[f64; 2]], weights: &[f64]) -> Vec<u64> {
    controls
        .iter()
        .flat_map(|point| point.iter())
        .chain(weights)
        .map(|value| value.to_bits())
        .collect()
}
