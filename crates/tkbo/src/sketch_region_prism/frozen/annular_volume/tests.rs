// SPDX-License-Identifier: GPL-2.0-only
//! Independent closed forms: the cubic loop (0,0),(4,0),(0,4),(0,0) bounds 12/5, the inner copy is
//! scaled by 1/2, so the annulus has area 9/5 and every sweep by |h| = 2 has volume 18/5, for every
//! stored/traversed reversal, reflection, sweep sign and request order.
use super::*;
use crate::sketch_region_prism::frozen::annular_history::fixture::{
    AnnularHistoryCase, fresh_embedding,
};
use crate::sketch_region_prism::frozen::annular_measures::AnnularFaceMeasure;
use crate::sketch_region_prism::frozen::annular_orientation::fixture::Variant;
use crate::sketch_region_prism::publication::annular::{
    AnnularPublicationRequest, PublishedAnnularPrism, publish_annular_prism, tests::options,
};
use crate::sketch_region_prism::publication::{
    PrismPublicationCheckpoint, PrismPublicationProgress,
};
use rustycad_tkernel::standard::Failure;

const _: () = {
    const fn owned<T: Send + Sync>() {}
    owned::<AnnularPrismVolume>();
    owned::<AnnularVolumeFailure>();
};
struct Continue;
impl PrismPublicationProgress for Continue {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}
fn with_published<T>(
    case: &AnnularHistoryCase,
    run: impl FnOnce(&PublishedAnnularPrism) -> T,
) -> T {
    case.with_region(|region| {
        let request = AnnularPublicationRequest {
            region,
            receipt: &case.receipt,
            bindings: &case.publication,
            copied: &case.names.copied,
            historical: &case.names.historical,
            requested: options(),
        };
        let embedding = fresh_embedding(region, &case.receipt);
        let published = publish_annular_prism(&request, embedding, &mut Continue);
        run(&published.unwrap_or_else(|f| panic!("publication refused at {:?}", f.phase)))
    })
}
fn certify(
    published: &PublishedAnnularPrism,
) -> Result<AnnularPrismVolume, Box<AnnularVolumeFailure>> {
    let orientation = published.validation().orientation();
    certify_annular_prism_volume(
        published.join(),
        orientation,
        published.measures(),
        1,
        || false,
    )
}
fn contains([lo, hi]: [f64; 2], value: f64) -> bool {
    lo <= value && value <= hi
}
/// Independent second route: world cap area (its own certified integral) times |h| = 2.
fn assert_closed_form(published: &PublishedAnnularPrism, volume: &AnnularPrismVolume) {
    let [lo, hi] = volume.enclosure();
    assert!(lo >= 0. && contains([lo, hi], 3.6), "{:?}", [lo, hi]);
    // Certified error claim against the closed form: |representative - 18/5| <= error bound.
    let error = (volume.representative() - 3.6).abs();
    assert!(
        error <= volume.absolute_error_bound() + 4. * f64::EPSILON,
        "{error}"
    );
    let source = volume.source_integral();
    let signed = f64::from(volume.jacobian_sign()) * source.representative();
    assert_eq!(volume.representative().to_bits(), signed.to_bits());
    assert_eq!(volume.absolute_error_bound(), source.absolute_error_bound());
    for face in published.measures().faces() {
        if let AnnularFaceMeasure::Cap(report) = &face.measure {
            let [a, b] = report.enclosure();
            assert!(
                a * 2. <= hi && lo <= b * 2.,
                "cap route {:?} vs {:?}",
                [a, b],
                [lo, hi]
            );
        }
    }
}
fn variants() -> Vec<(Variant, bool)> {
    let base = Variant::default();
    let mut rows = vec![(base, false), (base, true)];
    for edit in [
        |v: &mut Variant| v.stored_reverse = [true, false],
        |v: &mut Variant| v.traversed_reverse = [false, true],
        |v: &mut Variant| v.reflected = true,
        |v: &mut Variant| v.negative_sweep = true,
        |v: &mut Variant| (v.reflected, v.negative_sweep) = (true, true),
    ] {
        let mut variant = base;
        edit(&mut variant);
        rows.push((variant, false));
    }
    rows
}
#[test]
fn every_orientation_variant_has_the_closed_form_physical_volume() {
    let mut signs = std::collections::BTreeSet::new();
    for (variant, reversed) in variants() {
        let case = AnnularHistoryCase::variant(variant, reversed);
        with_published(&case, |published| {
            let volume = certify(published).unwrap_or_else(|f| panic!("{:?}", f.cause));
            assert!(volume.binds(published.join()));
            assert_closed_form(published, &volume);
            signs.insert(volume.jacobian_sign());
        });
    }
    assert_eq!(
        signs.into_iter().collect::<Vec<_>>(),
        [-1, 1],
        "both exact signs exercised"
    );
}
#[test]
fn foreign_orientation_version_and_cancellation_refuse_without_volume() {
    let variant = Variant {
        negative_sweep: true,
        ..Default::default()
    };
    let foreign = AnnularHistoryCase::variant(variant, false);
    let case = AnnularHistoryCase::nested();
    with_published(&foreign, |other| {
        with_published(&case, |published| {
            let (join, measures) = (published.join(), published.measures());
            let alien = other.validation().orientation();
            let cause = certify_annular_prism_volume(join, alien, measures, 1, || false)
                .err()
                .expect("foreign orientation")
                .cause;
            let operand = AnnularVolumeOperand::Orientation;
            assert_eq!(cause, AnnularVolumeCause::Binding(operand));
        })
    });
    with_published(&case, |published| {
        let (join, measures) = (published.join(), published.measures());
        let orientation = published.validation().orientation();
        let refuse = |version, cancel_at: usize| {
            let mut calls = 0;
            let cancel = move || {
                calls += 1;
                calls == cancel_at
            };
            certify_annular_prism_volume(join, orientation, measures, version, cancel)
                .err()
                .expect("expected a refusal")
        };
        let version = AnnularVolumeCause::Version {
            actual: 2,
            expected: 1,
        };
        assert_eq!(refuse(2, 0).cause, version);
        assert_eq!(
            refuse(1, 1).cause,
            AnnularVolumeCause::Cancelled(VolumeCheckpoint::Begin)
        );
        let finish = refuse(1, 2);
        assert!(finish.cancelled());
        assert_eq!(
            finish.cause,
            AnnularVolumeCause::Cancelled(VolumeCheckpoint::Finish)
        );
    });
}
#[test]
fn inconsistent_sign_evidence_is_incomplete() {
    let case = AnnularHistoryCase::nested();
    with_published(&case, |published| {
        let boundaries = *published.validation().orientation().boundaries();
        let mut swapped = boundaries;
        swapped.swap(0, 1);
        assert!(matches!(
            exact_sign(1, &swapped),
            Err(AnnularVolumeCause::SignFactors { .. })
        ));
        assert!(matches!(
            exact_sign(0, &boundaries),
            Err(AnnularVolumeCause::SignFactors { .. })
        ));
        let mut winding = boundaries;
        winding[1].native_winding = 0;
        assert!(exact_sign(-1, &winding).is_err());
        assert_eq!(exact_sign(-1, &boundaries), Ok(-1));
    });
}
#[test]
fn replay_is_identical_across_operations_and_threads() {
    let bytes = || {
        let case = AnnularHistoryCase::nested();
        with_published(&case, |p| {
            certify(p).expect("volume").replay_bytes().to_vec()
        })
    };
    let first = bytes();
    assert!(first.starts_with(b"Martensite.AnnularPrismVolume/v1\0"));
    assert_eq!(first, bytes());
    assert_eq!(
        first,
        std::thread::spawn(bytes).join().expect("volume thread")
    );
}
