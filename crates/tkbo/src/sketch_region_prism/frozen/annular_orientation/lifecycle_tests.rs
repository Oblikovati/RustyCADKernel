// SPDX-License-Identifier: GPL-2.0-only
use super::super::annular_join::operation_tests::joined;
use super::*;
fn certify(join: &AnnularFrozenDomain) -> AnnularOrientationCertificate {
    inspect_annular_orientation(join, AnnularOrientationOptions::v2(), || false)
        .unwrap_or_else(|_| panic!("actual orientation"))
}
#[test]
fn version_admission_precedes_callbacks_and_retains_exact_policy() {
    let join = joined();
    let failure =
        inspect_annular_orientation(&join, AnnularOrientationOptions { version: 99 }, || {
            panic!("unsupported version callback")
        })
        .err()
        .unwrap();
    assert_eq!(failure.requested.version, 99);
    assert_eq!(failure.phase, AnnularDirectionPhase::Admission);
    assert_eq!(failure.classification(), TwoLoopRefusalClass::Incomplete);
    assert_eq!(&*failure.source_binding, join.replay_bytes());
    assert!(matches!(
        failure.cause,
        AnnularDirectionCause::Version {
            actual: 99,
            expected: 2
        }
    ));
}
#[test]
fn every_checkpoint_cancels_without_partial_certificate_and_allows_reuse() {
    let join = joined();
    let expected = certify(&join).replay_bytes();
    let mut total = 0;
    assert!(
        inspect_annular_orientation(&join, AnnularOrientationOptions::v2(), || {
            total += 1;
            false
        })
        .is_ok()
    );
    assert_eq!(total, 6);
    for cutoff in 1..=total {
        assert_cutoff(&join, cutoff, &expected);
    }
}
fn assert_cutoff(join: &AnnularFrozenDomain, cutoff: usize, expected: &[u8]) {
    let mut calls = 0;
    let failure = inspect_annular_orientation(join, AnnularOrientationOptions::v2(), || {
        calls += 1;
        calls == cutoff
    })
    .err()
    .unwrap();
    assert_eq!(calls, cutoff);
    assert_eq!(failure.classification(), TwoLoopRefusalClass::Cancelled);
    assert_eq!(
        failure.phase,
        match cutoff {
            1 => AnnularDirectionPhase::Admission,
            6 => AnnularDirectionPhase::Commit,
            _ => AnnularDirectionPhase::Directions,
        }
    );
    assert_eq!(certify(join).replay_bytes(), expected);
}
#[test]
fn owned_getters_and_recomputed_rows_survive_native_source_mutation_and_drop() {
    let proof =
        super::fixture::with_variant(super::fixture::Variant::default(), |join, fixture| {
            let proof = certify(join);
            let before = proof.replay_bytes();
            let rows = proof.faces().to_vec();
            let edge = rustycad_tkbrep::topods::edge(&fixture.outer).unwrap();
            crate::sketch_region_prism::isolation_tests::set_pole(&edge, 9.);
            assert_eq!(proof.faces(), rows);
            assert_eq!(proof.replay_bytes(), before);
            proof
        });
    assert_eq!(proof.faces().len(), 4);
    assert_eq!(proof.boundaries()[0].native_winding, 1);
    assert!(!proof.replay_bytes().is_empty());
}
#[test]
fn concurrent_readers_and_independent_operations_have_identical_full_bytes() {
    let join = std::sync::Arc::new(joined());
    let expected = certify(&join).replay_bytes();
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let join = join.clone();
            std::thread::spawn(move || {
                (0..8)
                    .map(|_| certify(&join).replay_bytes())
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    for thread in threads {
        for actual in thread.join().unwrap() {
            assert_eq!(actual, expected);
        }
    }
    assert_eq!(certify(&joined()).replay_bytes(), expected);
}
#[test]
fn mismatched_actual_orientation_and_missing_full_path_are_typed() {
    let join = joined();
    let row = &join.faces()[0].chart.occurrence;
    let actual = lookup::observed(&join, row).unwrap();
    let expected = if actual == Orientation::Forward {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    assert!(matches!(
        lookup::check_direction(&join, row, expected),
        Err(AnnularDirectionCause::Direction { .. })
    ));
    assert_missing_paths(&join, row);
}
#[test]
fn different_actual_sweep_does_not_bind_nominal_certificate() {
    let proof = certify(&joined());
    super::fixture::with_variant(
        super::fixture::Variant {
            negative_sweep: true,
            ..Default::default()
        },
        |other, _| assert!(!proof.binds(other)),
    );
}
#[test]
fn annular_orientation_replay_probe() {
    println!(
        "ANNULAR_ORIENTATION_REPLAY={:?}",
        certify(&joined()).replay_bytes()
    );
}

fn assert_missing_paths(join: &AnnularFrozenDomain, row: &FrozenUseKey) {
    assert!(matches!(
        lookup::observed(join, &FrozenUseKey(vec![])),
        Err(AnnularDirectionCause::Count {
            actual: 0,
            expected: 1,
            ..
        })
    ));
    assert!(lookup::distinct(vec![row, row], 2).is_err());
}
