// SPDX-License-Identifier: GPL-2.0-only
//! Required native causes are preserved as typed owned values or explicitly refused.
use super::{tests::*, *};
use crate::sketch_region_validation::outcome::ValidationCause;

#[test]
fn nominal_fixture_native_alert_is_exact_dataless_intersection_failure() {
    let (_, receipt) = authored_receipt();
    let alerts: Vec<_> = native_findings(&receipt)
        .iter()
        .filter_map(|finding| match &finding.cause {
            ValidationCause::AnalyticNativeAlert(alert) => Some(alert),
            _ => None,
        })
        .collect();
    assert_eq!(alerts.len(), 1);
    assert!(
        rustycad_tkernel::message::downcast_alert::<crate::bop_algo::AlertIntersectionFailed>(
            &alerts[0].original
        )
        .is_some()
    );
}

fn native_findings(
    receipt: &super::super::PrismReceipt,
) -> &[crate::sketch_region_validation::outcome::ValidationFinding] {
    &receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .coverage
        .as_ref()
        .unwrap()
        .findings
}

#[test]
fn unknown_and_owner_bearing_alerts_explicitly_refuse_capture() {
    for owner_bearing in [false, true] {
        let (source, mut receipt) = authored_receipt();
        let bindings = authored_bindings(&receipt, &source);
        replace_alert(&mut receipt, owner_bearing);
        let outcome = freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress);
        assert!(matches!(
            outcome,
            PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
                cause: PrismFreezeCause::UnsupportedDiagnostic {
                    actual: "unprojectable native alert type/payload",
                    ..
                },
                ..
            })
        ));
    }
}
fn replace_alert(receipt: &mut super::super::PrismReceipt, owner_bearing: bool) {
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    let original: rustycad_tkernel::message::AlertHandle = if owner_bearing {
        std::sync::Arc::new(crate::bop_algo::AlertUnsupportedType::new(&diagnostic.body))
    } else {
        std::sync::Arc::new(rustycad_tkernel::message::BaseAlert)
    };
    let finding = diagnostic
        .coverage
        .as_mut()
        .unwrap()
        .findings
        .iter_mut()
        .find(|f| matches!(f.cause, ValidationCause::AnalyticNativeAlert(_)))
        .unwrap();
    finding.cause =
        ValidationCause::AnalyticNativeAlert(crate::bop_algo::NativeAlertObservation { original });
}
#[test]
fn captured_intersection_failure_is_typed_and_never_a_validity_pass() {
    let projection = fresh_projection();
    assert!(
        projection
            .coverage()
            .findings
            .iter()
            .any(|finding| matches!(
                finding.cause,
                super::coverage::FrozenValidationCause::AnalyticNativeAlert(
                    super::alerts::FrozenNativeAlert::IntersectionFailedAtFailGravity
                )
            ))
    );
    assert!(projection.history().deleted.is_empty());
    assert!(!projection.history().retired_intermediate_unbound.is_empty());
    assert!(projection.coverage().limitations.iter().any(|gap| matches!(
        gap,
        super::coverage::FrozenPublicationGap::IntermediateRetirementCompositionUnproven
    )));
    assert!(!projection.is_valid_body());
}
