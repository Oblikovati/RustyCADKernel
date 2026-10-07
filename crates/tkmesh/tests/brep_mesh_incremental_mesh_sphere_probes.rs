//! Port-only sphere probes of AllowQualityDecrease and of the BRepTools cleanup/reuse
//! order on a sphere face and solid; the upstream OCC31461 declaration itself is mirrored in
//! brep_mesh_incremental_mesh_suite.rs.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkmath::gp::{Axis3, Point3, Sphere3, StandardDirection3};
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
#[path = "support/mesh_counts.rs"]
mod mesh_counts;
use mesh_counts::shape_mesh_counts;

#[test]
fn sphere_face_quality_decrease_probe() {
    let sphere = Sphere3::new(
        Axis3::with_standard(Point3::default(), StandardDirection3::Z),
        10.0,
    )
    .unwrap();
    let maker = BRepBuilderAPIMakeFace::from_sphere(sphere).unwrap();
    let face = maker.face().unwrap();
    let shape = face.clone().into_shape();
    mesh_sphere(&shape, 0.01, false).unwrap();
    let fine = shape_mesh_counts(&shape);
    mesh_sphere(&shape, 0.1, true).unwrap();
    let coarse = shape_mesh_counts(&shape);
    assert!(coarse.0 < fine.0);
    assert!(coarse.1 < fine.1);
}

#[test]
fn sphere_solid_cleanup_reuse_keeps_existing_mesh() {
    let shape = sphere_shape();
    mesh_sphere(&shape, 0.01, false).unwrap();
    mesh_sphere(&shape, 0.1, true).unwrap();
    let coarse = shape_mesh_counts(&shape);
    rustycad_tkbrep::brep_tools::clean_geometry(&shape).unwrap();
    assert_eq!(shape_mesh_counts(&shape), coarse);
    mesh_sphere(&shape, 0.01, false).unwrap();
    rustycad_tkbrep::brep_tools::clean(&shape, false).unwrap();
    assert_eq!(shape_mesh_counts(&shape), coarse);
}

fn sphere_shape() -> Shape {
    let mut maker = BRepPrimAPIMakeSphere::new(10.0).unwrap();
    maker.shape().unwrap().clone()
}

fn mesh_sphere(
    shape: &rustycad_tkbrep::topods::Shape,
    deflection: f64,
    allow_quality_decrease: bool,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    BRepMeshIncrementalMesh::with_parameters(
        shape,
        &MeshParameters {
            deflection,
            allow_quality_decrease,
            ..Default::default()
        },
        &ProgressRange::default(),
    )?;
    Ok(())
}
