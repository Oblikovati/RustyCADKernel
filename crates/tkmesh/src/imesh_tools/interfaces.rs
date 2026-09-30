//! Native abstract strategy interfaces and ModelAlgo's exception boundary.
use super::MeshParameters;
use crate::imesh_data::{FaceHandle, MeshModel};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{
    impl_transient_interface,
    message::ProgressRange,
    standard::{Failure, Handle, StandardTransient, Transient},
};
use rustycad_tkmath::{geom_abs::SurfaceType, gp::Point3};
/// Curve sampling contract; `tessellator.points_nb()`.
pub trait CurveTessellator: Transient {
    /// Number of tessellation samples; `tessellator.points_nb()`.
    fn points_nb(&self) -> i32;
    /// Evaluate a sample, preserving output values when implementations return false.
    /// Example: `tessellator.value(1, &mut point, &mut parameter)?`.
    fn value(&self, index: i32, point: &mut Point3, parameter: &mut f64) -> Result<bool, Failure>;
}
impl_transient_interface!(
    dyn CurveTessellator,
    "IMeshTools_CurveTessellator",
    StandardTransient
);
/// Strategy that triangulates one discrete face; `algorithm.perform(&face, &parameters, &range)?`.
pub trait MeshAlgo: Transient {
    /// Run face processing; `algorithm.perform(&face, &parameters, &range)?`.
    fn perform(
        &self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure>;
}
impl_transient_interface!(dyn MeshAlgo, "IMeshTools_MeshAlgo", StandardTransient);
/// Select face meshing strategies; `factory.get_algo(kind, &parameters)?`.
pub trait MeshAlgoFactory: Transient {
    /// Strategy for the surface kind; `factory.get_algo(SurfaceType::Plane, &parameters)?`.
    fn get_algo(
        &self,
        surface_type: SurfaceType,
        parameters: &MeshParameters,
    ) -> Result<Handle<dyn MeshAlgo>, Failure>;
}
impl_transient_interface!(
    dyn MeshAlgoFactory,
    "IMeshTools_MeshAlgoFactory",
    StandardTransient
);
/// Model-processing strategy; native `Perform` converts any Standard_Failure to false.
pub trait ModelAlgo: Transient {
    /// Exception boundary; `algorithm.perform(&model, &parameters, &range)`.
    fn perform(
        &self,
        model: &Handle<dyn MeshModel>,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> bool {
        self.perform_internal(model, parameters, range)
            .unwrap_or(false)
    }
    /// Native protected implementation hook; `perform` owns failure conversion.
    /// Example: implement `perform_internal` in an edge tessellation strategy.
    fn perform_internal(
        &self,
        model: &Handle<dyn MeshModel>,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure>;
}
impl_transient_interface!(dyn ModelAlgo, "IMeshTools_ModelAlgo", StandardTransient);
/// Shape traversal receiver; `visitor.visit_face(&face)?`.
pub trait ShapeVisitor: Transient {
    /// Observe a face; `visitor.visit_face(&face)?`.
    fn visit_face(&self, face: &Face) -> Result<(), Failure>;
    /// Observe an edge; `visitor.visit_edge(&edge)?`.
    fn visit_edge(&self, edge: &Edge) -> Result<(), Failure>;
}
impl_transient_interface!(
    dyn ShapeVisitor,
    "IMeshTools_ShapeVisitor",
    StandardTransient
);
