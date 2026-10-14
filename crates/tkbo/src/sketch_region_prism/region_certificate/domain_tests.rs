// SPDX-License-Identifier: GPL-2.0-only
//! Independent Bernstein examples exercise closure, repeats, overlap and exhausted evidence.
use super::super::tests::{progress, request};
use super::{tests::*, *};
use crate::sketch_region_certification::census_bezier_fixture;
use crate::sketch_region_partition_bindings::binding_sources;

#[test]
fn scaled_reversed_and_reordered_inputs_keep_the_general_jordan_theorem() {
    for (scale, reverse) in [(0.5, false), (1., true), (2., false), (2., true)] {
        let shape = crate::closed_analytic_cubic::closed_analytic_cubic(scale, reverse);
        let (bridge, shapes) = fixture_for(vec![shape]);
        let result = retain_source_bound_prism(
            &bridge,
            &binding_sources(&shapes),
            request([scale, scale]),
            &mut progress(None, None),
        )
        .ok()
        .unwrap();
        assert_eq!(proof(&result).selected_winding, 1);
        assert!(!proof(&result).boundary_reversed);
    }
}
#[test]
fn retracing_open_and_extra_contact_boundaries_cannot_borrow_single_closure_proof() {
    for poles in [
        vec![[0., 0.], [4., 0.], [0., 0.]],
        vec![[0., 0.], [4., 0.], [0., 4.]],
        vec![[0., 0.], [-3., 3.], [0., -4.], [3., 3.], [0., 0.]],
    ] {
        let (bridge, shapes) = fixture_for(vec![census_bezier_fixture(&poles, None)]);
        let result = retain_source_bound_prism(
            &bridge,
            &binding_sources(&shapes),
            request([1., 1.]),
            &mut progress(None, None),
        );
        assert_unadmitted(result);
    }
}
fn assert_unadmitted(result: Result<PrismReceipt, Box<RegionPrismUnavailable>>) {
    match result {
        Ok(receipt) => assert_four_present(&receipt),
        Err(missing) => assert_eq!(missing.missing, obligations()),
    }
}
#[test]
fn partial_paired_search_cannot_inherit_old_complete_source_or_winding_status() {
    let (mut bridge, shapes) = fixture();
    bridge.census.paired.as_mut().unwrap().reports.clear();
    bridge.census.missing.clear();
    bridge.inspection.missing.clear();
    assert_four_present(&run(&bridge, &shapes));
}
#[test]
fn source_endpoint_tuple_must_be_exact_and_current() {
    let (mut bridge, shapes) = fixture();
    bridge.inspection.contacts[0]
        .bounded_construction
        .as_mut()
        .unwrap()
        .directed_contexts
        .clear();
    assert_four_present(&run(&bridge, &shapes));
}

#[test]
fn positive_rational_jordan_boundary_uses_the_same_complete_source_domain() {
    let poles = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let shape = census_bezier_fixture(&poles, Some(&[1., 2., 2., 1.]));
    let (bridge, shapes) = fixture_for(vec![shape]);
    // Equal positive inner weights preserve y/x=t/(1-t), proving interior injectivity.
    let receipt = run(&bridge, &shapes);
    assert_eq!(proof(&receipt).selected_winding, 1);
}
