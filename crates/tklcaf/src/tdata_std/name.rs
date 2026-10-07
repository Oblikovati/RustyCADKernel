// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_Name.hxx, .cxx and
// TDataStd_Comment.hxx, .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_Name` and `TDataStd_Comment`: the extended-string attributes.
use super::support::{find_or_add, reset_user_id};
use super::{
    TDataStdGenericExtString, TDataStdGenericExtStringBase,
    TDataStdGenericExtStringMembers as Members,
};
use crate::guid_from_text;
use crate::tdf::{TdfAttribute, TdfAttributeBase, TdfLabel, TdfRelocationTable, dump_attribute};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, StandardTransient, dump::DumpStream},
    tcollection::ExtendedString,
};
use std::io;

/// `TDataStd_Name::GetID()`.
const NAME_ID: Guid = guid_from_text("2a96b608-ec8b-11d0-bee7-080009dc3333");
/// `TDataStd_Comment::GetID()`.
const COMMENT_ID: Guid = guid_from_text("2a96b616-ec8b-11d0-bee7-080009dc3333");

/// Declares one generic-string attribute struct and its shared `TdfAttribute` members.
macro_rules! ext_string_attribute {
    ($(#[$doc:meta])* $name:ident, $occt:literal, $default_id:expr) => {
        $(#[$doc])*
        pub struct $name {
            transient: StandardTransient,
            base: TdfAttributeBase,
            strings: TDataStdGenericExtStringBase,
        }
        impl_transient!($name, $occt, dyn TDataStdGenericExtString, transient,
            interfaces: [dyn TdfAttribute, dyn TDataStdGenericExtString]);
        impl Default for $name {
            fn default() -> Self {
                Self {
                    transient: StandardTransient::default(),
                    base: TdfAttributeBase::default(),
                    strings: TDataStdGenericExtStringBase::with_id($default_id),
                }
            }
        }
        impl TDataStdGenericExtString for $name {
            fn generic_ext_string_base(&self) -> &TDataStdGenericExtStringBase {
                &self.strings
            }
        }
        impl $name {
            /// `GetID`: the default ID.
            pub fn get_id() -> Guid {
                $default_id
            }
            /// The default constructor: empty string, default ID.
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

/// The `TdfAttribute` members both string attributes share.
macro_rules! ext_string_members {
    ($default_id:expr) => {
        fn attribute_base(&self) -> &TdfAttributeBase {
            &self.base
        }
        fn id(&self) -> Guid {
            Members::id(self)
        }
        fn set_id(&self, guid: &Guid) -> Result<(), Failure> {
            Members::set_id(self, guid)
        }
        fn set_default_id(&self) -> Result<(), Failure> {
            reset_user_id(self, &self.strings.id, $default_id)
        }
        fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
            Members::restore(self, with)
        }
        /// `NewEmpty` (from `IMPLEMENT_DERIVED_ATTRIBUTE`): a new attribute of this type.
        fn new_empty(&self) -> Handle<dyn TdfAttribute> {
            Handle::new(Self::new()).cast()
        }
        fn paste(
            &self,
            into: &Handle<dyn TdfAttribute>,
            table: &Handle<TdfRelocationTable>,
        ) -> Result<(), Failure> {
            Members::paste(self, into, table)
        }
        fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
            Members::dump_json(self, stream, depth)
        }
    };
}

ext_string_attribute!(
    /// `TDataStd_Name`; `TDataStdName::set(&label, &name)`.
    ///
    /// ```
    /// use rustycad_tklcaf::{tdata_std::{TDataStdGenericExtString, TDataStdName}, tdf::TdfData};
    /// use rustycad_tkernel::tcollection::ExtendedString;
    /// let data = TdfData::new();
    /// let name = TDataStdName::set(&data.root(), &ExtendedString::new("part")).unwrap();
    /// assert_eq!(TDataStdGenericExtString::get(&*name), ExtendedString::new("part"));
    /// ```
    TDataStdName, "TDataStd_Name", NAME_ID
);
impl TDataStdName {
    /// `Set(label, string)`: finds or adds the name of the default ID and sets it.
    pub fn set(label: &TdfLabel, string: &ExtendedString) -> Result<Handle<TDataStdName>, Failure> {
        Self::set_with_id(label, &NAME_ID, string)
    }
    /// `Set(label, guid, string)`: the same with a user ID.
    pub fn set_with_id(
        label: &TdfLabel,
        guid: &Guid,
        string: &ExtendedString,
    ) -> Result<Handle<TDataStdName>, Failure> {
        let attribute = find_or_add(label, guid, true, Self::new)?;
        crate::live(&attribute, "TDataStd_Name::Set")?.set_value(string)?;
        Ok(attribute)
    }
}
impl TdfAttribute for TDataStdName {
    ext_string_members!(NAME_ID);
    /// `Dump`: the attribute fields, ` Name=|<string>|` and the ID.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)?;
        write!(stream, " Name=|{}|", self.get().to_utf8())?;
        writeln!(stream, "{}", self.id())
    }
}

ext_string_attribute!(
    /// `TDataStd_Comment`; `TDataStdComment::set(&label, &comment)`.
    TDataStdComment, "TDataStd_Comment", COMMENT_ID
);
impl TDataStdComment {
    /// `Set(label, string)`: finds or adds the comment (default ID, no `SetID`) and sets it.
    pub fn set(
        label: &TdfLabel,
        string: &ExtendedString,
    ) -> Result<Handle<TDataStdComment>, Failure> {
        let attribute = Self::set_empty(label)?;
        crate::live(&attribute, "TDataStd_Comment::Set")?.set_value(string)?;
        Ok(attribute)
    }
    /// `Set(label)`: finds or adds the comment.
    pub fn set_empty(label: &TdfLabel) -> Result<Handle<TDataStdComment>, Failure> {
        find_or_add(label, &COMMENT_ID, false, Self::new)
    }
}
impl TdfAttribute for TDataStdComment {
    ext_string_members!(COMMENT_ID);
    /// `Dump`: the attribute fields and `Comment=|<string>|`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)?;
        write!(stream, "Comment=|{}|", self.get().to_utf8())
    }
}
