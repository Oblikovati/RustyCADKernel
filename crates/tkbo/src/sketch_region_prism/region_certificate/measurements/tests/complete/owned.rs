// SPDX-License-Identifier: GPL-2.0-only
//! The scalar kernel needs captured operands, never a live receipt or mutable geometry owner.
use super::*;

#[test]
fn captured_complete_measures_survive_disposal_of_every_mutable_input_owner() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = full_report(&bridge, &shapes, &receipt, fast_options());
    let extrusion = report.measures.subset.extrusion;
    drop(receipt);
    drop(bridge);
    drop(shapes);
    let repeated = measure_bound_prism(&report.binding, extrusion, fast_options(), &mut || false)
        .map_err(CompletePrismMeasureFailure::from)
        .map_err(full_failure_name)
        .unwrap();
    assert_same_measures(&report.measures, &repeated);
}

fn assert_same_measures(expected: &CompletePrismMeasures, actual: &CompletePrismMeasures) {
    assert_eq!(actual.subset.first_cap, expected.subset.first_cap);
    assert_eq!(actual.subset.last_cap, expected.subset.last_cap);
    assert_eq!(actual.subset.signed_volume, expected.subset.signed_volume);
    assert_eq!(
        actual.subset.extrusion.map(f64::to_bits),
        expected.subset.extrusion.map(f64::to_bits)
    );
    assert_eq!(actual.side, expected.side);
    assert_eq!(actual.total, expected.total);
    assert_eq!(actual.requested, expected.requested);
}

#[test]
fn pure_scalar_failures_are_owned_and_keep_old_missing_evidence_payloads() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<BoundPrismMeasureFailure>();
    send_sync::<PrismScalarFailure>();
    send_sync::<ScalarOperandCause>();
    let cause = scalar_need::<usize>(None, "owned operand").unwrap_err();
    let legacy = PrismMeasureFailure::from(PrismScalarFailure::from(cause));
    assert!(matches!(
        legacy,
        PrismMeasureFailure::Region(RegionCertificateCause::MissingEvidence {
            hypothesis: "owned operand"
        })
    ));
    assert_eq!(scalar_need(Some(42), "owned value").unwrap(), 42);
}

#[test]
fn pure_scalar_conditions_keep_old_actual_and_expected_counts() {
    assert!(scalar_holds("exact equality", true).is_ok());
    let cause = scalar_holds("exact equality", false).unwrap_err();
    let legacy = RegionCertificateCause::from(cause);
    assert!(matches!(
        legacy,
        RegionCertificateCause::Hypothesis {
            name: "exact equality",
            actual: 0,
            expected: 1
        }
    ));
}

#[test]
fn side_operand_conversion_preserves_nested_binding_region_failure() {
    let cause = BoundPrismMeasureFailure::SideOperand(ScalarOperandCause::Missing {
        hypothesis: "positive height",
    });
    let CompletePrismMeasureFailure::Binding(cause) = CompletePrismMeasureFailure::from(cause)
    else {
        panic!("expected legacy binding refusal");
    };
    assert!(matches!(
        *cause,
        PrismBindingFailure::Region(RegionCertificateCause::MissingEvidence {
            hypothesis: "positive height"
        })
    ));
}
