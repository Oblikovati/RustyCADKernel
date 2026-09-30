//! Controlled strategies test orchestration, never substitute for a mesher.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient,
    message::{Algorithm, ProgressIndicatorHooks, ProgressRange, ProgressScope, Status},
    standard::{Failure, Handle, RuntimeTyped, SharedState, StandardTransient},
};
use rustycad_tkmesh::{
    brep_mesh_data::DiscreteModel,
    imesh_data::MeshModel,
    imesh_tools::{Context, MeshParameters, ModelAlgo, ModelBuilder},
};
use std::sync::Arc;

pub type PipelineCalls = Arc<SharedState<Vec<i32>>>;
pub struct RecordingModelBuilder {
    algorithm: Algorithm,
    pub outcome: i32,
    pub calls: PipelineCalls,
}
impl_transient!(RecordingModelBuilder, "RecordingModelBuilder", dyn ModelBuilder, algorithm, interfaces: [dyn ModelBuilder]);
impl RecordingModelBuilder {
    pub fn new(outcome: i32, calls: PipelineCalls) -> Self {
        Self {
            algorithm: Algorithm::with_runtime_type(Self::static_type()),
            outcome,
            calls,
        }
    }
}
impl ModelBuilder for RecordingModelBuilder {
    fn algorithm(&self) -> &Algorithm {
        &self.algorithm
    }
    fn perform_internal(
        &self,
        shape: &Shape,
        _: &MeshParameters,
    ) -> Result<Handle<dyn MeshModel>, Failure> {
        self.calls.write().push(0);
        if self.outcome == -2 {
            return Err(Failure::new("deliberate model construction failure"));
        }
        if self.outcome == -1 {
            self.algorithm.set_status(Status::FAIL1);
            return Ok(Handle::default());
        }
        if self.outcome == 0 {
            return Ok(Handle::default());
        }
        self.algorithm.set_status(Status::DONE1);
        Ok(Handle::new(DiscreteModel::new(shape)).cast())
    }
}
pub struct RecordingModelStage {
    transient: StandardTransient,
    pub outcome: i32,
    pub index: i32,
    pub calls: PipelineCalls,
}
impl_transient!(RecordingModelStage, "RecordingModelStage", dyn ModelAlgo, transient, interfaces: [dyn ModelAlgo]);
impl RecordingModelStage {
    pub fn new(outcome: i32, index: i32, calls: PipelineCalls) -> Self {
        Self {
            transient: StandardTransient::default(),
            outcome,
            index,
            calls,
        }
    }
}
impl ModelAlgo for RecordingModelStage {
    fn perform_internal(
        &self,
        _: &Handle<dyn MeshModel>,
        _: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.calls.write().push(self.index);
        self.calls.write().push(i32::from(range.is_active()));
        if self.outcome < 0 {
            return Err(Failure::new("deliberate model stage failure"));
        }
        Ok(self.outcome != 0)
    }
}
pub struct PipelineProgress {
    pub cancelled: bool,
    pub positions: Arc<SharedState<Vec<f64>>>,
}
impl ProgressIndicatorHooks for PipelineProgress {
    fn show(&self, _: &ProgressScope<'_>, position: f64, _: bool) {
        self.positions.write().push(position);
    }
    fn user_break(&self) -> bool {
        self.cancelled
    }
}
pub fn pipeline_context(
    builder_outcome: i32,
    stages: [i32; 5],
    clean: bool,
    calls: &PipelineCalls,
) -> Handle<Context> {
    let context = Handle::new(Context::new());
    context.change_parameters().clean_model = clean;
    if builder_outcome != -3 {
        context.set_model_builder(
            &Handle::new(RecordingModelBuilder::new(builder_outcome, calls.clone())).cast(),
        );
    }
    let algorithms: Vec<Handle<dyn ModelAlgo>> = stages
        .iter()
        .enumerate()
        .map(|(i, &outcome)| {
            if outcome == -2 {
                return Handle::default();
            }
            Handle::new(RecordingModelStage::new(
                outcome,
                i as i32 + 1,
                calls.clone(),
            ))
            .cast()
        })
        .collect();
    context.set_edge_discret(&algorithms[0]);
    context.set_model_healer(&algorithms[1]);
    context.set_pre_processor(&algorithms[2]);
    context.set_face_discret(&algorithms[3]);
    context.set_post_processor(&algorithms[4]);
    context
}
