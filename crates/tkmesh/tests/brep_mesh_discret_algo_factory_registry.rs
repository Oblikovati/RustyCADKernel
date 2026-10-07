//! Port-only checks of the discret-algorithm factory registry name rules and of the
//! incremental-mesh factory constructor, beyond the upstream GTest mirrors.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDiscretAlgoFactory, BRepMeshDiscretRoot, BRepMeshIncrementalMeshFactory,
    DiscretRootBase, factories, find_factory, register_factory, unregister_factory,
};

fn name_of(factory: &Handle<dyn BRepMeshDiscretAlgoFactory>) -> String {
    factory
        .get()
        .map(|f| f.name().to_owned())
        .unwrap_or_default()
}

#[test]
fn registry_custom_factory_registration_and_removal_match_name_rules() {
    let custom: Handle<dyn BRepMeshDiscretAlgoFactory> =
        Handle::new(FakeDiscretFactory::default()).cast();
    register_factory(&custom, false);
    register_factory(&custom, false);
    assert_eq!(name_of(&find_factory("factoryprobe")), "FactoryProbe");
    assert_eq!(name_of(factories().last().unwrap()), "FactoryProbe");
    let algorithm = custom.create_algorithm(&Shape::default(), 0.1, 0.5);
    algorithm.perform(&ProgressRange::default()).unwrap();
    assert!(!algorithm.is_done());
    unregister_factory("FACTORYPROBE");
    assert!(find_factory("FactoryProbe").is_null());
}

/// A named fake factory registered by the test.
#[derive(Default)]
struct FakeDiscretFactory {
    transient: StandardTransient,
}
impl_transient!(FakeDiscretFactory, "FakeDiscretFactory", dyn BRepMeshDiscretAlgoFactory, transient, interfaces: [dyn BRepMeshDiscretAlgoFactory]);

impl BRepMeshDiscretAlgoFactory for FakeDiscretFactory {
    fn name(&self) -> &str {
        "FactoryProbe"
    }

    fn create_algorithm(
        &self,
        _shape: &Shape,
        _linear_deflection: f64,
        _angular_deflection: f64,
    ) -> Handle<dyn BRepMeshDiscretRoot> {
        Handle::new(FakeDiscretAlgorithm::default()).cast()
    }
}

/// A fake algorithm whose Perform never succeeds.
#[derive(Default)]
struct FakeDiscretAlgorithm {
    transient: StandardTransient,
    base: DiscretRootBase,
}
impl_transient!(FakeDiscretAlgorithm, "FakeDiscretAlgorithm", dyn BRepMeshDiscretRoot, transient, interfaces: [dyn BRepMeshDiscretRoot]);

impl BRepMeshDiscretRoot for FakeDiscretAlgorithm {
    fn discret_root_base(&self) -> &DiscretRootBase {
        &self.base
    }

    fn perform(&self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        self.set_not_done();
        Ok(())
    }
}

#[test]
fn incremental_factory_constructor_restores_preferred_name_without_duplicates() {
    let count = || {
        factories()
            .iter()
            .filter(|factory| name_of(factory) == "FastDiscret")
            .count()
    };
    let prior_count = count();
    let _factory = BRepMeshIncrementalMeshFactory::new();
    assert_eq!(count(), prior_count);
    assert_eq!(
        name_of(&rustycad_tkmesh::brep_mesh::default_factory()),
        "FastDiscret"
    );
}
