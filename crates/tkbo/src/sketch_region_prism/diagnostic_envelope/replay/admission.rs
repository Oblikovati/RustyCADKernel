// SPDX-License-Identifier: GPL-2.0-only
//! V1 admission operands retain caller identities and exact refused floating-point bits.
use super::super::*;
use crate::sketch_region_prism::frozen::encode_frozen_numbers as numbers;
pub(super) fn append(value: &OwnedPrismAdmissionCause, bytes: &mut Vec<u8>) {
    bytes.push(tag(value));
    counts(value, bytes);
    geometry(value, bytes);
    if let OwnedPrismAdmissionCause::Names(value) = value {
        names(value, bytes);
    }
}
fn tag(value: &OwnedPrismAdmissionCause) -> u8 {
    use OwnedPrismAdmissionCause::*;
    match value {
        EmptyInput => 0,
        UnsupportedVersion { .. } => 1,
        MultipleBoundaries { .. } => 2,
        PoleCount { .. } => 3,
        NonfinitePole { .. } => 4,
        OpenBoundary { .. } => 5,
        InvalidExtrusion { .. } => 6,
        NonfinitePlane { .. } => 7,
        Names(_) => 8,
    }
}
fn counts(value: &OwnedPrismAdmissionCause, bytes: &mut Vec<u8>) {
    use OwnedPrismAdmissionCause::*;
    match value {
        UnsupportedVersion { actual, expected } => {
            bytes.extend(actual.to_le_bytes());
            bytes.extend(expected.to_le_bytes());
        }
        MultipleBoundaries { actual, supported } => boundaries(actual, *supported, bytes),
        _ => pole_count(value, bytes),
    }
}
fn boundaries(actual: &[CallerBoundaryId], supported: usize, bytes: &mut Vec<u8>) {
    words(&[actual.len() as u64], bytes);
    for value in actual {
        words(&[value.0], bytes);
    }
    words(&[supported as u64], bytes);
}
fn pole_count(value: &OwnedPrismAdmissionCause, bytes: &mut Vec<u8>) {
    if let OwnedPrismAdmissionCause::PoleCount {
        source,
        actual,
        minimum,
        maximum,
    } = value
    {
        words(
            &[source.0, *actual as u64, *minimum as u64, *maximum as u64],
            bytes,
        );
    }
}
fn geometry(value: &OwnedPrismAdmissionCause, bytes: &mut Vec<u8>) {
    pole_scalar(value, bytes);
    boundary_points(value, bytes);
    plane_scalar(value, bytes);
}
fn pole_scalar(value: &OwnedPrismAdmissionCause, bytes: &mut Vec<u8>) {
    if let OwnedPrismAdmissionCause::NonfinitePole {
        source,
        pole,
        axis,
        actual,
    } = value
    {
        words(&[source.0, *pole as u64, *axis as u64], bytes);
        numbers([*actual], bytes);
    }
}
fn boundary_points(value: &OwnedPrismAdmissionCause, bytes: &mut Vec<u8>) {
    match value {
        OwnedPrismAdmissionCause::OpenBoundary {
            source,
            first,
            last,
        } => {
            words(&[source.0], bytes);
            numbers(*first, bytes);
            numbers(*last, bytes);
        }
        OwnedPrismAdmissionCause::InvalidExtrusion { actual } => numbers(*actual, bytes),
        _ => {}
    }
}
fn plane_scalar(value: &OwnedPrismAdmissionCause, bytes: &mut Vec<u8>) {
    if let OwnedPrismAdmissionCause::NonfinitePlane {
        component,
        axis,
        actual,
    } = value
    {
        bytes.push(plane(*component));
        words(&[*axis as u64], bytes);
        numbers([*actual], bytes);
    }
}
fn plane(value: DeclaredPlaneComponent) -> u8 {
    match value {
        DeclaredPlaneComponent::Origin => 0,
        DeclaredPlaneComponent::Normal => 1,
        DeclaredPlaneComponent::U => 2,
        DeclaredPlaneComponent::V => 3,
    }
}
fn names(value: &SemanticNameFailure, bytes: &mut Vec<u8>) {
    match value {
        SemanticNameFailure::EqualEndpointUses { actual } => {
            bytes.push(0);
            words(&[actual.0], bytes);
        }
        SemanticNameFailure::NonfiniteSeed { axis, actual } => {
            bytes.push(1);
            words(&[*axis as u64], bytes);
            numbers([*actual], bytes);
        }
        SemanticNameFailure::UnknownToken { actual, version } => {
            bytes.push(2);
            words(&[*actual], bytes);
            bytes.extend(version.to_le_bytes());
        }
    }
}
pub(super) fn words(values: &[u64], bytes: &mut Vec<u8>) {
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
}
