// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_AsciiString.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_AsciiString`: an ASCII string attribute with a user ID.
use super::support::{cast_attribute, find_or_add, reset_user_id, set_backed_up, set_user_id};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::tdf::{TdfAttribute, TdfAttributeBase, TdfLabel, TdfRelocationTable, dump_attribute};
use rustycad_tkcdf::json_dump::{class_begin, field_string};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::AsciiString,
};
use std::io;

/// `TDataStd_AsciiString::GetID()`.
const ASCII_STRING_ID: Guid = guid_from_text("3bbefc60-e618-11d4-ba38-0060b0ee18ea");

/// `TDataStd_AsciiString`; `TDataStdAsciiString::set(&label, &text)`.
pub struct TDataStdAsciiString {
    transient: StandardTransient,
    base: TdfAttributeBase,
    string: SharedState<AsciiString>,
    id: SharedState<Guid>,
}
impl_transient!(TDataStdAsciiString, "TDataStd_AsciiString", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdAsciiString {
    /// `TDataStd_AsciiString()`: empty, default ID.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            string: SharedState::default(),
            id: SharedState::new(ASCII_STRING_ID),
        }
    }
}

impl TDataStdAsciiString {
    /// `GetID`: the default ID.
    pub fn get_id() -> Guid {
        ASCII_STRING_ID
    }
    /// `TDataStd_AsciiString()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label, string)`: finds or adds the attribute of the default ID and sets it.
    pub fn set(
        label: &TdfLabel,
        string: &AsciiString,
    ) -> Result<Handle<TDataStdAsciiString>, Failure> {
        Self::set_with_id(label, &ASCII_STRING_ID, string)
    }
    /// `Set(label, guid, string)`: the same with a user ID.
    pub fn set_with_id(
        label: &TdfLabel,
        guid: &Guid,
        string: &AsciiString,
    ) -> Result<Handle<TDataStdAsciiString>, Failure> {
        let attribute = find_or_add(label, guid, true, Self::new)?;
        crate::live(&attribute, "TDataStd_AsciiString::Set")?.set_value(string)?;
        Ok(attribute)
    }
    /// `Set(S)`: backed up when it changes.
    pub fn set_value(&self, string: &AsciiString) -> Result<(), Failure> {
        set_backed_up(self, &self.string, string)
    }
    /// `Get`; `TDataStdAsciiString::get(&attribute)`.
    pub fn get(&self) -> AsciiString {
        self.string.read().clone()
    }
    /// `IsEmpty`: the string is empty.
    pub fn is_empty(&self) -> bool {
        self.string.read().is_empty()
    }
}

impl TdfAttribute for TDataStdAsciiString {
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
        reset_user_id(self, &self.id, ASCII_STRING_ID)
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TDataStdAsciiString>(with, "TDataStd_AsciiString::Restore")?;
        let other = crate::live(&other, "TDataStd_AsciiString::Restore")?;
        let (string, id) = (other.get(), other.id());
        *self.string.write() = string;
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
        let target = cast_attribute::<TDataStdAsciiString>(into, "TDataStd_AsciiString::Paste")?;
        let target = crate::live(&target, "TDataStd_AsciiString::Paste")?;
        target.set_value(&self.get())?;
        target.set_id(&self.id())
    }
    /// `Dump`: the attribute fields, the string, ` Name=|<string>|`, the ID and `|`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)?;
        let text = String::from_utf8_lossy(self.string.read().as_bytes()).into_owned();
        write!(stream, "{text} Name=|{text}|")?;
        writeln!(stream, "{}|", self.id())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_AsciiString")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_string(
            stream,
            "myString",
            &String::from_utf8_lossy(self.string.read().as_bytes()),
        )
    }
}
