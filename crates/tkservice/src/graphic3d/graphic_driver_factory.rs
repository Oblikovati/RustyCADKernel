//! `Graphic3d_GraphicDriverFactory`: the registry of graphic driver factories. The factory
//! is abstract ([`Graphic3dGraphicDriverFactory`] over [`Graphic3dGraphicDriverFactoryBase`]);
//! the static registry is a process-wide list, as natively.
use super::Graphic3dGraphicDriver;
use crate::aspect::AspectDisplayConnection;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// The embedded state of a factory: its name.
pub struct Graphic3dGraphicDriverFactoryBase {
    /// `myName`.
    pub name: AsciiString,
}

impl Graphic3dGraphicDriverFactoryBase {
    /// The protected constructor; `Graphic3dGraphicDriverFactoryBase::new(&"VK".into())`.
    pub fn new(name: &AsciiString) -> Self {
        Self { name: name.clone() }
    }
}

/// A graphic driver factory, implemented by the driver crate.
pub trait Graphic3dGraphicDriverFactory: Transient {
    /// The embedded base; `factory.graphic_driver_factory_base()`.
    fn graphic_driver_factory_base(&self) -> &Graphic3dGraphicDriverFactoryBase;
    /// `CreateDriver`: a new driver on the display connection.
    fn create_driver(
        &self,
        display: &Handle<AspectDisplayConnection>,
    ) -> Result<Handle<dyn Graphic3dGraphicDriver>, Failure>;
    /// `Name`; `factory.name()`.
    fn name(&self) -> AsciiString {
        self.graphic_driver_factory_base().name.clone()
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dGraphicDriverFactory,
    "Graphic3d_GraphicDriverFactory",
    StandardTransient
);

/// The registered factories (`getDriverFactories()`).
fn factories() -> MutexGuard<'static, Vec<Handle<dyn Graphic3dGraphicDriverFactory>>> {
    static FACTORIES: OnceLock<Mutex<Vec<Handle<dyn Graphic3dGraphicDriverFactory>>>> =
        OnceLock::new();
    FACTORIES
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The name of a factory handle (empty for null).
fn name_of(factory: &Handle<dyn Graphic3dGraphicDriverFactory>) -> AsciiString {
    factory.get().map(|f| f.name()).unwrap_or_default()
}

/// `IsSameString(a, b, false)`: names compared without case.
fn same_name(first: &AsciiString, second: &AsciiString) -> bool {
    first.as_bytes().eq_ignore_ascii_case(second.as_bytes())
}

/// `RegisterFactory`: a preferred factory replaces the factories of its name and goes
/// first; another one is appended unless its name is registered.
/// Example: `graphic3d_graphic_driver_factory_register(&factory, true)`.
pub fn graphic3d_graphic_driver_factory_register(
    factory: &Handle<dyn Graphic3dGraphicDriverFactory>,
    is_preferred: bool,
) {
    let name = name_of(factory);
    if is_preferred {
        graphic3d_graphic_driver_factory_unregister(&name);
        factories().insert(0, factory.clone());
        return;
    }
    let mut list = factories();
    if !list.iter().any(|known| same_name(&name_of(known), &name)) {
        list.push(factory.clone());
    }
}

/// `UnregisterFactory`: removes every factory of the name (case ignored);
/// `graphic3d_graphic_driver_factory_unregister(&"VK".into())`.
pub fn graphic3d_graphic_driver_factory_unregister(name: &AsciiString) {
    factories().retain(|known| !same_name(&name_of(known), name));
}

/// `DefaultDriverFactory`: the first factory, or null; `graphic3d_graphic_driver_factory_default()`.
pub fn graphic3d_graphic_driver_factory_default() -> Handle<dyn Graphic3dGraphicDriverFactory> {
    factories().first().cloned().unwrap_or_default()
}

/// `DriverFactories`: the registered factories in order; `graphic3d_graphic_driver_factories()`.
pub fn graphic3d_graphic_driver_factories() -> Vec<Handle<dyn Graphic3dGraphicDriverFactory>> {
    factories().clone()
}
