// SPDX-License-Identifier: GPL-2.0-only
use super::super::annular_join::operation_tests::joined;
use super::*;

fn options() -> AnnularMeasureOptions {
    AnnularMeasureOptions {
        version: 2,
        cap_area: Default::default(),
        signed_sweep: Default::default(),
        side_area: PolynomialSideOptions {
            absolute_error_limit: 0.01,
            ..Default::default()
        },
        maximum_side_subdivisions: 8192,
        total_area: EnclosureSumOptions {
            absolute_error_limit: 0.021,
            ..Default::default()
        },
    }
}
fn measured(join: &AnnularFrozenDomain) -> AnnularPrismMeasures {
    inspect_annular_measures(join, options(), || false)
        .unwrap_or_else(|error| panic!("{:?}", error.cause))
}
#[test]
fn actual_annular_measures_keep_four_faces_two_cap_holes_and_independent_areas() {
    let join = joined();
    let report = measured(&join);
    assert!(report.matches(&join));
    assert_eq!(report.requested(), options());
    assert_eq!(report.faces().len(), 4);
    assert!(!join.projection().is_valid_body());
    assert_caps(&report);
    assert_sides(&report);
    assert_eq!(report.total_area().components().len(), 4);
    assert_eq!(
        report.signed_sweep().enclosure(),
        [
            f64::from_bits(0x400ccccccccccccc),
            f64::from_bits(0x400ccccccccccccd)
        ]
    );
    assert!(
        report.total_area().absolute_error_bound() <= options().total_area.absolute_error_limit
    );
}
fn assert_caps(report: &AnnularPrismMeasures) {
    let cap = [
        f64::from_bits(0x3ffccccccccccccc),
        f64::from_bits(0x3ffccccccccccccd),
    ];
    for row in &report.faces()[..2] {
        assert!(matches!(row.named.domain, NamedAnnularDomain::Cap { .. }));
        assert_eq!(row.named.domain.boundary_uses().len(), 2);
        assert_eq!(row.measure.enclosure(), cap);
        assert!(row.measure.absolute_error_bound() <= options().cap_area.absolute_error_limit);
    }
}
fn assert_sides(report: &AnnularPrismMeasures) {
    let oracle = crate::sketch_region_prism::region_certificate::measurements::tests::complete_tests::oracle_interval("side");
    let mut work = 0;
    for (row, scale) in report.faces()[2..].iter().zip([1., 0.5]) {
        let AnnularFaceMeasure::Side {
            requested,
            effective,
            report: side,
        } = &row.measure
        else {
            panic!("actual side")
        };
        assert_eq!(*requested, options().side_area);
        assert_eq!(*effective, side.effective_options());
        let observed = side.enclosure();
        assert!(observed[0] <= oracle[0] * scale && oracle[1] * scale <= observed[1]);
        work += side.work().subdivisions;
    }
    assert_eq!(report.side_subdivisions(), work);
}
#[test]
fn invalid_original_side_policy_is_not_hidden_by_zero_aggregate_budget() {
    let join = joined();
    let mut requested = options();
    requested.side_area.max_subdivisions = 65537;
    requested.maximum_side_subdivisions = 0;
    let error = inspect_annular_measures(&join, requested, || false)
        .err()
        .unwrap();
    assert!(
        matches!(error.cause, AnnularMeasureCause::OriginalSidePolicy(PolynomialSideFailure::Policy(actual)) if actual == requested.side_area)
    );
    assert_eq!(error.phase, AnnularMeasurePhase::Admission);
    assert!(error.partial.faces.is_empty());
}
#[test]
fn whole_report_and_terminal_partial_evidence_are_send_sync() {
    static_assertions::assert_impl_all!(AnnularPrismMeasures: Send, Sync);
    static_assertions::assert_impl_all!(AnnularMeasureFailure: Send, Sync);
    static_assertions::assert_impl_all!(AnnularMeasuredFace: Send, Sync);
    static_assertions::assert_impl_all!(AnnularMeasureOptions: Send, Sync);
}

#[path = "baseline_tests.rs"]
mod baseline;
