//! `Storage_Root`: a named root object of a set of stored data.
use crate::standard::{Handle, StandardPersistent, StandardTransient};
use crate::tcollection::AsciiString;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Default)]
struct RootFields {
    name: AsciiString,
    type_name: AsciiString,
    object: Handle<StandardPersistent>,
    reference: i32,
}

/// A root shared by handles: `Storage_RootData::UpdateRoot` edits the same object that an
/// earlier `Roots()` sequence holds, so the fields sit behind a lock.
///
/// ```
/// use rustycad_tkernel::storage::Root;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let root = Root::with_reference(&AsciiString::new("doc"), 3, &AsciiString::new("PDoc"));
/// assert_eq!(root.reference(), 3);
/// assert_eq!(root.type_name().as_bytes(), b"PDoc");
/// ```
#[derive(Default)]
pub struct Root {
    parent: StandardTransient,
    fields: RwLock<RootFields>,
}

impl Root {
    /// `Storage_Root(theName, theObject)`: reference 0 and an empty type; `Root::new(&name, &object)`.
    pub fn new(name: &AsciiString, object: &Handle<StandardPersistent>) -> Self {
        Self::from_fields(RootFields {
            name: name.clone(),
            object: object.clone(),
            ..RootFields::default()
        })
    }

    /// `Storage_Root(theName, theRef, theType)`: a retrieved root without its object yet.
    /// Example: `Root::with_reference(&name, 1, &type_name)`.
    pub fn with_reference(name: &AsciiString, reference: i32, type_name: &AsciiString) -> Self {
        Self::from_fields(RootFields {
            name: name.clone(),
            type_name: type_name.clone(),
            reference,
            ..RootFields::default()
        })
    }

    fn from_fields(fields: RootFields) -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: RwLock::new(fields),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, RootFields> {
        self.fields.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self) -> RwLockWriteGuard<'_, RootFields> {
        self.fields.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// `SetName`; `root.set_name(&AsciiString::new("doc"))`.
    pub fn set_name(&self, name: &AsciiString) {
        self.write().name = name.clone();
    }

    /// `Name`; `root.name()`.
    pub fn name(&self) -> AsciiString {
        self.read().name.clone()
    }

    /// `SetObject`; `root.set_object(&object)`.
    pub fn set_object(&self, object: &Handle<StandardPersistent>) {
        self.write().object = object.clone();
    }

    /// `Object`: the root's persistent object, null for a retrieved root; `root.object()`.
    pub fn object(&self) -> Handle<StandardPersistent> {
        self.read().object.clone()
    }

    /// `Type`; `root.type_name()`.
    pub fn type_name(&self) -> AsciiString {
        self.read().type_name.clone()
    }

    /// `SetReference`; `root.set_reference(2)`.
    pub fn set_reference(&self, reference: i32) {
        self.write().reference = reference;
    }

    /// `Reference`; `root.reference()`.
    pub fn reference(&self) -> i32 {
        self.read().reference
    }

    /// `SetType`; `root.set_type(&AsciiString::new("PDocStd_Document"))`.
    pub fn set_type(&self, type_name: &AsciiString) {
        self.write().type_name = type_name.clone();
    }
}

crate::impl_transient!(Root, "Storage_Root", StandardTransient, parent);
