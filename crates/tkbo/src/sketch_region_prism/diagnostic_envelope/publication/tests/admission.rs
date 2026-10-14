// SPDX-License-Identifier: GPL-2.0-only
//! Exact requested bits, rather than floating-point PartialEq, govern policy correspondence.
use super::*;
#[test]
fn policy_comparison_preserves_nan_payload_and_signed_zero() {
    with_diagnostic_context(|_, options| {
        let mut first = options;
        first.measures.scalars.total_area.absolute_error_limit =
            f64::from_bits(0x7ff8_0000_0000_0042);
        assert!(super::super::admission::same_policy(first, first));
        let mut other = first;
        other.measures.scalars.total_area.absolute_error_limit =
            f64::from_bits(0x7ff8_0000_0000_0043);
        assert!(!super::super::admission::same_policy(first, other));
        first.measures.scalars.total_area.absolute_error_limit = 0.;
        other.measures.scalars.total_area.absolute_error_limit = -0.;
        assert!(!super::super::admission::same_policy(first, other));
    });
}
#[test]
fn earlier_attribution_gap_prevents_qualifying_even_identical_operation_tokens() {
    with_diagnostic_context(|bindings, options| {
        let cause = PublicationStageCause::Join(Box::new(join(
            PrismProofJoinCause::MissingClosureQuery,
            bindings,
        )));
        let mut gaps = vec![PublicationDiagnosticGap::CallerDictionary];
        let result = projection::outer(&outer(cause, options), bindings, &mut gaps);
        assert!(matches!(
            stage(&result).cause,
            OwnedPublicationStageCause::Unprojected(_)
        ));
        assert_eq!(gaps.len(), 2);
    });
}
#[test]
fn foreign_join_operation_is_incomplete_without_rewriting_observed_ids() {
    with_diagnostic_context(|bindings, options| {
        let mut child = join(PrismProofJoinCause::MissingClosureQuery, bindings);
        child.operation = AuthoredPrismId(900);
        let value = outer(PublicationStageCause::Join(Box::new(child)), options);
        let mut gaps = Vec::new();
        let result = projection::outer(&value, bindings, &mut gaps);
        assert!(matches!(
            stage(&result).cause,
            OwnedPublicationStageCause::Unprojected(_)
        ));
        assert_eq!(
            gaps[0],
            PublicationDiagnosticGap::Operation {
                actual: AuthoredPrismId(900),
                expected: bindings.operation,
            }
        );
    });
}

#[test]
fn unprojected_entity_keeps_full_authored_key_without_claiming_caller_qualification() {
    with_diagnostic_context(|bindings, options| {
        let mut child = join(PrismProofJoinCause::MissingClosureQuery, bindings);
        let key = crate::sketch_region_prism::frozen::FrozenPrismKey {
            operation: bindings.operation,
            source: AuthoredPrismId(999),
            role: crate::sketch_region_prism::frozen::FrozenSweepRole::Generated,
            kind: 4,
        };
        child.entity = Some(key);
        let value = outer(PublicationStageCause::Join(Box::new(child)), options);
        let mut gaps = Vec::new();
        projection::outer(&value, bindings, &mut gaps);
        assert_eq!(gaps[0], PublicationDiagnosticGap::UnprojectedEntity(key));
    });
}
#[test]
fn observed_history_operation_mismatch_is_explicit_not_inferred_from_outer_token() {
    with_diagnostic_context(|bindings, options| {
        let mut value = outer(PublicationStageCause::Binding { proof: "fixture" }, options);
        value.cause = PrismPublicationCause::HistoryCancelled {
            operation: AuthoredPrismId(900),
            phase: PrismHistoryPhase::Capture,
        };
        let mut gaps = Vec::new();
        let result = projection::outer(&value, bindings, &mut gaps);
        assert_history_observation(result);
        assert_eq!(
            gaps,
            vec![PublicationDiagnosticGap::Operation {
                actual: AuthoredPrismId(900),
                expected: bindings.operation,
            }]
        );
    });
}
fn assert_history_observation(value: OwnedPublicationDiagnosticCause) {
    assert!(matches!(
        value,
        OwnedPublicationDiagnosticCause::HistoryCancelled {
            operation: AuthoredPrismId(900),
            phase: PrismHistoryPhase::Capture,
        }
    ));
}
