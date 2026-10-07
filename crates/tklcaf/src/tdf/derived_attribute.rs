// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_DerivedAttribute.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_DerivedAttribute`: the process-wide registry of attribute types by name.
//!
//! Natively `IMPLEMENT_DERIVED_ATTRIBUTE` registers each creator from a static initializer
//! when its library loads. Rust has no static initializers: the queries first register the
//! TKLCAF attributes once (`tdata_std::register_derived_attributes`, in the native file
//! order); another toolkit calls [`TdfDerivedAttribute::register`] for its own attributes
//! before the registry is read. The prototypes sit in an `NCollection_DataMap` with the
//! native `Standard_CStringHasher`, so `Attributes` lists them in the native map order.
use super::TdfAttribute;
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Handle, c_string::CStringHasher},
    tcollection::AsciiString,
};
use std::sync::{Mutex, Once, PoisonError};

/// `TDF_DerivedAttribute::NewDerived`: makes a new attribute of one type.
pub type TdfNewDerived = fn() -> Handle<dyn TdfAttribute>;

/// One pending registration (`CreatorData`).
struct Creator {
    create: TdfNewDerived,
    name_space: Option<&'static str>,
    type_name: Option<&'static str>,
}

/// The native globals: the pending creators, and the prototypes (`Attributes()`) and type
/// names (`Types()`) by dynamic type name, one map here since they share their keys.
struct Registry {
    creators: Vec<Creator>,
    derived: DataMap<String, (Handle<dyn TdfAttribute>, AsciiString), CStringHasher>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            creators: Vec::new(),
            derived: DataMap::with_hasher(CStringHasher, 1),
        }
    }
}

/// The one-time registration of the TKLCAF attributes (their library load natively).
static TOOLKIT_REGISTRATION: Once = Once::new();

static REGISTRY: Mutex<Option<Registry>> = Mutex::new(None);

/// `TDF_DerivedAttribute`.
pub struct TdfDerivedAttribute;
impl TdfDerivedAttribute {
    /// `Register(theNewAttributeFunction, theNameSpace, theTypeName)`: records a creator,
    /// initialized on the next query; returns the function.
    pub fn register(
        create: TdfNewDerived,
        name_space: Option<&'static str>,
        type_name: Option<&'static str>,
    ) -> TdfNewDerived {
        lock_registry(|registry| {
            registry.creators.push(Creator {
                create,
                name_space,
                type_name,
            })
        });
        create
    }
    /// `Attribute(theType)`: the prototype of a dynamic type name, null when unknown.
    pub fn attribute(type_name: &str) -> Handle<dyn TdfAttribute> {
        with_registry(|registry| {
            initialize(registry);
            registry
                .derived
                .find(&type_name.to_owned())
                .map(|(prototype, _)| prototype.clone())
                .unwrap_or_default()
        })
    }
    /// `TypeName(theType)`: `NameSpace:TypeName` (or the dynamic type name), empty when
    /// unknown.
    pub fn type_name(type_name: &str) -> AsciiString {
        with_registry(|registry| {
            initialize(registry);
            registry
                .derived
                .find(&type_name.to_owned())
                .map(|(_, name)| name.clone())
                .unwrap_or_default()
        })
    }
    /// `Attributes(theList)`: appends every prototype.
    pub fn attributes(list: &mut Vec<Handle<dyn TdfAttribute>>) {
        with_registry(|registry| {
            initialize(registry);
            list.extend(
                registry
                    .derived
                    .values()
                    .map(|(prototype, _)| prototype.clone()),
            );
        });
    }
}

/// Runs `action` on the registry, after the one-time TKLCAF registration.
fn with_registry<R>(action: impl FnOnce(&mut Registry) -> R) -> R {
    TOOLKIT_REGISTRATION.call_once(crate::tdata_std::register_derived_attributes);
    lock_registry(action)
}

/// Runs `action` on the locked registry.
fn lock_registry<R>(action: impl FnOnce(&mut Registry) -> R) -> R {
    let mut guard = REGISTRY.lock().unwrap_or_else(PoisonError::into_inner);
    action(guard.get_or_insert_with(Registry::default))
}

/// The file-static `Initialize`: makes one prototype per pending creator.
fn initialize(registry: &mut Registry) {
    for creator in std::mem::take(&mut registry.creators) {
        let prototype = (creator.create)();
        let Some(dynamic) = prototype.dynamic_type().map(|t| t.name().to_owned()) else {
            continue;
        };
        let mut name = String::new();
        if let Some(space) = creator.name_space.filter(|space| !space.is_empty()) {
            name.push_str(space);
            name.push(':');
        }
        name.push_str(creator.type_name.unwrap_or(&dynamic));
        registry
            .derived
            .bind(dynamic, (prototype, AsciiString::new(&name)));
    }
}
