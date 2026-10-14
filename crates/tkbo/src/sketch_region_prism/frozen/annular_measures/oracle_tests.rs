// SPDX-License-Identifier: GPL-2.0-only
//! Exact rational transforms of the independently retained closed-cubic side enclosure.
use super::*;
use crate::sketch_region_prism::frozen::annular_orientation::fixture::{Variant, with_variant};

fn interval(quantity: &str) -> [f64; 2] {
    let fixture = include_str!("../../../../tests/fixtures/annular_measures/material.tsv");
    let fields: Vec<_> = fixture
        .lines()
        .find(|line| line.starts_with(&format!("{quantity}\t")))
        .unwrap()
        .split('\t')
        .collect();
    [
        f64::from_bits(fields[1].parse().unwrap()),
        f64::from_bits(fields[2].parse().unwrap()),
    ]
}
fn encloses(actual: [f64; 2], expected: [f64; 2]) {
    assert!(
        actual[0] <= expected[0] && expected[1] <= actual[1],
        "actual {actual:?} must enclose independent {expected:?}"
    );
}
fn assert_material(report: &AnnularPrismMeasures, negative: bool) {
    assert_eq!(report.faces().len(), 4);
    for (face, quantity) in report
        .faces()
        .iter()
        .zip(["cap", "cap", "outer_side", "inner_side"])
    {
        encloses(face.measure.enclosure(), interval(quantity));
    }
    encloses(report.total_area().enclosure(), interval("total"));
    assert_sweep(report, negative);
    assert!(
        report.total_area().absolute_error_bound()
            <= report.requested().total_area.absolute_error_limit
    );
}
fn assert_sweep(report: &AnnularPrismMeasures, negative: bool) {
    let sweep = interval("signed_sweep");
    assert_eq!(
        report.signed_sweep().enclosure(),
        if negative {
            [-sweep[1], -sweep[0]]
        } else {
            sweep
        }
    );
}
#[test]
fn independent_total_and_signed_material_bounds_survive_all_direction_generators() {
    for variant in variants() {
        with_variant(variant, |join, _| {
            let report = measured(join);
            assert_material(&report, variant.reflected ^ variant.negative_sweep);
            assert_named_rows(join, &report);
            assert_eq!(
                report
                    .faces()
                    .iter()
                    .map(|r| r.named.chart.trims.len())
                    .sum::<usize>(),
                12
            );
            assert_eq!(super::super::replay::encode(&report), report.replay_bytes());
        });
    }
}
fn variants() -> Vec<Variant> {
    let mut rows = vec![Variant::default()];
    for mask in [[true, false], [false, true], [true, true]] {
        rows.push(Variant {
            stored_reverse: mask,
            ..Default::default()
        });
        rows.push(Variant {
            traversed_reverse: mask,
            ..Default::default()
        });
    }
    placed_variants(&mut rows);
    rows
}
fn placed_variants(rows: &mut Vec<Variant>) {
    for (reflected, negative_sweep) in [(true, false), (false, true), (true, true)] {
        rows.push(Variant {
            reflected,
            negative_sweep,
            ..Default::default()
        });
        rows.push(Variant {
            reflected,
            negative_sweep,
            stored_reverse: [true, false],
            traversed_reverse: [false, true],
        });
    }
}

#[test]
fn source_request_reordering_preserves_material_quantities_and_named_allocation_order() {
    use crate::sketch_region_prism::frozen::annular_orientation::fixture::with_source_order;
    let forward = with_source_order(Variant::default(), false, |join, _| measured(join));
    let reverse = with_source_order(Variant::default(), true, |join, _| measured(join));
    assert_material(&reverse, false);
    assert_eq!(forward.faces(), reverse.faces());
    assert_eq!(forward.side_subdivisions(), reverse.side_subdivisions());
    assert_eq!(forward.signed_sweep(), reverse.signed_sweep());
    assert_eq!(forward.total_area(), reverse.total_area());
}

fn assert_named_rows(join: &AnnularFrozenDomain, report: &AnnularPrismMeasures) {
    for named in join.faces() {
        let found: Vec<_> = report
            .faces()
            .iter()
            .filter(|row| &row.named == named)
            .collect();
        assert_eq!(
            found.len(),
            1,
            "one complete exact named face/trim/domain row"
        );
        assert_eq!(&found[0].named, named);
    }
}
