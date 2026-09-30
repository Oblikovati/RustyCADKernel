//! Native preprocessing preserves reuse flags and selectively removes outdated representations.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, Transient},
};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::{
    brep_mesh::BRepMeshModelPreProcessor,
    imesh_data::{MeshModel, Status},
    imesh_tools::{MeshParameters, ModelAlgo},
};
#[path = "support/pre_processor_fixture.rs"]
mod fixture;
#[path = "support/post_processor_fixture.rs"]
mod post_fixture;
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
fn perform(model: &Handle<dyn MeshModel>, allow_decrease: bool) -> bool {
    BRepMeshModelPreProcessor::new().perform(
        model,
        &MeshParameters {
            allow_quality_decrease: allow_decrease,
            ..Default::default()
        },
        &ProgressRange::default(),
    )
}
#[test]
fn null_model_and_algorithm_ancestry_match_native() {
    let processor = BRepMeshModelPreProcessor::new();
    assert!(processor.is_kind_name("IMeshTools_ModelAlgo"));
    assert!(processor.is_instance_name("BRepMesh_ModelPreProcessor"));
    assert!(!perform(&Handle::default(), false));
}
#[test]
fn source_deflection_controls_reuse_but_estimate_is_retained() {
    let model = fixture::existing_model(&[1, 0, 0, 0, 1, 0, 0, 0]);
    assert!(perform(&model, false));
    let face = model.get_face(0).unwrap();
    assert!(face.status().is_set(Status::Reused));
    assert_eq!(face.get_deflection(), 0.1);
    assert!(
        !BRepTool::triangulation(&face.get_face().unwrap(), mesh_purpose::NONE)
            .unwrap()
            .0
            .is_null()
    );
}
#[test]
fn invalid_triangle_indices_remove_face_and_edge_polygons() {
    for invalid in [1, 2] {
        let model = fixture::existing_model(&[2, 0, 0, 1, 0, invalid, 0, 0]);
        let face = model.get_face(0).unwrap();
        let (mesh, location) =
            BRepTool::triangulation(&face.get_face().unwrap(), mesh_purpose::NONE).unwrap();
        assert!(perform(&model, false));
        assert!(face.status().is_set(Status::Outdated));
        assert!(
            BRepTool::triangulation(&face.get_face().unwrap(), mesh_purpose::NONE)
                .unwrap()
                .0
                .is_null()
        );
        assert!(
            BRepTool::polygon_on_triangulation(
                &model.get_edge(0).unwrap().get_edge().unwrap(),
                &mesh,
                &location
            )
            .unwrap()
            .is_null()
        );
    }
}
#[test]
fn quality_decrease_rejects_overresolved_existing_mesh() {
    let retained = fixture::existing_model(&[1, 0, 0, 2, 0, 0, 0, 0]);
    assert!(perform(&retained, false));
    assert!(
        retained
            .get_face(0)
            .unwrap()
            .status()
            .is_set(Status::Reused)
    );
    let removed = fixture::existing_model(&[1, 0, 0, 2, 0, 0, 0, 0]);
    assert!(perform(&removed, true));
    assert!(
        removed
            .get_face(0)
            .unwrap()
            .status()
            .is_set(Status::Outdated)
    );
}
#[test]
fn free_edge_cleanup_requires_outdated_flag() {
    for status in [0, 64] {
        let model = fixture::existing_model(&[0, status, 0, 0, 0, 0, 0, 0]);
        let topology = model.get_edge(0).unwrap().get_edge().unwrap();
        assert!(!BRepTool::polygon3d(&topology).unwrap().0.is_null());
        assert!(perform(&model, false));
        assert_eq!(
            BRepTool::polygon3d(&topology).unwrap().0.is_null(),
            status == 64
        );
    }
}
#[test]
fn cone_seam_gets_matching_three_dimensional_and_parametric_samples() {
    let parameters = MeshParameters {
        min_size: 0.001,
        deflection: 0.01,
        ..Default::default()
    };
    let model = fixture::cone_model(1, &parameters);
    let before: Vec<_> = (0..model.edges_nb())
        .map(|i| model.get_edge(i).unwrap().get_curve().parameters_nb())
        .collect();
    assert!(BRepMeshModelPreProcessor::new().perform(
        &model,
        &parameters,
        &ProgressRange::default()
    ));
    let mut amplified = false;
    for (index, count) in before.into_iter().enumerate() {
        let edge = model.get_edge(index as i32).unwrap();
        if edge.get_curve().parameters_nb() <= count {
            continue;
        }
        amplified = true;
        assert_eq!(count, 2);
        assert_eq!(
            edge.get_pcurve(0).unwrap().parameters_nb(),
            edge.get_curve().parameters_nb()
        );
        assert_eq!(
            edge.get_pcurve(1).unwrap().parameters_nb(),
            edge.get_curve().parameters_nb()
        );
    }
    assert!(amplified);
}
fn cone_seam(model: &Handle<dyn MeshModel>) -> rustycad_tkmesh::imesh_data::EdgeHandle {
    use rustycad_tkg3d::top_abs::Orientation;
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap();
        if edge.pcurves_nb() < 2 {
            continue;
        }
        let face = edge.get_pcurve(0).unwrap().get_face();
        if edge.get_pcurve_for(&face, Orientation::Forward).unwrap()
            != edge.get_pcurve_for(&face, Orientation::Reversed).unwrap()
        {
            return edge;
        }
    }
    panic!("cone has no seam; expected two distinct pcurves on one face");
}
#[test]
fn missing_geometric_pcurves_preserve_prior_3d_insertions() {
    let parameters = MeshParameters {
        min_size: 0.001,
        deflection: 0.01,
        ..Default::default()
    };
    let model = fixture::cone_model(1, &parameters);
    fixture::alter_cone(&model, 2);
    let seam = cone_seam(&model);
    assert!(BRepMeshModelPreProcessor::new().perform(
        &model,
        &parameters,
        &ProgressRange::default()
    ));
    assert!(seam.get_curve().parameters_nb() > 2);
    assert_eq!(seam.get_pcurve(0).unwrap().parameters_nb(), 2);
    assert_eq!(seam.get_pcurve(1).unwrap().parameters_nb(), 2);
}
#[test]
fn reversed_discrete_parameters_keep_descending_insertions() {
    let parameters = MeshParameters {
        min_size: 0.001,
        deflection: 0.01,
        ..Default::default()
    };
    let model = fixture::cone_model(1, &parameters);
    fixture::alter_cone(&model, 1);
    assert!(BRepMeshModelPreProcessor::new().perform(
        &model,
        &parameters,
        &ProgressRange::default()
    ));
    let curve = cone_seam(&model).get_curve();
    assert!(curve.parameters_nb() > 2);
    for index in 1..curve.parameters_nb() {
        assert!(curve.get_parameter(index - 1).unwrap() > curve.get_parameter(index).unwrap());
    }
}
#[test]
fn zero_uv_span_existing_samples_and_face_failure_skip_amplification() {
    let parameters = MeshParameters {
        min_size: 0.001,
        deflection: 0.01,
        ..Default::default()
    };
    for mode in [3, 4, 5] {
        let model = fixture::cone_model(1, &parameters);
        fixture::alter_cone(&model, mode);
        let curve = cone_seam(&model).get_curve();
        let count = curve.parameters_nb();
        assert!(BRepMeshModelPreProcessor::new().perform(
            &model,
            &parameters,
            &ProgressRange::default()
        ));
        assert_eq!(curve.parameters_nb(), count);
    }
}
