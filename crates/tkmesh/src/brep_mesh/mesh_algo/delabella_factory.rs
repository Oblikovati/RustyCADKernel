// Rust translation of BRepMesh_DelabellaMeshAlgoFactory.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::algo_object::{BRepMeshDelaunayBaseMeshAlgo, FaceMeshingCore};
use super::deflection_control::DelaunayDeflectionControlMeshAlgo;
use super::delabella_base::{BRepMeshDelabellaBaseMeshAlgo, DelabellaTriangulation};
use super::delaunay_insertion::DelaunayNodeInsertionMeshAlgo;
use super::generator::{CustomDelaunayGenerator, CustomGenerator, DelaunayGenerator};
use crate::brep_mesh::{
    BoundaryParamsRangeSplitter, ConeRangeSplitter, CylinderRangeSplitter, DefaultRangeSplitter,
    NURBSRangeSplitter, RangeSplitter, SphereRangeSplitter, TorusRangeSplitter,
};
use crate::imesh_tools::{MeshAlgo, MeshAlgoFactory, MeshParameters};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::SurfaceType;

/// `BRepMesh_CustomDelaunayBaseMeshAlgo<BRepMesh_DelabellaBaseMeshAlgo>`: the base of
/// the Delabella node-insertion and deflection-control algorithms.
type DelabellaDelaunayGenerator = CustomDelaunayGenerator<CustomGenerator<DelabellaTriangulation>>;

/// `BRepMesh_DelabellaMeshAlgoFactory`: Delabella-based algorithms of a complexity
/// suited to each surface type.
///
/// ```
/// use rustycad_tkmath::geom_abs::SurfaceType;
/// use rustycad_tkmesh::brep_mesh::BRepMeshDelabellaMeshAlgoFactory;
/// use rustycad_tkmesh::imesh_tools::{MeshAlgoFactory, MeshParameters};
/// let factory = BRepMeshDelabellaMeshAlgoFactory::new();
/// let algo = factory.get_algo(SurfaceType::Plane, &MeshParameters::default())?;
/// assert!(algo.is_kind_name("BRepMesh_DelabellaBaseMeshAlgo"));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepMeshDelabellaMeshAlgoFactory {
    transient: StandardTransient,
}
impl_transient!(
    BRepMeshDelabellaMeshAlgoFactory,
    "BRepMesh_DelabellaMeshAlgoFactory",
    dyn MeshAlgoFactory,
    transient,
    interfaces: [dyn MeshAlgoFactory]
);

impl BRepMeshDelabellaMeshAlgoFactory {
    /// `BRepMesh_DelabellaMeshAlgoFactory()`; `BRepMeshDelabellaMeshAlgoFactory::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

fn delabella_algo(core: impl FaceMeshingCore + 'static) -> Handle<dyn MeshAlgo> {
    Handle::new(BRepMeshDelabellaBaseMeshAlgo::from_core(core)).cast()
}

/// `new NodeInsertionMeshAlgo<R>::Type`, optionally with surface nodes pre-processed.
fn node_insertion<R: RangeSplitter + Default + Send + 'static>(
    pre_process: bool,
) -> Handle<dyn MeshAlgo> {
    let mut algo = DelaunayNodeInsertionMeshAlgo::<R, DelabellaDelaunayGenerator>::default();
    algo.set_pre_process_surface_nodes(pre_process);
    delabella_algo(algo)
}

/// `new DeflectionControlMeshAlgo<R>::Type` with surface nodes pre-processed.
fn deflection_control<R: RangeSplitter + Default + Send + 'static>() -> Handle<dyn MeshAlgo> {
    let mut algo = DelaunayDeflectionControlMeshAlgo::<R, DelabellaDelaunayGenerator>::default();
    algo.set_pre_process_surface_nodes(true);
    delabella_algo(algo)
}

/// The cylinder choice: the Watson algorithms (`DefaultNodeInsertionMeshAlgo` with
/// internal vertices, else `DefaultBaseMeshAlgo`).
fn cylinder_choice(parameters: &MeshParameters) -> Handle<dyn MeshAlgo> {
    if !parameters.internal_vertices_mode {
        return Handle::new(BRepMeshDelaunayBaseMeshAlgo::new()).cast();
    }
    let algo = DelaunayNodeInsertionMeshAlgo::<CylinderRangeSplitter, DelaunayGenerator>::default();
    Handle::new(BRepMeshDelaunayBaseMeshAlgo::from_core(algo)).cast()
}

/// The plane choice: Delabella node insertion with internal vertices, else the plain
/// Delabella algorithm.
fn plane_choice(parameters: &MeshParameters) -> Handle<dyn MeshAlgo> {
    if parameters.internal_vertices_mode {
        return node_insertion::<DefaultRangeSplitter>(false);
    }
    Handle::new(BRepMeshDelabellaBaseMeshAlgo::new()).cast()
}

impl MeshAlgoFactory for BRepMeshDelabellaMeshAlgoFactory {
    /// `GetAlgo(theSurfaceType, theParameters)`. Unlike `BRepMesh_MeshAlgoFactory`,
    /// `EnableControlSurfaceDeflectionAllSurfaces` is ignored and extrusion, Bezier,
    /// B-spline and other surfaces share the NURBS splitter, as natively.
    fn get_algo(
        &self,
        surface_type: SurfaceType,
        parameters: &MeshParameters,
    ) -> Result<Handle<dyn MeshAlgo>, Failure> {
        Ok(match surface_type {
            SurfaceType::Plane => plane_choice(parameters),
            SurfaceType::Sphere => node_insertion::<SphereRangeSplitter>(true),
            SurfaceType::Cylinder => cylinder_choice(parameters),
            SurfaceType::Cone => node_insertion::<ConeRangeSplitter>(true),
            SurfaceType::Torus => node_insertion::<TorusRangeSplitter>(true),
            SurfaceType::SurfaceOfRevolution => deflection_control::<BoundaryParamsRangeSplitter>(),
            _ => deflection_control::<NURBSRangeSplitter>(),
        })
    }
}
