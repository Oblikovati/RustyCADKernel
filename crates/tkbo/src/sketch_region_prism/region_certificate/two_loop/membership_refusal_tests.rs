// SPDX-License-Identifier: GPL-2.0-only
//! Equal parent lists cannot identify output occurrences or excuse missing actual candidate rows.
use super::{failure::*, fixture::*, operation_tests::*, outcome::*};
use crate::sketch_region_candidates::GeneratedTopologyObservation;

#[test]
fn extra_generated_area_and_missing_descendant_never_disappear_from_universe_coverage() {
    for duplicate in [true, false] {
        let mut fixture = TwoLoopFixture::nested();
        let copy = fixture.bridge.mapped.copied.as_mut().unwrap();
        let GeneratedTopologyObservation::Observed(areas) = &mut copy.generated_topology else {
            panic!("actual areas")
        };
        if duplicate {
            let mut other = TwoLoopFixture::nested();
            let GeneratedTopologyObservation::Observed(other_areas) = &mut other
                .bridge
                .mapped
                .copied
                .as_mut()
                .unwrap()
                .generated_topology
            else {
                panic!("other actual areas")
            };
            areas.push(other_areas.pop().unwrap());
        } else {
            areas[0].members.pop();
        }
        assert!(matches!(
            inspect(&fixture, &mut OperationProgress),
            TwoLoopRegionOutcome::Incomplete(_)
        ));
    }
}
#[test]
fn actual_distinct_same_parent_members_keep_separate_authored_keys() {
    let fixture = TwoLoopFixture::nested();
    let proof = accepted(inspect(&fixture, &mut OperationProgress));
    let ring = proof.candidates().iter().find(|c| c.name == 301).unwrap();
    let complement = proof.candidates().iter().find(|c| c.name == 303).unwrap();
    let first = ring.wires.iter().find(|w| w.source == OUTER).unwrap();
    let second = &complement.wires[0];
    assert_eq!(first.parents, second.parents);
    assert_ne!(first.name, second.name);
}
#[test]
fn duplicated_member_parent_is_retained_or_refused_and_never_silently_deduplicated() {
    let mut fixture = TwoLoopFixture::nested();
    let batch = fixture
        .bridge
        .mapped
        .copied
        .as_mut()
        .unwrap()
        .completion
        .as_mut()
        .unwrap_or_else(|_| panic!("batch"));
    let relation = batch
        .candidates
        .iter_mut()
        .flat_map(|c| &mut c.relations)
        .find(|r| !r.generated.is_empty())
        .unwrap();
    relation.generated.push(relation.generated[0].clone());
    assert!(matches!(
        inspect(&fixture, &mut OperationProgress),
        TwoLoopRegionOutcome::Incomplete(_)
    ));
}
