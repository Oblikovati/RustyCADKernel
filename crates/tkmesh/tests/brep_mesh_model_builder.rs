//! Concrete native model construction with the actual production box/sphere builders.
#[path = "support/model_builder_fixture.rs"]
mod fixtures;
use rustycad_tkernel::{message::Status, standard::Handle};
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshModelBuilder, BRepMeshShapeVisitor},
    brep_mesh_data::DiscreteModel,
    imesh_data::{MeshModel, Status as MeshStatus},
    imesh_tools::{MeshParameters, ModelBuilder, ShapeVisitor},
};
#[test]
fn production_primitives_build_discrete_face_wire_and_pcurve_incidence() {
    for (kind, faces, edges) in [(4, 1, 4), (5, 6, 12), (6, 1, 3), (7, 1, 4)] {
        let shape = fixtures::model_shape(kind);
        let builder = BRepMeshModelBuilder::new();
        let model = builder.perform(&shape, &MeshParameters::default());
        assert!(!model.is_null());
        assert!(builder.get_status().is_set(Status::DONE1));
        assert_eq!(model.faces_nb(), faces);
        assert_eq!(model.edges_nb(), edges);
        assert_eq!(model.get_max_size(), 0.001);
        for index in 0..faces {
            let face = model.get_face(index).unwrap();
            assert_eq!(face.wires_nb(), 1);
            assert!(face.is_valid());
        }
    }
}
#[test]
fn empty_shapes_and_invalid_boundaries_keep_native_status_meanings() {
    let builder = BRepMeshModelBuilder::new();
    assert!(
        builder
            .perform(&fixtures::model_shape(0), &Default::default())
            .is_null()
    );
    assert!(builder.get_status().is_set(Status::FAIL1));
    for (kind, expected) in [(8, MeshStatus::Failure), (9, MeshStatus::UnorientedWire)] {
        let model = builder.perform(&fixtures::model_shape(kind), &Default::default());
        assert!(!model.is_null());
        assert!(builder.get_status().is_set(Status::DONE1));
        assert!(model.get_face(0).unwrap().status().is_equal(expected));
    }
}
#[test]
fn relative_deflection_uses_native_bounds_and_visitor_deduplicates_edges() {
    let shape = fixtures::model_shape(5);
    let parameters = MeshParameters {
        relative: true,
        ..Default::default()
    };
    let model = BRepMeshModelBuilder::new().perform(&shape, &parameters);
    assert_eq!(model.get_max_size(), (3.0 + 1e-7) - (-1e-7));
    let free = fixtures::model_shape(2);
    let edge = rustycad_tkbrep::topods::edge(&free).unwrap();
    let discrete = Handle::new(DiscreteModel::new(&free));
    let visitor = BRepMeshShapeVisitor::new(&discrete.cast());
    visitor.visit_edge(&edge).unwrap();
    visitor
        .visit_edge(&rustycad_tkbrep::topods::edge(&edge.reversed()).unwrap())
        .unwrap();
    assert_eq!(discrete.edges_nb(), 1);
    assert!(discrete.get_edge(0).unwrap().is_free());
}

#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
