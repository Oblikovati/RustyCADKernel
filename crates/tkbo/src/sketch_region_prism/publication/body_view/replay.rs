// SPDX-License-Identifier: GPL-2.0-only
//! New V1 body bytes reuse existing scalar/key encoders and retain the full naming/publication binding.
use super::{result::*, rows::*};
use crate::sketch_region_prism::{
    exact_tolerance::encode_stored_tolerance_observation, frozen::encode_frozen_surface,
    owned_request::names::*,
};
use crate::sketch_region_source_conformance::encoding::*;

pub(super) fn encode(body: &OwnedNamedPrismBody) -> Vec<u8> {
    let mut bytes = b"Martensite.NamedPrismBody\0".to_vec();
    bytes.extend(body.version().to_le_bytes());
    block(&body.naming_binding, &mut bytes);
    let names = SemanticReplayEnvelope::new(body.plan.clone(), &[]);
    block(names.replay_bytes(), &mut bytes);
    encode_semantic_use_key(&mut bytes, &body.root);
    append_rows(&body.rows, &mut bytes);
    bytes
}
pub(super) fn append_rows(rows: &PrismBodyRows, bytes: &mut Vec<u8>) {
    append(&rows.members, bytes, member);
    append(&rows.occurrences, bytes, occurrence);
    append(&rows.faces, bytes, face);
    append(&rows.endpoints, bytes, endpoint);
    append(&rows.coedges, bytes, coedge);
    append(&rows.wires, bytes, wire);
}
fn append<T>(rows: &[T], bytes: &mut Vec<u8>, encode: fn(&T, &mut Vec<u8>)) {
    bytes.extend((rows.len() as u64).to_le_bytes());
    for row in rows {
        encode(row, bytes);
    }
}
fn block(value: &[u8], bytes: &mut Vec<u8>) {
    bytes.extend((value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}
pub(super) fn member(row: &PrismBodyMember, bytes: &mut Vec<u8>) {
    encode_semantic_member_key(bytes, &row.key);
    bytes.push(u8::from(row.output_slot.is_some()));
    if let Some(slot) = row.output_slot {
        bytes.extend([slot.stage as u8, slot.role as u8]);
    }
    bytes.push(u8::from(row.rounded_world_point.is_some()));
    if let Some(point) = row.rounded_world_point {
        numbers(point, bytes);
    }
    bytes.push(u8::from(row.curve.is_some()));
    if let Some(value) = &row.curve {
        space(value, bytes);
    }
    bytes.push(u8::from(row.tolerance.is_some()));
    if let Some(value) = row.tolerance {
        encode_stored_tolerance_observation(value, bytes);
    }
}
pub(super) fn occurrence(row: &PrismBodyOccurrence, bytes: &mut Vec<u8>) {
    encode_semantic_use_key(bytes, &row.key);
    bytes.push(row.orientation as u8);
    placement(row.placement, bytes);
    append(&row.children, bytes, |key, bytes| {
        encode_semantic_use_key(bytes, key)
    });
    bytes.push(u8::from(row.local_vertex.is_some()));
    if let Some(point) = row.local_vertex {
        numbers(point, bytes);
    }
}
pub(super) fn face(row: &PrismBodyFace, bytes: &mut Vec<u8>) {
    encode_semantic_member_key(bytes, &row.key);
    encode_frozen_surface(&row.surface, bytes);
    append(&row.trims, bytes, trim);
}
pub(super) fn trim(row: &PrismBodyTrim, bytes: &mut Vec<u8>) {
    encode_semantic_use_key(bytes, &row.occurrence);
    bytes.push(row.role as u8);
    curve(&row.pcurve, bytes);
    numbers(row.native_interval, bytes);
    space(&row.space, bytes);
}
pub(super) fn endpoint(row: &PrismBodyEdgeEndpoints, bytes: &mut Vec<u8>) {
    encode_semantic_member_key(bytes, &row.edge);
    encode_semantic_member_key(bytes, &row.first);
    encode_semantic_member_key(bytes, &row.last);
}
pub(super) fn coedge(row: &PrismBodyCoedge, bytes: &mut Vec<u8>) {
    encode_semantic_use_key(bytes, &row.occurrence);
    encode_semantic_member_key(bytes, &row.face);
    encode_semantic_member_key(bytes, &row.wire);
    encode_semantic_member_key(bytes, &row.edge);
    bytes.push(u8::from(row.reversed));
}
pub(super) fn wire(row: &PrismBodyWireTraversal, bytes: &mut Vec<u8>) {
    encode_semantic_use_key(bytes, &row.occurrence);
    append(&row.coedges, bytes, |key, bytes| {
        encode_semantic_use_key(bytes, key)
    });
}
