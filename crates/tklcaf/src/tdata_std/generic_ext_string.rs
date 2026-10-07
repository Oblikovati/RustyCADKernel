// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_GenericExtString.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_GenericExtString`: the abstract extended-string attribute with a user ID
//! (`TDataStd_Name`, `TDataStd_Comment` and the TKCAF/TKXCAF strings derive from it).
use super::support::{cast_attribute, set_backed_up, set_user_id};
use crate::json::dump_attribute_json_base;
use crate::tdf::{TdfAttribute, TdfRelocationTable};
use rustycad_tkcdf::json_dump::{class_begin, field_guid, field_string};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Guid, Handle, SharedState, dump::DumpStream},
    tcollection::ExtendedString,
};
use std::io;

/// The protected fields of `TDataStd_GenericExtString`: the string and the ID.
#[derive(Default)]
pub struct TDataStdGenericExtStringBase {
    /// Native `myString`.
    pub string: SharedState<ExtendedString>,
    /// Native `myID`.
    pub id: SharedState<Guid>,
}
impl TDataStdGenericExtStringBase {
    /// The base of a derived attribute with its default ID (`myID = GetID()` in the
    /// derived constructors).
    pub fn with_id(id: Guid) -> Self {
        Self {
            string: SharedState::default(),
            id: SharedState::new(id),
        }
    }
}

/// `TDataStd_GenericExtString`'s members over the embedded base.
pub trait TDataStdGenericExtString: TdfAttribute {
    /// The embedded base with the native protected fields.
    fn generic_ext_string_base(&self) -> &TDataStdGenericExtStringBase;
    /// `Set(S)` (virtual): backed up when the string changes.
    fn set_value(&self, string: &ExtendedString) -> Result<(), Failure> {
        set_backed_up(self, &self.generic_ext_string_base().string, string)
    }
    /// `Get` (virtual): the string.
    fn get(&self) -> ExtendedString {
        self.generic_ext_string_base().string.read().clone()
    }
}
impl_transient_interface!(
    dyn TDataStdGenericExtString,
    "TDataStd_GenericExtString",
    dyn TdfAttribute
);

/// The native `TDataStd_GenericExtString` overrides of `TDF_Attribute`, called by the
/// `TdfAttribute` impls of the derived attributes.
pub struct TDataStdGenericExtStringMembers;
impl TDataStdGenericExtStringMembers {
    /// `ID`: the stored ID.
    pub fn id<A: TDataStdGenericExtString + ?Sized>(attribute: &A) -> Guid {
        *attribute.generic_ext_string_base().id.read()
    }
    /// `SetID(guid)`: backed up when it changes.
    pub fn set_id<A: TDataStdGenericExtString + ?Sized>(
        attribute: &A,
        guid: &Guid,
    ) -> Result<(), Failure> {
        set_user_id(attribute, &attribute.generic_ext_string_base().id, guid)
    }
    /// `Restore`: the string and the ID of another generic string.
    pub fn restore<A: TDataStdGenericExtString + ?Sized>(
        attribute: &A,
        with: &Handle<dyn TdfAttribute>,
    ) -> Result<(), Failure> {
        let other = cast_attribute::<dyn TDataStdGenericExtString>(
            with,
            "TDataStd_GenericExtString::Restore",
        )?;
        let other = crate::live(&other, "TDataStd_GenericExtString::Restore")?;
        let (string, id) = (other.get(), Self::id(other));
        let base = attribute.generic_ext_string_base();
        *base.string.write() = string;
        *base.id.write() = id;
        Ok(())
    }
    /// `Paste`: sets the string and the ID of another generic string.
    pub fn paste<A: TDataStdGenericExtString + ?Sized>(
        attribute: &A,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<dyn TDataStdGenericExtString>(
            into,
            "TDataStd_GenericExtString::Paste",
        )?;
        let target = crate::live(&target, "TDataStd_GenericExtString::Paste")?;
        target.set_value(&attribute.get())?;
        target.set_id(&Self::id(attribute))
    }
    /// `DumpJson`: the base dump, the string and the ID.
    pub fn dump_json<A: TDataStdGenericExtString + ?Sized>(
        attribute: &A,
        stream: &mut DumpStream,
        depth: i32,
    ) -> io::Result<()> {
        class_begin(stream, "TDataStd_GenericExtString")?;
        dump_attribute_json_base(attribute, stream, depth)?;
        field_string(stream, "Get()", &attribute.get().to_utf8())?;
        field_guid(stream, "myID", Self::id(attribute))
    }
}
