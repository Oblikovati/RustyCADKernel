// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_DerivedAttribute.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_DerivedAttribute`: the process-wide registry of attribute types by name.
//!
//! Natively `IMPLEMENT_DERIVED_ATTRIBUTE` registers each creator from a static initializer
//! when its library loads. Rust has no static initializers: a toolkit calls
//! [`TdfDerivedAttribute::register`] for its attributes (TKLCAF does it in
//! `tdata_std::register_derived_attributes`) before the registry is read.
use super::TdfAttribute;
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

/// `TDF_DerivedAttribute::NewDerived`: makes a new attribute of one type.
pub type TdfNewDerived = fn() -> Handle<dyn TdfAttribute>;

/// One pending registration (`CreatorData`).
struct Creator {
    create: TdfNewDerived,
    name_space: Option<&'static str>,
    type_name: Option<&'static str>,
}

#[derive(Default)]
struct Registry {
    creators: Vec<Creator>,
    /// By dynamic type name: the prototype attribute and the type name.
    derived: BTreeMap<String, (Handle<dyn TdfAttribute>, AsciiString)>,
    /// The registration order of `derived`, the native map iteration stand-in.
    order: Vec<String>,
}

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
        with_registry(|registry| {
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
                .get(type_name)
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
                .get(type_name)
                .map(|(_, name)| name.clone())
                .unwrap_or_default()
        })
    }
    /// `Attributes(theList)`: appends every prototype.
    pub fn attributes(list: &mut Vec<Handle<dyn TdfAttribute>>) {
        with_registry(|registry| {
            initialize(registry);
            for name in &registry.order {
                if let Some((prototype, _)) = registry.derived.get(name) {
                    list.push(prototype.clone());
                }
            }
        });
    }
}

fn with_registry<R>(action: impl FnOnce(&mut Registry) -> R) -> R {
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
        if !registry.derived.contains_key(&dynamic) {
            registry.order.push(dynamic.clone());
        }
        registry
            .derived
            .insert(dynamic, (prototype, AsciiString::new(&name)));
    }
}
