// SPDX-License-Identifier: GPL-2.0-only
//! Unsupported upstream diagnostics never imply complete projection of their unknown payload.
use super::*;
fn unsupported() -> PrismFreezeCause {
    PrismFreezeCause::UnsupportedDiagnostic {
        stage: Some(crate::sketch_region_validation::ValidationStage::SelfIntersection),
        actual: "unprojectable native alert type/payload",
    }
}
fn unsupported_capture() -> Box<PrismFreezeFailure> {
    Box::new(PrismFreezeFailure {
        operation: AuthoredPrismId(401),
        phase: FreezePhase::Evidence,
        entity: None,
        cause: unsupported(),
    })
}
#[test]
fn unsupported_frozen_diagnostic_stays_projection_incomplete() {
    let cases = [
        PrismProofJoinCause::Link(Box::new(unsupported())),
        PrismProofJoinCause::FailedCapture(unsupported_capture()),
        PrismProofJoinCause::IncompleteCapture(unsupported_capture()),
    ];
    for cause in cases {
        let error = retained(NamedRefusalFixture::new().run(cause, PrismRefusalClass::Failed));
        assert_eq!(error.into_original().0, PrismRefusalClass::Failed);
    }
}
fn nested_unsupported() -> PrismFreezeCause {
    let key = FrozenPrismKey {
        operation: AuthoredPrismId(401),
        source: AuthoredPrismId(403),
        role: FrozenSweepRole::Generated,
        kind: 4,
    };
    PrismFreezeCause::Entities {
        primary: key,
        related: vec![key, key],
        occurrence: Some(FrozenUseKey(vec![(key, AuthoredPrismId(404))])),
        cause: Box::new(unsupported()),
    }
}
#[test]
fn nested_frozen_diagnostic_retains_complete_entity_context_and_payload() {
    let before = format!("{:?}", nested_unsupported());
    let cause = PrismProofJoinCause::Link(Box::new(nested_unsupported()));
    let error = retained(NamedRefusalFixture::new().run(cause, PrismRefusalClass::Incomplete));
    let (class, local) = error.into_original();
    assert_eq!(class, PrismRefusalClass::Incomplete);
    let PrismProofJoinCause::Link(cause) = local.cause else {
        panic!("expected link")
    };
    assert_eq!(format!("{cause:?}"), before);
}
