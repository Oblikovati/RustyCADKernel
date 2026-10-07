//! BRepMesh_DelabellaMeshAlgoFactory and the Delabella context: algorithm choice per
//! surface type and mesh totals equal to the pinned native build (tkmesh_probe
//! `mesh_delabella_shape`).
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmesh::brep_mesh::{BRepMeshDelabellaMeshAlgoFactory, BRepMeshIncrementalMesh};
use rustycad_tkmesh::imesh_tools::{MeshAlgoFactory, MeshAlgoType, MeshParameters};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
#[path = "support/mesh_counts.rs"]
mod mesh_counts;
use mesh_counts::shape_mesh_counts;

fn algo_type(surface: SurfaceType, internal_vertices_mode: bool) -> &'static str {
    let parameters = MeshParameters {
        internal_vertices_mode,
        ..MeshParameters::default()
    };
    let algo = BRepMeshDelabellaMeshAlgoFactory::new()
        .get_algo(surface, &parameters)
        .unwrap();
    algo.dynamic_type().map_or("null", |kind| kind.name())
}

#[test]
fn factory_uses_delabella_except_on_cylinders() {
    const DELABELLA: &str = "BRepMesh_DelabellaBaseMeshAlgo";
    const WATSON: &str = "BRepMesh_DelaunayBaseMeshAlgo";
    assert_eq!(algo_type(SurfaceType::Plane, false), DELABELLA);
    assert_eq!(algo_type(SurfaceType::Plane, true), DELABELLA);
    assert_eq!(algo_type(SurfaceType::Cylinder, false), WATSON);
    assert_eq!(algo_type(SurfaceType::Cylinder, true), WATSON);
    for surface in [
        SurfaceType::Sphere,
        SurfaceType::Cone,
        SurfaceType::Torus,
        SurfaceType::SurfaceOfRevolution,
        SurfaceType::SurfaceOfExtrusion,
        SurfaceType::BSplineSurface,
        SurfaceType::OtherSurface,
    ] {
        assert_eq!(algo_type(surface, false), DELABELLA, "{surface:?}");
    }
}

fn delabella_counts(
    shape: &rustycad_tkbrep::topods::Shape,
    deflection: f64,
    internal_vertices_mode: bool,
) -> Result<(i32, i32), Failure> {
    let parameters = MeshParameters {
        mesh_algo: MeshAlgoType::Delabella,
        deflection,
        internal_vertices_mode,
        ..MeshParameters::default()
    };
    BRepMeshIncrementalMesh::with_parameters(shape, &parameters, &ProgressRange::default())?;
    Ok(shape_mesh_counts(shape))
}

#[test]
fn delabella_meshes_match_native_totals() -> Result<(), Failure> {
    let solid = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)?
        .shape()?
        .clone();
    assert_eq!(delabella_counts(&solid, 0.5, false)?, (24, 12));
    let solid = BRepPrimAPIMakeCylinder::new(5.0, 10.0).shape()?.clone();
    assert_eq!(delabella_counts(&solid, 0.1, false)?, (106, 100));
    let solid = BRepPrimAPIMakeCone::new(10.0, 5.0, 10.0)?.shape()?.clone();
    assert_eq!(delabella_counts(&solid, 0.5, true)?, (134, 152));
    let solid = BRepPrimAPIMakeSphere::new(10.0)?.shape()?.clone();
    assert_eq!(delabella_counts(&solid, 0.1, false)?, (507, 978));
    Ok(())
}
