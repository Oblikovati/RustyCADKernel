//! Native stage order, status selection, progress spans, and cleanup.
use super::MeshContext;
use rustycad_tkernel::{
    impl_transient,
    message::{Algorithm, ProgressRange, ProgressScope, Status},
    standard::{Handle, RuntimeTyped, SharedState},
};

/// Native orchestration over an assigned context; `MeshBuilder::with_context(&context)`.
pub struct MeshBuilder {
    algorithm: Algorithm,
    context: SharedState<Handle<dyn MeshContext>>,
}
impl_transient!(MeshBuilder, "IMeshTools_MeshBuilder", Algorithm, algorithm);
impl Default for MeshBuilder {
    fn default() -> Self {
        Self::with_context(&Handle::default())
    }
}
impl std::ops::Deref for MeshBuilder {
    type Target = Algorithm;
    fn deref(&self) -> &Self::Target {
        &self.algorithm
    }
}
impl MeshBuilder {
    /// Initially null context; `MeshBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Retain the supplied context; `MeshBuilder::with_context(&context)`.
    pub fn with_context(context: &Handle<dyn MeshContext>) -> Self {
        Self {
            algorithm: Algorithm::with_runtime_type(Self::static_type()),
            context: SharedState::new(context.clone()),
        }
    }
    /// Replace context; `builder.set_context(&context)`.
    pub fn set_context(&self, context: &Handle<dyn MeshContext>) {
        *self.context.write() = context.clone();
    }
    /// Shared context; `builder.get_context()`.
    pub fn get_context(&self) -> Handle<dyn MeshContext> {
        self.context.snapshot()
    }
    /// Execute the native stage pipeline; `builder.perform(&ProgressRange::default())`.
    pub fn perform(&self, range: &ProgressRange<'_>) {
        self.clear_status();
        let context = self.get_context();
        if context.is_null() {
            self.set_status(Status::FAIL1);
            return;
        }
        let scope = ProgressScope::with_literal(range, "Mesh Perform", 10., false);
        let status = Self::run_stages(&context, &scope);
        self.set_status(status);
        if status != Status::FAIL8 {
            drop(scope.next(1.));
        }
        context.clean();
    }
    fn run_stages(context: &Handle<dyn MeshContext>, scope: &ProgressScope<'_>) -> Status {
        if !context.build_model() {
            return Self::model_failure(context);
        }
        if !context.discretize_edges() {
            return Status::FAIL3;
        }
        if !context.heal_model() {
            return Status::FAIL4;
        }
        if !context.pre_process_model() {
            return Status::FAIL5;
        }
        if !context.discretize_faces(&scope.next(9.)) {
            return if scope.more() {
                Status::FAIL6
            } else {
                Status::FAIL8
            };
        }
        if !context.post_process_model() {
            return Status::FAIL7;
        }
        Status::DONE1
    }
    fn model_failure(context: &Handle<dyn MeshContext>) -> Status {
        let builder = context.get_model_builder();
        if builder.is_null() {
            return Status::FAIL1;
        }
        if builder.get_status().is_set(Status::FAIL1) {
            return Status::WARN1;
        }
        Status::FAIL2
    }
}
