// SPDX-License-Identifier: GPL-2.0-only
//! Exact bounded join codec operands; synthetic local causes are never issued invocation packets.
use super::*;
use crate::sketch_region_prism::{
    frozen::FreezePhase,
    region_certificate::{
        RegionCertificateCause, current_binding::PrismBindingFailure, embedding::*,
    },
};
#[test]
fn scalar_join_causes_have_independently_expected_literal_bytes() {
    use PrismProofJoinCause as C;
    with_diagnostic_context(|bindings, _| {
        assert_join_bytes(bindings, C::MissingClosureQuery, vec![0]);
        assert_join_bytes(
            bindings,
            C::Version {
                actual: 9,
                expected: 1,
            },
            vec![1, 9, 0, 0, 0, 1, 0, 0, 0],
        );
    });
}
#[test]
fn stale_join_retains_exact_representation_label_bytes() {
    use PrismProofJoinCause as C;
    with_diagnostic_context(|bindings, _| {
        assert_join_bytes(
            bindings,
            C::Stale {
                representation: "x",
            },
            vec![3, 1, 0, 0, 0, 0, 0, 0, 0, b'x'],
        );
    });
}
#[test]
fn absent_and_present_cancellation_phase_and_binding_have_distinct_bytes() {
    use PrismProofJoinCause as C;
    with_diagnostic_context(|bindings, _| {
        assert_join_bytes(bindings, C::Cancelled { freeze_phase: None }, vec![2, 0]);
        let cause = C::Cancelled {
            freeze_phase: Some(FreezePhase::Commit),
        };
        assert_join_bytes(bindings, cause, vec![2, 1, 5]);
        let cause = C::Binding(Box::new(PrismBindingFailure::Cancelled));
        assert_join_bytes(bindings, cause, vec![4, 0]);
    });
}
fn assert_join_bytes(
    bindings: &PrismPublicationBindings,
    cause: PrismProofJoinCause,
    suffix: Vec<u8>,
) {
    let value = join(cause, bindings);
    let (_, actual) = joined::project(&value, PrismPublicationClass::Cancelled, bindings).unwrap();
    let mut expected = b"Martensite.PublicationJoinDiagnostic/v1\0".to_vec();
    expected.extend(257u64.to_le_bytes());
    expected.extend([0, 3, 0]);
    expected.extend(suffix);
    assert_eq!(actual, expected);
}
#[test]
fn mapped_native_binding_keeps_exact_provenance_without_working_owner() {
    with_diagnostic_context(|bindings, _| {
        let native = Failure::out_of_range("exact primitive native").with_exception_parent("");
        let cause = PrismBindingFailure::Region(RegionCertificateCause::Mapped(native.clone()));
        let value = join(PrismProofJoinCause::Binding(Box::new(cause)), bindings);
        let (result, _) = joined::project(&value, PrismPublicationClass::Failed, bindings).unwrap();
        let OwnedPrismJoinCause::Binding(cause) = result.cause() else {
            panic!("expected binding")
        };
        let OwnedPrismBindingCause::Region(OwnedPrismRegionCause::Mapped(actual)) = cause.as_ref()
        else {
            panic!("expected native")
        };
        assert_eq!(actual, &native);
        assert_eq!(actual.explicit_exception_parent(), Some(""));
    });
}
#[test]
fn embedding_version_preserves_options_phase_and_typed_operands() {
    with_diagnostic_context(|bindings, options| {
        let cause = embedding(
            options.join.embedding,
            PrismEmbeddingCause::Version {
                actual: 9,
                expected: 1,
            },
        );
        let value = join(PrismProofJoinCause::Embedding(Box::new(cause)), bindings);
        let (result, _) =
            joined::project(&value, PrismPublicationClass::Incomplete, bindings).unwrap();
        assert_embedding_version(&result, options.join.embedding);
    });
}
fn assert_embedding_version(result: &OwnedPrismJoinRefusal, requested: PrismEmbeddingOptions) {
    let OwnedPrismJoinCause::Embedding(actual) = result.cause() else {
        panic!("expected embedding")
    };
    assert_eq!(actual.requested, requested);
    assert_eq!(actual.phase, PrismEmbeddingPhase::Admission);
    assert!(matches!(
        actual.cause,
        OwnedPrismEmbeddingCause::Version {
            actual: 9,
            expected: 1
        }
    ));
}

fn embedding(
    requested: PrismEmbeddingOptions,
    cause: PrismEmbeddingCause,
) -> PrismEmbeddingFailure {
    PrismEmbeddingFailure {
        requested,
        phase: PrismEmbeddingPhase::Admission,
        original: None,
        cause,
    }
}
#[test]
fn unimplemented_precise_embedding_payload_is_incomplete_not_stringified() {
    with_diagnostic_context(|bindings, options| {
        let cause = embedding(
            options.join.embedding,
            PrismEmbeddingCause::PolynomialRequired {
                curve_id: 5,
                explicit_weights: vec![1., f64::from_bits(0x7ff8_0000_0000_0042)],
            },
        );
        let value = join(PrismProofJoinCause::Embedding(Box::new(cause)), bindings);
        let result = joined::project(&value, PrismPublicationClass::Incomplete, bindings);
        assert!(matches!(
            result,
            Err(PublicationPendingPayload::JoinEmbedding)
        ));
        assert_original_polynomial_required(&value);
    });
}
fn assert_original_polynomial_required(value: &PrismProofJoinFailure) {
    let PrismProofJoinCause::Embedding(retained) = &value.cause else {
        panic!("expected original")
    };
    let PrismEmbeddingCause::PolynomialRequired {
        curve_id,
        explicit_weights,
    } = &retained.cause
    else {
        panic!("expected weights")
    };
    assert_eq!(*curve_id, 5);
    assert_eq!(explicit_weights[1].to_bits(), 0x7ff8_0000_0000_0042);
}
