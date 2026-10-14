// SPDX-License-Identifier: GPL-2.0-only
//! Pin callback positions and nested cancellation causes before borrowing the evaluator.
use super::*;

#[test]
fn legacy_orientation_callback_and_cause_receipt() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let polls = mutation_tests::count_polls(&bridge, &shapes, &receipt);
    let rows: Vec<_> = [1, polls - 1, polls]
        .map(|cutoff| cancellation_row(&bridge, &shapes, &receipt, cutoff))
        .into_iter()
        .collect();
    let encoded = format!("{{\"legacy_orientation_polls\":{polls},\"cancellations\":{rows:?}}}");
    assert_eq!(encoded, include_str!("callbacks-v1.json").trim());
    println!("{encoded}");
}

fn cancellation_row(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    cutoff: usize,
) -> String {
    let mut polls = 0;
    let error = attempt(bridge, shapes, receipt, options(), || {
        polls += 1;
        polls == cutoff
    })
    .err()
    .unwrap();
    assert_eq!(error.requested, options());
    error_row(cutoff, polls, error)
}

fn error_row(cutoff: usize, polls: usize, error: PrismOrientationFailure) -> String {
    format!(
        "{cutoff}/{polls}/{:?}/{}/{}",
        error.phase,
        error.original.is_some(),
        cancellation_cause(error.cause)
    )
}

fn cancellation_cause(cause: PrismOrientationCause) -> String {
    match cause {
        PrismOrientationCause::Binding(cause) => {
            assert!(matches!(*cause, PrismBindingFailure::Cancelled));
            "binding.cancelled".into()
        }
        PrismOrientationCause::Embedding(cause) => embedding_cancel(*cause),
        _ => panic!("cancellation cause; expected nested binding cancellation"),
    }
}

fn embedding_cancel(cause: PrismEmbeddingFailure) -> String {
    let PrismEmbeddingCause::Binding(binding) = cause.cause else {
        panic!("embedding cancellation cause; expected binding cancellation")
    };
    assert!(matches!(*binding, PrismBindingFailure::Cancelled));
    format!(
        "embedding.{:?}.binding.cancelled/{}",
        cause.phase,
        cause.original.is_some()
    )
}
