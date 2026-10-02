//! Incremental mesh entry point for supported planar boundary meshes.
use super::{
    BRepMeshEdgeDiscret, BRepMeshModelBuilder, planar_face_discret::BRepMeshPlanarFaceDiscret,
};
use crate::imesh_data::MeshModel;
use crate::imesh_tools::{MeshParameters, ModelAlgo, ModelBuilder};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::{message::ProgressRange, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Incremental mesh facade for planar faces; `BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())?`.
pub struct BRepMeshIncrementalMesh;

impl BRepMeshIncrementalMesh {
    /// Reproduces OCCT's empty-compound no-op; `BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())?`.
    pub fn with_parameters(
        shape: &Shape,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<Self, Failure> {
        validate_parameters(parameters)?;
        if is_empty_compound(shape)? {
            return Ok(Self);
        }
        Self::mesh_planar_shape(shape, &normalized_parameters(parameters), range)?;
        Ok(Self)
    }

    fn mesh_planar_shape(
        shape: &Shape,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let builder = BRepMeshModelBuilder::new();
        let model = builder.perform(shape, parameters);
        if model.is_null() || model_faces(&model) == 0 {
            return Err(unsupported_shape(shape));
        }
        if !BRepMeshEdgeDiscret::new().perform(&model, parameters, range) {
            return Err(unsupported_shape(shape));
        }
        let mesher = BRepMeshPlanarFaceDiscret::new();
        if !mesher.perform(&model, parameters, range) {
            return Err(unsupported_shape(shape));
        }
        Ok(())
    }
}

fn model_faces(model: &rustycad_tkernel::standard::Handle<dyn MeshModel>) -> i32 {
    model.faces_nb()
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
