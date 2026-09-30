//! Native refinement, internal vertices, surface bounds and constructor failures.
use rustycad_tkernel::standard::Transient;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmesh::{
    brep_mesh::BRepMeshCurveTessellator,
    imesh_tools::{CurveTessellator, MeshParameters},
};
#[path = "support/curve_tessellator_fixture.rs"]
mod fixtures;
fn settings() -> MeshParameters {
    MeshParameters {
        min_size: 0.001,
        ..Default::default()
    }
}
#[test]
fn invalid_minimum_size_preserves_native_failure() {
    let (edge, _) = fixtures::refinement_fixture(0, false, false);
    let result = BRepMeshCurveTessellator::new(&edge, &MeshParameters::default(), 2);
    assert_eq!(result.err().unwrap().exception_type(), "Standard_Failure");
}
#[test]
fn cylindrical_pcurve_refines_straight_representation() {
    let (edge, _face) = fixtures::refinement_fixture(0, false, false);
    edge.set_deflection(0.01);
    edge.set_angular_deflection(0.5);
    edge.set_same_param(true);
    edge.set_same_range(true);
    let tessellator = BRepMeshCurveTessellator::new(&edge, &settings(), 2).unwrap();
    assert!(tessellator.points_nb() > 2);
    assert!(tessellator.is_kind_name("IMeshTools_CurveTessellator"));
    edge.set_same_param(false);
    assert_eq!(
        BRepMeshCurveTessellator::new(&edge, &settings(), 2)
            .unwrap()
            .points_nb(),
        2
    );
}
#[test]
fn internal_vertex_is_inserted_at_native_parameter() {
    let (edge, _face) = fixtures::refinement_fixture(0, false, true);
    edge.set_deflection(0.01);
    edge.set_angular_deflection(0.5);
    let tessellator = BRepMeshCurveTessellator::new(&edge, &settings(), 2).unwrap();
    let mut point = Point3::default();
    let mut parameter = 0.0;
    let mut found = false;
    for index in 1..=tessellator.points_nb() {
        tessellator
            .value(index, &mut point, &mut parameter)
            .unwrap();
        found |= parameter == 0.37;
    }
    assert!(found);
}
#[test]
fn located_bezier_surface_rejects_outside_uv_samples() {
    let (edge, face) = fixtures::refinement_fixture(1, true, true);
    edge.set_deflection(0.01);
    edge.set_angular_deflection(0.5);
    let tessellator =
        BRepMeshCurveTessellator::on_face(&edge, Orientation::Forward, &face, &settings(), 2)
            .unwrap();
    let mut point = Point3::default();
    let mut parameter = 0.0;
    assert!(!tessellator.value(1, &mut point, &mut parameter).unwrap());
    assert_eq!(parameter, -1.0);
    assert!(
        !tessellator
            .value(tessellator.points_nb(), &mut point, &mut parameter)
            .unwrap()
    );
}
