//! Incremental mesh entry point for supported planar boundary meshes.
use super::{
    BRepMeshEdgeDiscret, BRepMeshModelBuilder, BRepMeshModelHealer, BRepMeshModelPostProcessor,
    BRepMeshModelPreProcessor, planar_face_discret::BRepMeshPlanarFaceDiscret,
};
use crate::imesh_data::{MeshModel, Status};
use crate::imesh_tools::{MeshParameters, ModelAlgo, ModelBuilder};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::{message::ProgressRange, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Incremental mesh facade for planar faces; `BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())?`.
pub struct BRepMeshIncrementalMesh {
    status_flags: i32,
}

impl BRepMeshIncrementalMesh {
    /// Reproduces OCCT's empty-compound no-op; `BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())?`.
    pub fn with_parameters(
        shape: &Shape,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<Self, Failure> {
        validate_parameters(parameters)?;
        if is_empty_compound(shape)? {
            return Ok(Self { status_flags: 0 });
        }
        let status_flags =
            Self::mesh_planar_shape(shape, &normalized_parameters(parameters), range)?;
        Ok(Self { status_flags })
    }

    /// Accumulated face and wire status mask; `mesher.get_status_flags()`.
    pub fn get_status_flags(&self) -> i32 {
        self.status_flags
    }

    fn mesh_planar_shape(
        shape: &Shape,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<i32, Failure> {
        let builder = BRepMeshModelBuilder::new();
        let model = builder.perform(shape, parameters);
        if model.is_null() {
            return Err(unsupported_shape(shape));
        }
        if model_faces(&model) == 0 {
            return mesh_free_edges(shape, &model, parameters, range);
        }
        if !BRepMeshEdgeDiscret::new().perform(&model, parameters, range) {
            return Err(unsupported_shape(shape));
        }
        if !BRepMeshModelHealer::new().perform(&model, parameters, range) {
            return Err(unsupported_shape(shape));
        }
        if !BRepMeshModelPreProcessor::new().perform(&model, parameters, range) {
            return Err(unsupported_shape(shape));
        }
        let mesher = BRepMeshPlanarFaceDiscret::new();
        if !mesher.perform(&model, parameters, range) {
            mark_failed_faces(&model)?;
            return model_status_flags(&model);
        }
        if !BRepMeshModelPostProcessor::new().perform(&model, parameters, range) {
            return Err(unsupported_shape(shape));
        }
        model_status_flags(&model)
    }
}

fn model_faces(model: &rustycad_tkernel::standard::Handle<dyn MeshModel>) -> i32 {
    model.faces_nb()
}

fn mesh_free_edges(
    shape: &Shape,
    model: &rustycad_tkernel::standard::Handle<dyn MeshModel>,
    parameters: &MeshParameters,
    range: &ProgressRange<'_>,
) -> Result<i32, Failure> {
    if model.edges_nb() == 0 {
        return Err(unsupported_shape(shape));
    }
    if !BRepMeshEdgeDiscret::new().perform(model, parameters, range) {
        return Err(unsupported_shape(shape));
    }
    model_status_flags(model)
}

fn mark_failed_faces(
    model: &rustycad_tkernel::standard::Handle<dyn MeshModel>,
) -> Result<(), Failure> {
    for index in 0..model.faces_nb() {
        model.get_face(index)?.status().set_status(Status::Failure);
    }
    Ok(())
}

fn model_status_flags(
    model: &rustycad_tkernel::standard::Handle<dyn MeshModel>,
) -> Result<i32, Failure> {
    let mut flags = 0;
    for face_index in 0..model.faces_nb() {
        let face = model.get_face(face_index)?;
        flags |= face.status().get_status_mask();
        for wire_index in 0..face.wires_nb() {
            flags |= face.get_wire(wire_index)?.status().get_status_mask();
        }
    }
    Ok(flags)
}

fn unsupported_shape(shape: &Shape) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!(
            "BRepMesh_IncrementalMesh shape {:?}; expected planar faces",
            shape.shape_type()
        ),
    )
}

fn validate_parameters(parameters: &MeshParameters) -> Result<(), Failure> {
    let invalid_linear = parameters.deflection < rustycad_tkernel::precision::confusion();
    let invalid_angle = parameters.angle < rustycad_tkernel::precision::angular();
    if invalid_linear || invalid_angle {
        return Err(Failure::with_message(
            "Standard_NumericError",
            "BRepMesh_IncrementalMesh::initParameters : invalid parameter value",
        ));
    }
    Ok(())
}

fn normalized_parameters(parameters: &MeshParameters) -> MeshParameters {
    let mut normalized = *parameters;
    if normalized.deflection_interior < rustycad_tkernel::precision::confusion() {
        normalized.deflection_interior = normalized.deflection;
    }
    if normalized.min_size < rustycad_tkernel::precision::confusion() {
        normalized.min_size = (MeshParameters::rel_min_size()
            * normalized.deflection.min(normalized.deflection_interior))
        .max(rustycad_tkernel::precision::confusion());
    }
    if normalized.angle_interior < rustycad_tkernel::precision::angular() {
        normalized.angle_interior = 2.0 * normalized.angle;
    }
    normalized
}

fn is_empty_compound(shape: &Shape) -> Result<bool, Failure> {
    if shape.shape_type()? != ShapeEnum::Compound {
        return Ok(false);
    }
    Ok(!ShapeIterator::new(shape, false, false).more())
}
