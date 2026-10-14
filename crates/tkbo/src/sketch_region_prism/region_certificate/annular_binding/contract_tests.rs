// SPDX-License-Identifier: GPL-2.0-only
//! Actual-operation cancellation, source rebind and fresh-read ownership checks.
use super::*;
use rustycad_tkernel::standard::Failure;
#[derive(Default)]
struct RecordedProgress { stages: Vec<AnnularBindingStage>, stop: Option<usize>, fail: Option<AnnularBindingStage> }
impl AnnularBindingProgress for RecordedProgress {
    fn checkpoint(&mut self, stage: AnnularBindingStage) -> Result<bool, Failure> {
        self.stages.push(stage);
        if self.fail == Some(stage) { return Err(Failure::with_message("Standard_Failure", "authored annular callback failure")); }
        Ok(self.stop == Some(self.stages.len() - 1))
    }
}
#[test]
fn every_observed_cancellation_prevents_annular_binding_issuance() {
    tests::with_prepared(|request, receipt| {
        let mut baseline = RecordedProgress::default();
        assert!(bind_annular_prism_v2(request, tests::fresh_region(request), receipt, &mut baseline).is_ok());
        for (index, stage) in baseline.stages.iter().enumerate() {
            let mut progress = RecordedProgress { stop: Some(index), ..Default::default() };
            let Err(failure) = bind_annular_prism_v2(request, tests::fresh_region(request), receipt, &mut progress) else { panic!("expected cancellation at {index}/{stage:?}"); };
            assert_eq!(failure.classification(), TwoLoopRefusalClass::Cancelled);
            assert_eq!(failure.stage, *stage);
            assert_eq!(progress.stages, baseline.stages[..=index]);
        }
        let mut final_progress = RecordedProgress::default();
        assert!(bind_annular_prism_v2(request, tests::fresh_region(request), receipt, &mut final_progress).is_ok());
        assert_eq!(final_progress.stages, baseline.stages);
    });
}
#[test]
fn callback_failure_keeps_operation_stage_and_original_typed_cause() {
    for stage in [AnnularBindingStage::Admission, AnnularBindingStage::Copy, AnnularBindingStage::Charts, AnnularBindingStage::Commit] {
        tests::with_actual(|request, region, receipt| {
            let mut progress = RecordedProgress { fail: Some(stage), ..Default::default() };
            let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut progress) else { panic!("expected callback failure"); };
            assert_eq!(failure.classification(), TwoLoopRefusalClass::Failed);
            assert_eq!((failure.operation, failure.selected_candidate, failure.stage), (77,301,stage));
            assert!(matches!(*failure.cause, AnnularBindingCause::Callback { .. }));
        });
    }
}
struct MutateSourceAtCommit { source: rustycad_tkbrep::topods::Edge, changed: bool }
impl AnnularBindingProgress for MutateSourceAtCommit {
    fn checkpoint(&mut self, stage: AnnularBindingStage) -> Result<bool, Failure> {
        if stage == AnnularBindingStage::Commit {
            super::super::super::isolation_tests::set_pole(&self.source, 5.);
            self.changed = true;
        }
        Ok(false)
    }
}
#[test]
fn final_commit_callback_source_mutation_cannot_publish_stale_region_binding() {
    tests::with_actual(|request, region, receipt| {
        let mut progress = MutateSourceAtCommit { source: rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap(), changed: false };
        let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut progress) else { panic!("expected final source rebind refusal"); };
        assert!(progress.changed);
        assert_eq!(failure.stage, AnnularBindingStage::Commit);
        assert!(matches!(*failure.cause, AnnularBindingCause::Region(_)));
    });
}
#[test]
fn successful_owned_binding_reads_survive_source_mutation_and_input_drop() {
    let binding = tests::with_actual(|request, region, receipt| {
        let binding = bind_annular_prism_v2(request, region, receipt, &mut RecordedProgress::default()).unwrap_or_else(|_| panic!("actual binding"));
        let before = fresh_bytes(&binding);
        super::super::super::isolation_tests::set_pole(&rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap(), 5.);
        assert_eq!(fresh_bytes(&binding), before);
        assert!(rebind_annular_prism_v2(request, receipt, &binding).is_err());
        binding
    });
    assert_eq!(fresh_bytes(&binding), binding.replay_bytes());
}
fn fresh_bytes(binding: &AnnularPrismBinding) -> Vec<u8> {
    replay::encode(&binding.region, &binding.operands).unwrap_or_else(|_| panic!("fresh owned read encoding"))
}
#[test]
fn owned_binding_and_failure_envelopes_have_no_thread_confined_state() {
    fn check<T: Send + Sync>() {}
    check::<AnnularPrismBinding>();
    check::<AnnularChartIdentity>();
    check::<AnnularTrimIdentity>();
    check::<AnnularBindingFailure<'static>>();
    check::<AnnularBindingCause<'static>>();
}
#[test]
fn all_current_parent_dispositions_are_owned_without_deduplicating_occurrences() {
    let binding = tests::actual_binding();
    assert_eq!(binding.parents().copies.len(), 9);
    assert!(!binding.parents().images.is_empty());
    assert!(binding.parents().retained_units > binding.parents().images.len());
    let face = binding.parents().copies.iter().find(|row| row.key.wire.is_none()).unwrap();
    let expected: Vec<_> = [101, 202].into_iter().flat_map(|source| [TwoLoopMemberRole::Boundary, TwoLoopMemberRole::NativeStart, TwoLoopMemberRole::NativeEnd].map(|role| TwoLoopOriginalUse { source: RegionSourceId(source), role })).collect();
    assert_eq!(face.selected[2], expected);
    assert_eq!(face.copied[2], face.selected[2]);
}
#[test]
fn actual_callback_census_retains_all_stage_observations() {
    let started = std::time::Instant::now();
    tests::with_actual(|request, region, receipt| {
        let prepared = started.elapsed();
        let mut progress = RecordedProgress::default();
        assert!(bind_annular_prism_v2(request, region, receipt, &mut progress).is_ok());
        println!("{{\"annular_checkpoint_count\":{},\"prepare_micros\":{},\"binding_micros\":{}}}", progress.stages.len(), prepared.as_micros(), started.elapsed().as_micros() - prepared.as_micros());
        assert_eq!(progress.stages.first(), Some(&AnnularBindingStage::Admission));
        assert_eq!(progress.stages.last(), Some(&AnnularBindingStage::Commit));
    });
}
