//! Native edge-stage status, model lifetime and tessellation insertion regressions.
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, Transient},
};
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshEdgeDiscret, BRepMeshModelBuilder},
    imesh_data::Status,
    imesh_tools::{MeshParameters, ModelAlgo, ModelBuilder},
};
#[path = "support/edge_discret_fixture.rs"]
mod existing;
#[path = "support/model_builder_fixture.rs"]
mod fixtures;
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
fn settings() -> MeshParameters {
    MeshParameters {
        min_size: 0.001,
        deflection: 0.1,
        ..Default::default()
    }
}
#[test]
fn null_model_and_native_strategy_base() {
    let stage = BRepMeshEdgeDiscret::new();
    assert!(stage.is_kind_name("IMeshTools_ModelAlgo"));
    assert!(!stage.perform(&Handle::default(), &settings(), &ProgressRange::default()));
    assert!(stage.process_edge(0).is_err());
}
#[test]
fn production_box_discretization_releases_model() {
    let parameters = settings();
    let model = BRepMeshModelBuilder::new().perform(&fixtures::model_shape(5), &parameters);
    let weak = model.downgrade();
    let stage = BRepMeshEdgeDiscret::new();
    assert!(stage.perform(&model, &parameters, &ProgressRange::default()));
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap();
        assert!(!edge.status().is_set(Status::Failure));
        assert_eq!(edge.get_curve().parameters_nb(), 2);
    }
    existing::clear_model_curves(&model);
    drop(model);
    assert!(weak.upgrade().is_null());
}
#[test]
fn bad_parameters_mark_edges_failed_but_stage_succeeds() {
    let model = BRepMeshModelBuilder::new().perform(&fixtures::model_shape(4), &settings());
    assert!(BRepMeshEdgeDiscret::new().perform(
        &model,
        &MeshParameters::default(),
        &ProgressRange::default()
    ));
    for index in 0..model.edges_nb() {
        assert!(
            model
                .get_edge(index)
                .unwrap()
                .status()
                .is_set(Status::Failure)
        );
    }
}
#[test]
fn existing_free_polygon_is_reused_without_curve_population() {
    let model = BRepMeshModelBuilder::new().perform(&fixtures::model_shape(2), &settings());
    existing::seed_free_polygons(&model, 1);
    assert!(BRepMeshEdgeDiscret::new().perform(&model, &settings(), &ProgressRange::default()));
    let edge = model.get_edge(0).unwrap();
    assert!(edge.status().is_set(Status::Reused));
    assert_eq!(edge.get_curve().parameters_nb(), 0);
}
#[test]
fn interior_tessellation_keeps_existing_endpoints() {
    let model = BRepMeshModelBuilder::new().perform(&fixtures::model_shape(4), &settings());
    let edge = model.get_edge(0).unwrap();
    edge.set_deflection(0.1);
    edge.set_angular_deflection(0.5);
    let tessellator = BRepMeshEdgeDiscret::create_edge_tessellator(&edge, &settings(), 5).unwrap();
    edge.get_curve().add_point(Default::default(), -7.0);
    edge.get_curve().add_point(Default::default(), 9.0);
    BRepMeshEdgeDiscret::tessellate_3d(&edge, &tessellator, false).unwrap();
    assert_eq!(edge.get_curve().parameters_nb(), 5);
    assert_eq!(edge.get_curve().get_parameter(0).unwrap(), -7.0);
    assert_eq!(edge.get_curve().get_parameter(4).unwrap(), 9.0);
    BRepMeshEdgeDiscret::tessellate_2d(&edge, true).unwrap();
    assert_eq!(edge.get_pcurve(0).unwrap().parameters_nb(), 5);
}

#[path = "support/mutating_tessellator.rs"]
mod callbacks;
#[test]
fn endpoint_points_are_read_after_tessellator_callbacks() {
    let model = BRepMeshModelBuilder::new().perform(&fixtures::model_shape(4), &settings());
    let edge = model.get_edge(0).unwrap();
    let (first, last) = rustycad_tkbrep::topexp::vertices(&edge.get_edge().unwrap(), false);
    let callback = Handle::new(callbacks::MutatingEndpoints {
        transient: Default::default(),
        first,
        last,
    })
    .cast();
    BRepMeshEdgeDiscret::tessellate_3d(&edge, &callback, true).unwrap();
    assert_eq!(edge.get_curve().get_point(0).unwrap().x(), 7.0);
    assert_eq!(edge.get_curve().get_point(1).unwrap().x(), 14.0);
}
