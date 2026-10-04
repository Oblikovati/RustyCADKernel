//! `Storage_CallBack`, `Storage_DefaultCallBack` and `Storage_TypedCallBack`.
use super::{BaseDriver, Schema};
use crate::standard::{Failure, Handle, StandardPersistent, StandardTransient, Transient};
use crate::tcollection::AsciiString;
use std::{
    fmt,
    sync::{PoisonError, RwLock},
};

/// `Storage_CallBack`: creates, registers, writes and reads the persistent objects of one
/// type. Native and Rust callbacks are shared through owning handles.
/// Custom callbacks embed `StandardTransient` and register their concrete runtime descriptor
/// with `impl_transient!`, so a retrieved binding keeps its native type ancestry.
/// `Send + Sync` permits sharing across Rust threads; custom implementations must keep any
/// mutable callback state synchronized, as `Schema::type_binding` returns a shared callback.
pub trait CallBack: Transient {
    /// `New`: a fresh object of the callback's type; `callback.new_persistent()`.
    fn new_persistent(&self) -> Handle<StandardPersistent>;

    /// `Add`: registers the objects `persistent` references with `schema`.
    /// Example: `callback.add(&object, &schema)?`.
    fn add(&self, persistent: &Handle<StandardPersistent>, schema: &Schema) -> Result<(), Failure>;

    /// `Write`: writes the object's fields; `callback.write(&object, &mut driver, &schema)?`.
    fn write(
        &self,
        persistent: &Handle<StandardPersistent>,
        driver: &mut dyn BaseDriver,
        schema: &Schema,
    ) -> Result<(), Failure>;

    /// `Read`: reads the object's fields; `callback.read(&object, &mut driver, &schema)?`.
    fn read(
        &self,
        persistent: &Handle<StandardPersistent>,
        driver: &mut dyn BaseDriver,
        schema: &Schema,
    ) -> Result<(), Failure>;
}

/// `Storage_DefaultCallBack`: plain `Standard_Persistent` objects whose data is skipped.
///
/// ```
/// use rustycad_tkernel::storage::{CallBack, DefaultCallBack};
/// assert!(DefaultCallBack::default().new_persistent().is_some());
/// ```
#[derive(Clone, Default)]
pub struct DefaultCallBack {
    parent: StandardTransient,
}

crate::impl_transient!(
    DefaultCallBack,
    "Storage_DefaultCallBack",
    dyn CallBack,
    parent,
    interfaces: [dyn CallBack]
);

crate::impl_transient_interface!(dyn CallBack, "Storage_CallBack", StandardTransient);

impl fmt::Debug for DefaultCallBack {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DefaultCallBack")
    }
}

impl CallBack for DefaultCallBack {
    fn new_persistent(&self) -> Handle<StandardPersistent> {
        Handle::new(StandardPersistent::default())
    }

    fn add(&self, _: &Handle<StandardPersistent>, _: &Schema) -> Result<(), Failure> {
        Ok(())
    }

    fn write(
        &self,
        _: &Handle<StandardPersistent>,
        _: &mut dyn BaseDriver,
        _: &Schema,
    ) -> Result<(), Failure> {
        Ok(())
    }

    /// Skips the object's data with the driver's `SkipObject`.
    fn read(
        &self,
        _: &Handle<StandardPersistent>,
        driver: &mut dyn BaseDriver,
        _: &Schema,
    ) -> Result<(), Failure> {
        driver.skip_object()
    }
}

struct TypedFields {
    type_name: AsciiString,
    callback: Handle<dyn CallBack>,
    index: i32,
}

/// `Storage_TypedCallBack`: a callback bound to a type name and a type index.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkernel::storage::{CallBack, DefaultCallBack, TypedCallBack};
/// use rustycad_tkernel::tcollection::AsciiString;
/// let callback = Handle::new(DefaultCallBack::default()).cast::<dyn CallBack>();
/// let typed = TypedCallBack::new(&AsciiString::new("PDoc"), &callback);
/// typed.set_index(3);
/// assert_eq!(typed.index(), 3);
/// ```
pub struct TypedCallBack {
    parent: StandardTransient,
    fields: RwLock<TypedFields>,
}

