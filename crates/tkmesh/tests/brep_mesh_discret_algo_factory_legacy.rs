//! Direct Rust mirrors of the pinned discret-algorithm factory regressions.
#![allow(non_snake_case)]
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDiscretFactory, default_factory, factories, find_factory,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__Factories_AtLeastOneRegistered() {
    assert!(!factories().is_empty(), "No factories registered");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__DefaultFactory_ReturnsValid() {
    assert!(!default_factory().is_null(), "DefaultFactory returned null");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__FindFactory_FastDiscret() {
    let factory = find_factory("FastDiscret");
    assert!(!factory.is_null(), "FastDiscret factory not found");
    assert_eq!(factory.name(), "FastDiscret");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__FindFactory_NonExistent_ReturnsNull() {
    assert!(
        find_factory("NonExistentFactory").is_null(),
        "FindFactory should return null for non-existent factory"
    );
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__CreateAlgorithm_ReturnsValid() {
    let factory = default_factory();
    assert!(!factory.is_null());
    let algorithm = factory.create_algorithm(&box_shape(), 0.1, 0.5);
    assert!(!algorithm.is_null(), "CreateAlgorithm returned null");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__CreateAlgorithm_CanMesh() {
    let factory = default_factory();
    assert!(!factory.is_null());
    let algorithm = factory.create_algorithm(&box_shape(), 0.1, 0.5);
    assert!(!algorithm.is_null());
    algorithm.perform(&ProgressRange::default()).unwrap();
    assert!(algorithm.is_done(), "Meshing failed");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__DiscretFactory_UsesRegistry() {
    let algorithm = BRepMeshDiscretFactory::get().discret(&box_shape(), 0.1, 0.5);
    assert!(!algorithm.is_null(), "Discret returned null");
    algorithm.perform(&ProgressRange::default()).unwrap();
    assert!(algorithm.is_done(), "Meshing failed");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__DiscretFactory_SetDefaultName() {
    let factory = BRepMeshDiscretFactory::get();
    assert!(factory.set_default_name("FastDiscret"));
    assert_eq!(factory.default_name(), "FastDiscret");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__RegisterFactory_Uniqueness() {
    let count = factories()
        .iter()
        .filter(|factory| factory.get().is_some_and(|f| f.name() == "FastDiscret"))
        .count();
    assert_eq!(
        count, 1,
        "FastDiscret factory should be registered exactly once"
    );
}

fn box_shape() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
