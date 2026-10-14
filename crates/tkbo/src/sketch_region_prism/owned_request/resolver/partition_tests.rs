// SPDX-License-Identifier: GPL-2.0-only
use super::{source_tests::*, *};
use crate::sketch_region::{PartitionMemberQueries, PartitionMemberQuery, PartitionObservation};
fn first_stages(
    prepared: &PreparedPolynomialPrism,
) -> (Vec<NamedSourceContext>, Vec<NamedSourceContext>) {
    let observed = initial_copy(prepared);
    let original = original(prepared, &observed);
    let copied = checked(copied::copied_contexts(&observed, &original));
    (original, copied)
}
fn queries(prepared: &mut PreparedPolynomialPrism) -> &mut Vec<PartitionMemberQuery> {
    let bridge = prepared.retained.bridge.as_mut().unwrap();
    let PartitionObservation::Captured(partition) = &mut bridge.mapped.arrangement.partition else {
        panic!("actual partition required");
    };
    let PartitionMemberQueries::Captured { rows, .. } = &mut partition.member_queries else {
        panic!("actual native queries required");
    };
    rows
}
fn directed_tokens(names: &[NamedSourceContext], stage: SemanticNameStage) -> Vec<u64> {
    let mut result: Vec<_> = names
        .iter()
        .map(|row| {
            SemanticNameSlot {
                stage,
                role: row.directed_use,
            }
            .token()
            .value()
        })
        .collect();
    result.sort();
    result
}
#[test]
fn actual_partition_and_candidate_copy_resolve_predeclared_aggregate_roles() {
    for reversed in [false, true] {
        let prepared = operation(reversed);
        let (original, copied) = first_stages(&prepared);
        let partition = checked(partition::partition_contexts(&prepared, &original, &copied));
        assert_endpoint_roles(&partition, reversed);
        assert_eq!(
            directed_tokens(&partition, SemanticNameStage::Partition),
            [1029, 1030, 1031, 1059]
        );
        let next = checked(operands::candidate_copy(&prepared));
        let candidate = checked(copied::copied_contexts(next, &partition));
        assert_endpoint_roles(&candidate, reversed);
        assert_eq!(
            directed_tokens(&candidate, SemanticNameStage::CandidateCopy),
            [1285, 1286, 1287, 1315]
        );
    }
}
#[test]
fn actual_partition_query_order_does_not_author_roles() {
    let mut prepared = operation(true);
    let (original, copied) = first_stages(&prepared);
    queries(&mut prepared).reverse();
    let partition = checked(partition::partition_contexts(&prepared, &original, &copied));
    assert_endpoint_roles(&partition, true);
    assert_eq!(
        directed_tokens(&partition, SemanticNameStage::Partition),
        [1029, 1030, 1031, 1059]
    );
}
#[test]
fn native_split_or_deleted_partition_answers_leave_correspondence_unresolved() {
    for generated in [false, true] {
        let mut prepared = operation(false);
        let (original, copied) = first_stages(&prepared);
        let query = &mut queries(&mut prepared)[0];
        if generated {
            query.generated = Ok(vec![query.copied.clone()]);
        } else {
            query.deleted = Ok(true);
        }
        assert!(matches!(
            partition::partition_contexts(&prepared, &original, &copied),
            Err(ResolutionCause::History(_))
        ));
    }
}
#[test]
fn swapped_native_copy_query_targets_cannot_rename_original_endpoints() {
    let mut prepared = operation(false);
    let (original, copied) = first_stages(&prepared);
    let query = queries(&mut prepared);
    let source = query[1].copied.clone();
    query[1].copied = query[2].copied.clone();
    query[2].copied = source;
    assert!(partition::partition_contexts(&prepared, &original, &copied).is_err());
}
#[test]
fn equal_count_duplicate_partition_query_cannot_replace_an_original_use() {
    let mut prepared = operation(false);
    let (original, copied) = first_stages(&prepared);
    let query = queries(&mut prepared);
    query[1] = query[2].clone();
    assert!(partition::partition_contexts(&prepared, &original, &copied).is_err());
}
