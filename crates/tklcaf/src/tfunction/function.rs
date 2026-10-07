// Rust port of ApplicationFramework/TKLCAF/TFunction/TFunction_Function.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TFunction_Function`: the attribute that marks a label as a function, with the GUID of
//! its driver and its failure code.
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdata_std::support::cast_attribute;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable, dump_attribute,
};
use rustycad_tkcdf::json_dump::{class_begin, field_guid, field_integer};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
};
use std::io;

/// `TFunction_Function::GetID()`.
const FUNCTION_ID: Guid = guid_from_text("5b35ca00-5b78-11d1-8940-080009dc3333");

/// The native fields `myDriverGUID` and `myFailure`.
#[derive(Default)]
struct FunctionState {
    driver_guid: Guid,
    failure: i32,
}

/// `TFunction_Function`; `TFunctionFunction::set_with_driver(&label, &driver_id)`.
///
/// ```
/// use rustycad_tklcaf::{tfunction::TFunctionFunction, tdf::TdfData, guid_from_text};
/// let data = TdfData::new();
/// let driver = guid_from_text("0f2a9a10-0000-4000-8000-000000000002");
/// let function = TFunctionFunction::set_with_driver(&data.root(), &driver).unwrap();
/// assert_eq!(function.get_driver_guid(), driver);
/// assert!(!function.failed());
/// ```
#[derive(Default)]
pub struct TFunctionFunction {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<FunctionState>,
}
impl_transient!(TFunctionFunction, "TFunction_Function", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TFunctionFunction {
    /// `Set(L)`: the function of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        let mut found: Handle<Self> = Handle::default();
        if label.find_attribute_as(&FUNCTION_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        label.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `Set(L, DriverID)`: as `Set(L)`, then the driver GUID.
    pub fn set_with_driver(label: &TdfLabel, driver_id: &Guid) -> Result<Handle<Self>, Failure> {
        let function = Self::set(label)?;
        live(&function, "TFunction_Function::Set")?.set_driver_guid(driver_id)?;
        Ok(function)
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        FUNCTION_ID
    }
    /// `TFunction_Function()`: the null driver GUID, no failure.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetDriverGUID`.
    pub fn get_driver_guid(&self) -> Guid {
        self.state.read().driver_guid
    }
    /// `SetDriverGUID(guid)`: backed up when it changes (OCC2932).
    pub fn set_driver_guid(&self, guid: &Guid) -> Result<(), Failure> {
        if self.state.read().driver_guid == *guid {
            return Ok(());
        }
        self.backup()?;
        self.state.write().driver_guid = *guid;
        Ok(())
    }
    /// `Failed`: the failure code is not 0.
    pub fn failed(&self) -> bool {
        self.get_failure() != 0
    }
    /// `SetFailure(mode)`: backed up when it changes (OCC2932).
    pub fn set_failure(&self, mode: i32) -> Result<(), Failure> {
        if self.state.read().failure == mode {
            return Ok(());
        }
        self.backup()?;
        self.state.write().failure = mode;
        Ok(())
    }
    /// `GetFailure`.
    pub fn get_failure(&self) -> i32 {
        self.state.read().failure
    }
}

impl TdfAttribute for TFunctionFunction {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        FUNCTION_ID
    }
    /// `Restore(other)`: the failure code and the driver GUID.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TFunction_Function::Restore")?;
        let other = live(&other, "TFunction_Function::Restore")?;
        let (failure, driver_guid) = (other.get_failure(), other.get_driver_guid());
        let mut state = self.state.write();
        (state.failure, state.driver_guid) = (failure, driver_guid);
        Ok(())
    }
    /// `Paste(into, RT)`: `into` takes the failure code and the driver GUID.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<Self>(into, "TFunction_Function::Paste")?;
        let target = live(&target, "TFunction_Function::Paste")?;
        target.set_failure(self.get_failure())?;
        target.set_driver_guid(&self.get_driver_guid())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Dump`: the attribute dump.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)
    }
    /// `DumpJson`: the driver GUID and the failure code.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TFunction_Function")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_guid(stream, "myDriverGUID", self.get_driver_guid())?;
        field_integer(stream, "myFailure", self.get_failure())
    }
}
