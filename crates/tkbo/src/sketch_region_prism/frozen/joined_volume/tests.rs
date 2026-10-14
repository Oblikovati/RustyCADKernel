// SPDX-License-Identifier: GPL-2.0-only
use super::super::proof_join::operation_tests::JoinFixture;
use super::*;
use crate::sketch_region_prism::region_certificate::{
    CompletePrismMeasureOptions, PrismMeasureOptions,
};
use rustycad_tkmath::predicates::roots::metric_bounds::*;
#[path = "tests_arithmetic.rs"]
mod arithmetic;
#[path = "tests_domains.rs"]
mod domains;

fn children(join: &JoinedPrismProof) -> (JoinedPrismOrientationCertificate, JoinedPrismMeasures) {
    let directions =
        inspect_joined_prism_orientation(join, JoinedPrismOrientationOptions::v1(), || false)
            .ok()
            .unwrap();
    let measures = inspect_joined_prism_measures(join, options(), || false)
        .ok()
        .unwrap();
    (directions, measures)
}
fn options() -> JoinedPrismMeasureOptions {
    JoinedPrismMeasureOptions::v1(CompletePrismMeasureOptions {
        subset: PrismMeasureOptions {
            cap_area: Default::default(),
            signed_volume: Default::default(),
        },
        side_area: PolynomialSideOptions {
            absolute_error_limit: 0.01,
            ..Default::default()
        },
        total_area: EnclosureSumOptions {
            absolute_error_limit: 0.02,
            ..Default::default()
        },
    })
}
fn interpreted(join: &JoinedPrismProof) -> JoinedPrismVolume {
    let (directions, measures) = children(join);
    interpret_joined_prism_volume(join, &directions, &measures, 1, || false).unwrap()
}
#[test]
fn positive_volume_reuses_exact_binding_scalar_error_and_options() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let volume = interpreted(&join);
    assert_eq!(
        volume.enclosure(),
        [
            f64::from_bits(0x4013333333333333),
            f64::from_bits(0x4013333333333334)
        ]
    );
    assert_eq!(volume.representative(), volume.enclosure()[0]);
    assert_policies(&volume);
    assert!(volume.matches_join(&join));
    assert_eq!(volume.binding_bytes(), join.replay_bytes());
    assert!(!join.projection().is_valid_body());
}
fn assert_policies(volume: &JoinedPrismVolume) {
    assert_eq!(
        volume.absolute_error_bound(),
        volume.source_integral().absolute_error_bound()
    );
    assert_eq!(volume.requested(), options().scalars.subset.signed_volume);
    assert_eq!(volume.effective(), volume.requested());
    assert_eq!(
        volume.evidence(),
        JoinedVolumeEvidence::ExactSignTransportOfCertifiedIntegral
    );
}
#[test]
fn version_and_both_foreign_proofs_refuse_before_callbacks() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let other = domains::fixture(false, false, -2.)
        .compose(|| false)
        .ok()
        .unwrap();
    let (directions, measures) = children(&join);
    let (foreign_directions, foreign_measures) = children(&other);
    check_version(&join, &directions, &measures);
    for (d, m, proof) in [
        (&foreign_directions, &measures, VolumeOperand::Orientation),
        (&directions, &foreign_measures, VolumeOperand::Measures),
    ] {
        check_refusal(&join, d, m, 1, JoinedPrismVolumeCause::Binding(proof));
    }
}
fn check_version(
    join: &JoinedPrismProof,
    d: &JoinedPrismOrientationCertificate,
    m: &JoinedPrismMeasures,
) {
    check_refusal(
        join,
        d,
        m,
        9,
        JoinedPrismVolumeCause::Version { expected: 1 },
    );
}
fn check_refusal(
    join: &JoinedPrismProof,
    d: &JoinedPrismOrientationCertificate,
    m: &JoinedPrismMeasures,
    version: u32,
    expected: JoinedPrismVolumeCause,
) {
    let error = interpret_joined_prism_volume(join, d, m, version, || {
        panic!("invalid operands must not poll")
    })
    .err()
    .unwrap();
    assert_eq!(error.cause, expected);
    assert_eq!(error.operation, join.projection().operation);
    assert_eq!(error.entity, *join.original());
    assert_eq!(error.class, JoinedVolumeFailureClass::Incomplete);
}
#[test]
fn both_cancellation_checkpoints_return_no_scalar_and_allow_reuse() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let (directions, measures) = children(&join);
    let expected = interpreted(&join).replay_bytes();
    for (stop, checkpoint) in [(1, VolumeCheckpoint::Begin), (2, VolumeCheckpoint::Finish)] {
        let mut polls = 0;
        let error = interpret_joined_prism_volume(&join, &directions, &measures, 1, || {
            polls += 1;
            polls == stop
        })
        .err()
        .unwrap();
        assert_eq!(error.cause, JoinedPrismVolumeCause::Cancelled(checkpoint));
        assert_eq!(error.class, JoinedVolumeFailureClass::Cancelled);
        assert_eq!(polls, stop);
        assert_eq!(interpreted(&join).replay_bytes(), expected);
    }
}
#[test]
fn owned_volume_survives_all_input_drops_and_is_thread_independent() {
    fn immutable<T: Send + Sync>() {}
    immutable::<JoinedPrismVolume>();
    immutable::<JoinedPrismVolumeFailure>();
    immutable::<VolumeSignFactors>();
    let volume = interpreted(&JoinFixture::new().compose(|| false).ok().unwrap());
    let expected = volume.replay_bytes();
    let actual = std::thread::spawn(move || (volume.enclosure(), volume.replay_bytes()))
        .join()
        .unwrap();
    assert!(actual.0[0] <= 4.8 && 4.8 <= actual.0[1]);
    assert_eq!(actual.1, expected);
}
#[test]
fn joined_volume_process_replay() {
    let volume = interpreted(&JoinFixture::new().compose(|| false).ok().unwrap());
    println!(
        "joined-volume-v1={}",
        volume
            .replay_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}

#[test]
fn independently_computed_and_shared_read_results_equal_serial_bytes() {
    let baseline = interpreted(&JoinFixture::new().compose(|| false).ok().unwrap());
    let expected = baseline.replay_bytes();
    let workers: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                interpreted(&JoinFixture::new().compose(|| false).ok().unwrap()).replay_bytes()
            })
        })
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    check_shared_reads(baseline, &expected);
}
fn check_shared_reads(baseline: JoinedPrismVolume, expected: &[u8]) {
    let shared = std::sync::Arc::new(baseline);
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let owned = shared.clone();
            std::thread::spawn(move || (owned.enclosure(), owned.replay_bytes()))
        })
        .collect();
    for reader in readers {
        let (bounds, bytes) = reader.join().unwrap();
        assert_eq!(bounds, shared.enclosure());
        assert_eq!(bytes, expected);
    }
}
