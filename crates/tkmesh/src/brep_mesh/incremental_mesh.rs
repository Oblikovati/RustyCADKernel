//! Minimal `BRepMesh_IncrementalMesh` constructor behavior for empty compounds.
use crate::imesh_tools::MeshParameters;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::{message::ProgressRange, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Incremental mesh facade while face discretization is being ported.
pub struct BRepMeshIncrementalMesh;

impl BRepMeshIncrementalMesh {
    /// Reproduces OCCT's empty-compound no-op; `BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())?`.
    pub fn with_parameters(
        shape: &Shape,
        parameters: &MeshParameters,
        _range: &ProgressRange<'_>,
    ) -> Result<Self, Failure> {
        validate_parameters(parameters)?;
        if !is_empty_compound(shape)? {
            return Err(Failure::with_message(
                "Standard_NotImplemented",
                "BRepMesh_IncrementalMesh currently supports only an empty compound",
            ));
        }
        Ok(Self)
    }
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

fn is_empty_compound(shape: &Shape) -> Result<bool, Failure> {
    if shape.shape_type()? != ShapeEnum::Compound {
        return Ok(false);
    }
    Ok(!ShapeIterator::new(shape, false, false).more())
}
