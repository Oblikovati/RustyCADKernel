//! Deflection overload and strict interval regression coverage.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, Wire},
};
use rustycad_tkernel::{
    ncollection::IncrementalAllocatorObject,
    standard::{Handle, Transient},
};
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshModelBuilder, Deflection},
    brep_mesh_data::DiscreteWire,
    imesh_data::WireHandle,
    imesh_tools::{MeshParameters, ModelBuilder},
};
#[path = "support/model_builder_fixture.rs"]
mod fixtures;
#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
#[test]
fn strict_consistency_and_transient_ancestry() {
    assert!(Deflection::is_consistent(1.0, 1.0, true, 0.1));
    assert!(!Deflection::is_consistent(0.9, 1.0, true, 0.1));
    assert!(!Deflection::is_consistent(1.1, 1.0, true, 0.1));
    assert!(Deflection::is_consistent(-1.0, 1.0, false, 0.1));
    assert!(!Deflection::is_consistent(f64::NAN, 1.0, false, 0.1));
    assert!(Deflection::default().is_kind_name("Standard_Transient"));
}
#[test]
fn null_and_empty_shape_bounds_remain_distinct() {
    assert_eq!(
        Deflection::compute_absolute(&Shape::default(), 0.1, 10.0).unwrap(),
        0.1
    );
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    assert!(Deflection::compute_absolute(&compound, 0.1, 10.0).is_err());
}
#[test]
fn empty_wire_uses_configured_deflection() {
    let allocator = Handle::new(IncrementalAllocatorObject::default());
    let wire: WireHandle = Handle::new(DiscreteWire::new(&Wire::default(), 0, &allocator)).cast();
    let parameters = MeshParameters {
        deflection: 0.25,
        ..Default::default()
    };
    Deflection::compute_wire(&wire, &parameters).unwrap();
    assert_eq!(wire.get_deflection(), 0.25);
}
#[test]
fn production_model_deflection_overloads() {
    let parameters = MeshParameters {
        deflection: 0.25,
        deflection_interior: 0.5,
        ..Default::default()
    };
    let model = BRepMeshModelBuilder::new().perform(&fixtures::model_shape(5), &parameters);
    for index in 0..model.edges_nb() {
        Deflection::compute_edge(
            &model.get_edge(index).unwrap(),
            model.get_max_size(),
            &parameters,
        )
        .unwrap();
    }
    for index in 0..model.faces_nb() {
        let face = model.get_face(index).unwrap();
        for index in 0..face.wires_nb() {
            Deflection::compute_wire(&face.get_wire(index).unwrap(), &parameters).unwrap();
        }
        Deflection::compute_face(&face, &parameters).unwrap();
        assert_eq!(face.get_deflection(), 0.5);
    }
}
