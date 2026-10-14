// SPDX-License-Identifier: GPL-2.0-only
use super::fixture::{Variant, with_variant};
use super::*;
#[test]
fn actual_nominal_parameterized_fixture_preserves_existing_join_bytes() {
    let old = super::super::annular_join::operation_tests::joined();
    with_variant(Variant::default(), |new, _| {
        assert_eq!(new.replay_bytes(), old.replay_bytes())
    });
}
#[test]
fn actual_stored_reversal_is_distinct_from_traversal_reversal() {
    for (stored_reverse, traversed_reverse) in [
        ([true, false], [false, false]),
        ([false, true], [false, false]),
        ([true, true], [false, false]),
        ([false, false], [true, false]),
        ([false, false], [false, true]),
        ([false, false], [true, true]),
    ] {
        let variant = Variant {
            stored_reverse,
            traversed_reverse,
            ..Variant::default()
        };
        with_variant(variant, |join, _| assert_variant(join, variant));
    }
}
#[test]
fn actual_reflection_and_negative_sweep_change_exact_jacobian_sign() {
    for (reflected, negative_sweep) in [(true, false), (false, true), (true, true)] {
        let variant = Variant {
            reflected,
            negative_sweep,
            ..Variant::default()
        };
        with_variant(variant, |join, _| assert_variant(join, variant));
    }
}
fn assert_variant(join: &AnnularFrozenDomain, variant: Variant) {
    let proof = inspect_annular_orientation(join, AnnularOrientationOptions::v2(), || false)
        .unwrap_or_else(|failure| panic!("variant orientation: {:?}", failure.cause));
    assert_eq!(
        proof.jacobian_sign(),
        if variant.reflected ^ variant.negative_sweep {
            -1
        } else {
            1
        }
    );
    assert_boundary_provenance(&proof, variant);
    assert!(proof.binds(join));
    assert_eq!(
        proof
            .faces()
            .iter()
            .map(|row| row.trims().len())
            .sum::<usize>(),
        12
    );
}

fn assert_boundary_provenance(proof: &AnnularOrientationCertificate, variant: Variant) {
    for (index, boundary) in proof.boundaries().iter().enumerate() {
        assert_eq!(
            boundary.native_winding,
            if variant.stored_reverse[index] { -1 } else { 1 }
        );
        assert_eq!(boundary.reversed, variant.traversed_reverse[index]);
    }
}
