//! Native postprocessor publication, reuse and parallel face regression coverage.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, Transient},
};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::{
    brep_mesh::BRepMeshModelPostProcessor,
    imesh_tools::{MeshParameters, ModelAlgo},
};
#[path = "support/post_processor_fixture.rs"]
mod fixtures;
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
#[test]
fn null_model_and_runtime_base() {
    let processor = BRepMeshModelPostProcessor::new();
    assert!(processor.is_kind_name("IMeshTools_ModelAlgo"));
    assert!(!processor.perform(
        &Handle::default(),
        &MeshParameters::default(),
        &ProgressRange::default()
    ));
}
#[test]
fn free_polygon_published_unless_reused() {
    for status in [0, 128] {
        let model = fixtures::post_model(&[0, status, 0, 3, 1, 1, 0]);
        assert!(BRepMeshModelPostProcessor::new().perform(
            &model,
            &MeshParameters::default(),
            &ProgressRange::default()
        ));
        let (polygon, _) =
            BRepTool::polygon3d(&model.get_edge(0).unwrap().get_edge().unwrap()).unwrap();
        assert_eq!(polygon.is_null(), status == 128);
        if !polygon.is_null() {
            assert_eq!(polygon.nb_nodes(), 3);
            assert_eq!(polygon.deflection(), 0.125);
        }
    }
}
#[test]
fn face_estimation_and_shared_parameters_in_parallel() {
    let model = fixtures::post_model(&[2, 0, 0, 2, 2, 1, 1]);
    let parameters = MeshParameters {
        in_parallel: true,
        ..Default::default()
    };
    assert!(BRepMeshModelPostProcessor::new().perform(
        &model,
        &parameters,
        &ProgressRange::default()
    ));
    let meshes: Vec<_> = (0..2)
        .map(|index| {
            BRepTool::triangulation(
                &model.get_face(index).unwrap().get_face().unwrap(),
                mesh_purpose::NONE,
            )
            .unwrap()
            .0
        })
        .collect();
    assert!(meshes[0].parameters() == meshes[1].parameters());
    assert!((meshes[0].deflection() - 0.3).abs() < 1e-15);
    assert_eq!(meshes[0].parameters().deflection(), parameters.deflection);
}
#[test]
fn failed_or_reused_faces_keep_previous_mesh_state() {
    for status in [4, 128, 132] {
        let model = fixtures::post_model(&[1, 0, status, 2, 1, 1, 0]);
        assert!(BRepMeshModelPostProcessor::new().perform(
            &model,
            &MeshParameters::default(),
            &ProgressRange::default()
        ));
        let (mesh, _) = BRepTool::triangulation(
            &model.get_face(0).unwrap().get_face().unwrap(),
            mesh_purpose::NONE,
        )
        .unwrap();
        assert_eq!(mesh.deflection(), 7.0);
        assert!(mesh.parameters().is_null());
    }
}
