// SPDX-License-Identifier: GPL-2.0-only
use super::*;

fn full_receipt(report: &CompletePrismMeasureInspection) -> String {
    format!(
        "{{\"operation\":\"martensite.prism-complete-measures\",\"behavior\":1,\"side\":{:?},\"total\":{:?},\"work\":{},\"winding\":{},\"body\":\"incomplete\"}}",
        report.measures.side.enclosure().map(f64::to_bits),
        report.measures.total.enclosure().map(f64::to_bits),
        report.measures.side.work().subdivisions,
        report.binding.region.selected_winding
    )
}
fn fresh_full_receipt() -> String {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    full_receipt(&full_report(&bridge, &shapes, &receipt, fast_options()))
}
#[test]
fn complete_prism_measure_process_receipt() {
    let receipt = fresh_full_receipt();
    assert!(receipt.contains("\"body\":\"incomplete\""));
    println!("{receipt}");
}
#[test]
fn complete_independent_workers_match_serial_replay() {
    let expected = fresh_full_receipt();
    let workers: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(fresh_full_receipt))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
}
#[test]
fn final_callback_original_copy_and_side_edits_cannot_publish_stale_measures() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut polls = 0;
    full_attempt(&bridge, &shapes, &receipt, fast_options(), || {
        polls += 1;
        false
    })
    .ok()
    .unwrap();
    for target in 0..3 {
        assert_final_edit_is_refused(target, polls);
        assert_final_edit_is_refused(target, polls - 6);
    }
}
fn assert_final_edit_is_refused(target: usize, last_poll: usize) {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut polls = 0;
    let failure = full_attempt(&bridge, &shapes, &receipt, fast_options(), || {
        polls += 1;
        if polls == last_poll {
            edit_target(target, &shapes, &receipt);
        }
        false
    })
    .err()
    .unwrap();
    assert!(matches!(failure, CompletePrismMeasureFailure::Binding(_)));
}
#[test]
fn complete_cancellation_at_initial_and_final_poll_remains_retryable() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let (report, polls) = report_with_poll_count(&bridge, &shapes, &receipt);
    for cutoff in [1, polls - 6, polls - 5, polls] {
        assert_cancel_cutoff(&bridge, &shapes, &receipt, cutoff, polls);
    }
    assert_eq!(
        full_receipt(&report),
        full_receipt(&full_report(&bridge, &shapes, &receipt, fast_options()))
    );
}
fn assert_cancel_stage(cutoff: usize, final_poll: usize, error: CompletePrismMeasureFailure) {
    // v1 total polls once before, three components, once after; outer binding polls once.
    match final_poll - cutoff {
        6 => assert!(
            matches!(error, CompletePrismMeasureFailure::Side { cause, .. }
            if matches!(*cause.cause, PolynomialSideFailure::Cancelled))
        ),
        5 => assert!(matches!(
            error,
            CompletePrismMeasureFailure::Total(EnclosureSumError {
                cause: EnclosureSumCause::Cancelled,
                ..
            })
        )),
        _ => assert!(matches!(
            error,
            CompletePrismMeasureFailure::Binding(cause) if matches!(*cause, PrismBindingFailure::Cancelled)
        )),
    }
}

fn report_with_poll_count(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) -> (CompletePrismMeasureInspection, usize) {
    let mut polls = 0;
    let report = full_attempt(bridge, shapes, receipt, fast_options(), || {
        polls += 1;
        false
    })
    .ok()
    .unwrap();
    (report, polls)
}
fn assert_cancel_cutoff(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    cutoff: usize,
    polls: usize,
) {
    let mut current = 0;
    let error = full_attempt(bridge, shapes, receipt, fast_options(), || {
        current += 1;
        current == cutoff
    })
    .err()
    .unwrap();
    assert_cancel_stage(cutoff, polls, error);
}
