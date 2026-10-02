//! Direct Rust mirrors of the pinned discret-algorithm factory regressions.
#![allow(non_snake_case)]
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDiscretAlgoFactory, BRepMeshDiscretFactory, BRepMeshIncrementalMeshFactory, factories,
    find_factory, register_factory, unregister_factory,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use std::sync::Arc;

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

#[test]
fn registry_custom_factory_registration_and_removal_match_name_rules() {
    let custom = Arc::new(TestFactory("FactoryProbe"));
    register_factory(custom.clone(), false);
    register_factory(custom, false);
    assert_eq!(find_factory("factoryprobe").unwrap().name(), "FactoryProbe");
    assert_eq!(factories().last().unwrap().name(), "FactoryProbe");
    unregister_factory("FACTORYPROBE");
    assert!(find_factory("FactoryProbe").is_none());
}

fn box_shape() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

struct TestFactory(&'static str);

impl BRepMeshDiscretAlgoFactory for TestFactory {
    fn name(&self) -> &str {
        self.0
    }

    fn create_algorithm(
        &self,
        _shape: &Shape,
        _linear_deflection: f64,
        _angular_deflection: f64,
    ) -> Box<dyn rustycad_tkmesh::brep_mesh::BRepMeshDiscretRoot> {
        Box::new(TestAlgorithm)
    }
}

struct TestAlgorithm;

impl rustycad_tkmesh::brep_mesh::BRepMeshDiscretRoot for TestAlgorithm {
    fn perform(&mut self) {}

    fn is_done(&self) -> bool {
        false
    }
}

#[test]
fn incremental_factory_constructor_restores_preferred_name_without_duplicates() {
    let prior_count = factories()
        .iter()
        .filter(|factory| factory.name() == "FastDiscret")
        .count();
    let _factory = BRepMeshIncrementalMeshFactory::new();
    assert_eq!(
        factories()
            .iter()
            .filter(|factory| factory.name() == "FastDiscret")
            .count(),
        prior_count
    );
    assert_eq!(
        rustycad_tkmesh::brep_mesh::default_factory()
            .unwrap()
            .name(),
        "FastDiscret"
    );
}
