// SPDX-License-Identifier: GPL-2.0-only
//! Reviewed retained construction is shared; expected closure points and sweep limits are authored here.
use super::{tests::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::StoredToleranceCap,
    frozen::annular_orientation::fixture::{Variant, with_source_order, with_variant},
};
#[test]
fn reflected_reversed_and_negative_sweep_annuli_keep_exact_conformance() {
    for variant in variants() {
        with_variant(variant, |join, _| assert_variant(join, variant));
    }
}
fn assert_variant(join: &AnnularFrozenDomain, variant: Variant) {
    let proof = conformance(join);
    let expected_z: f64 = if variant.negative_sweep { -2. } else { 2. };
    assert_eq!(
        proof.operation_limits().extrusion[2].to_bits(),
        expected_z.to_bits()
    );
    assert_variant_closures(&proof, variant);
    let metadata = certify_annular_prism_tolerances(
        join,
        &proof,
        StoredToleranceCap::report(0.).unwrap(),
        &mut Continue,
    )
    .ok()
    .unwrap();
    assert_eq!(metadata.metadata().len(), 14);
    assert!(!join.projection().is_valid_body());
}
fn variants() -> [Variant; 13] {
    [
        variant([false, false], [false, false], false, false),
        variant([true, false], [false, false], false, false),
        variant([false, true], [false, false], false, false),
        variant([true, true], [false, false], false, false),
        variant([false, false], [true, false], false, false),
        variant([false, false], [false, true], false, false),
        variant([false, false], [true, true], false, false),
        variant([false, false], [false, false], true, false),
        variant([false, false], [false, false], false, true),
        variant([false, false], [false, false], true, true),
        variant([true, false], [false, true], false, false),
        variant([true, true], [true, true], false, false),
        variant([true, true], [true, true], true, true),
    ]
}
fn variant(
    stored_reverse: [bool; 2],
    traversed_reverse: [bool; 2],
    reflected: bool,
    negative_sweep: bool,
) -> Variant {
    Variant {
        stored_reverse,
        traversed_reverse,
        reflected,
        negative_sweep,
    }
}
#[test]
fn source_request_row_order_does_not_choose_boundary_identity() {
    for reversed in [false, true] {
        with_source_order(Variant::default(), reversed, |join, _| {
            let proof = conformance(join);
            let mut sources: Vec<_> = proof
                .evidence()
                .source_reports
                .iter()
                .map(|row| row.source.0)
                .collect();
            sources.sort_unstable();
            assert_eq!(sources, [101, 101, 101, 202, 202, 202]);
            assert_eq!(proof.evidence().closures.len(), 20);
        });
    }
}

fn assert_variant_closures(proof: &AnnularPrismConformance, variant: Variant) {
    for closure in &proof.evidence().closures {
        let x = if variant.reflected { -0.5 } else { 0.5 };
        let expected = if closure.context.source.0 == 101 {
            [0.; 3]
        } else {
            [x, 0.5, 0.]
        };
        assert_eq!(closure.local_point, expected);
        assert!(
            closure
                .certificate
                .proven_squared_upper
                .compare_f64(0.)
                .unwrap()
                .is_eq()
        );
    }
}
