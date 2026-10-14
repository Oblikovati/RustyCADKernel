// SPDX-License-Identifier: GPL-2.0-only
//! Corrupt derived operands, never forge the private actual binding constructor.
use super::super::super::annular_binding::tests::actual_binding;
use super::super::annular_table::ten_rows;
use super::*;

#[test]
fn directed_trim_order_never_supplies_rim_or_seam_identity() {
    let binding = actual_binding();
    let mut domains = capture_annular_domains(&binding).unwrap();
    let before = ten_rows(&domains).unwrap();
    domains.first.trims.reverse();
    domains.last.trims.reverse();
    domains.outer.trims.rotate_left(1);
    domains.inner.trims.rotate_right(1);
    assert_eq!(ten_rows(&domains).unwrap(), before);
}
#[test]
fn every_cap_rim_is_required_even_though_both_have_the_same_role() {
    let binding = actual_binding();
    for slot in 0..2 {
        let mut domains = capture_annular_domains(&binding).unwrap();
        domains.first.trims.remove(slot);
        assert!(ten_rows(&domains).is_err());
        let mut domains = capture_annular_domains(&binding).unwrap();
        domains.last.trims.remove(slot);
        assert!(ten_rows(&domains).is_err());
    }
}
#[test]
fn duplicated_rim_contexts_do_not_hide_a_missing_source() {
    let binding = actual_binding();
    let mut domains = capture_annular_domains(&binding).unwrap();
    domains.first.trims[0] = domains.first.trims[1].clone();
    assert!(ten_rows(&domains).is_err());
}
#[test]
fn rim_lookup_refuses_foreign_ancestry_even_with_the_same_scalar_occurrence() {
    let binding = actual_binding();
    let mut domains = capture_annular_domains(&binding).unwrap();
    let before = domains.first.trims[0].operand.clone();
    domains.first.trims[0].identity.source.child_slots.push(999);
    assert_eq!(domains.first.trims[0].operand, before);
    assert!(ten_rows(&domains).is_err());
}
#[test]
fn each_native_seam_endpoint_is_required_by_the_actual_full_context() {
    let binding = actual_binding();
    let domains = capture_annular_domains(&binding).unwrap();
    let boundary = selection::boundary(&binding, binding.region().nesting().outer).unwrap();
    for role in [PrismTrimRole::Start, PrismTrimRole::End] {
        let mut trims = domains.outer.trims.clone();
        let trim = trims.iter_mut().find(|t| t.identity.role == role).unwrap();
        trim.identity.source.child_slots.push(999);
        assert!(selection::seam_endpoints(boundary, &trims).is_err());
    }
}
#[test]
fn a_foreign_side_output_fails_the_shared_physical_rim_check() {
    let binding = actual_binding();
    let mut domains = capture_annular_domains(&binding).unwrap();
    let inner_rim = domains
        .inner
        .trims
        .iter()
        .find(|t| t.identity.role == PrismTrimRole::First)
        .unwrap();
    let foreign = inner_rim.operand.output.clone();
    let outer_rim = domains
        .outer
        .trims
        .iter_mut()
        .find(|t| t.identity.role == PrismTrimRole::First)
        .unwrap();
    outer_rim.operand.output = foreign;
    assert!(ten_rows(&domains).is_err());
}
