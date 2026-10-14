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
