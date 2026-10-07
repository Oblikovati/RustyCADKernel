//! Direct Rust mirrors of the pinned discret-algorithm factory regressions.
#![allow(non_snake_case)]
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmesh::brep_mesh::{BRepMeshDiscretFactory, factories, find_factory};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__Factories_AtLeastOneRegistered() {
    assert!(!factories().is_empty(), "No factories registered");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__DefaultFactory_ReturnsValid() {
    assert!(rustycad_tkmesh::brep_mesh::default_factory().is_some());
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__FindFactory_FastDiscret() {
    let factory = find_factory("FastDiscret").expect("FastDiscret factory not found");
    assert_eq!(factory.name(), "FastDiscret");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__FindFactory_NonExistent_ReturnsNull() {
    assert!(find_factory("NonExistentFactory").is_none());
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__CreateAlgorithm_ReturnsValid() {
    let shape = box_shape();
    let factory = rustycad_tkmesh::brep_mesh::default_factory().unwrap();
    let _algorithm = factory.create_algorithm(&shape, 0.1, 0.5);
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__CreateAlgorithm_CanMesh() {
    let shape = box_shape();
    let factory = rustycad_tkmesh::brep_mesh::default_factory().unwrap();
    let mut algorithm = factory.create_algorithm(&shape, 0.1, 0.5);
    assert!(!algorithm.is_done());
    algorithm.perform();
    assert!(algorithm.is_done(), "Meshing failed");
}

#[test]
fn BRepMesh_DiscretAlgoFactoryTest__DiscretFactory_UsesRegistry() {
    let shape = box_shape();
    let mut algorithm = BRepMeshDiscretFactory::get()
        .discret(&shape, 0.1, 0.5)
        .expect("Discret returned null");
    algorithm.perform();
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
    let factories = factories();
    let count = factories
        .iter()
        .filter(|factory| factory.name() == "FastDiscret")
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
