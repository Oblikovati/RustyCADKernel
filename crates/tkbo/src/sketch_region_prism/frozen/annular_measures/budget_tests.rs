// SPDX-License-Identifier: GPL-2.0-only
use super::*;

fn refused(
    join: &AnnularFrozenDomain,
    requested: AnnularMeasureOptions,
) -> Box<AnnularMeasureFailure> {
    inspect_annular_measures(join, requested, || false)
        .err()
        .expect("typed incomplete measure")
}
fn side_failure(failure: &AnnularMeasureFailure) -> &PolynomialSideError {
    let AnnularMeasureCause::Side(side) = &failure.cause else {
        panic!("expected side failure: {:?}", failure.cause)
    };
    assert_eq!(failure.phase, AnnularMeasurePhase::Sides);
    assert!(failure.entity.is_some());
    assert_eq!(
        failure.classification(),
        JoinedMeasureFailureClass::Incomplete
    );
    side
}
#[test]
fn aggregate_exact_and_one_short_include_terminal_child_work() {
    let join = joined();
    let mut requested = exact_budget(&join);
    requested.maximum_side_subdivisions -= 1;
    let failure = refused(&join, requested);
    let terminal = side_failure(&failure);
    assert!(matches!(
        *terminal.cause,
        PolynomialSideFailure::WorkLimit { .. }
    ));
    assert_eq!(failure.partial.faces.len(), 3);
    assert_eq!(
        failure.partial.side_subdivisions,
        requested.maximum_side_subdivisions
    );
    assert_terminal_work(&failure, terminal);
}
fn exact_budget(join: &AnnularFrozenDomain) -> AnnularMeasureOptions {
    let mut requested = options();
    requested.maximum_side_subdivisions = measured(join).side_subdivisions();
    let exact = inspect_annular_measures(join, requested, || false).unwrap();
    assert_eq!(
        exact.side_subdivisions(),
        requested.maximum_side_subdivisions
    );
    requested
}
fn assert_terminal_work(failure: &AnnularMeasureFailure, terminal: &PolynomialSideError) {
    let AnnularFaceMeasure::Side { report, .. } = &failure.partial.faces[2].measure else {
        panic!("completed outer side")
    };
    assert!(terminal.work.subdivisions > 0);
    assert_eq!(
        failure.partial.side_subdivisions,
        report.work().subdivisions + terminal.work.subdivisions
    );
    assert_eq!(
        terminal.requested_options.max_subdivisions,
        failure.requested.maximum_side_subdivisions - report.work().subdivisions
    );
    assert_eq!(
        terminal.requested_options.absolute_error_limit,
        failure.requested.side_area.absolute_error_limit
    );
}
#[test]
fn zero_shared_budget_retains_caps_sweep_and_actual_zero_work_failure() {
    let join = joined();
    let mut requested = options();
    requested.maximum_side_subdivisions = 0;
    let failure = refused(&join, requested);
    let terminal = side_failure(&failure);
    assert!(matches!(
        *terminal.cause,
        PolynomialSideFailure::WorkLimit { requested: 0, .. }
    ));
    assert_eq!(terminal.work.subdivisions, 0);
    assert_eq!(failure.partial.side_subdivisions, 0);
    assert_eq!(failure.partial.faces.len(), 2);
    assert!(failure.partial.signed_sweep.is_some());
    assert!(failure.partial.total.is_none());
    assert_eq!(failure.requested, requested);
    assert_eq!(failure.binding.as_ref(), join.replay_bytes());
}
#[test]
fn zero_budget_can_complete_when_the_original_coarse_limit_is_met() {
    let join = joined();
    let mut requested = options();
    requested.maximum_side_subdivisions = 0;
    requested.side_area.absolute_error_limit = 100.;
    requested.total_area.absolute_error_limit = 200.;
    let report = inspect_annular_measures(&join, requested, || false).unwrap();
    assert_eq!(report.side_subdivisions(), 0);
    assert_eq!(report.faces().len(), 4);
    assert_eq!(report.requested(), requested);
}
#[test]
fn total_limit_failure_keeps_all_completed_faces_without_success() {
    let join = joined();
    let mut requested = options();
    requested.total_area.absolute_error_limit = 0.;
    let failure = refused(&join, requested);
    assert_eq!(failure.phase, AnnularMeasurePhase::Total);
    assert!(matches!(&failure.cause, AnnularMeasureCause::Total(error)
        if matches!(error.cause, EnclosureSumCause::ErrorLimit { requested: 0., .. })));
    assert_eq!(failure.partial.faces.len(), 4);
    assert!(failure.partial.signed_sweep.is_some());
    assert!(failure.partial.total.is_none());
    assert_eq!(
        failure.classification(),
        JoinedMeasureFailureClass::Incomplete
    );
}
