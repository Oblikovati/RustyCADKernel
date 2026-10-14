// SPDX-License-Identifier: GPL-2.0-only
//! Legacy adapters preserve exact causes while pure consumers receive no owner-bearing variants.
use super::{hypothesis::*, types::RegionCertificateCause};
use static_assertions::assert_impl_all;

#[test]
fn owned_hypothesis_checks_keep_count_expectations() {
    assert_eq!(require("unique side", 1, 1), Ok(()));
    assert_eq!(
        require("unique side", 2, 1),
        Err(RegionHypothesisCause::Condition {
            name: "unique side",
            actual: 2,
            expected: 1,
        })
    );
}

#[test]
fn owned_hypothesis_checks_keep_boolean_expectations() {
    assert_eq!(holds("source binding", true), Ok(()));
    assert_eq!(
        holds("source binding", false),
        Err(RegionHypothesisCause::Condition {
            name: "source binding",
            actual: 0,
            expected: 1,
        })
    );
}

#[test]
fn owned_hypothesis_checks_keep_missing_expectations() {
    assert_eq!(need(Some(7), "height"), Ok(7));
    assert_eq!(
        need::<usize>(None, "height"),
        Err(RegionHypothesisCause::Missing {
            hypothesis: "height",
        })
    );
}

#[test]
fn owned_hypothesis_legacy_adapter_preserves_missing_error_shape() {
    let missing = RegionCertificateCause::from(RegionHypothesisCause::Missing {
        hypothesis: "height",
    });
    assert!(matches!(
        missing,
        RegionCertificateCause::MissingEvidence {
            hypothesis: "height"
        }
    ));
}

#[test]
fn owned_hypothesis_legacy_adapter_preserves_count_error_shape() {
    let cause = RegionHypothesisCause::Condition {
        name: "unique trim",
        actual: 3,
        expected: 1,
    };
    assert!(matches!(
        RegionCertificateCause::from(cause.clone()),
        RegionCertificateCause::Hypothesis {
            name: "unique trim",
            actual: 3,
            expected: 1,
        }
    ));
    assert_legacy_embedding_cause(cause);
}

fn assert_legacy_embedding_cause(cause: RegionHypothesisCause) {
    use super::{current_binding::PrismBindingFailure, embedding::PrismEmbeddingCause};
    let PrismEmbeddingCause::Binding(binding) = PrismEmbeddingCause::from(cause) else {
        panic!("expected unchanged legacy binding layer");
    };
    assert!(matches!(
        *binding,
        PrismBindingFailure::Region(RegionCertificateCause::Hypothesis {
            name: "unique trim",
            actual: 3,
            expected: 1,
        })
    ));
}

#[test]
fn owned_hypothesis_wrappers_keep_legacy_success_and_refusal_values() {
    assert!(super::types::require("one", 1, 1).is_ok());
    assert!(matches!(
        super::types::require("one", 0, 1),
        Err(RegionCertificateCause::Hypothesis {
            name: "one",
            actual: 0,
            expected: 1,
        })
    ));
    assert_eq!(super::types::need(Some(9), "present").unwrap(), 9);
    assert!(matches!(
        super::types::need::<usize>(None, "absent"),
        Err(RegionCertificateCause::MissingEvidence {
            hypothesis: "absent",
        })
    ));
    assert!(super::types::holds("true", true).is_ok());
    assert_impl_all!(RegionHypothesisCause: Send, Sync);
}
