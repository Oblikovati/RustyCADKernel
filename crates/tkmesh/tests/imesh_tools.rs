//! Independent IMeshTools contracts, without a substitute mesher.
#[path = "support/tools_fixture.rs"]
mod fixtures;
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, Transient},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmesh::{
    imesh_data::MeshShape,
    imesh_tools::{MeshAlgoType, MeshParameters, ModelAlgo, ShapeExplorer},
};
#[test]
fn meshing_parameters_keep_native_unnormalized_defaults() {
    let parameters = MeshParameters::default();
    assert_eq!(parameters.mesh_algo, MeshAlgoType::Default);
    assert_eq!((parameters.angle, parameters.deflection), (0.5, 0.001));
    assert_eq!(
        (
            parameters.angle_interior,
            parameters.deflection_interior,
            parameters.min_size
        ),
        (-1.0, -1.0, -1.0)
    );
    assert!(!parameters.in_parallel && !parameters.relative);
    assert!(
        parameters.internal_vertices_mode
            && parameters.control_surface_deflection
            && parameters.clean_model
    );
    assert!(!parameters.enable_control_surface_deflection_all_surfaces);
    assert!(
        !parameters.adjust_min_size
            && !parameters.force_face_deflection
            && !parameters.allow_quality_decrease
    );
    assert_eq!(MeshParameters::rel_min_size(), 0.1);
    assert_eq!(MeshAlgoType::Watson as i32, 0);
    assert_eq!(MeshAlgoType::Delabella as i32, 1);
}
#[test]
fn model_algorithm_boundary_catches_failures_and_preserves_boolean_results() {
    for outcome in [-1, 0, 1] {
        let algorithm = fixtures::OracleModelAlgorithm::new(outcome);
        assert_eq!(
            algorithm.perform(
                &Handle::default(),
                &MeshParameters::default(),
                &ProgressRange::default()
            ),
            outcome > 0
        );
        assert!(algorithm.is_kind_name("IMeshTools_ModelAlgo"));
    }
}
#[test]
fn explorer_orders_free_edges_then_faces_and_suppresses_located_duplicates() {
    let (shape, identities) = fixtures::explorer_fixture(Orientation::Reversed, true);
    let explorer = ShapeExplorer::new(&shape);
    let visitor = Handle::new(fixtures::RecordingShapeVisitor::default());
    assert!(explorer.get_shape().is_same(&shape));
    explorer.accept(&visitor.cast()).unwrap();
    let visited = visitor.get().unwrap().shapes.snapshot();
    assert_eq!(visited.len(), 5);
    assert!(visited[0].is_partner(&identities[0]));
    assert!(visited[0].location().is_identity());
    assert_eq!(visited[0].orientation(), Orientation::Reversed);
    let faces: Vec<_> = visited
        .iter()
        .filter(|s| s.shape_type().unwrap() == ShapeEnum::Face)
        .collect();
    assert_eq!(faces.len(), 2);
    assert!(
        faces
            .iter()
            .all(|s| s.orientation() == Orientation::Forward)
    );
    assert!(faces[0].is_partner(&identities[3]));
    assert!(faces[1].is_partner(&identities[2]));
    explorer.set_shape(&Default::default());
    explorer.accept(&visitor.cast()).unwrap();
    assert_eq!(visitor.get().unwrap().shapes.read().len(), 5);
    assert!(explorer.is_kind_name("IMeshData_Shape"));
}
