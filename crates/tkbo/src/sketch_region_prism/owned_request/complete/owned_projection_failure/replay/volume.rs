// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::frozen::joined_volume::*;
pub(super) fn append(value: &JoinedPrismVolumeFailure, bytes: &mut Vec<u8>) {
    words(&[value.operation.0], bytes);
    bytes.extend(value.requested_version.to_le_bytes());
    words(
        &[
            value.entity.source,
            value.entity.member.0,
            value.entity.directed_use.0,
        ],
        bytes,
    );
    class(value.class, bytes);
    cause(&value.cause, bytes);
}
pub(super) fn class(value: JoinedVolumeFailureClass, bytes: &mut Vec<u8>) {
    match value {
        JoinedVolumeFailureClass::Incomplete => bytes.push(0),
        JoinedVolumeFailureClass::Cancelled => bytes.push(1),
    }
}
fn cause(value: &JoinedPrismVolumeCause, bytes: &mut Vec<u8>) {
    use JoinedPrismVolumeCause::*;
    bytes.push(cause_tag(value));
    match value {
        Version { expected } => bytes.extend(expected.to_le_bytes()),
        Binding(value) => bytes.push(operand_tag(*value)),
        Cancelled(value) => bytes.push(checkpoint_tag(*value)),
        SignFactors(value) => factors(*value, bytes),
        ScalarContradiction {
            signed_enclosure,
            exact_sign,
        } => scalar(*signed_enclosure, *exact_sign, bytes),
    }
}
fn cause_tag(value: &JoinedPrismVolumeCause) -> u8 {
    match value {
        JoinedPrismVolumeCause::Version { .. } => 0,
        JoinedPrismVolumeCause::Binding(_) => 1,
        JoinedPrismVolumeCause::Cancelled(_) => 2,
        JoinedPrismVolumeCause::SignFactors(_) => 3,
        JoinedPrismVolumeCause::ScalarContradiction { .. } => 4,
    }
}
fn operand_tag(value: VolumeOperand) -> u8 {
    match value {
        VolumeOperand::Orientation => 0,
        VolumeOperand::Measures => 1,
    }
}
fn checkpoint_tag(value: VolumeCheckpoint) -> u8 {
    match value {
        VolumeCheckpoint::Begin => 0,
        VolumeCheckpoint::Finish => 1,
    }
}
fn scalar(enclosure: [f64; 2], sign: i8, bytes: &mut Vec<u8>) {
    words(&enclosure.map(f64::to_bits), bytes);
    bytes.push(sign as u8);
}
fn factors(value: VolumeSignFactors, bytes: &mut Vec<u8>) {
    bytes.extend([
        value.candidate_facing as u8,
        value.boundary_traversal as u8,
        value.increasing_parameter as u8,
        value.jacobian as u8,
    ]);
    bytes.extend(value.selected_winding.to_le_bytes());
}
