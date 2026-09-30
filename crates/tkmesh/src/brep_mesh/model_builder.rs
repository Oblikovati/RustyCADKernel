//! Build the actual discrete BRep model after native bounding-box validation.
use super::{BRepMeshShapeVisitor, ShapeTool};
use crate::{
    brep_mesh_data::DiscreteModel,
    imesh_data::MeshModel,
    imesh_tools::{MeshParameters, ModelBuilder, ShapeExplorer},
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient,
    message::{Algorithm, Status},
    standard::{Failure, Handle, RuntimeTyped},
};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
/// Concrete native model construction; `BRepMeshModelBuilder::new().perform(&shape, &parameters)`.
pub struct BRepMeshModelBuilder {
    algorithm: Algorithm,
}
impl_transient!(BRepMeshModelBuilder,"BRepMesh_ModelBuilder",dyn ModelBuilder,algorithm,interfaces:[dyn ModelBuilder]);
impl Default for BRepMeshModelBuilder {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::with_runtime_type(Self::static_type()),
        }
    }
}
impl BRepMeshModelBuilder {
    /// Empty algorithm state; `BRepMeshModelBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    fn model_size(bounds: &BoundingBox3, parameters: &MeshParameters) -> Result<f64, Failure> {
        let mut size = if parameters.deflection < parameters.deflection_interior {
            parameters.deflection_interior
        } else {
            parameters.deflection
        };
        if parameters.relative {
            ShapeTool::box_max_dimension(bounds, &mut size)?;
        }
        Ok(size)
    }
}
impl std::ops::Deref for BRepMeshModelBuilder {
    type Target = Algorithm;
    fn deref(&self) -> &Self::Target {
        &self.algorithm
    }
}
impl ModelBuilder for BRepMeshModelBuilder {
    fn algorithm(&self) -> &Algorithm {
        &self.algorithm
    }
    fn perform_internal(
        &self,
        shape: &Shape,
        parameters: &MeshParameters,
    ) -> Result<Handle<dyn MeshModel>, Failure> {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(shape, &mut bounds, false)?;
        if bounds.is_void() {
            self.set_status(Status::FAIL1);
            return Ok(Handle::default());
        }
        let model = Handle::new(DiscreteModel::new(shape));
        model.set_max_size(Self::model_size(&bounds, parameters)?);
        let visitor = Handle::new(BRepMeshShapeVisitor::new(&model.cast()));
        ShapeExplorer::new(shape).accept(&visitor.cast())?;
        self.set_status(Status::DONE1);
        Ok(model.cast())
    }
}
