// SPDX-License-Identifier: GPL-2.0-only
//! Full annular request operands (ADR0062): explicit outer/inner roles, authored names and every
//! requested limit; refused NaN payloads keep their bits and nothing is admitted here.
use super::super::annular::{OwnedAnnularPolicy, OwnedAnnularPrismRequest};
use super::*;
use crate::sketch_region_prism::frozen::proof_join::encode_joined_root_options as roots;
use crate::sketch_region_prism::region_certificate::two_loop::{
    TwoLoopRegionBehavior, TwoLoopRegionOptions,
};

/// Encode every original annular operand; `encode_owned_annular_request(&request)`.
pub(in crate::sketch_region_prism) fn encode_owned_annular_request(
    request: &OwnedAnnularPrismRequest,
) -> Vec<u8> {
    let mut bytes = b"Martensite.OwnedAnnularPrismRequest/v1\0".to_vec();
    bytes.extend(request.version.to_le_bytes());
    word(request.operation.0, &mut bytes);
    boundary(&request.outer, &mut bytes);
    boundary(&request.inner, &mut bytes);
    for name in request.regions.all() {
        word(name, &mut bytes);
    }
    numbers(request.inner_interior, &mut bytes);
    numbers(request.material_seed, &mut bytes);
    frame(request.declared_plane, &mut bytes);
    numbers(request.extrusion, &mut bytes);
    policy(request.policy, &mut bytes);
    bytes
}
fn policy(policy: OwnedAnnularPolicy, bytes: &mut Vec<u8>) {
    roots(policy.census.roots, bytes);
    numbers([policy.census.max_world_error], bytes);
    roots(policy.source.roots, bytes);
    let source = &policy.source;
    numbers(
        [source.original_plane_limit, source.construction_limit],
        bytes,
    );
    roots(policy.contacts.roots, bytes);
    numbers(policy.contacts.native_parameter_limits, bytes);
    numbers([policy.contacts.world_endpoint_limit], bytes);
    roots(policy.classification_roots, bytes);
    numbers([policy.spatial_limit, policy.parameter_limit], bytes);
    region(policy.region, bytes);
}
fn region(options: TwoLoopRegionOptions, bytes: &mut Vec<u8>) {
    // Explicit behavior tag: a later behavior must add its own tag, never reuse this one.
    let TwoLoopRegionBehavior::NestedPolynomialV2 = options.behavior;
    bytes.push(2);
    roots(options.winding, bytes);
    word(options.maximum_retained_units as u64, bytes);
    word(options.maximum_poles as u64, bytes);
}

#[cfg(test)]
#[path = "annular_tests.rs"]
mod tests;
