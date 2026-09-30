//! Native stage-pipeline oracle twin.
use super::super::{flag, int, real};
use rustycad_tkernel::{
    message::{Algorithm, ProgressIndicator, Status},
    standard::{Handle, SharedState},
};
use rustycad_tkmesh::imesh_tools::{Context, MeshBuilder};
use std::sync::Arc;
#[path = "../../tests/support/pipeline_fixture.rs"]
mod fixtures;
use fixtures::*;
pub fn pipeline_case(builder_outcome: i32, stages: [i32; 5], clean: bool, cancelled: bool) {
    let calls = PipelineCalls::default();
    let context = pipeline_context(builder_outcome, stages, clean, &calls);
    let builder = Handle::new(MeshBuilder::new());
    if builder_outcome != -4 {
        builder.set_context(&context.cast());
    }
    let positions = Arc::new(SharedState::new(Vec::new()));
    let progress = ProgressIndicator::new(Box::new(PipelineProgress {
        cancelled,
        positions: positions.clone(),
    }));
    builder.set_status(Status::WARN3);
    builder.perform(&progress.start());
    for status in [
        Status::DONE1,
        Status::WARN1,
        Status::WARN3,
        Status::FAIL1,
        Status::FAIL2,
        Status::FAIL3,
        Status::FAIL4,
        Status::FAIL5,
        Status::FAIL6,
        Status::FAIL7,
        Status::FAIL8,
    ] {
        flag(builder.get_status().is_set(status));
    }
    flag(context.get_model().is_null());
    flag(builder.get_context().is_null());
    flag(builder.cast::<Algorithm>().is_null());
    flag(context.cast::<Context>().is_null());
    report_pipeline_trace(&calls, &positions);
}
fn report_pipeline_trace(calls: &PipelineCalls, positions: &Arc<SharedState<Vec<f64>>>) {
    int(calls.read().len() as i32);
    for &value in calls.read().iter() {
        int(value);
    }
    int(positions.read().len() as i32);
    for &value in positions.read().iter() {
        real(value);
    }
    println!();
}
