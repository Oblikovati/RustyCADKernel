// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::frozen::annular_orientation::fixture::{Variant, with_variant};

fn fresh_snapshot(
    report: &AnnularPrismMeasures,
) -> (Vec<u8>, Vec<AnnularMeasuredFace>, [f64; 2], [f64; 2]) {
    (
        super::super::replay::encode(report),
        report.faces().to_vec(),
        report.signed_sweep().enclosure(),
        report.total_area().enclosure(),
    )
}
#[test]
fn owned_getters_survive_source_mutation_and_drop() {
    let (report, before) = with_variant(Variant::default(), |join, fixture| {
        let report = measured(join);
        let before = fresh_snapshot(&report);
        let edge = rustycad_tkbrep::topods::edge(&fixture.outer).unwrap();
        crate::sketch_region_prism::isolation_tests::set_pole(&edge, 9.);
        assert_eq!(fresh_snapshot(&report), before);
        (report, before)
    });
    assert_eq!(fresh_snapshot(&report), before);
    assert_eq!(report.replay_bytes(), before.0);
}
#[test]
fn same_owned_domain_replays_identically_across_threads() {
    let join = joined();
    let serial = measured(&join).replay_bytes().to_vec();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| measured(&join).replay_bytes().to_vec()))
            .collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), serial);
        }
    });
    with_variant(
        Variant {
            negative_sweep: true,
            ..Default::default()
        },
        |different, _| {
            assert!(!measured(&join).matches(different));
        },
    );
}
#[test]
fn every_callback_cancel_preserves_owned_input_and_clean_reuse() {
    let join = joined();
    let before = join.replay_bytes().to_vec();
    let mut count = 0;
    let complete = inspect_annular_measures(&join, options(), || {
        count += 1;
        false
    })
    .unwrap();
    let phases: Vec<_> = (1..=count)
        .map(|cutoff| assert_cancel(&join, cutoff))
        .collect();
    assert_cancel_phases(&phases);
    assert_eq!(join.replay_bytes(), before);
    assert_eq!(fresh_snapshot(&measured(&join)), fresh_snapshot(&complete));
    println!("ANNULAR_MEASURE_CHECKPOINTS={count}");
}
fn assert_cancel(join: &AnnularFrozenDomain, cutoff: usize) -> AnnularMeasurePhase {
    let mut calls = 0;
    let failure = inspect_annular_measures(join, options(), || {
        calls += 1;
        calls == cutoff
    })
    .err()
    .unwrap();
    assert_eq!(calls, cutoff);
    assert_eq!(
        failure.classification(),
        JoinedMeasureFailureClass::Cancelled
    );
    assert_eq!(failure.binding.as_ref(), join.replay_bytes());
    assert!(failure.partial.side_subdivisions <= options().maximum_side_subdivisions);
    assert_partial_work(&failure);
    failure.phase
}
fn assert_partial_work(failure: &AnnularMeasureFailure) {
    if let AnnularMeasureCause::Side(terminal) = &failure.cause {
        let completed: u32 = failure
            .partial
            .faces
            .iter()
            .map(|face| match &face.measure {
                AnnularFaceMeasure::Side { report, .. } => report.work().subdivisions,
                _ => 0,
            })
            .sum();
        assert_eq!(
            failure.partial.side_subdivisions,
            completed + terminal.work.subdivisions
        );
    }
}

#[test]
fn callback_source_edits_do_not_alias_the_sealed_measure_operands() {
    with_variant(Variant::default(), |join, fixture| {
        let before = fresh_snapshot(&measured(join));
        let edge = rustycad_tkbrep::topods::edge(&fixture.inner).unwrap();
        let mut changed = false;
        let report = inspect_annular_measures(join, options(), || {
            if !changed {
                crate::sketch_region_prism::isolation_tests::set_pole(&edge, 7.);
                changed = true;
            }
            false
        })
        .unwrap();
        assert!(changed);
        assert_eq!(fresh_snapshot(&report), before);
    });
}
#[test]
fn failed_partial_rows_survive_source_drop_and_remain_typed_incomplete() {
    let failure = with_variant(Variant::default(), |join, _| {
        let mut requested = options();
        requested.total_area.absolute_error_limit = 0.;
        inspect_annular_measures(join, requested, || false)
            .err()
            .unwrap()
    });
    assert_eq!(
        failure.classification(),
        JoinedMeasureFailureClass::Incomplete
    );
    assert_eq!(failure.partial.faces.len(), 4);
    assert!(failure.partial.signed_sweep.is_some());
    assert!(failure.partial.total.is_none());
    assert_eq!(failure.partial.side_subdivisions, 77);
    assert_eq!(
        failure.partial.faces[0].measure.enclosure(),
        [
            f64::from_bits(0x3ffccccccccccccc),
            f64::from_bits(0x3ffccccccccccccd)
        ]
    );
}

fn assert_cancel_phases(phases: &[AnnularMeasurePhase]) {
    for phase in [
        AnnularMeasurePhase::Admission,
        AnnularMeasurePhase::Caps,
        AnnularMeasurePhase::Sweep,
        AnnularMeasurePhase::Sides,
        AnnularMeasurePhase::Total,
        AnnularMeasurePhase::Commit,
    ] {
        assert!(
            phases.contains(&phase),
            "expected cancellation at {phase:?}"
        );
    }
    assert_eq!(phases.last(), Some(&AnnularMeasurePhase::Commit));
}
