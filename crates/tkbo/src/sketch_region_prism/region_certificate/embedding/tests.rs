// SPDX-License-Identifier: GPL-2.0-only
use super::super::tests::{fixture, fixture_for, run};
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;

fn options() -> PrismEmbeddingOptions {
    PrismEmbeddingOptions::v1(BezierRegularityOptions::v1(RootIsolationOptions::new(1e-6)))
}
fn attempt(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    options: PrismEmbeddingOptions,
    cancelled: impl FnMut() -> bool,
) -> Result<PrismEmbeddingEvidence, PrismEmbeddingFailure> {
    inspect_prism_embedding(
        bridge,
        &binding_sources(shapes),
        receipt,
        options,
        cancelled,
    )
}
fn embedded(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) -> PrismEmbeddingEvidence {
    attempt(bridge, shapes, receipt, options(), || false).unwrap_or_else(|failure| {
        panic!(
            "embedding {:?}: {}",
            failure.phase,
            failure_name(&failure.cause)
        )
    })
}
fn failure_name(cause: &PrismEmbeddingCause) -> String {
    match cause {
        PrismEmbeddingCause::Version { actual, expected } => {
            format!("version {actual}; expected {expected}")
        }
        PrismEmbeddingCause::Binding(cause) => binding_failure_name(cause),
        PrismEmbeddingCause::Regularity(cause) => cause.to_string(),
        PrismEmbeddingCause::NonRegular(report) => format!("{:?}", report.decision()),
        PrismEmbeddingCause::PolynomialRequired {
            curve_id,
            explicit_weights,
        } => format!(
            "curve {curve_id}, explicit weights {explicit_weights:?}; expected unweighted polynomial"
        ),
    }
}
fn binding_failure_name(cause: &PrismBindingFailure) -> String {
    match cause {
        PrismBindingFailure::Region(cause) => format!("{cause:?}"),
        PrismBindingFailure::Chart(cause) => format!("{cause:?}"),
        _ => "typed binding failure".into(),
    }
}

#[test]
fn embedding_cubic_complete_six_rows_preserve_incomplete_body() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = embedded(&bridge, &shapes, &receipt);
    assert_eq!(
        report.binding().signed_jacobian.compare_f64(2.).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(report.domains().len(), 3);
    assert_eq!(report.incidences().len(), 6);
    assert!(matches!(
        report.regularity().decision(),
        BezierRegularityDecision::Regular { .. }
    ));
    assert_eq!(report.requested(), options());
    assert_body_unpromoted(&receipt);
}
fn assert_body_unpromoted(receipt: &PrismReceipt) {
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(
        receipt
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::PrismInjectivityAndSurfaceCharts))
    );
    assert!(
        !receipt
            .diagnostic
            .as_ref()
            .unwrap()
            .coverage
            .as_ref()
            .unwrap()
            .complete()
    );
}
#[test]
fn embedding_evidence_and_immutable_views_are_send_sync() {
    fn check<T: Send + Sync>() {}
    check::<PrismEmbeddingOptions>();
    check::<PrismEmbeddingEvidence>();
    check::<PrismFaceDomain>();
    check::<PrismFaceIncidence>();
    check::<PrismAuthoredTrimUse>();
    check::<PrismDomainKind>();
    check::<PrismIncidenceConclusion>();
}
#[path = "tests/domains.rs"]
mod domain_tests;
#[path = "tests/mutation.rs"]
mod mutation_tests;
#[path = "tests/refusal.rs"]
mod refusal_tests;
#[path = "tests/replay.rs"]
mod replay_tests;

#[test]
fn embedding_typed_failure_payloads_stay_below_large_error_threshold() {
    assert!(std::mem::size_of::<PrismEmbeddingCause>() < 128);
    assert!(std::mem::size_of::<PrismEmbeddingFailure>() < 128);
}
