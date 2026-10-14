// SPDX-License-Identifier: GPL-2.0-only
//! Literal signs are from analytic material normals at t=1/2, not the implementation's q/J helpers.
use super::fixture::{Variant, with_variant};
use super::*;
use Orientation::{Forward as F, Reversed as R};
#[test]
fn actual_stored_reversals_match_independent_four_face_and_cap_rim_tables() {
    for (stored_reverse, faces, rims) in [
        ([false, false], [R, F, R, F], [R, F, F, R]),
        ([true, false], [R, F, F, F], [F, F, R, R]),
        ([false, true], [R, F, R, R], [R, R, F, F]),
        ([true, true], [R, F, F, R], [F, R, R, F]),
    ] {
        assert_tables(
            Variant {
                stored_reverse,
                ..Default::default()
            },
            faces,
            rims,
        );
    }
}
#[test]
fn actual_traversal_reversal_preserves_analytic_material_directions() {
    for traversed_reverse in [[true, false], [false, true], [true, true]] {
        assert_tables(
            Variant {
                traversed_reverse,
                ..Default::default()
            },
            [R, F, R, F],
            [R, F, F, R],
        );
    }
}
#[test]
fn actual_reflection_and_sweep_tables_follow_material_normals() {
    for (reflected, negative_sweep, faces, rims) in [
        (true, false, [F, R, F, R], [F, R, R, F]),
        (false, true, [F, R, F, R], [F, R, R, F]),
        (true, true, [R, F, R, F], [R, F, F, R]),
    ] {
        assert_tables(
            Variant {
                reflected,
                negative_sweep,
                ..Default::default()
            },
            faces,
            rims,
        );
    }
}
fn assert_tables(variant: Variant, faces: [Orientation; 4], rims: [Orientation; 4]) {
    with_variant(variant, |join, _| {
        let proof = inspect_annular_orientation(join, AnnularOrientationOptions::v2(), || false)
            .unwrap_or_else(|failure| panic!("literal orientation {:?}", failure.cause));
        for row in proof.faces() {
            assert_face(row, &faces, &rims);
        }
    });
}
fn face_slot(row: &AnnularFaceDirection) -> usize {
    match &row.named().domain {
        NamedAnnularDomain::Cap { .. } => match row.named().chart.role {
            PrismImageRole::First => 0,
            PrismImageRole::Last => 1,
            _ => panic!("cap role"),
        },
        NamedAnnularDomain::Side { boundary } => match boundary.source.0 {
            101 => 2,
            202 => 3,
            _ => panic!("authored source boundary"),
        },
    }
}
fn assert_face(row: &AnnularFaceDirection, faces: &[Orientation; 4], rims: &[Orientation; 4]) {
    let slot = face_slot(row);
    assert_eq!(row.orientations(), (faces[slot], faces[slot]));
    for trim in row.trims() {
        let expected = if slot < 2 {
            rims[2 * slot + source_slot(trim.boundary())]
        } else {
            side_direction(faces[slot], trim.named().role)
        };
        assert_eq!(trim.orientations(), (expected, expected));
    }
}
fn source_slot(source: RegionSourceId) -> usize {
    match source.0 {
        101 => 0,
        202 => 1,
        _ => panic!("authored source boundary"),
    }
}
fn side_direction(face: Orientation, role: PrismTrimRole) -> Orientation {
    match (face, role) {
        (F, PrismTrimRole::First | PrismTrimRole::End)
        | (R, PrismTrimRole::Last | PrismTrimRole::Start) => R,
        (R, PrismTrimRole::First | PrismTrimRole::End)
        | (F, PrismTrimRole::Last | PrismTrimRole::Start) => F,
        _ => panic!("directed annular side"),
    }
}