impl Default for TypedCallBack {
    fn default() -> Self {
        Self::new(&AsciiString::default(), &Handle::default())
    }
}

impl TypedCallBack {
    /// `Storage_TypedCallBack(aTypeName, aCallBack)` with index 0; `TypedCallBack::new(&name, &callback)`.
    pub fn new(type_name: &AsciiString, callback: &Handle<dyn CallBack>) -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: RwLock::new(TypedFields {
                type_name: type_name.clone(),
                callback: callback.clone(),
                index: 0,
            }),
        }
    }

    fn edit<R>(&self, change: impl FnOnce(&mut TypedFields) -> R) -> R {
        change(&mut self.fields.write().unwrap_or_else(PoisonError::into_inner))
    }

    fn view<R>(&self, read: impl FnOnce(&TypedFields) -> R) -> R {
        read(&self.fields.read().unwrap_or_else(PoisonError::into_inner))
    }

    /// `SetType`; `typed.set_type(&name)`.
    pub fn set_type(&self, type_name: &AsciiString) {
        self.edit(|fields| fields.type_name = type_name.clone());
    }

    /// `Type`; `typed.type_name()`.
    pub fn type_name(&self) -> AsciiString {
        self.view(|fields| fields.type_name.clone())
    }

    /// `SetCallBack`; `typed.set_callback(&callback)`.
    pub fn set_callback(&self, callback: &Handle<dyn CallBack>) {
        self.edit(|fields| fields.callback = callback.clone());
    }

    /// `CallBack`: the bound callback, null when unbound; `typed.callback()`.
    pub fn callback(&self) -> Handle<dyn CallBack> {
        self.view(|fields| fields.callback.clone())
    }

    /// `SetIndex`; `typed.set_index(1)`.
    pub fn set_index(&self, index: i32) {
        self.edit(|fields| fields.index = index);
    }

    /// `Index`; `typed.index()`.
    pub fn index(&self) -> i32 {
        self.view(|fields| fields.index)
    }
}

crate::impl_transient!(
    TypedCallBack,
    "Storage_TypedCallBack",
    StandardTransient,
    parent
);

/// An `NCollection_HArray1<occ::handle<Storage_CallBack>>(1, len)`: the per-type callback
/// table `Storage_Schema::Write` and `FSD_BinaryFile::ReadCompleteInfo` index by type number.
pub(crate) struct CallBackTable(Vec<Handle<dyn CallBack>>);

impl CallBackTable {
    /// `len` null callbacks; a negative length fails like the native allocation.
    pub(crate) fn new(length: i32) -> Result<Self, Failure> {
        let length = usize::try_from(length).map_err(|_| {
            Failure::with_type(
                "Standard_OutOfMemory",
                &format!("NCollection_Array1 bounds 1..={length}; expected a non-negative size"),
                "",
            )
        })?;
        Ok(Self(vec![Handle::default(); length]))
    }

    fn slot(&self, member: &str, index: i32) -> Result<usize, Failure> {
        let length = self.0.len();
        usize::try_from(index)
            .ok()
            .filter(|index| (1..=length).contains(index))
            .map(|index| index - 1)
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_OutOfRange",
                    &format!("NCollection_Array1::{member} index {index}; expected 1..={length}"),
                    "",
                )
            })
    }

    /// `SetValue(index, callback)`, `Standard_OutOfRange` outside `1..=len`.
    pub(crate) fn set_value(
        &mut self,
        index: i32,
        callback: &Handle<dyn CallBack>,
    ) -> Result<(), Failure> {
        let slot = self.slot("SetValue", index)?;
        self.0[slot] = callback.clone();
        Ok(())
    }

    /// `Value(index)` dereferenced: out of range is `Standard_OutOfRange`, a null callback
    /// (a native null-pointer call) is `Standard_NullObject`.
    pub(crate) fn value(&self, index: i32, caller: &str) -> Result<&dyn CallBack, Failure> {
        let slot = self.slot("Value", index)?;
        self.0[slot].get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                &format!("{caller}: type {index} has a null callback; expected a bound type"),
                "",
            )
        })
    }
}
