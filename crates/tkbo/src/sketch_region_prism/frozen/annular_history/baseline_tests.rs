// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form tree census: two closed edges, one annulus and two rejected one-wire domains.
use super::super::annular_fixture::with_bound;
use crate::sketch_region::*;
use crate::sketch_region_candidates::GeneratedTopologyObservation;
use crate::sketch_region_prism::PrismSweepQueries;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[test]
fn actual_annular_native_history_keeps_two_source_and_three_candidate_rosters() {
    with_bound(|request, receipt, _, _| {
        let PartitionObservation::Captured(partition) =
            &request.bridge.mapped.arrangement.partition
        else {
            panic!("expected retained partition");
        };
        assert_partition_roster(partition);
        let copied = request.bridge.mapped.copied.as_ref().unwrap();
        assert_copy_roster(&copied.contexts, 7);
        assert_eq!(copied.links.len(), 7);
        let GeneratedTopologyObservation::Observed(areas) = &copied.generated_topology else {
            panic!("expected complete actual generated areas");
        };
        let mut lengths = areas
            .iter()
            .map(|a| observed_len(&a.contexts))
            .collect::<Vec<_>>();
        lengths.sort();
        assert_eq!(lengths, [5, 5, 9]);
        assert_eq!(receipt.input.rejected.len(), 2);
        assert_eq!(receipt.input.selected.as_ref().unwrap().relations.len(), 9);
    });
}
fn assert_partition_roster(partition: &OwnedPartitionSnapshot) {
    assert_eq!(partition.input_contexts.len(), 2);
    assert_eq!(partition.sources.len(), 2);
    assert_eq!(partition.vertices.len(), 2);
    assert_eq!(partition.source_shape_count, Some(4));
    for source in &partition.input_contexts {
        assert_copy_roster(&source.observation, 3);
    }
    let PartitionMemberQueries::Captured {
        history_requested,
        history_available,
        rows,
    } = &partition.member_queries
    else {
        panic!("expected exhaustive native partition queries");
    };
    assert!(*history_requested && *history_available);
    assert_eq!(rows.len(), 6);
    for row in rows {
        assert!(row.modified.as_ref().unwrap().is_empty());
        assert!(row.generated.as_ref().unwrap().is_empty());
        assert!(!row.deleted.as_ref().unwrap());
    }
}
fn assert_copy_roster(observed: &RegionCopyContextObservation, expected: usize) {
    let RegionCopyContextObservation::Captured(copied) = observed else {
        panic!("expected actual copier context observations");
    };
    assert_eq!(copied.source.len(), expected);
    assert_eq!(copied.copied.len(), expected);
    assert_eq!(copied.queries.len(), expected);
    assert!(
        copied
            .queries
            .iter()
            .all(|q| q.images.as_ref().unwrap().len() == 1)
    );
}
fn observed_len(observed: &RegionContextObservation) -> usize {
    let RegionContextObservation::Captured(rows) = observed else {
        panic!("expected complete actual context census");
    };
    rows.len()
}
#[test]
fn actual_annular_sweep_keeps_false_queries_and_two_distinct_wire_retirements() {
    with_bound(|_, receipt, _, _| {
        let body = receipt.diagnostic.as_ref().unwrap();
        let PrismSweepQueries::Captured(rows) = &body.sweep_queries else {
            panic!("expected exhaustive native sweep queries");
        };
        assert_eq!(rows.len(), 9);
        let retired = rows.iter().filter(|r| r.retired).collect::<Vec<_>>();
        assert_eq!(retired.len(), 2);
        assert!(
            retired
                .iter()
                .all(|r| r.query.output.shape_type().unwrap() == ShapeEnum::Wire)
        );
        assert_eq!(body.native_retired.len(), 2);
        assert_eq!(rows.iter().filter(|r| !r.retired).count(), 7);
        assert!(receipt.input.input_deleted.is_empty());
    });
}
