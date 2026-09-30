//! Native model construction and pipeline transition tests.
#[path = "support/pipeline_fixture.rs"]
mod fixtures;
use fixtures::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::{Algorithm, ProgressIndicator, ProgressRange, Status},
    standard::{Handle, SharedState, StandardTransient},
};
use rustycad_tkmesh::imesh_tools::{Context, MeshBuilder, ModelBuilder};
use std::sync::Arc;

#[test]
fn model_builder_clears_prior_status_and_catches_failure() {
    for outcome in -2..=1 {
        let builder = Handle::new(RecordingModelBuilder::new(
            outcome,
            PipelineCalls::default(),
        ));
        let base: Handle<Algorithm> = builder.cast();
        base.set_status(Status::WARN3);
        let result = builder.perform(&Shape::default(), &Default::default());
        assert_eq!(result.is_null(), outcome != 1);
        assert!(!base.get_status().is_warn());
        assert_eq!(base.get_status().is_set(Status::FAIL2), outcome == -2);
        assert_eq!(base.get_status().is_set(Status::FAIL1), outcome == -1);
        assert_eq!(base.get_status().is_set(Status::DONE1), outcome == 1);
        assert!(!base.cast::<dyn ModelBuilder>().is_null());
    }
}
#[test]
fn required_and_optional_context_stages_preserve_null_semantics() {
    let calls = PipelineCalls::default();
    let context = pipeline_context(1, [-2; 5], false, &calls);
    assert!(!context.heal_model());
    assert!(!context.pre_process_model());
    assert!(!context.post_process_model());
    assert!(context.build_model());
    assert!(!context.discretize_edges());
    assert!(!context.discretize_faces(&ProgressRange::default()));
    assert!(context.heal_model());
    assert!(context.pre_process_model());
    assert!(context.post_process_model());
    context.clean();
    assert!(!context.get_model().is_null());
    context.set_model_builder(&Handle::default());
    assert!(!context.build_model());
    assert!(!context.get_model().is_null());
    context.change_parameters().clean_model = true;
    context.clean();
    assert!(context.get_model().is_null());
}
#[test]
fn mesh_builder_stage_failure_statuses_and_cleanup_are_native() {
    for failure in 0..=5 {
        let mut stages = [1; 5];
        if failure > 0 {
            stages[failure - 1] = 0;
        }
        for clean in [false, true] {
            let context = pipeline_context(1, stages, clean, &PipelineCalls::default());
            let builder = MeshBuilder::with_context(&context.cast());
            builder.perform(&ProgressRange::default());
            let status = if failure == 0 {
                Status::DONE1
            } else {
                Status(Status::FAIL3.0 + failure as i32 - 1)
            };
            assert!(builder.get_status().is_set(status));
            assert_eq!(context.get_model().is_null(), clean);
            assert!(
                builder
                    .get_context()
                    .get()
                    .unwrap()
                    .context_base()
                    .get_parameters()
                    .clean_model
                    == clean
            );
        }
    }
}
#[test]
fn builder_empty_context_and_model_failure_statuses_are_native() {
    let builder = Handle::new(MeshBuilder::new());
    builder.perform(&ProgressRange::default());
    assert!(builder.get_status().is_set(Status::FAIL1));
    assert!(builder.get_context().is_null());
    for (outcome, status) in [
        (-3, Status::FAIL1),
        (-2, Status::FAIL2),
        (-1, Status::WARN1),
        (0, Status::FAIL2),
        (1, Status::DONE1),
    ] {
        let context = pipeline_context(outcome, [1; 5], true, &PipelineCalls::default());
        builder.set_context(&context.cast());
        builder.perform(&ProgressRange::default());
        assert!(builder.get_status().is_set(status));
        assert!(!builder.cast::<Algorithm>().is_null());
        assert!(!builder.cast::<StandardTransient>().is_null());
        assert!(!builder.get_context().cast::<Context>().is_null());
    }
}
#[test]
fn face_failure_on_cancel_sets_fail8_and_still_cleans() {
    let positions = Arc::new(SharedState::new(Vec::new()));
    let progress = ProgressIndicator::new(Box::new(PipelineProgress {
        cancelled: true,
        positions: positions.clone(),
    }));
    let context = pipeline_context(1, [1, 1, 1, 0, 1], true, &PipelineCalls::default());
    let builder = MeshBuilder::with_context(&context.cast());
    builder.perform(&progress.start());
    assert!(builder.get_status().is_set(Status::FAIL8));
    assert!(context.get_model().is_null());
    assert_eq!(*positions.read().first().unwrap(), 0.0);
    assert_eq!(progress.get_position(), 1.0);
}
