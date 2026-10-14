// SPDX-License-Identifier: GPL-2.0-only
//! Independent expectations come from the closed-form census of two nested one-edge loops:
//! each loop has one edge and two closure-vertex uses; the annulus keeps both loops, each
//! rejected domain keeps exactly one, and only the two copied annulus wires retire in the sweep.
use super::fixture::AnnularHistoryCase;
use super::*;
use crate::sketch_region::RegionSourceId;
use rustycad_tkg3d::top_abs::ShapeEnum;

const OUTER: RegionSourceId = RegionSourceId(101);
const INNER: RegionSourceId = RegionSourceId(202);

pub(super) fn inspected(case: &AnnularHistoryCase) -> Box<AnnularCompleteHistory> {
    case.run(
        |request| match inspect_annular_prism_history(request, |_| false) {
            AnnularHistoryOutcome::Inspected(value) => value,
            AnnularHistoryOutcome::Incomplete(f) | AnnularHistoryOutcome::Failed(f) => {
                panic!("{:?}", f.finding())
            }
            AnnularHistoryOutcome::Cancelled { phase, .. } => panic!("cancelled at {phase:?}"),
        },
    )
}
fn kind(history: &AnnularCompleteHistory, key: &PrismHistoryUseKey) -> u8 {
    let found: Vec<_> = history
        .contexts()
        .iter()
        .filter(|c| &c.key == key)
        .collect();
    assert_eq!(found.len(), 1);
    found[0].occurrence.member.kind
}
fn count<T>(rows: &[T], keep: impl Fn(&T) -> bool) -> usize {
    rows.iter().filter(|r| keep(r)).count()
}
#[test]
fn actual_annular_history_proves_all_six_original_uses_with_kept_caps() {
    fn check<T: Send + Sync>() {}
    check::<AnnularCompleteHistory>();
    check::<PrismHistoryFinding>();
    let case = AnnularHistoryCase::nested();
    let history = inspected(&case);
    assert!(history.binds(&case.names.joined));
    let continued = history.continued();
    assert_eq!(continued.len(), 6);
    assert_eq!(count(continued, |c| c.source == OUTER), 3);
    assert_eq!(count(continued, |c| c.source == INNER), 3);
    assert!(history.deleted_originals().is_empty());
    assert!(!history.is_valid_body());
    let kept =
        |c: &AnnularOriginalContinuation| !c.kept_first.is_empty() && !c.kept_last.is_empty();
    assert!(continued.iter().all(kept));
    assert!(case.run(|request| reproject_annular_prism_history(request, &history).is_ok()));
}
#[test]
fn intermediate_wire_retirements_are_separate_from_original_deletion() {
    let history = inspected(&AnnularHistoryCase::nested());
    let retired = history.retired_wires();
    let sources: Vec<_> = retired.iter().map(|r| r.source).collect();
    assert_eq!(sources, [OUTER, INNER]);
    let wire = ShapeEnum::Wire as u8;
    assert!(retired.iter().all(|r| kind(&history, &r.selected) == wire));
    let binding = history.source_binding();
    assert_eq!(binding.selected_candidate, 301);
    assert_eq!(binding.rejected_candidates, [302, 303]);
    assert_eq!(history.rosters().len(), 10);
    assert!(history.deleted_originals().is_empty());
}
#[test]
fn every_source_keeps_its_own_empty_and_false_native_partition_answers() {
    let history = inspected(&AnnularHistoryCase::nested());
    let sources = history.sources();
    let roles: Vec<_> = sources.iter().map(|s| s.source).collect();
    assert_eq!(roles, [OUTER, INNER]);
    sources.iter().for_each(assert_unchanged_source);
    let outer = sources[0].partition_queries.iter();
    let outer: Vec<_> = outer.map(|q| &q.continued).collect();
    let inner = &sources[1].partition_queries;
    assert!(inner.iter().all(|q| !outer.contains(&&q.continued)));
}
fn assert_unchanged_source(source: &AnnularSourceHistory) {
    let queries = &source.partition_queries;
    assert_eq!((source.initial_copy.len(), queries.len()), (3, 3));
    for query in queries {
        assert_eq!(query.original.source, source.source.0);
        assert!(query.modified.is_empty() && query.generated.is_empty() && !query.deleted);
    }
    let endpoints = source.metadata.endpoints.len();
    assert_eq!((source.closure.len(), endpoints), (4, 2));
    assert!(!source.metadata.images_present);
}
/// Kind grouping would give each edge both source edges as parents; the actual continuation
/// gives each edge exactly its own source edge and each vertex its own two closure uses.
#[test]
fn partition_parents_follow_each_source_continuation_not_shape_kind() {
    let history = inspected(&AnnularHistoryCase::nested());
    let rows = history.partition_relations();
    assert_eq!(rows.len(), 7);
    for row in rows {
        let modified = &row.parents[1];
        match kind(&history, &row.output) {
            k if k == ShapeEnum::Compound as u8 => assert_eq!(row.parents[2].len(), 6),
            k if k == ShapeEnum::Edge as u8 => assert_eq!(modified.len(), 1),
            k if k == ShapeEnum::Vertex as u8 => {
                assert_eq!(modified.len(), 2);
                assert_eq!(modified[0].source, modified[1].source);
            }
            other => panic!("unexpected partition kind {other}"),
        }
    }
}
#[test]
fn authored_candidates_bind_sealed_dispositions_and_receipts() {
    use TwoLoopCandidateDisposition as D;
    let history = inspected(&AnnularHistoryCase::nested());
    let rows = history.candidates();
    let summary = |c: &AnnularCandidateHistory| (c.name, c.disposition, c.generated.len());
    let summary: Vec<_> = rows.iter().map(summary).collect();
    let expected = [
        (301, D::SelectedAnnulus, 9),
        (302, D::RejectedInnerDomain, 5),
        (303, D::RejectedOuterComplement, 5),
    ];
    assert_eq!(summary, expected);
    assert!(rows.iter().all(|c| c.relations.len() == c.generated.len()));
    let copy = history.candidate_copy();
    assert_eq!((copy.queries.len(), copy.links.len()), (7, 7));
    let container = kind(&history, &copy.administrative_container);
    assert_eq!(container, ShapeEnum::Compound as u8);
}
#[test]
fn authored_candidate_wires_keep_their_source_roles() {
    let history = inspected(&AnnularHistoryCase::nested());
    let wires = |c: &AnnularCandidateHistory| -> Vec<_> {
        c.wires.iter().map(|w| (w.name, w.source)).collect()
    };
    let rows: Vec<_> = history.candidates().iter().map(wires).collect();
    let expected = [
        vec![(401, OUTER), (402, INNER)],
        vec![(403, INNER)],
        vec![(404, OUTER)],
    ];
    assert_eq!(rows, expected);
}
#[test]
fn each_copied_use_reaches_the_annulus_and_the_one_rejected_domain_of_its_source() {
    let history = inspected(&AnnularHistoryCase::nested());
    assert_eq!(history.traces().len(), 6);
    for row in history.continued() {
        let outer = row.source == OUTER;
        let expected = if outer {
            (303, 404, 401)
        } else {
            (302, 403, 402)
        };
        let trace = &row.trace;
        let actual = (
            trace.rejected_candidate,
            trace.rejected_wire,
            trace.selected_wire,
        );
        assert_eq!(actual, expected);
        assert_ne!(trace.selected, trace.rejected);
    }
}
#[test]
fn captured_flat_parents_bind_the_same_native_row_and_keep_proof_order() {
    let case = AnnularHistoryCase::nested();
    let history = inspected(&case);
    let captured = &case.names.joined.embedding().binding().parents().captured;
    let rows = history.captured_parents();
    assert_eq!(rows.len(), captured.len());
    for (row, raw) in rows.iter().zip(captured) {
        assert_eq!(row.flat.len(), raw.parents.len());
        assert_eq!(row.role, FrozenSweepRole::from(raw.identity.role));
        let sources = row.flat.iter().zip(&raw.parents);
        assert!(
            sources
                .into_iter()
                .all(|(key, role)| key.source == role.source.0)
        );
    }
    let queries = history.sweep_queries();
    assert_eq!((queries.len(), count(queries, |q| !q.retired)), (9, 7));
}
fn cancelled_at(case: &AnnularHistoryCase, cutoff: PrismHistoryPhase) -> Vec<PrismHistoryPhase> {
    let mut calls = Vec::new();
    let outcome = case.run(|request| {
        let stop = |phase| {
            calls.push(phase);
            phase == cutoff
        };
        match inspect_annular_prism_history(request, stop) {
            AnnularHistoryOutcome::Cancelled { phase, operation } => (phase, operation),
            _ => panic!("expected cancellation at {cutoff:?}"),
        }
    });
    assert_eq!(outcome, (cutoff, AuthoredPrismId(77)));
    calls
}
#[test]
fn every_coarse_checkpoint_cancels_without_history_in_order() {
    use PrismHistoryPhase::*;
    let case = AnnularHistoryCase::nested();
    assert_eq!(cancelled_at(&case, Admission), [Admission]);
    assert_eq!(cancelled_at(&case, Capture), [Admission, Capture]);
    assert_eq!(cancelled_at(&case, Commit), [Admission, Capture, Commit]);
    inspected(&case);
}
#[test]
fn independent_operations_replay_identical_bytes_without_native_addresses() {
    let first = inspected(&AnnularHistoryCase::nested());
    let second = inspected(&AnnularHistoryCase::nested());
    assert_eq!(first.replay_bytes(), second.replay_bytes());
    assert_eq!(
        first.measured_operand_rows(),
        second.measured_operand_rows()
    );
    let prefix = b"martensite-complete-annular-history-v2;";
    assert!(first.replay_bytes().starts_with(prefix));
    assert_eq!(first.requested(), PrismHistoryOptions::annular_v2());
}
