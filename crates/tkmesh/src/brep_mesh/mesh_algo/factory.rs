// Rust translation of BRepMesh_MeshAlgoFactory.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::algo_object::{BRepMeshDelaunayBaseMeshAlgo, FaceMeshingCore};
use super::deflection_control::DelaunayDeflectionControlMeshAlgo;
use super::delaunay_insertion::DelaunayNodeInsertionMeshAlgo;
use super::generator::DelaunayGenerator;
use crate::brep_mesh::{
    BoundaryParamsRangeSplitter, ConeRangeSplitter, CylinderRangeSplitter, DefaultRangeSplitter,
    ExtrusionRangeSplitter, NURBSRangeSplitter, RangeSplitter, SphereRangeSplitter,
    TorusRangeSplitter, UndefinedRangeSplitter,
};
use crate::imesh_tools::{MeshAlgo, MeshAlgoFactory, MeshParameters};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::SurfaceType;

/// `BRepMesh_MeshAlgoFactory`: the Watson algorithm suited to each surface type.
///
/// ```
/// use rustycad_tkmath::geom_abs::SurfaceType;
/// use rustycad_tkmesh::brep_mesh::BRepMeshMeshAlgoFactory;
/// use rustycad_tkmesh::imesh_tools::{MeshAlgoFactory, MeshParameters};
/// let algo = BRepMeshMeshAlgoFactory::new().get_algo(SurfaceType::Sphere, &MeshParameters::default())?;
/// assert!(!algo.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepMeshMeshAlgoFactory {
    transient: StandardTransient,
}
impl_transient!(
    BRepMeshMeshAlgoFactory,
    "BRepMesh_MeshAlgoFactory",
    dyn MeshAlgoFactory,
    transient,
    interfaces: [dyn MeshAlgoFactory]
);

impl BRepMeshMeshAlgoFactory {
    /// `BRepMesh_MeshAlgoFactory()`; `BRepMeshMeshAlgoFactory::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// `new DeflectionControlMeshAlgo<R>::Type`.
fn deflection_control<R: RangeSplitter + Default + Send + 'static>() -> Handle<dyn MeshAlgo> {
    algo(DelaunayDeflectionControlMeshAlgo::<R, DelaunayGenerator>::default())
}

/// `new NodeInsertionMeshAlgo<R>::Type` (the Delaunay node insertion).
fn node_insertion<R: RangeSplitter + Default + Send + 'static>() -> Handle<dyn MeshAlgo> {
    algo(DelaunayNodeInsertionMeshAlgo::<R, DelaunayGenerator>::default())
}

/// `new BaseMeshAlgo::Type`.
fn base_algo() -> Handle<dyn MeshAlgo> {
    Handle::new(BRepMeshDelaunayBaseMeshAlgo::new()).cast()
}

fn algo(core: impl FaceMeshingCore + 'static) -> Handle<dyn MeshAlgo> {
    Handle::new(BRepMeshDelaunayBaseMeshAlgo::from_core(core)).cast()
}

/// The plane and cylinder choice: deflection control on all surfaces, else node
/// insertion with internal vertices, else the base algorithm.
fn planar_choice<R: RangeSplitter + Default + Send + 'static>(
    parameters: &MeshParameters,
) -> Handle<dyn MeshAlgo> {
    if parameters.enable_control_surface_deflection_all_surfaces {
        return deflection_control::<R>();
    }
    if parameters.internal_vertices_mode {
        return node_insertion::<R>();
    }
    base_algo()
}

/// The analytic choice: deflection control on all surfaces, else node insertion.
fn analytic_choice<R: RangeSplitter + Default + Send + 'static>(
    parameters: &MeshParameters,
) -> Handle<dyn MeshAlgo> {
    if parameters.enable_control_surface_deflection_all_surfaces {
        return deflection_control::<R>();
    }
    node_insertion::<R>()
}

impl MeshAlgoFactory for BRepMeshMeshAlgoFactory {
    /// `GetAlgo(theSurfaceType, theParameters)`.
    fn get_algo(
        &self,
        surface_type: SurfaceType,
        parameters: &MeshParameters,
    ) -> Result<Handle<dyn MeshAlgo>, Failure> {
        Ok(match surface_type {
            SurfaceType::Plane => planar_choice::<DefaultRangeSplitter>(parameters),
            SurfaceType::Sphere => analytic_choice::<SphereRangeSplitter>(parameters),
            SurfaceType::Cylinder => planar_choice::<CylinderRangeSplitter>(parameters),
            SurfaceType::Cone => analytic_choice::<ConeRangeSplitter>(parameters),
            SurfaceType::Torus => analytic_choice::<TorusRangeSplitter>(parameters),
            SurfaceType::SurfaceOfRevolution => deflection_control::<BoundaryParamsRangeSplitter>(),
            SurfaceType::SurfaceOfExtrusion => deflection_control::<ExtrusionRangeSplitter>(),
            SurfaceType::BezierSurface | SurfaceType::BSplineSurface => {
                deflection_control::<NURBSRangeSplitter>()
            }
            _ => deflection_control::<UndefinedRangeSplitter>(),
        })
    }
}
