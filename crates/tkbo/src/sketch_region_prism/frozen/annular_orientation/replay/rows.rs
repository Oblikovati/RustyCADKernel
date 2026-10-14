// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn boundary(bytes: &mut Vec<u8>, value: &AnnularBoundaryDirection) {
    word(bytes, value.source.0);
    bytes.extend(value.traversed_winding.to_le_bytes());
    bytes.extend([
        u8::from(value.reversed),
        value.native_winding as u8,
        value.material_side as u8,
    ]);
}
pub(super) fn face(bytes: &mut Vec<u8>, value: &AnnularFaceDirection) {
    named_face(bytes, &value.named.chart);
    domain(bytes, &value.named.domain);
    dispositions(bytes, &value.parents);
    direction(bytes, value.actual);
    direction(bytes, value.expected);
    word(bytes, value.trims.len() as u64);
    for trim in &value.trims {
        conclusion(bytes, trim);
    }
}
fn named_face(bytes: &mut Vec<u8>, value: &JoinedPrismFace) {
    member(bytes, value.definition);
    occurrence(bytes, &value.occurrence);
    word(bytes, value.source.0);
    image_role(bytes, value.role);
    parents(bytes, &value.parents);
    word(bytes, value.trims.len() as u64);
    for trim in &value.trims {
        named_trim(bytes, trim);
    }
}
fn domain(bytes: &mut Vec<u8>, value: &NamedAnnularDomain) {
    match value {
        NamedAnnularDomain::Cap { outer, inner } => {
            bytes.push(1);
            wire(bytes, outer);
            wire(bytes, inner);
        }
        NamedAnnularDomain::Side { boundary } => {
            bytes.push(2);
            wire(bytes, boundary);
        }
    }
}
fn wire(bytes: &mut Vec<u8>, value: &NamedAnnularWire) {
    word(bytes, value.source.0);
    occurrence(bytes, &value.occurrence);
}
fn named_trim(bytes: &mut Vec<u8>, value: &JoinedPrismTrim) {
    occurrence(bytes, &value.occurrence);
    word(bytes, value.source.0);
    trim_role(bytes, value.role);
    parents(bytes, &value.parents);
    encoding::numbers(value.interval, bytes);
}
fn conclusion(bytes: &mut Vec<u8>, value: &AnnularTrimDirection) {
    named_trim(bytes, &value.named);
    word(bytes, value.boundary.0);
    dispositions(bytes, &value.parents);
    direction(bytes, value.actual);
    direction(bytes, value.expected);
}
