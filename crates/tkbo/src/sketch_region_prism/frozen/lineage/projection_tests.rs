// SPDX-License-Identifier: GPL-2.0-only
use super::tests::{fixture, inspected, run};
use super::*;

#[test]
fn supplied_projection_reuses_exact_observations_without_second_freeze() {
    let (_, receipt, publication, authored) = fixture();
    let original = inspected(run(&receipt, &publication, &authored));
    let bound = super::super::bindings::bind_projection(&receipt, &publication).unwrap();
    let projected = inspected(project_prism_lineage(
        &bound,
        original.projection().unwrap(),
        &authored,
    ));
    assert_eq!(projected.lineage_bytes(), original.lineage_bytes());
    assert!(projected.projection().is_none());
    assert_eq!(projected.coverage(), original.coverage());
}
#[test]
fn supplied_projection_refuses_wrong_operation_or_missing_directed_keys() {
    let (_, receipt, publication, authored) = fixture();
    let original = inspected(run(&receipt, &publication, &authored));
    let bound = super::super::bindings::bind_projection(&receipt, &publication).unwrap();
    let mut changed = original.projection().unwrap().clone();
    changed.operation = AuthoredPrismId(9876);
    assert!(matches!(
        project_prism_lineage(&bound, &changed, &authored),
        PrismLineageOutcome::Incomplete(_)
    ));
    changed.operation = publication.operation;
    changed.uses.pop();
    assert!(matches!(
        project_prism_lineage(&bound, &changed, &authored),
        PrismLineageOutcome::Incomplete(_)
    ));
}
#[test]
fn supplied_projection_retains_missing_historical_observation_failure() {
    let (_, mut receipt, publication, authored) = fixture();
    let original = inspected(run(&receipt, &publication, &authored));
    receipt.copied.as_mut().unwrap().observations.pop();
    let bound = super::super::bindings::bind_projection(&receipt, &publication).unwrap();
    let PrismLineageOutcome::Incomplete(result) =
        project_prism_lineage(&bound, original.projection().unwrap(), &authored)
    else {
        panic!("missing observation must remain incomplete");
    };
    assert_eq!(
        result.coverage(),
        (
            CopyLineageCoverage::Incomplete,
            CopyLineageCoverage::Blocked
        )
    );
    assert!(matches!(
        result.findings()[0].cause,
        CopyLineageCause::Count {
            actual: 4,
            expected: 5,
            ..
        }
    ));
}
