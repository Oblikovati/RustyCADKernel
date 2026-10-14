// SPDX-License-Identifier: GPL-2.0-only
use super::fixture::*;
use crate::sketch_region_candidates::{
    CandidateClassification, SeedSelection, classify_candidates,
};
use crate::sketch_region_certification::domain_certificate;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;

#[test]
fn independent_nested_loops_have_complete_pair_coverage_and_three_actual_regions() {
    let fixture = TwoLoopFixture::nested();
    assert_eq!(fixture.sources().map(|row| row.identity), [OUTER, INNER]);
    let census = &fixture.bridge.census;
    assert_eq!(census.domains.len(), 3);
    for domain in &census.domains {
        let proof = domain_certificate(census, domain).expect("complete actual self/cross domain");
        let expected = usize::from(domain.occurrences[0] == domain.occurrences[1]);
        assert_eq!(proof.contact_count(), expected);
    }
    check_nested_candidates(&fixture);
}
fn check_nested_candidates(fixture: &TwoLoopFixture) {
    let copied = fixture.bridge.mapped.copied.as_ref().unwrap();
    let batch = copied
        .completion
        .as_ref()
        .unwrap_or_else(|_| panic!("complete candidates"));
    assert_eq!(batch.candidates.len(), 3);
    assert_eq!(
        batch
            .candidates
            .iter()
            .map(|c| c.wires.len())
            .sum::<usize>(),
        4
    );
    let result = classify_candidates(
        batch,
        [0.25, 0.25],
        RootIsolationOptions::new(0.001),
        || false,
    )
    .unwrap_or_else(|_| panic!("exact complete winding"));
    let CandidateClassification::Incomplete {
        selection: SeedSelection::Unique { candidate },
        ..
    } = result
    else {
        panic!("ring seed; expected unique annular candidate");
    };
    assert_eq!(batch.candidates[candidate].wires.len(), 2);
}
