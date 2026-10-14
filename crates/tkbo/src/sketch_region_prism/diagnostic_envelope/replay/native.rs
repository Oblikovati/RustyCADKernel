// SPDX-License-Identifier: GPL-2.0-only
//! Preserve stored native provenance without display conversion or inferred exception ancestry.
use crate::sketch_region_prism::frozen::proof_join::encode_joined_block;
use rustycad_tkernel::standard::Failure;
pub(in crate::sketch_region_prism::diagnostic_envelope) fn append(
    value: &Failure,
    bytes: &mut Vec<u8>,
) {
    encode_joined_block(value.exception_type().as_bytes(), bytes);
    optional(value.explicit_exception_parent().map(str::as_bytes), bytes);
    encode_joined_block(value.message_bytes(), bytes);
    encode_joined_block(value.stack_bytes(), bytes);
    optional(value.derived_message_bytes(), bytes);
}
fn optional(value: Option<&[u8]>, bytes: &mut Vec<u8>) {
    bytes.push(u8::from(value.is_some()));
    if let Some(value) = value {
        encode_joined_block(value, bytes);
    }
}
