//! Native polygon publication and triangulation deflection estimation.
use crate::{
    imesh_data::{FaceHandle, MeshModel, Status},
    imesh_tools::{MeshParameters, ModelAlgo},
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    osd::Parallel,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::poly::{TriangulationParameters, mesh_purpose};
mod polygons;
/// Publishes discrete curves and mesh parameters; `processor.perform(&model, &parameters, &range)`.
#[derive(Default)]
pub struct BRepMeshModelPostProcessor {
    transient: StandardTransient,
}
impl_transient!(BRepMeshModelPostProcessor,"BRepMesh_ModelPostProcessor",dyn ModelAlgo,transient,interfaces:[dyn ModelAlgo]);
impl BRepMeshModelPostProcessor {
    /// Empty stateless postprocessor; `BRepMeshModelPostProcessor::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}
impl ModelAlgo for BRepMeshModelPostProcessor {
    fn perform_internal(
        &self,
        model: &Handle<dyn MeshModel>,
        parameters: &MeshParameters,
        _range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if model.is_null() {
            return Ok(false);
        }
        // OCCT 3d097a03 forces edge publication sequentially because edges may share TShapes.
        for index in 0..model.edges_nb() {
            polygons::commit_edge(&model.get_edge(index)?)?;
        }
        estimate_faces(model, parameters)?;
        Ok(true)
    }
}
fn estimate_faces(
    model: &Handle<dyn MeshModel>,
    parameters: &MeshParameters,
) -> Result<(), Failure> {
    let settings = Handle::new(TriangulationParameters::new(
        parameters.deflection,
        parameters.angle,
        parameters.min_size,
    ));
    if !parameters.in_parallel {
        for index in 0..model.faces_nb() {
            estimate_face(&model.get_face(index)?, &settings)?;
        }
        return Ok(());
    }
    estimate_parallel_faces(model, &settings)
}
fn estimate_parallel_faces(
    model: &Handle<dyn MeshModel>,
    settings: &Handle<TriangulationParameters>,
) -> Result<(), Failure> {
    let result = SharedState::new(Ok(()));
    Parallel::for_range(
        0,
        model.faces_nb(),
        |index| {
            if let Err(error) = model
                .get_face(index)
                .and_then(|face| estimate_face(&face, settings))
            {
                *result.write() = Err(error);
            }
        },
        false,
    )?;
    std::mem::replace(&mut *result.write(), Ok(()))
}
fn estimate_face(
    face: &FaceHandle,
    settings: &Handle<TriangulationParameters>,
) -> Result<(), Failure> {
    if skip_face(face) {
        return Ok(());
    }
    let topology = face.get_face()?;
    rustycad_tktopalgo::brep_lib::update_deflection(&topology)?;
    let (triangulation, _) = BRepTool::triangulation(&topology, mesh_purpose::NONE)?;
    if !triangulation.is_null() {
        triangulation.set_parameters(settings.clone());
    }
    Ok(())
}
fn skip_face(face: &FaceHandle) -> bool {
    face.status().is_set(Status::Failure) || face.status().is_set(Status::Reused)
}
