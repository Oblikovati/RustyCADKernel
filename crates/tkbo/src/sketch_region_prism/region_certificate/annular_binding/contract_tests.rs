// SPDX-License-Identifier: GPL-2.0-only
//! Actual-operation cancellation, source rebind and fresh-read ownership checks.
use super::*;
use rustycad_tkernel::standard::Failure;
#[derive(Default)]
struct RecordedProgress {
    stages: Vec<AnnularBindingStage>,
    stop: Option<usize>,
    fail: Option<AnnularBindingStage>,
}
impl AnnularBindingProgress for RecordedProgress {
    fn checkpoint(&mut self, stage: AnnularBindingStage) -> Result<bool, Failure> {
        self.stages.push(stage);
        if self.fail == Some(stage) {
            return Err(Failure::with_message(
                "Standard_Failure",
                "authored annular callback failure",
            ));
        }
        Ok(self.stop == Some(self.stages.len() - 1))
    }
}
#[test]
fn every_observed_cancellation_prevents_annular_binding_issuance() {
    tests::with_prepared(|request, receipt| {
        let baseline = completed_progress(request, receipt);
        for (index, stage) in baseline.stages.iter().enumerate() {
            cancelled_at(request, receipt, &baseline.stages, index, *stage);
        }
        let final_progress = completed_progress(request, receipt);
        assert_eq!(final_progress.stages, baseline.stages);
    });
}
#[test]
fn callback_failure_keeps_operation_stage_and_original_typed_cause() {
    for stage in [
        AnnularBindingStage::Admission,
        AnnularBindingStage::Copy,
        AnnularBindingStage::Charts,
        AnnularBindingStage::Commit,
    ] {
        tests::with_actual(|request, region, receipt| {
            assert_callback_failure(request, region, receipt, stage);
        });
    }
}
struct MutateSourceAtCommit {
    source: rustycad_tkbrep::topods::Edge,
    changed: bool,
}
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
        let mut progress = MutateSourceAtCommit {
            source: rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap(),
            changed: false,
        };
        let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut progress) else {
            panic!("expected final source rebind refusal");
        };
        assert!(progress.changed);
        assert_eq!(failure.stage, AnnularBindingStage::Commit);
        assert!(matches!(*failure.cause, AnnularBindingCause::Region(_)));
    });
}
#[test]
fn successful_owned_binding_reads_survive_source_mutation_and_input_drop() {
    let binding = tests::with_actual(|request, region, receipt| {
        let binding =
            bind_annular_prism_v2(request, region, receipt, &mut RecordedProgress::default())
                .unwrap_or_else(|_| panic!("actual binding"));
        let before = fresh_bytes(&binding);
        super::super::super::isolation_tests::set_pole(
            &rustycad_tkbrep::topods::edge(request.sources[0].shape).unwrap(),
            5.,
        );
        assert_eq!(fresh_bytes(&binding), before);
        assert!(rebind_annular_prism_v2(request, receipt, &binding).is_err());
        binding
    });
    assert_eq!(fresh_bytes(&binding), binding.replay_bytes());
}
fn fresh_bytes(binding: &AnnularPrismBinding) -> Vec<u8> {
    replay::encode(&binding.region, &binding.operands)
        .unwrap_or_else(|_| panic!("fresh owned read encoding"))
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
    assert_eq!(binding.parents().captured.len(), 16);
    assert!(binding.retained_units() >= binding.hierarchy_budget().retained_units);
    assert!(binding.retained_units() <= binding.hierarchy_budget().maximum_retained_units);
    assert!(binding.parents().retained_units > binding.parents().images.len());
    let face = binding
        .parents()
        .copies
        .iter()
        .find(|row| row.key.wire.is_none())
        .unwrap();
    let expected = expected_original_parents();
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
        println!(
            "{{\"annular_checkpoint_count\":{},\"prepare_micros\":{},\"binding_micros\":{}}}",
            progress.stages.len(),
            prepared.as_micros(),
            started.elapsed().as_micros() - prepared.as_micros()
        );
        assert_eq!(
            progress.stages.first(),
            Some(&AnnularBindingStage::Admission)
        );
        assert_eq!(progress.stages.last(), Some(&AnnularBindingStage::Commit));
    });
}

