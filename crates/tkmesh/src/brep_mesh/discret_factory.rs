// Rust translation of BRepMesh_DiscretAlgoFactory, BRepMesh_IncrementalMeshFactory and
// BRepMesh_DiscretFactory (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The registry of named meshing algorithm factories.
use super::BRepMeshIncrementalMesh;
use super::discret_root::BRepMeshDiscretRoot;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use std::sync::{Mutex, OnceLock, PoisonError, RwLock};

/// `BRepMesh_DiscretAlgoFactory`: a named maker of meshing algorithms.
pub trait BRepMeshDiscretAlgoFactory: Transient + Send + Sync {
    /// `Name()`: the registry name; `factory.name()`.
    fn name(&self) -> &str;
    /// `CreateAlgorithm(theShape, theLinDeflection, theAngDeflection)` (pure virtual).
    fn create_algorithm(
        &self,
        shape: &Shape,
        lin_deflection: f64,
        ang_deflection: f64,
    ) -> Handle<dyn BRepMeshDiscretRoot>;
}
impl_transient_interface!(
    dyn BRepMeshDiscretAlgoFactory,
    "BRepMesh_DiscretAlgoFactory",
    StandardTransient
);

type FactoryHandle = Handle<dyn BRepMeshDiscretAlgoFactory>;

/// The process-wide registry, holding the self-registered incremental mesh factory
/// from the start (the native static initializer).
fn registry() -> &'static RwLock<Vec<FactoryHandle>> {
    static REGISTRY: OnceLock<RwLock<Vec<FactoryHandle>>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        RwLock::new(vec![
            Handle::new(BRepMeshIncrementalMeshFactory::unregistered()).cast(),
        ])
    })
}

/// The registered factories, in default-selection order; `factories().len()`.
pub fn factories() -> Vec<FactoryHandle> {
    registry()
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// `RegisterFactory(theFactory, theIsPreferred)`: a preferred factory replaces the
/// same-named ones at the front; otherwise it is appended unless the name (compared
/// without case) is taken; `register_factory(&factory, false)`.
pub fn register_factory(factory: &FactoryHandle, preferred: bool) {
    let Some(name) = factory.get().map(|f| f.name().to_owned()) else {
        return;
    };
    if preferred {
        unregister_factory(&name);
        registry()
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(0, factory.clone());
        return;
    }
    let mut entries = registry().write().unwrap_or_else(PoisonError::into_inner);
    if !entries.iter().any(|entry| same_name(entry, &name)) {
        entries.push(factory.clone());
    }
}

/// `UnregisterFactory(theName)`: removes every factory of that name (any case).
pub fn unregister_factory(name: &str) {
    registry()
        .write()
        .unwrap_or_else(PoisonError::into_inner)
        .retain(|entry| !same_name(entry, name));
}

/// `DefaultFactory()`: the first registered factory, or a null handle.
pub fn default_factory() -> FactoryHandle {
    factories().into_iter().next().unwrap_or_default()
}

/// `FindFactory(theName)`: the first factory of that name (any case), or null.
pub fn find_factory(name: &str) -> FactoryHandle {
    factories()
        .into_iter()
        .find(|entry| same_name(entry, name))
        .unwrap_or_default()
}

/// `TCollection_AsciiString::IsSameString(.., .., false)`.
fn same_name(factory: &FactoryHandle, name: &str) -> bool {
    factory
        .get()
        .is_some_and(|f| f.name().eq_ignore_ascii_case(name))
}

/// `BRepMesh_IncrementalMeshFactory`: the "FastDiscret" factory of
/// BRepMesh_IncrementalMesh.
///
/// ```
/// use rustycad_tkmesh::brep_mesh::{BRepMeshDiscretAlgoFactory, find_factory};
/// assert_eq!(find_factory("fastdiscret").get().map(|f| f.name()), Some("FastDiscret"));
/// ```
#[derive(Default)]
pub struct BRepMeshIncrementalMeshFactory {
    transient: StandardTransient,
}
impl_transient!(BRepMeshIncrementalMeshFactory, "BRepMesh_IncrementalMeshFactory", dyn BRepMeshDiscretAlgoFactory, transient, interfaces: [dyn BRepMeshDiscretAlgoFactory]);

impl BRepMeshIncrementalMeshFactory {
    /// `BRepMesh_IncrementalMeshFactory()`: a factory registered as preferred;
    /// `BRepMeshIncrementalMeshFactory::new()`.
    pub fn new() -> Handle<Self> {
        let factory = Handle::new(Self::unregistered());
        register_factory(&factory.clone().cast(), true);
        factory
    }

    fn unregistered() -> Self {
        Self::default()
    }
}

impl BRepMeshDiscretAlgoFactory for BRepMeshIncrementalMeshFactory {
    fn name(&self) -> &str {
        "FastDiscret"
    }
    /// `CreateAlgorithm`: a BRepMesh_IncrementalMesh with these deflections and the
    /// default parallel flag, not yet performed.
    fn create_algorithm(
        &self,
        shape: &Shape,
        lin_deflection: f64,
        ang_deflection: f64,
    ) -> Handle<dyn BRepMeshDiscretRoot> {
        BRepMeshIncrementalMesh::discret(shape, lin_deflection, ang_deflection).1
    }
}

/// `BRepMesh_DiscretFactory`: the global selector of the default algorithm name.
pub struct BRepMeshDiscretFactory {
    default_name: Mutex<String>,
}

impl BRepMeshDiscretFactory {
    /// `Get()`: the global instance, default name "FastDiscret";
    /// `BRepMeshDiscretFactory::get().default_name()`.
    pub fn get() -> &'static Self {
        static FACTORY: OnceLock<BRepMeshDiscretFactory> = OnceLock::new();
        FACTORY.get_or_init(|| BRepMeshDiscretFactory {
            default_name: Mutex::new("FastDiscret".to_owned()),
        })
    }

    /// `SetDefaultName(theName)`: false (and no change) when no factory has the name;
    /// `BRepMeshDiscretFactory::get().set_default_name("FastDiscret")`.
    pub fn set_default_name(&self, name: &str) -> bool {
        if find_factory(name).is_null() {
            return false;
        }
        *self
            .default_name
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = name.to_owned();
        true
    }

    /// `DefaultName()`; `BRepMeshDiscretFactory::get().default_name()`.
    pub fn default_name(&self) -> String {
        self.default_name
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// `Discret(theShape, theLinDeflection, theAngDeflection)`: an algorithm of the
    /// named (or first) factory, or a null handle without factory;
    /// `BRepMeshDiscretFactory::get().discret(&shape, 0.1, 0.5)`.
    pub fn discret(&self, shape: &Shape, lin: f64, ang: f64) -> Handle<dyn BRepMeshDiscretRoot> {
        let name = self.default_name();
        let factory = if name.is_empty() {
            default_factory()
        } else {
            find_factory(&name)
        };
        match factory.get() {
            Some(factory) => factory.create_algorithm(shape, lin, ang),
            None => Handle::default(),
        }
    }
}
