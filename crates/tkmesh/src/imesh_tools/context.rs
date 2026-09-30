//! Cached model and independently replaceable native processing strategies.
use super::{MeshParameters, ModelAlgo, ModelBuilder};
use crate::imesh_data::{
    MeshModel, MeshShape,
    shape::{ShapeState, mesh_shape_members},
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{
        Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, Transient,
        project_interface,
    },
};
use std::sync::RwLockWriteGuard;

/// Native concrete context; `Context::new()` starts with null strategies and model.
pub struct Context {
    transient: StandardTransient,
    shape: ShapeState,
    model_builder: SharedState<Handle<dyn ModelBuilder>>,
    model: SharedState<Handle<dyn MeshModel>>,
    edge_discret: SharedState<Handle<dyn ModelAlgo>>,
    model_healer: SharedState<Handle<dyn ModelAlgo>>,
    pre_processor: SharedState<Handle<dyn ModelAlgo>>,
    face_discret: SharedState<Handle<dyn ModelAlgo>>,
    post_processor: SharedState<Handle<dyn ModelAlgo>>,
    parameters: SharedState<MeshParameters>,
}
impl_transient!(Context, "IMeshTools_Context", dyn MeshShape, transient, interfaces: [dyn MeshShape, dyn MeshContext]);
mesh_shape_members!(Context);
impl Default for Context {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            shape: ShapeState::new(&Shape::default()),
            model_builder: SharedState::default(),
            model: SharedState::default(),
            edge_discret: SharedState::default(),
            model_healer: SharedState::default(),
            pre_processor: SharedState::default(),
            face_discret: SharedState::default(),
            post_processor: SharedState::default(),
            parameters: SharedState::default(),
        }
    }
}
impl Context {
    /// Empty native context; `Context::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Cached model; `context.get_model().is_null()` before model construction.
    pub fn get_model(&self) -> Handle<dyn MeshModel> {
        self.model.snapshot()
    }
    /// Input parameter snapshot; `context.get_parameters().clean_model`.
    pub fn get_parameters(&self) -> MeshParameters {
        self.parameters.snapshot()
    }
    /// Scoped mutable parameters; `context.change_parameters().clean_model = false`.
    pub fn change_parameters(&self) -> RwLockWriteGuard<'_, MeshParameters> {
        self.parameters.write()
    }
    /// Construct the cached model through the assigned strategy; `context.build_model()`.
    pub fn build_model(&self) -> bool {
        let builder = self.get_model_builder();
        if builder.is_null() {
            return false;
        }
        let model = builder.perform(&self.get_shape(), &self.get_parameters());
        let success = !model.is_null();
        *self.model.write() = model;
        success
    }
    fn process_model(
        &self,
        algorithm: Handle<dyn ModelAlgo>,
        required: bool,
        range: &ProgressRange<'_>,
    ) -> bool {
        let model = self.get_model();
        if model.is_null() {
            return false;
        }
        if algorithm.is_null() {
            return !required;
        }
        algorithm.perform(&model, &self.get_parameters(), range)
    }
    /// Required edge strategy; `context.discretize_edges()`.
    pub fn discretize_edges(&self) -> bool {
        self.process_model(self.get_edge_discret(), true, &ProgressRange::default())
    }
    /// Optional model healing; `context.heal_model()`.
    pub fn heal_model(&self) -> bool {
        self.process_model(self.get_model_healer(), false, &ProgressRange::default())
    }
    /// Optional preparation; `context.pre_process_model()`.
    pub fn pre_process_model(&self) -> bool {
        self.process_model(self.get_pre_processor(), false, &ProgressRange::default())
    }
    /// Required face strategy with caller progress; `context.discretize_faces(&range)`.
    pub fn discretize_faces(&self, range: &ProgressRange<'_>) -> bool {
        self.process_model(self.get_face_discret(), true, range)
    }
    /// Optional final processing; `context.post_process_model()`.
    pub fn post_process_model(&self) -> bool {
        self.process_model(self.get_post_processor(), false, &ProgressRange::default())
    }
    /// Release the model when CleanModel is set; `context.clean()`.
    pub fn clean(&self) {
        if self.get_parameters().clean_model {
            *self.model.write() = Handle::default();
        }
    }
}
macro_rules! context_strategy_accessors {
    ($field:ident, $get:ident, $set:ident, $interface:ty) => {
        impl Context {
            #[doc = concat!("Shared strategy; `context.", stringify!($get), "()`.")]
            pub fn $get(&self) -> Handle<$interface> {
                self.$field.snapshot()
            }
            #[doc = concat!("Replace strategy; `context.", stringify!($set), "(&strategy)`.")]
            pub fn $set(&self, strategy: &Handle<$interface>) {
                *self.$field.write() = strategy.clone();
            }
        }
    };
}
context_strategy_accessors!(
    model_builder,
    get_model_builder,
    set_model_builder,
    dyn ModelBuilder
);
context_strategy_accessors!(
    edge_discret,
    get_edge_discret,
    set_edge_discret,
    dyn ModelAlgo
);
context_strategy_accessors!(
    model_healer,
    get_model_healer,
    set_model_healer,
    dyn ModelAlgo
);
context_strategy_accessors!(
    pre_processor,
    get_pre_processor,
    set_pre_processor,
    dyn ModelAlgo
);
context_strategy_accessors!(
    face_discret,
    get_face_discret,
    set_face_discret,
    dyn ModelAlgo
);
context_strategy_accessors!(
    post_processor,
    get_post_processor,
    set_post_processor,
    dyn ModelAlgo
);

/// Virtual context operations; derived contexts embed an actual `Context`.
/// Example: implement `context_base` and override `build_model` for a custom context.
pub trait MeshContext: MeshShape {
    /// Actual context subobject; `context.context_base().get_parameters()`.
    fn context_base(&self) -> &Context;
    /// Virtual model construction; `context.build_model()`.
    fn build_model(&self) -> bool {
        self.context_base().build_model()
    }
    /// Virtual edge discretization; `context.discretize_edges()`.
    fn discretize_edges(&self) -> bool {
        self.context_base().discretize_edges()
    }
    /// Virtual healing; `context.heal_model()`.
    fn heal_model(&self) -> bool {
        self.context_base().heal_model()
    }
    /// Virtual preparation; `context.pre_process_model()`.
    fn pre_process_model(&self) -> bool {
        self.context_base().pre_process_model()
    }
    /// Virtual face discretization; `context.discretize_faces(&range)`.
    fn discretize_faces(&self, range: &ProgressRange<'_>) -> bool {
        self.context_base().discretize_faces(range)
    }
    /// Virtual postprocessing; `context.post_process_model()`.
    fn post_process_model(&self) -> bool {
        self.context_base().post_process_model()
    }
    /// Virtual cleanup; `context.clean()`.
    fn clean(&self) {
        self.context_base().clean();
    }
}
impl MeshContext for Context {
    fn context_base(&self) -> &Context {
        self
    }
}
impl std::ops::Deref for dyn MeshContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        self.context_base()
    }
}
// One descriptor serves both views of this concrete native class; its embedded
// Context remains separately projectable for derived native classes.
impl RuntimeTyped for dyn MeshContext {
    const TYPE_NAME: &'static str = "IMeshTools_Context";
    fn static_type() -> &'static RuntimeType {
        Context::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        project_interface::<Self>(object)
    }
}
