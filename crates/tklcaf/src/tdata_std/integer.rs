// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_Integer.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_Integer`: an integer value attribute with a user ID.
use super::support::{
    address_of, cast_attribute, find_or_add, reset_user_id, set_backed_up, set_user_id,
};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfReference, TdfRelocationTable,
    dump_attribute,
};
use rustycad_tkcdf::json_dump::{class_begin, field_integer};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
};
use std::io;

/// `TDataStd_Integer::GetID()`.
const INTEGER_ID: Guid = guid_from_text("2a96b606-ec8b-11d0-bee7-080009dc3333");

/// `TDataStd_Integer`; `TDataStdInteger::set(&label, 5)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdInteger, tdf::TdfData};
/// let data = TdfData::new();
/// let attribute = TDataStdInteger::set(&data.root(), 5).unwrap();
/// assert_eq!(TDataStdInteger::get(&attribute), 5);
/// ```
pub struct TDataStdInteger {
    transient: StandardTransient,
    base: TdfAttributeBase,
    value: SharedState<i32>,
    id: SharedState<Guid>,
}
impl_transient!(TDataStdInteger, "TDataStd_Integer", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdInteger {
    /// `TDataStd_Integer()`: value -1, the default ID.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            value: SharedState::new(-1),
            id: SharedState::new(INTEGER_ID),
        }
    }
}

impl TDataStdInteger {
    /// `GetID`: the default ID.
    pub fn get_id() -> Guid {
        INTEGER_ID
    }
    /// `TDataStd_Integer()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label, value)`: finds or adds the attribute of the default ID and sets it.
    pub fn set(label: &TdfLabel, value: i32) -> Result<Handle<TDataStdInteger>, Failure> {
        Self::set_with_id(label, &INTEGER_ID, value)
    }
    /// `Set(label, guid, value)`: the same with a user ID.
    pub fn set_with_id(
        label: &TdfLabel,
        guid: &Guid,
        value: i32,
    ) -> Result<Handle<TDataStdInteger>, Failure> {
        let attribute = find_or_add(label, guid, true, Self::new)?;
        crate::live(&attribute, "TDataStd_Integer::Set")?.set_value(value)?;
        Ok(attribute)
    }
    /// `Set(V)`: backed up when it changes (OCC2932).
    pub fn set_value(&self, value: i32) -> Result<(), Failure> {
        set_backed_up(self, &self.value, &value)
    }
    /// `Get`; `TDataStdInteger::get(&attribute)`.
    pub fn get(&self) -> i32 {
        *self.value.read()
    }
    /// `IsCaptured`: the label holds a `TDF_Reference`.
    pub fn is_captured(&self) -> Result<bool, Failure> {
        self.label().is_attribute(&TdfReference::get_id())
    }
}

impl TdfAttribute for TDataStdInteger {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        *self.id.read()
    }
    fn set_id(&self, guid: &Guid) -> Result<(), Failure> {
        set_user_id(self, &self.id, guid)
    }
    fn set_default_id(&self) -> Result<(), Failure> {
        reset_user_id(self, &self.id, INTEGER_ID)
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TDataStdInteger>(with, "TDataStd_Integer::Restore")?;
        let (value, id) = other.get().map(|o| (o.get(), o.id())).unwrap_or_default();
        *self.value.write() = value;
        *self.id.write() = id;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TDataStdInteger>(into, "TDataStd_Integer::Paste")?;
        let target = crate::live(&target, "TDataStd_Integer::Paste")?;
        target.set_value(self.get())?;
        target.set_id(&self.id())
    }
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(
            stream,
            "Integer:: {} : {}{}",
            address_of(self),
            self.get(),
            self.id()
        )?;
        write!(stream, "\nAttribute fields: ")?;
        dump_attribute(self, stream)
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_Integer")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_integer(stream, "myValue", self.get())
    }
}
