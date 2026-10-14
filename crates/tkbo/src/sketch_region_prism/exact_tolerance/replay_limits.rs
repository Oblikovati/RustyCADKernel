// SPDX-License-Identifier: GPL-2.0-only
//! New receipts retain all operation option bits without changing the legacy frozen encoding.
use crate::sketch_region_prism::frozen::{
    FrozenPrismLimits, encode_frozen_numbers, proof_join::encode_joined_root_options,
};
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
pub(super) fn limits(value: &FrozenPrismLimits, bytes: &mut Vec<u8>) {
    encode_frozen_numbers(value.seed, bytes);
    encode_frozen_numbers(value.extrusion, bytes);
    encode_joined_root_options(value.roots, bytes);
    encode_frozen_numbers([value.spatial_limit, value.parameter_limit], bytes);
    optional(value.effective_seed, bytes);
    optional(value.effective_extrusion, bytes);
    roots(value.effective_roots, bytes);
    optional(value.effective_spatial.map(|x| [x]), bytes);
    optional(value.effective_parameter.map(|x| [x]), bytes);
    optional(value.achieved_error.map(|x| [x]), bytes);
}
fn optional<const N: usize>(value: Option<[f64; N]>, bytes: &mut Vec<u8>) {
    bytes.push(u8::from(value.is_some()));
    if let Some(value) = value {
        encode_frozen_numbers(value, bytes);
    }
}
fn roots(value: Option<RootIsolationOptions>, bytes: &mut Vec<u8>) {
    bytes.push(u8::from(value.is_some()));
    if let Some(value) = value {
        encode_joined_root_options(value, bytes);
    }
}
