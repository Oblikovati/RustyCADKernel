// Rust port of ApplicationFramework/TKLCAF/TFunction/TFunction_Scope.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TFunction_Scope`: the functions of a framework by ID, kept on the root label.
use super::TFunctionLogbook;
use crate::guid_from_text;
use crate::live;
use crate::tdata_std::support::cast_attribute;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable, dump_attribute,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::DoubleMap,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
};
use std::io;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// `TFunction_Scope::GetID()`.
const SCOPE_ID: Guid = guid_from_text("F2DE4EFF-7FE8-40a3-AAD5-5B6DDEA83469");

/// `NCollection_DoubleMap<int, TDF_Label>`: the functions by ID.
pub type TFunctionFunctionMap = DoubleMap<i32, TdfLabel>;

/// `TFunction_Scope`; `TFunctionScope::set(&label)` (on the root label). `GetFunctions`
/// and `ChangeFunctions` lend the read and write guards of the map.
///
/// ```
/// use rustycad_tklcaf::{tfunction::TFunctionScope, tdf::TdfData};
/// let data = TdfData::new();
/// let function = data.root().find_child(1, true).unwrap();
/// let scope = TFunctionScope::set(&function).unwrap();
/// assert!(scope.add_function(&function).unwrap());
/// assert_eq!((scope.get_function_id(&function).unwrap(), scope.get_free_id()), (1, 2));
/// ```
pub struct TFunctionScope {
    transient: StandardTransient,
    base: TdfAttributeBase,
    functions: SharedState<TFunctionFunctionMap>,
    free_id: SharedState<i32>,
}
impl_transient!(TFunctionScope, "TFunction_Scope", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TFunctionScope {
    /// `TFunction_Scope()`: no function, the next ID is 1.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            functions: SharedState::new(TFunctionFunctionMap::default()),
            free_id: SharedState::new(1),
        }
    }
}

impl TFunctionScope {
    /// `Set(Access)`: the scope of the root label, added when absent.
    pub fn set(access: &TdfLabel) -> Result<Handle<Self>, Failure> {
        let root = access.root()?;
        let mut found: Handle<Self> = Handle::default();
        if root.find_attribute_as(&SCOPE_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        root.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        SCOPE_ID
    }
    /// `TFunction_Scope()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `AddFunction(L)`: false when the label is a function already; else backs up and
    /// binds it to the next free ID.
    pub fn add_function(&self, label: &TdfLabel) -> Result<bool, Failure> {
        if self.has_function_label(label) {
            return Ok(false);
        }
        self.backup()?;
        let id = {
            let mut free_id = self.free_id.write();
            *free_id += 1;
            *free_id - 1
        };
        self.functions.write().bind(id, label.clone())?;
        Ok(true)
    }
    /// `RemoveFunction(L)`: false when the label is not a function; else backs up and unbinds.
    pub fn remove_function_label(&self, label: &TdfLabel) -> Result<bool, Failure> {
        if !self.has_function_label(label) {
            return Ok(false);
        }
        self.backup()?;
        Ok(self.functions.write().unbind2(label))
    }
    /// `RemoveFunction(ID)`: false when the ID is unbound; else backs up and unbinds.
    pub fn remove_function(&self, id: i32) -> Result<bool, Failure> {
        if !self.has_function(id) {
            return Ok(false);
        }
        self.backup()?;
        Ok(self.functions.write().unbind1(&id))
    }
    /// `RemoveAllFunctions`: unless empty, backs up and clears.
    pub fn remove_all_functions(&self) -> Result<(), Failure> {
        if self.functions.read().is_empty() {
            return Ok(());
        }
        self.backup()?;
        self.functions.write().clear(false);
        Ok(())
    }
    /// `HasFunction(ID)`.
    pub fn has_function(&self, id: i32) -> bool {
        self.functions.read().is_bound1(&id)
    }
    /// `HasFunction(L)`.
    pub fn has_function_label(&self, label: &TdfLabel) -> bool {
        self.functions.read().is_bound2(label)
    }
    /// `GetFunction(L)`: the ID; `Standard_NoSuchObject` for an unknown label.
    pub fn get_function_id(&self, label: &TdfLabel) -> Result<i32, Failure> {
        self.functions.read().find2(label).copied()
    }
    /// `GetFunction(ID)`: the label; `Standard_NoSuchObject` for an unknown ID.
    pub fn get_function(&self, id: i32) -> Result<TdfLabel, Failure> {
        self.functions.read().find1(&id).cloned()
    }
    /// `GetLogbook`: the logbook of the scope's label (null when absent).
    pub fn get_logbook(&self) -> Result<Handle<TFunctionLogbook>, Failure> {
        let mut logbook = Handle::default();
        self.find_attribute_as(&TFunctionLogbook::get_id(), &mut logbook)?;
        Ok(logbook)
    }
    /// `GetFunctions`.
    pub fn get_functions(&self) -> RwLockReadGuard<'_, TFunctionFunctionMap> {
        self.functions.read()
    }
    /// `ChangeFunctions`: the map itself, without backup, as natively.
    pub fn change_functions(&self) -> RwLockWriteGuard<'_, TFunctionFunctionMap> {
        self.functions.write()
    }
    /// `SetFreeID(ID)`: backed up when it changes.
    pub fn set_free_id(&self, id: i32) -> Result<(), Failure> {
        if self.get_free_id() == id {
            return Ok(());
        }
        self.backup()?;
        *self.free_id.write() = id;
        Ok(())
    }
    /// `GetFreeID`.
    pub fn get_free_id(&self) -> i32 {
        *self.free_id.read()
    }
}

impl TdfAttribute for TFunctionScope {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        SCOPE_ID
    }
    /// `Restore(other)`: the functions (assigned, as the native `operator=`) and the next ID.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TFunction_Scope::Restore")?;
        let other = live(&other, "TFunction_Scope::Restore")?;
        self.functions.write().assign(&other.functions.read());
        *self.free_id.write() = other.get_free_id();
        Ok(())
    }
    /// `Paste`: nothing, as natively.
    fn paste(
        &self,
        _into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Dump`: the attribute dump.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)
    }
}
