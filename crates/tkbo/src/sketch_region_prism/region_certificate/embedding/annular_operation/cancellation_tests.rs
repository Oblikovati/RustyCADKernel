// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{annular_binding::tests::with_actual, two_loop::TwoLoopRefusalClass};
use super::tests::{binding, options};
use super::*;

fn observed_checkpoints() -> usize {
    with_actual(|request, region, receipt| {
        let bound = binding(request, region, receipt);
        count_checkpoints(request, receipt, bound)
    })
}
fn count_checkpoints(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    bound: AnnularPrismBinding,
) -> usize {
    let mut count = 0;
    assert!(
        inspect_bound_annular_embedding(request, receipt, bound, options(), || {
            count += 1;
            false
        })
        .is_ok()
    );
    count
}
#[test]
fn every_actual_external_cancellation_cutoff_prevents_embedding_issuance() {
    use super::super::super::annular_binding::tests::fresh_region;
    with_actual(|request, region, receipt| {
        let count = count_checkpoints(request, receipt, binding(request, region, receipt));
        assert!(count > 3);
        for stop in 1..=count {
            assert_cancelled_at(
                request,
                receipt,
                binding(request, fresh_region(request), receipt),
                stop,
            );
        }
        let final_count = count_checkpoints(
            request,
            receipt,
            binding(request, fresh_region(request), receipt),
        );
        assert_eq!(final_count, count);
    });
}
fn assert_cancelled_at(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    bound: AnnularPrismBinding,
    stop: usize,
) {
    let mut count = 0;
    let failure = inspect_bound_annular_embedding(request, receipt, bound, options(), || {
        count += 1;
        count == stop
    })
    .err()
    .unwrap();
    assert_eq!(count, stop);
    assert_eq!(failure.classification(), TwoLoopRefusalClass::Cancelled);
    assert_eq!(failure.operation, request.identities.operation);
}
#[test]
fn final_callback_mutation_refuses_the_stale_complete_proposal() {
    let final_ordinal = observed_checkpoints();
    with_actual(|request, region, receipt| {
        let bound = binding(request, region, receipt);
        let edge = rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap();
        let mut count = 0;
        let failure = inspect_bound_annular_embedding(request, receipt, bound, options(), || {
            count += 1;
            if count == final_ordinal {
                crate::sketch_region_prism::isolation_tests::set_pole(&edge, 5.);
            }
            false
        })
        .err()
        .unwrap();
        assert_eq!(count, final_ordinal);
        assert_eq!(failure.phase, AnnularEmbeddingPhase::Commit);
        assert!(matches!(failure.cause, AnnularEmbeddingCause::Binding(_)));
    });
}
#[test]
fn source_mutation_before_entry_refuses_before_regularity_work() {
    with_actual(|request, region, receipt| {
        let bound = binding(request, region, receipt);
        let edge = rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap();
        crate::sketch_region_prism::isolation_tests::set_pole(&edge, 5.);
        let mut count = 0;
        let failure = inspect_bound_annular_embedding(request, receipt, bound, options(), || {
            count += 1;
            false
        })
        .err()
        .unwrap();
        assert_eq!(count, 1);
        assert_eq!(failure.phase, AnnularEmbeddingPhase::CurrentBinding);
        assert!(matches!(failure.cause, AnnularEmbeddingCause::Binding(_)));
    });
}
