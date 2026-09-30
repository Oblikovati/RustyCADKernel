//! The native model-builder exception boundary retains the real Message_Algorithm base.
use super::MeshParameters;
use crate::imesh_data::MeshModel;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient_interface,
    message::{Algorithm, Status},
    standard::{Failure, Handle, Transient},
};

/// Abstract model construction; `builder.perform(&shape, &parameters)`.
pub trait ModelBuilder: Transient {
    /// The actual message base subobject; `builder.algorithm().get_status()`.
    fn algorithm(&self) -> &Algorithm;
    /// Clear old statuses and convert native failures to Fail2 and a null model.
    /// Example: `builder.perform(&shape, &MeshParameters::default())`.
    fn perform(&self, shape: &Shape, parameters: &MeshParameters) -> Handle<dyn MeshModel> {
        self.algorithm().clear_status();
        match self.perform_internal(shape, parameters) {
            Ok(model) => model,
            Err(_) => {
                self.algorithm().set_status(Status::FAIL2);
                Handle::default()
            }
        }
    }
    /// Native protected hook; implement to construct the actual discrete topology.
    /// Example: `fn perform_internal(&self, shape: &Shape, parameters: &MeshParameters) -> Result<Handle<dyn MeshModel>, Failure>`.
    fn perform_internal(
        &self,
        shape: &Shape,
        parameters: &MeshParameters,
    ) -> Result<Handle<dyn MeshModel>, Failure>;
}
impl_transient_interface!(dyn ModelBuilder, "IMeshTools_ModelBuilder", Algorithm);
impl std::ops::Deref for dyn ModelBuilder {
    type Target = Algorithm;
    fn deref(&self) -> &Self::Target {
        self.algorithm()
    }
}
