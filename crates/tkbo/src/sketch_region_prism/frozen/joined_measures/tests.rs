// SPDX-License-Identifier: GPL-2.0-only
use super::super::proof_join::operation_tests::JoinFixture;
use super::*;

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
fn measured(join: &JoinedPrismProof) -> JoinedPrismMeasures {
    inspect_joined_prism_measures(join, options(), || false).unwrap()
}
#[test]
fn joined_measures_bind_named_faces_and_do_not_promote_body_validity() {
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let report = measured(&join);
    assert!(report.matches_join(&join));
    assert_eq!(report.binding_bytes(), join.replay_bytes());
    assert_eq!(report.requested(), options());
    assert_eq!(report.named_faces().len(), 3);
    assert!(!join.projection().is_valid_body());
    check_closed_form_caps(&report);
    check_shared_scalar_results(&report, &join);
}
fn check_closed_form_caps(report: &JoinedPrismMeasures) {
    for face in report.named_faces() {
        let area = report.face_area(&face.occurrence).unwrap();
        assert_eq!(area.face, face);
        let expected = if face.role == PrismImageRole::Generated {
            0.01
        } else {
            1e-12
        };
        assert!(area.scalar.absolute_error_bound() <= expected);
        if face.role != PrismImageRole::Generated {
            contains(area.scalar.enclosure(), 2.4);
        }
    }
    contains(report.signed_volume().enclosure(), 4.8);
    assert_eq!(report.total_area().components().len(), 3);
}
fn check_shared_scalar_results(report: &JoinedPrismMeasures, join: &JoinedPrismProof) {
    let legacy = measure_bound_prism(
        join.embedding().binding(),
        join.projection().limits().extrusion,
        options().scalars,
        &mut || false,
    )
    .unwrap();
    assert_eq!(report.measures.subset.first_cap, legacy.subset.first_cap);
    assert_eq!(report.measures.subset.last_cap, legacy.subset.last_cap);
    assert_eq!(
        report.measures.subset.signed_volume,
        legacy.subset.signed_volume
    );
    assert_eq!(report.measures.side, legacy.side);
    assert_eq!(report.measures.total, legacy.total);
}
fn contains(enclosure: [f64; 2], expected: f64) {
    assert!(
        enclosure[0] <= expected && expected <= enclosure[1],
        "{enclosure:?} must contain {expected}"
    );
}
#[test]
fn joined_measure_result_and_failures_are_send_sync() {
    static_assertions::assert_impl_all!(JoinedPrismMeasures: Send, Sync);
    static_assertions::assert_impl_all!(JoinedPrismMeasureFailure: Send, Sync);
    static_assertions::assert_impl_all!(JoinedPrismMeasureCause: Send, Sync);
    static_assertions::assert_impl_all!(NamedPrismArea<'static>: Send, Sync);
    static_assertions::assert_impl_all!(PrismAreaReport<'static>: Send, Sync);
}

#[path = "tests_binding.rs"]
mod binding;
#[path = "tests_refusal.rs"]
mod refusal;
#[path = "tests_replay.rs"]
mod replay_tests;

#[test]
fn full_joined_surface_reports_contain_independent_exact_integration_oracles() {
    use crate::sketch_region_prism::region_certificate::measurements::tests::complete_tests::{
        assert_contains_interval, oracle_interval,
    };
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let report = measured(&join);
    assert_contains_interval(report.measures.side.enclosure(), oracle_interval("side"));
    assert_contains_interval(report.total_area().enclosure(), oracle_interval("total"));
    let cap = [
        f64::from_bits(0x4003333333333333),
        f64::from_bits(0x4003333333333334),
    ];
    assert_eq!(report.measures.subset.first_cap.enclosure(), cap);
    assert_eq!(report.measures.subset.last_cap.enclosure(), cap);
    assert_eq!(
        report.signed_volume().enclosure(),
        cap.map(|value| value * 2.)
    );
}
