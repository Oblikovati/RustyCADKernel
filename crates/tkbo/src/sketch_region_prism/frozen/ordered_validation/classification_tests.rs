// SPDX-License-Identifier: GPL-2.0-only
use super::{classification::*, joined_orientation::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::JoinedEvidenceClass, region_certificate::orientation::*,
};
use rustycad_tkg3d::top_abs::Orientation;
#[test]
fn witnessed_orientation_and_unsupported_evidence_have_distinct_classes() {
    let direction = JoinedPrismOrientationCause::Direction {
        actual: Orientation::Forward,
        expected: Orientation::Reversed,
    };
    assert_eq!(
        orientation_class(&direction),
        OrderedPrismFailureClass::Failed
    );
    assert_zero_jacobian_failure();
}
#[test]
fn missing_orientation_evidence_and_cancellation_keep_their_classes() {
    let missing = JoinedPrismOrientationCause::Count {
        subject: "named cap",
        actual: 0,
        expected: 1,
    };
    assert_eq!(
        orientation_class(&missing),
        OrderedPrismFailureClass::Incomplete
    );
    assert_eq!(
        orientation_class(&JoinedPrismOrientationCause::Cancelled),
        OrderedPrismFailureClass::Cancelled
    );
}
fn assert_zero_jacobian_failure() {
    let pure = PrismOrientationEvaluationFailure {
        phase: PrismOrientationEvaluationPhase::Winding,
        cause: PrismOrientationEvaluationCause::ZeroJacobian,
    };
    assert_eq!(
        orientation_class(&JoinedPrismOrientationCause::Orientation(Box::new(pure))),
        OrderedPrismFailureClass::Failed
    );
}
#[test]
fn original_named_orientation_failure_is_retained_without_flattening() {
    let join = test_support::JoinFixture::new()
        .compose(|| false)
        .ok()
        .unwrap();
    let occurrence = join.faces()[0].occurrence.clone();
    let failure = named_direction_failure(&join, occurrence.clone());
    let stop = orientation_stop(failure);
    assert_eq!(stop.class, OrderedPrismFailureClass::Failed);
    let OrderedPrismCause::Orientation(cause) = stop.cause else {
        panic!("expected original orientation")
    };
    assert_eq!(cause.phase, JoinedPrismOrientationPhase::Names);
    assert!(
        matches!(cause.cause, JoinedPrismOrientationCause::Named { occurrence: actual, .. } if *actual == occurrence)
    );
}
#[test]
fn child_class_mapping_never_treats_missing_or_cancelled_as_failed() {
    let cases = [
        (
            JoinedEvidenceClass::Failed,
            OrderedPrismFailureClass::Failed,
        ),
        (
            JoinedEvidenceClass::Incomplete,
            OrderedPrismFailureClass::Incomplete,
        ),
        (
            JoinedEvidenceClass::Cancelled,
            OrderedPrismFailureClass::Cancelled,
        ),
    ];
    for (actual, expected) in cases {
        assert_eq!(OrderedPrismFailureClass::from(actual), expected);
    }
}
#[test]
fn mismatched_complete_binding_is_incomplete_and_keeps_proof_identity() {
    assert!(super::checks::binding(true, "owned proof").is_ok());
    let refused = super::checks::binding(false, "owned proof").err().unwrap();
    assert_eq!(refused.class, OrderedPrismFailureClass::Incomplete);
    assert!(matches!(
        refused.cause,
        OrderedPrismCause::Binding {
            proof: "owned proof"
        }
    ));
}

fn named_direction_failure(
    join: &JoinedPrismProof,
    occurrence: super::super::FrozenUseKey,
) -> JoinedPrismOrientationFailure {
    JoinedPrismOrientationFailure {
        operation: join.projection().operation,
        requested: JoinedPrismOrientationOptions::v1(),
        phase: JoinedPrismOrientationPhase::Names,
        cause: JoinedPrismOrientationCause::Named {
            occurrence: Box::new(occurrence),
            cause: Box::new(JoinedPrismOrientationCause::Direction {
                actual: Orientation::Forward,
                expected: Orientation::Reversed,
            }),
        },
    }
}
