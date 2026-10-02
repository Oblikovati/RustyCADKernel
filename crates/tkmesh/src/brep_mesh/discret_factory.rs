//! Registry and compatibility factory for incremental BRep meshing.
use super::BRepMeshIncrementalMesh;
use crate::imesh_tools::MeshParameters;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Common meshing operation returned by a discret algorithm factory.
pub trait BRepMeshDiscretRoot: Send {
    /// Run the mesher once; `algorithm.perform()`.
    fn perform(&mut self);
    /// Whether the most recent `perform` call succeeded; `algorithm.is_done()`.
    fn is_done(&self) -> bool;
}

/// Factory interface for named BRep meshing algorithms.
pub trait BRepMeshDiscretAlgoFactory: Send + Sync {
    /// The case-insensitive registry name; `factory.name() == "FastDiscret"`.
    fn name(&self) -> &str;
    /// Create an algorithm for the given shape and deflection values.
    fn create_algorithm(
        &self,
        shape: &Shape,
        linear_deflection: f64,
        angular_deflection: f64,
    ) -> Box<dyn BRepMeshDiscretRoot>;
}

/// Global registry adapter for named meshing algorithms.
pub struct BRepMeshDiscretFactory;

impl BRepMeshDiscretFactory {
    /// Return the process-wide factory; `BRepMeshDiscretFactory::get().default_name()`.
    pub fn get() -> &'static Self {
        static FACTORY: BRepMeshDiscretFactory = BRepMeshDiscretFactory;
        ensure_builtin_factory();
        &FACTORY
    }

    /// Select a registered name, retaining the previous choice on failure.
    /// Example: `assert!(BRepMeshDiscretFactory::get().set_default_name("FastDiscret"));`
    pub fn set_default_name(&self, name: &str) -> bool {
        if find_factory(name).is_none() {
            return false;
        }
        *default_name()
            .lock()
            .expect("default factory lock poisoned") = name.to_owned();
        true
    }

    /// Return the currently selected factory name; `get().default_name() == "FastDiscret"`.
    pub fn default_name(&self) -> String {
        default_name()
            .lock()
            .expect("default factory lock poisoned")
            .clone()
    }

    /// Create an algorithm from the selected factory, or the first registry entry.
    /// Example: `let algo = BRepMeshDiscretFactory::get().discret(&shape, 0.1, 0.5);`
    pub fn discret(
        &self,
        shape: &Shape,
        linear_deflection: f64,
        angular_deflection: f64,
    ) -> Option<Box<dyn BRepMeshDiscretRoot>> {
        let selected_name = self.default_name();
        let factory = if selected_name.is_empty() {
            default_factory()
        } else {
            find_factory(&selected_name)
        }?;
        Some(factory.create_algorithm(shape, linear_deflection, angular_deflection))
    }
}

/// Built-in meshing factory registered as `FastDiscret`.
pub struct BRepMeshIncrementalMeshFactory;

impl BRepMeshIncrementalMeshFactory {
    /// Register a new built-in factory as the preferred registry entry.
    /// Example: `let _factory = BRepMeshIncrementalMeshFactory::new();`
    pub fn new() -> Arc<Self> {
        let factory = Arc::new(Self);
        register_factory(factory.clone(), true);
        factory
    }
}

impl BRepMeshDiscretAlgoFactory for BRepMeshIncrementalMeshFactory {
    fn name(&self) -> &str {
        "FastDiscret"
    }

    fn create_algorithm(
        &self,
        shape: &Shape,
        linear_deflection: f64,
        angular_deflection: f64,
    ) -> Box<dyn BRepMeshDiscretRoot> {
        Box::new(IncrementalMeshAlgorithm {
            shape: shape.clone(),
            parameters: MeshParameters {
                deflection: linear_deflection,
                angle: angular_deflection,
                ..Default::default()
            },
            is_done: false,
        })
    }
}

struct IncrementalMeshAlgorithm {
    shape: Shape,
    parameters: MeshParameters,
    is_done: bool,
}

impl BRepMeshDiscretRoot for IncrementalMeshAlgorithm {
    fn perform(&mut self) {
        self.is_done = BRepMeshIncrementalMesh::with_parameters(
            &self.shape,
            &self.parameters,
            &ProgressRange::default(),
        )
        .is_ok();
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

/// Register a factory once, or move it to the front when preferred.
pub fn register_factory(factory: Arc<dyn BRepMeshDiscretAlgoFactory>, preferred: bool) {
    let mut entries = registry().write().expect("factory registry lock poisoned");
    if preferred {
        entries.retain(|entry| !same_name(entry.name(), factory.name()));
        entries.insert(0, factory);
        return;
    }
    if entries
        .iter()
        .any(|entry| same_name(entry.name(), factory.name()))
    {
        return;
    }
    entries.push(factory);
}

/// Remove every registry entry matching `name` without regard to ASCII case.
pub fn unregister_factory(name: &str) {
    registry()
        .write()
        .expect("factory registry lock poisoned")
        .retain(|entry| !same_name(entry.name(), name));
}

/// Return the registered factories in default-selection order.
pub fn factories() -> Vec<Arc<dyn BRepMeshDiscretAlgoFactory>> {
    ensure_builtin_factory();
    registry()
        .read()
        .expect("factory registry lock poisoned")
        .clone()
}

/// Find a registered factory using OCCT's case-insensitive name comparison.
pub fn find_factory(name: &str) -> Option<Arc<dyn BRepMeshDiscretAlgoFactory>> {
    ensure_builtin_factory();
    registry()
        .read()
        .expect("factory registry lock poisoned")
        .iter()
        .find(|factory| same_name(factory.name(), name))
        .cloned()
}

/// Return the first registered factory, if any.
pub fn default_factory() -> Option<Arc<dyn BRepMeshDiscretAlgoFactory>> {
    ensure_builtin_factory();
    registry()
        .read()
        .expect("factory registry lock poisoned")
        .first()
        .cloned()
}

fn registry() -> &'static RwLock<Vec<Arc<dyn BRepMeshDiscretAlgoFactory>>> {
    static REGISTRY: OnceLock<RwLock<Vec<Arc<dyn BRepMeshDiscretAlgoFactory>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(vec![Arc::new(BRepMeshIncrementalMeshFactory)]))
}

fn default_name() -> &'static Mutex<String> {
    static DEFAULT_NAME: OnceLock<Mutex<String>> = OnceLock::new();
    DEFAULT_NAME.get_or_init(|| Mutex::new("FastDiscret".to_owned()))
}

fn ensure_builtin_factory() {
    let _ = registry();
}

fn same_name(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}
