// SPDX-License-Identifier: GPL-2.0-only
//! Encode requested limits separately; the codec does not invent effective or achieved settings.
use super::*;
use crate::sketch_region_prism::frozen::proof_join::encode_joined_root_options as roots;
pub(super) fn encode(policy: OwnedPrismPolicy, bytes: &mut Vec<u8>) {
    roots(policy.census.roots, bytes);
    numbers([policy.census.max_world_error], bytes);
    roots(policy.source.roots, bytes);
    numbers(
        [
            policy.source.original_plane_limit,
            policy.source.construction_limit,
        ],
        bytes,
    );
    roots(policy.contacts.roots, bytes);
    numbers(policy.contacts.native_parameter_limits, bytes);
    numbers([policy.contacts.world_endpoint_limit], bytes);
    roots(policy.classification_roots, bytes);
    numbers([policy.spatial_limit, policy.parameter_limit], bytes);
    bytes.extend(encode_publication_request_options(policy.publication));
}
