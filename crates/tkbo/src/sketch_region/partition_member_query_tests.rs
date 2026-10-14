// SPDX-License-Identifier: GPL-2.0-only
use super::arrangement::{ArrangementExecutor, MappedArrangementExecutor};
use super::copy_inputs::CopiedSources;
use super::*;
use crate::bop_algo::BOPAlgoBuilder;
use rustycad_tkernel::standard::Failure;

fn query_fixture() -> (CopiedSources, BOPAlgoBuilder) {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let inputs = [ArrangementSource {
        identity: RegionSourceId(61),
        shape: &edge,
    }];
    let copied = copy_sources(&inputs).unwrap_or_else(|_| panic!("expected actual source copy"));
    let builder = MappedArrangementExecutor {
        observation: None,
        space_capture: None,
    }
    .execute(&copied)
    .unwrap_or_else(|_| panic!("expected actual mapped builder"));
    assert!(!builder.options().has_errors());
    (copied, builder)
}
fn captured_queries(copied: &CopiedSources, builder: &BOPAlgoBuilder) -> Vec<PartitionMemberQuery> {
    let PartitionMemberQueries::Captured {
        history_requested: true,
        history_available: true,
        rows,
    } = partition_member_queries::capture_member_queries(copied, builder)
    else {
        panic!("expected actual filled history queries");
    };
    rows
}
#[test]
fn actual_partition_queries_keep_every_original_use_and_empty_false_answers() {
    let (copied, builder) = query_fixture();
    let rows = captured_queries(&copied, &builder);
    assert_eq!(rows.len(), 3);
    for (row, (original, member)) in rows.iter().zip(&copied.members) {
        assert_eq!(&row.original, original);
        assert_eq!(row.copied, PartitionOccurrenceIdentity::capture(member));
        assert!(*row.supported.as_ref().unwrap());
        assert!(!*row.deleted.as_ref().unwrap());
        assert_query_lists(row, member, &builder);
    }
    assert!(
        rows.iter()
            .any(|row| row.generated.as_ref().unwrap().is_empty())
    );
}
fn assert_query_lists(
    row: &PartitionMemberQuery,
    member: &rustycad_tkbrep::topods::Shape,
    builder: &BOPAlgoBuilder,
) {
    for (observed, actual) in [
        (&row.modified, builder.modified(member)),
        (&row.generated, builder.generated(member)),
    ] {
        let expected: Vec<_> = actual
            .unwrap()
            .iter()
            .map(PartitionOccurrenceIdentity::capture)
            .collect();
        assert_eq!(observed.as_ref().unwrap(), &expected);
    }
}

#[test]
fn unavailable_native_history_keeps_fallback_answers_explicitly_unavailable() {
    let (copied, _) = query_fixture();
    let builder = BOPAlgoBuilder::new();
    let PartitionMemberQueries::Captured {
        history_requested,
        history_available,
        rows,
    } = partition_member_queries::capture_member_queries(&copied, &builder)
    else {
        panic!("expected observed unavailable history");
    };
    assert!(history_requested && !history_available);
    assert!(
        rows.iter()
            .all(|row| row.modified.as_ref().unwrap().is_empty())
    );
    assert!(rows.iter().all(|row| !row.deleted.as_ref().unwrap()));
}
#[test]
fn filled_history_accessor_never_synthesizes_availability() {
    let mut base = crate::bop_algo::BuilderShape::new();
    assert!(!base.has_filled_history());
    base.history();
    assert!(base.has_filled_history());
    base.set_to_fill_history(false);
    assert!(!base.has_filled_history());
}
#[test]
fn native_partition_query_errors_remain_typed_in_owned_snapshots() {
    let (mut copied, builder) = query_fixture();
    copied.members[0].1 = rustycad_tkbrep::topods::Shape::default();
    let rows = captured_queries(&copied, &builder);
    assert!(rows[0].kind.is_err());
    let typed: &Failure = rows[0].kind.as_ref().unwrap_err();
    assert_eq!(typed.exception_type(), "Standard_NullObject");
    assert!(rows[0].supported.is_err());
    assert!(rows[0].modified.is_err());
    // Native History deliberately permits null generation origins; retain its actual empty answer.
    assert!(rows[0].generated.as_ref().unwrap().is_empty());
    assert!(rows[0].deleted.is_err());
    fn require<T: Send + Sync>() {}
    require::<PartitionMemberQuery>();
    require::<PartitionMemberQueries>();
}
#[test]
fn partition_query_retention_incremental_work_measurement() {
    let (copied, builder) = query_fixture();
    let iterations = 1000;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(captured_queries(&copied, &builder));
    }
    let elapsed = start.elapsed().as_nanos();
    let rows = captured_queries(&copied, &builder);
    let images: usize = rows
        .iter()
        .map(|row| row.modified.as_ref().unwrap().len() + row.generated.as_ref().unwrap().len())
        .sum();
    println!(
        "{{\"measurement\":\"partition_query_retention\",\"iterations\":{iterations},\"elapsed_ns\":{elapsed},\"rows\":{},\"images\":{images},\"row_inline_bytes\":{}}}",
        rows.len(),
        std::mem::size_of::<PartitionMemberQuery>()
    );
    assert_eq!(rows.len(), 3);
}
