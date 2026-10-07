//! Port-only checks of the discret-algorithm factory registry name rules and of the
//! incremental-mesh factory constructor, beyond the upstream GTest mirrors.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDiscretAlgoFactory, BRepMeshIncrementalMeshFactory, factories, find_factory,
    register_factory, unregister_factory,
};
use std::sync::Arc;

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