#[test]
fn directed_endpoint_roles_cannot_hide_split_physical_copy_definition() {
    tests::with_actual(|request, region, receipt| {
        let mut capture = capture::copy_inputs(request, &region, receipt)
            .unwrap_or_else(|_| panic!("actual copy"));
        let endpoint = capture
            .members
            .iter_mut()
            .find(|m| m.key.wire == Some(401) && m.key.role == TwoLoopMemberRole::NativeEnd)
            .unwrap();
        endpoint.copied.occurrence.topology_address = 0;
        assert!(correspondence::preserve_physical_sharing(&capture.members).is_err());
    });
}
#[test]
fn captured_trim_parent_union_cannot_substitute_equal_count_foreign_parent() {
    tests::with_actual(|request, region, receipt| {
        let copy = capture::copy_inputs(request, &region, receipt)
            .unwrap_or_else(|_| panic!("actual copy"));
        let boundaries = boundaries::capture(&region, &copy, receipt)
            .unwrap_or_else(|_| panic!("actual boundaries"));
        let (mut charts, _) = charts::capture(receipt, &copy, &boundaries, &mut || false)
            .unwrap_or_else(|_| panic!("actual charts"));
        assert!(parents::capture(request, receipt, &copy, 0, &charts).is_ok());
        charts[0].chart.captured.trims[0].parents[0].source = RegionSourceId(999);
        let Err(cause) = parents::capture(request, receipt, &copy, 0, &charts) else {
            panic!("expected parent bridge refusal");
        };
        assert_parent_bridge_cause(*cause);
    });
}
#[test]
fn legacy_flat_parents_keep_first_seen_order_while_classes_keep_duplicates() {
    tests::with_actual(|request, _, _| {
        let parents = native_original_parents(request);
        let relation = crate::sketch_region::ArrangementRelation {
            output: rustycad_tkbrep::topods::Shape::default(),
            preserved: vec![parents[1].clone(), parents[0].clone(), parents[1].clone()],
            modified: vec![parents[0].clone(), parents[1].clone()],
            generated: vec![parents[1].clone()],
        };
        let flat = super::super::super::history::all_parents(&relation);
        assert_eq!(flat, [parents[1].clone(), parents[0].clone()]);
        let rows: Vec<_> = flat.iter().map(std::slice::from_ref).collect();
        let mapped = map_two_loop_original_parents(request, &rows)
            .unwrap_or_else(|_| panic!("exact authored singleton parents"));
        assert_flat_parent_mapping(&mapped);
        assert_eq!(
            relation.preserved.len() + relation.modified.len() + relation.generated.len(),
            6
        );
    });
}

fn completed_progress(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
) -> RecordedProgress {
    let mut progress = RecordedProgress::default();
    assert!(
        bind_annular_prism_v2(
            request,
            tests::fresh_region(request),
            receipt,
            &mut progress
        )
        .is_ok()
    );
    progress
}

fn cancelled_at(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    baseline: &[AnnularBindingStage],
    index: usize,
    stage: AnnularBindingStage,
) {
    let mut progress = RecordedProgress {
        stop: Some(index),
        ..Default::default()
    };
    let Err(failure) = bind_annular_prism_v2(
        request,
        tests::fresh_region(request),
        receipt,
        &mut progress,
    ) else {
        panic!("expected cancellation at {index}/{stage:?}");
    };
    assert_eq!(failure.classification(), TwoLoopRefusalClass::Cancelled);
    assert_eq!(failure.stage, stage);
    assert_eq!(progress.stages, baseline[..=index]);
}

fn assert_callback_failure(
    request: &TwoLoopRegionRequest<'_, '_>,
    region: Box<TwoLoopRegionCertificate>,
    receipt: &PrismReceipt,
    stage: AnnularBindingStage,
) {
    let mut progress = RecordedProgress {
        fail: Some(stage),
        ..Default::default()
    };
    let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut progress) else {
        panic!("expected callback failure");
    };
    assert_eq!(failure.classification(), TwoLoopRefusalClass::Failed);
    assert_eq!(
        (failure.operation, failure.selected_candidate, failure.stage),
        (77, 301, stage)
    );
    assert!(matches!(
        *failure.cause,
        AnnularBindingCause::Callback { .. }
    ));
}

fn expected_original_parents() -> Vec<TwoLoopOriginalUse> {
    [101, 202]
        .into_iter()
        .flat_map(|source| {
            [
                TwoLoopMemberRole::Boundary,
                TwoLoopMemberRole::NativeStart,
                TwoLoopMemberRole::NativeEnd,
            ]
            .map(|role| TwoLoopOriginalUse {
                source: RegionSourceId(source),
                role,
            })
        })
        .collect()
}

fn assert_parent_bridge_cause(cause: AnnularBindingCause<'_>) {
    assert!(matches!(
        cause,
        AnnularBindingCause::Count {
            required: "captured chart/trim parents equal legacy ordered unique projection",
            actual: 0,
            expected: 1
        }
    ));
}

fn native_original_parents(
    request: &TwoLoopRegionRequest<'_, '_>,
) -> Vec<crate::sketch_region::OriginalMemberIdentity> {
    request
        .sources
        .iter()
        .map(|source| {
            let actual = PartitionOccurrenceIdentity::capture(source.shape);
            crate::sketch_region::OriginalMemberIdentity {
                source: source.identity,
                topology_address: actual.topology_address,
                location: actual.location,
                orientation: actual.orientation,
            }
        })
        .collect()
}

fn assert_flat_parent_mapping(mapped: &TwoLoopMappedParents) {
    assert_eq!(
        mapped.rows,
        [
            vec![TwoLoopOriginalUse {
                source: RegionSourceId(202),
                role: TwoLoopMemberRole::Boundary
            }],
            vec![TwoLoopOriginalUse {
                source: RegionSourceId(101),
                role: TwoLoopMemberRole::Boundary
            }]
        ]
    );
}
