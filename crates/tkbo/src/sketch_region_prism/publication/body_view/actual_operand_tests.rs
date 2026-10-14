// SPDX-License-Identifier: GPL-2.0-only
//! Every actual accepted geometry and metadata operand is compared without evaluation or rounding.
use super::*;
use crate::sketch_region_prism::{exact_tolerance::*, frozen::*, owned_request::*};
use crate::sketch_region_source_conformance::encoding::{curve, numbers, space};

#[test]
fn all_actual_projected_member_and_face_operands_preserve_exact_bytes() {
    let fixture = default_resolved_prism_fixture();
    let body = project_named_prism_body(&fixture.publication, &fixture.names).unwrap();
    let projection = fixture.publication.join().projection();
    for (source, row) in projection.nodes().iter().zip(body.members()) {
        assert_eq!(row.key(), fixture.names.member(&source.key).unwrap());
        assert_member_operands(source, row);
        assert_tolerance(
            row,
            source,
            fixture.publication.validation().tolerance().metadata(),
        );
    }
    for (source, row) in projection.faces().iter().zip(body.faces()) {
        assert_face_operands(source, row);
    }
}
fn assert_member_operands(source: &FrozenPrismNode, row: &PrismBodyMember) {
    assert_eq!(
        source.point.map(|p| p.map(f64::to_bits)),
        row.rounded_world_point().map(|p| p.map(f64::to_bits))
    );
    let mut expected = Vec::new();
    let mut actual = Vec::new();
    if let Some(value) = &source.curve {
        space(value, &mut expected);
    }
    if let Some(value) = &row.curve {
        space(value, &mut actual);
    }
    assert_eq!(actual, expected);
}
fn assert_tolerance(
    row: &PrismBodyMember,
    source: &FrozenPrismNode,
    observations: &[JoinedToleranceMetadata],
) {
    let expected: Vec<_> = observations
        .iter()
        .filter(|m| m.entity == source.key)
        .collect();
    assert_eq!(expected.len(), usize::from(row.tolerance().is_some()));
    if let Some(actual) = row.tolerance() {
        let mut expected_bytes = Vec::new();
        let mut actual_bytes = Vec::new();
        encode_stored_tolerance_observation(expected[0].observation, &mut expected_bytes);
        encode_stored_tolerance_observation(actual, &mut actual_bytes);
        assert_eq!(actual_bytes, expected_bytes);
    }
}
fn assert_face_operands(source: &FrozenPrismFace, row: &PrismBodyFace) {
    let mut expected = Vec::new();
    let mut actual = Vec::new();
    encode_frozen_surface(&source.surface, &mut expected);
    encode_frozen_surface(&row.surface, &mut actual);
    assert_eq!(actual, expected);
    assert_eq!(source.trims.len(), row.trims().len());
    for (source, projected) in source.trims.iter().zip(row.trims()) {
        assert_trim_operands(source, projected);
    }
}
fn assert_trim_operands(source: &FrozenPrismTrim, row: &PrismBodyTrim) {
    let mut expected = Vec::new();
    let mut actual = Vec::new();
    curve(&source.pcurve, &mut expected);
    curve(&row.pcurve, &mut actual);
    space(&source.space, &mut expected);
    space(&row.space, &mut actual);
    numbers(source.interval, &mut expected);
    numbers(row.native_interval(), &mut actual);
    assert_eq!(source.role, row.role());
    assert_eq!(actual, expected);
}
