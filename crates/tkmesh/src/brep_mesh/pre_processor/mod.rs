//! Native cone seam amplification, mesh reuse checks and outdated polygon removal.
use super::{ConeRangeSplitter, Deflection, ShapeTool};
use crate::{
    imesh_data::{FaceHandle, MeshModel, Status},
    imesh_tools::{MeshParameters, ModelAlgo},
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkmath::poly::mesh_purpose;
mod cleanup;
mod consistency;
mod seam;
mod split_curve;
/// Native model preprocessing stage; `BRepMeshModelPreProcessor::new()`.
#[derive(Default)]
pub struct BRepMeshModelPreProcessor {
    transient: StandardTransient,
}
impl_transient!(BRepMeshModelPreProcessor,"BRepMesh_ModelPreProcessor",dyn ModelAlgo,transient,interfaces:[dyn ModelAlgo]);
impl BRepMeshModelPreProcessor {
    /// Construct the stateless native stage; `BRepMeshModelPreProcessor::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}
impl ModelAlgo for BRepMeshModelPreProcessor {
    fn perform_internal(
        &self,
        model: &Handle<dyn MeshModel>,
        parameters: &MeshParameters,
        _range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if model.is_null() {
            return Ok(false);
        }
        crate::model_parallel::model_indices(model.faces_nb(), parameters.in_parallel, |index| {
            seam::amplify_face_seam(&model.get_face(index)?, parameters)
        })?;
        crate::model_parallel::model_indices(model.faces_nb(), parameters.in_parallel, |index| {
            consistency::check_face_mesh(&model.get_face(index)?, parameters.allow_quality_decrease)
        })?;
        cleanup::clear_outdated_polygons(model)?;
        Ok(true)
    }
}
