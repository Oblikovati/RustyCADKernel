// SPDX-License-Identifier: GPL-2.0-only
//! Regression for fixed 99a107f89 review: admission must precede legacy matrix/fault allocation.
use super::{
    census_budget, failure::*, fixture::*, fixture_bindings, operation_tests::*, source_budget,
    work::*,
};
use crate::sketch_region_source_conformance::*;

fn source_units(fixture: &TwoLoopFixture, maximum: usize) -> TwoLoopCheck<'_, usize> {
    let mut progress = OperationProgress;
    let mut options = fixture_bindings::options();
    options.maximum_retained_units = maximum;
    let mut work = TwoLoopWork::new(77, options, &mut progress);
    source_budget::admit(&fixture.bridge.census, &mut work)?;
    Ok(work.retained_units)
}
#[test]
fn full_nested_source_roster_has_inclusive_limit_and_tiny_refusal() {
    let fixture = TwoLoopFixture::nested();
    let units = source_units(&fixture, 8192).unwrap_or_else(|_| panic!("bounded fixture"));
    assert!(units > 100);
    assert_eq!(source_units(&fixture, units).ok(), Some(units));
    for maximum in [1, units - 1] {
        let failure = source_units(&fixture, maximum).err().unwrap();
        assert!(
            matches!(failure.cause, TwoLoopRegionCause::Limit { requested, measured, .. }
            if requested == maximum && measured > maximum)
        );
        assert!(failure.retained_units <= maximum);
    }
}
fn duplicate_entry(source: &OriginalConstructionEvidence) -> OriginalConstructionEvidence {
    OriginalConstructionEvidence {
        occurrence: source.occurrence,
        original: source.original.clone(),
        stages: source
            .stages
            .iter()
            .map(|stage| ConstructionStageEvidence {
                stage: stage.stage,
                requested_limit: stage.requested_limit,
                effective_limit: stage.effective_limit,
                effective_roots: stage.effective_roots,
                bisections: stage.bisections,
                decision: Ok(stage.decision.as_ref().unwrap().clone()),
            })
            .collect(),
    }
}
fn inject_entries(fixture: &mut TwoLoopFixture) {
    let census = &mut fixture.bridge.census;
    let old = census.source_binding.as_ref().unwrap().inspection();
    let report = SourceConformanceInspection {
        entries: (0..64).map(|_| duplicate_entry(&old.entries[0])).collect(),
        requested: old.requested,
        originals: old.originals.clone(),
        constructions: old.constructions.clone(),
        bisections: old.bisections,
    };
    census.source_binding = Some(bind_source_conformance(
        &census.inputs,
        report.requested,
        report,
    ));
}
#[test]
fn injected_source_matrix_is_refused_before_source_fault_cloning() {
    let mut fixture = TwoLoopFixture::nested();
    let baseline = source_units(&fixture, 8192).unwrap_or_else(|_| panic!("baseline"));
    inject_entries(&mut fixture);
    let failure = source_units(&fixture, baseline).err().unwrap();
    assert!(matches!(failure.cause, TwoLoopRegionCause::Limit { .. }));
    assert!(failure.local.is_none());
    assert!(source_units(&fixture, 8192).unwrap_or_else(|_| panic!("injected units")) > baseline);
    assert!(super::source_tests::capture(&fixture).is_err());
}
fn census_units(fixture: &TwoLoopFixture, maximum: usize) -> TwoLoopCheck<'_, usize> {
    let mut progress = OperationProgress;
    let mut options = fixture_bindings::options();
    options.maximum_retained_units = maximum;
    let mut work = TwoLoopWork::new(77, options, &mut progress);
    census_budget::admit(&fixture.bridge.census, &mut work)?;
    Ok(work.retained_units)
}
#[test]
fn pair_roster_counts_nested_parent_and_coefficient_operands_before_validation() {
    let mut fixture = TwoLoopFixture::nested();
    let baseline = census_units(&fixture, 8192).unwrap_or_else(|_| panic!("baseline"));
    assert_eq!(census_units(&fixture, baseline).ok(), Some(baseline));
    assert!(census_units(&fixture, baseline - 1).is_err());
    let row = &mut fixture.bridge.census.paired.as_mut().unwrap().occurrences[0];
    row.source.controls.push([9., 9.]);
    let parent = row.source.parents[0].clone();
    row.relations[0].generated.push(parent);
    assert_eq!(census_units(&fixture, 8192).ok(), Some(baseline + 2));
    assert!(matches!(
        census_units(&fixture, baseline).err().unwrap().cause,
        TwoLoopRegionCause::Limit { .. }
    ));
}
