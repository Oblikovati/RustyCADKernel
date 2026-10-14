// SPDX-License-Identifier: GPL-2.0-only
//! Every requested policy dimension remains distinguishable before admission or clamping.
use super::tests::changed;
use super::*;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
type RootSlot = fn(&mut OwnedPolynomialPrismRequest) -> &mut RootIsolationOptions;
type ScalarSlot = fn(&mut OwnedPolynomialPrismRequest) -> &mut f64;
type VersionSlot = fn(&mut OwnedPolynomialPrismRequest) -> &mut u32;
#[test]
fn each_root_policy_preserves_version_width_work_and_degree() {
    let slots: [RootSlot; 5] = [
        |r| &mut r.policy.census.roots,
        |r| &mut r.policy.source.roots,
        |r| &mut r.policy.contacts.roots,
        |r| &mut r.policy.classification_roots,
        |r| &mut r.policy.publication.join.embedding.regularity.roots,
    ];
    for slot in slots {
        changed(|r| slot(r).version += 1);
        changed(|r| slot(r).max_width = -0.);
        changed(|r| slot(r).max_bisections += 1);
        changed(|r| slot(r).max_degree += 1);
    }
}
#[test]
fn scalar_policies_preserve_signed_zero_and_all_requested_error_dimensions() {
    for slot in scalar_slots() {
        changed(|r| *slot(r) = -0.);
        changed(|r| *slot(r) = f64::from_bits(0x7ff8_0000_0000_0042));
    }
}
fn scalar_slots() -> [ScalarSlot; 12] {
    [
        |r| &mut r.policy.census.max_world_error,
        |r| &mut r.policy.source.original_plane_limit,
        |r| &mut r.policy.source.construction_limit,
        |r| &mut r.policy.contacts.native_parameter_limits[0],
        |r| &mut r.policy.contacts.native_parameter_limits[1],
        |r| &mut r.policy.contacts.world_endpoint_limit,
        |r| &mut r.policy.spatial_limit,
        |r| &mut r.policy.parameter_limit,
        |r| &mut measures(r).subset.cap_area.absolute_error_limit,
        |r| &mut measures(r).subset.signed_volume.absolute_error_limit,
        |r| &mut measures(r).side_area.absolute_error_limit,
        |r| &mut measures(r).total_area.absolute_error_limit,
    ]
}
#[test]
fn every_nested_behavior_version_and_side_work_budget_changes_request_bytes() {
    for slot in version_slots() {
        changed(|r| *slot(r) += 1);
    }
}
fn version_slots() -> [VersionSlot; 13] {
    [
        |r| &mut r.version,
        |r| &mut publication(r).version,
        |r| &mut publication(r).join.version,
        |r| &mut publication(r).join.embedding.version,
        |r| &mut publication(r).join.embedding.regularity.version,
        |r| &mut publication(r).validation.version,
        |r| &mut publication(r).history.version,
        |r| &mut publication(r).measures.version,
        |r| &mut measures(r).subset.cap_area.behavior_version,
        |r| &mut measures(r).subset.signed_volume.behavior_version,
        |r| &mut measures(r).side_area.behavior_version,
        |r| &mut measures(r).total_area.behavior_version,
        |r| &mut measures(r).side_area.max_subdivisions,
    ]
}
#[test]
fn history_bounds_stored_degree_and_integrator_count_caps_remain_distinct() {
    changed(|r| publication(r).history.limits.maximum_depth += 1);
    changed(|r| publication(r).history.limits.maximum_uses += 1);
    changed(|r| publication(r).history.limits.maximum_rows += 1);
    changed(|r| publication(r).join.embedding.regularity.max_stored_degree += 1);
    changed(|r| measures(r).subset.cap_area.max_degree += 1);
    changed(|r| measures(r).subset.cap_area.max_curves += 1);
    changed(|r| measures(r).subset.signed_volume.max_degree += 1);
    changed(|r| measures(r).subset.signed_volume.max_curves += 1);
    changed(|r| measures(r).side_area.max_degree += 1);
    changed(|r| measures(r).total_area.max_components += 1);
}
#[test]
fn explicit_stored_cap_mode_and_value_are_both_bound() {
    use crate::sketch_region_prism::exact_tolerance::StoredToleranceCap;
    changed(|r| r.policy.publication.validation.stored_cap = StoredToleranceCap::NoLimit);
    changed(|r| {
        r.policy.publication.validation.stored_cap = StoredToleranceCap::report(1e-6).unwrap()
    });
    changed(|r| {
        r.policy.publication.validation.stored_cap = StoredToleranceCap::refuse(2e-6).unwrap()
    });
}

fn publication(
    request: &mut OwnedPolynomialPrismRequest,
) -> &mut crate::sketch_region_prism::publication::PrismPublicationOptions {
    let policy = &mut request.policy;
    &mut policy.publication
}
fn measures(
    request: &mut OwnedPolynomialPrismRequest,
) -> &mut crate::sketch_region_prism::region_certificate::CompletePrismMeasureOptions {
    let policy = &mut request.policy.publication;
    &mut policy.measures.scalars
}
