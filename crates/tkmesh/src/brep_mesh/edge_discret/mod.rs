//! Native edge discretization, existing polygon reuse and per-edge failure flags.
use super::{BRepMeshCurveTessellator, Deflection, EdgeTessellationExtractor, ShapeTool};
use crate::{
    imesh_data::{EdgeHandle, FaceHandle, MeshModel, Status},
    imesh_tools::{CurveTessellator, MeshParameters, ModelAlgo},
};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::top_abs::Orientation;
mod process;
mod tessellate;
#[derive(Default)]
struct EdgeDiscretState {
    model: Handle<dyn MeshModel>,
    parameters: MeshParameters,
}
/// Edge processing strategy; `BRepMeshEdgeDiscret::new().perform(&model, &parameters, &range)`.
#[derive(Default)]
pub struct BRepMeshEdgeDiscret {
    transient: StandardTransient,
    state: SharedState<EdgeDiscretState>,
}
impl_transient!(BRepMeshEdgeDiscret,"BRepMesh_EdgeDiscret",dyn ModelAlgo,transient,interfaces:[dyn ModelAlgo]);
impl BRepMeshEdgeDiscret {
    /// Empty native edge stage; `BRepMeshEdgeDiscret::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Free-edge tessellation factory; `BRepMeshEdgeDiscret::create_edge_tessellator(&edge, &parameters, 2)?`.
    pub fn create_edge_tessellator(
        edge: &EdgeHandle,
        parameters: &MeshParameters,
        minimum_points: i32,
    ) -> Result<Handle<dyn CurveTessellator>, Failure> {
        Ok(Handle::new(BRepMeshCurveTessellator::new(
            edge,
            parameters,
            minimum_points,
        )?)
        .cast())
    }
    /// SameParameter chooses the 3D constructor; `BRepMeshEdgeDiscret::create_face_tessellator(&edge, orientation, &face, &parameters, 2)?`.
    pub fn create_face_tessellator(
        edge: &EdgeHandle,
        orientation: Orientation,
        face: &FaceHandle,
        parameters: &MeshParameters,
        minimum_points: i32,
    ) -> Result<Handle<dyn CurveTessellator>, Failure> {
        if edge.get_same_param() {
            return Self::create_edge_tessellator(edge, parameters, minimum_points);
        }
        Ok(Handle::new(BRepMeshCurveTessellator::on_face(
            edge,
            orientation,
            face,
            parameters,
            minimum_points,
        )?)
        .cast())
    }
    /// Existing polygon extraction factory; `BRepMeshEdgeDiscret::create_edge_tessellation_extractor(&edge, &face)?`.
    pub fn create_edge_tessellation_extractor(
        edge: &EdgeHandle,
        face: &FaceHandle,
    ) -> Result<Handle<dyn CurveTessellator>, Failure> {
        Ok(Handle::new(EdgeTessellationExtractor::new(edge, face)?).cast())
    }
    /// Native function-call operator during a model run; `stage.process_edge(index)?`.
    pub fn process_edge(&self, index: i32) -> Result<(), Failure> {
        let (model, parameters) = {
            let state = self.state.read();
            (state.model.clone(), state.parameters)
        };
        let model = model.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "null model; expected active edge discretization",
            )
        })?;
        let edge = model.get_edge(index)?;
        if process::discretize_edge(&edge, model.get_max_size(), &parameters).is_err() {
            edge.status().set_status(Status::Failure);
        }
        Ok(())
    }
}
impl ModelAlgo for BRepMeshEdgeDiscret {
    fn perform_internal(
        &self,
        model: &Handle<dyn MeshModel>,
        parameters: &MeshParameters,
        _range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        *self.state.write() = EdgeDiscretState {
            model: model.clone(),
            parameters: *parameters,
        };
        if model.is_null() {
            return Ok(false);
        }
        crate::model_parallel::model_indices(model.edges_nb(), parameters.in_parallel, |index| {
            self.process_edge(index)
        })?;
        self.state.write().model.nullify();
        Ok(true)
    }
}
