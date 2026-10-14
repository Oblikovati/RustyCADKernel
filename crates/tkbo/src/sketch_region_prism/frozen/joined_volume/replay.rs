// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn encode(volume: &JoinedPrismVolume) -> Vec<u8> {
    let mut bytes = b"Martensite joined conditional volume V1\0".to_vec();
    sign_words(&mut bytes, volume.factors);
    scalar_words(&mut bytes, volume);
    block(&mut bytes, &volume.binding);
    block(&mut bytes, &volume.directions);
    block(&mut bytes, &volume.measures);
    bytes
}
fn sign_words(bytes: &mut Vec<u8>, factors: VolumeSignFactors) {
    for sign in [
        factors.candidate_facing,
        factors.boundary_traversal,
        factors.increasing_parameter,
        factors.jacobian,
    ] {
        bytes.push(sign as u8);
    }
    bytes.extend_from_slice(&factors.selected_winding.to_le_bytes());
}
fn scalar_words(bytes: &mut Vec<u8>, volume: &JoinedPrismVolume) {
    for value in [
        volume.enclosure[0],
        volume.enclosure[1],
        volume.representative,
        volume.absolute_error_bound(),
    ] {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
}
fn block(bytes: &mut Vec<u8>, payload: &[u8]) {
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(payload);
}
