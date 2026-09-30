//! Native boundary repair, actual refinement, failure flags and model ownership.
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, Transient},
};
use rustycad_tkmesh::{
    brep_mesh::BRepMeshModelHealer,
    imesh_data::Status,
    imesh_tools::{MeshParameters, ModelAlgo},
};
#[path = "support/face_checker_fixture.rs"]
mod checker_fixture;
#[path = "support/model_healer_fixture.rs"]
mod fixture;
#[path = "support/model_builder_fixture.rs"]
mod model_fixture;
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
fn parameters() -> MeshParameters {
    MeshParameters {
        min_size: 0.001,
        deflection: 0.1,
        ..Default::default()
    }
}
#[test]
fn native_runtime_type_null_model_and_default_constructor() {
    let healer = BRepMeshModelHealer::default();
    assert!(healer.is_instance_name("BRepMesh_ModelHealer"));
    assert!(healer.is_kind_name("IMeshTools_ModelAlgo"));
    assert!(!healer.perform(&Handle::default(), &parameters(), &ProgressRange::default()));
    assert!(healer.process_index(0).is_err());
}
#[test]
fn production_box_repairs_endpoints_and_releases_model() {
    let model = fixture::healer_model(5, 1, &parameters());
    let owner = model.downgrade();
    let healer = BRepMeshModelHealer::new();
    assert!(healer.perform(&model, &parameters(), &ProgressRange::default()));
    for i in 0..model.faces_nb() {
        let face = model.get_face(i).unwrap();
        assert!(!face.status().is_set(Status::Failure));
        assert!(!face.get_wire(0).unwrap().status().is_set(Status::OpenWire));
    }
    drop(model);
    assert!(owner.upgrade().is_null());
    assert!(healer.process_index(0).is_err());
}
#[test]
fn empty_pcurves_fail_per_face_without_failing_stage() {
    let model = fixture::healer_model(4, 3, &parameters());
    assert!(BRepMeshModelHealer::new().perform(&model, &parameters(), &ProgressRange::default()));
    assert!(model.get_face(0).unwrap().status().is_set(Status::Failure));
}
#[test]
fn outdated_edges_mark_face_without_marking_connected_wire_open() {
    let model = fixture::healer_model(4, 4, &parameters());
    assert!(BRepMeshModelHealer::new().perform(&model, &parameters(), &ProgressRange::default()));
    let face = model.get_face(0).unwrap();
    assert!(face.status().is_set(Status::Outdated));
    assert!(!face.get_wire(0).unwrap().status().is_set(Status::OpenWire));
}
#[test]
fn crossing_wire_exhausts_exactly_five_deflection_reductions() {
    let model = fixture::healer_model(11, 1, &parameters());
    let mut expected = 0.01;
    for _ in 0..5 {
        expected /= 3.0;
    }
    assert!(BRepMeshModelHealer::new().perform(&model, &parameters(), &ProgressRange::default()));
    assert_eq!(model.get_edge(0).unwrap().get_deflection(), expected);
    let face = model.get_face(0).unwrap();
    assert!(face.status().is_set(Status::SelfIntersectingWire));
    assert!(face.status().is_set(Status::Failure));
    assert!(face.get_wire(0).unwrap().status().is_set(Status::OpenWire));
}
#[test]
fn two_edge_wire_adds_samples_after_degenerate_face_check() {
    let model = fixture::healer_model(13, 1, &parameters());
    assert!(BRepMeshModelHealer::new().perform(&model, &parameters(), &ProgressRange::default()));
    for i in 0..2 {
        let edge = model.get_edge(i).unwrap();
        assert_eq!(edge.get_curve().parameters_nb(), 3);
        assert_eq!(edge.get_deflection(), 0.01 / 3.0);
    }
}
#[test]
fn parallel_amplification_and_repeated_perform_preserve_failure_flags() {
    let settings = MeshParameters {
        in_parallel: true,
        min_size: 10.0,
        ..parameters()
    };
    let model = fixture::healer_model(11, 1, &settings);
    let healer = BRepMeshModelHealer::new();
    assert!(healer.perform(&model, &settings, &ProgressRange::default()));
    let reduced = model.get_edge(0).unwrap().get_deflection();
    assert!(healer.perform(&model, &settings, &ProgressRange::default()));
    assert_eq!(model.get_edge(0).unwrap().get_deflection(), reduced);
}
#[test]
fn face_operator_catches_missing_active_map() {
    let model = fixture::healer_model(4, 0, &parameters());
    let face = model.get_face(0).unwrap();
    BRepMeshModelHealer::new().process_face(&face);
    assert!(face.status().is_set(Status::Failure));
}

#[test]
fn distinct_vertices_use_strict_distance_and_missing_vertices_stay_open() {
    for (mode, open) in [(6, false), (7, false), (9, true), (10, true), (11, true)] {
        let model = fixture::healer_model(4, mode, &parameters());
        assert!(BRepMeshModelHealer::new().perform(
            &model,
            &parameters(),
            &ProgressRange::default()
        ));
        let wire = model.get_face(0).unwrap().get_wire(0).unwrap();
        assert_eq!(wire.status().is_set(Status::OpenWire), open, "mode {mode}");
    }
}
