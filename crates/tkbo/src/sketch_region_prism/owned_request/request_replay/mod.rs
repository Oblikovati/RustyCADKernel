// SPDX-License-Identifier: GPL-2.0-only
//! Full pre-admission operands; refused NaN payloads and source multiplicity remain evidence.
mod annular;
mod policy;
mod publication;
use super::{names::SemanticSelectionIntent, request::*};
use crate::sketch_region_prism::frozen::encode_frozen_numbers as numbers;
pub(in crate::sketch_region_prism) use annular::encode_owned_annular_request;
pub(crate) use publication::encode_publication_request_options;
/// Encode every original input and limit; `encode_owned_prism_request(&request)` performs no admission.
pub(crate) fn encode_owned_prism_request(request: &OwnedPolynomialPrismRequest) -> Vec<u8> {
    let mut bytes = b"Martensite.OwnedPolynomialPrismRequest/v1\0".to_vec();
    bytes.extend(request.version.to_le_bytes());
    word(request.operation.0, &mut bytes);
    word(request.sources.len() as u64, &mut bytes);
    for source in &request.sources {
        boundary(source, &mut bytes);
    }
    frame(request.declared_plane, &mut bytes);
    selection(request.selection, &mut bytes);
    numbers(request.extrusion, &mut bytes);
    policy::encode(request.policy, &mut bytes);
    bytes
}
fn boundary(source: &OwnedPolynomialBoundary, bytes: &mut Vec<u8>) {
    for value in [
        source.source.0,
        source.revision.0,
        source.closure.0,
        source.native_start.0,
        source.native_end.0,
    ] {
        word(value, bytes);
    }
    bytes.push(u8::from(source.reversed_traversal));
    word(source.world_poles.len() as u64, bytes);
    for pole in &source.world_poles {
        numbers(*pole, bytes);
    }
}
fn frame(plane: rustycad_tkmath::gp::Plane3, bytes: &mut Vec<u8>) {
    let axes = plane.position();
    for point in [
        axes.location().coordinates(),
        axes.direction().coordinates(),
        axes.x_direction().coordinates(),
        axes.y_direction().coordinates(),
    ] {
        numbers(*point, bytes);
    }
}
fn selection(intent: SemanticSelectionIntent, bytes: &mut Vec<u8>) {
    let SemanticSelectionIntent::BoundedRegionContaining { seed } = intent;
    bytes.push(1);
    numbers(seed, bytes);
}
fn word(value: u64, bytes: &mut Vec<u8>) {
    let encoded = value.to_le_bytes();
    bytes.extend(encoded);
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod operand_tests;
#[cfg(test)]
mod policy_tests;
