// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_GenericEmpty, TDataStd_Tick,
// TDataStd_Directory and TDataStd_NoteBook (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_GenericEmpty` and the content-free attributes derived from it.
use super::support::find_or_add;
use super::{TDataStdInteger, TDataStdReal};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::tdf::{TdfAttribute, TdfAttributeState, TdfLabel, TdfTagSource};
use rustycad_tkcdf::json_dump::class_begin;
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Guid, Handle, RuntimeTyped, dump::DumpStream, exceptions},
};
use std::io;

/// `TDataStd_GenericEmpty`: an attribute without contents (empty `Restore` and `Paste`).
pub trait TDataStdGenericEmpty: TdfAttribute {}
impl_transient_interface!(
    dyn TDataStdGenericEmpty,
    "TDataStd_GenericEmpty",
    dyn TdfAttribute
);

/// The native `TDataStd_GenericEmpty::DumpJson`, called by the derived attributes.
pub fn dump_generic_empty<A: TDataStdGenericEmpty + ?Sized>(
    attribute: &A,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "TDataStd_GenericEmpty")?;
    dump_attribute_json_base(attribute, stream, depth)
}

/// Declares one content-free attribute derived from `TDataStd_GenericEmpty`, with its fixed ID
/// and dump text; other toolkits (TKCAF's `TDataXtd_Point`...) declare theirs with it. The
/// optional `on_create` hook runs in the constructor (TKCAF registers its derived attributes
/// there, since Rust has no library-load initializers).
///
/// ```
/// rustycad_tklcaf::tdata_std_empty_attribute!(
///     /// A marker attribute.
///     Marker, "Marker", rustycad_tklcaf::guid_from_text("40DC60CD-30B9-41be-B002-4169EFB34EA6"), "Marker"
/// );
/// assert_eq!(Marker::get_id(), rustycad_tklcaf::guid_from_text("40DC60CD-30B9-41be-B002-4169EFB34EA6"));
/// ```
#[macro_export]
macro_rules! tdata_std_empty_attribute {
    ($(#[$doc:meta])* $name:ident, $occt:literal, $id:expr, $text:literal) => {
        $crate::tdata_std_empty_attribute!($(#[$doc])* $name, $occt, $id, $text, on_create: || {});
    };
    ($(#[$doc:meta])* $name:ident, $occt:literal, $id:expr, $text:literal, on_create: $hook:expr) => {
        $(#[$doc])*
        pub struct $name {
            transient: ::rustycad_tkernel::standard::StandardTransient,
            base: $crate::tdf::TdfAttributeBase,
        }
        ::rustycad_tkernel::impl_transient!($name, $occt, dyn $crate::tdata_std::TDataStdGenericEmpty, transient,
            interfaces: [dyn $crate::tdf::TdfAttribute, dyn $crate::tdata_std::TDataStdGenericEmpty]);
        impl $crate::tdata_std::TDataStdGenericEmpty for $name {}
        impl ::std::default::Default for $name {
            fn default() -> Self {
                ($hook)();
                Self {
                    transient: ::rustycad_tkernel::standard::StandardTransient::default(),
                    base: $crate::tdf::TdfAttributeBase::default(),
                }
            }
        }
        impl $name {
            /// `GetID`: the fixed ID.
            pub fn get_id() -> ::rustycad_tkernel::standard::Guid {
                $id
            }
            /// The default constructor.
            pub fn new() -> Self {
                Self::default()
            }
        }
        impl $crate::tdf::TdfAttribute for $name {
            fn attribute_base(&self) -> &$crate::tdf::TdfAttributeBase {
                &self.base
            }
            fn id(&self) -> ::rustycad_tkernel::standard::Guid {
                $id
            }
            /// `Restore` of `TDataStd_GenericEmpty`: nothing to restore.
            fn restore(
                &self,
                _with: &::rustycad_tkernel::standard::Handle<dyn $crate::tdf::TdfAttribute>,
            ) -> ::std::result::Result<(), ::rustycad_tkernel::standard::Failure> {
                Ok(())
            }
            /// `NewEmpty` (from `IMPLEMENT_DERIVED_ATTRIBUTE`).
            fn new_empty(&self) -> ::rustycad_tkernel::standard::Handle<dyn $crate::tdf::TdfAttribute> {
                ::rustycad_tkernel::standard::Handle::new(Self::new()).cast()
            }
            /// `Paste` of `TDataStd_GenericEmpty`: nothing to paste.
            fn paste(
                &self,
                _into: &::rustycad_tkernel::standard::Handle<dyn $crate::tdf::TdfAttribute>,
                _table: &::rustycad_tkernel::standard::Handle<$crate::tdf::TdfRelocationTable>,
            ) -> ::std::result::Result<(), ::rustycad_tkernel::standard::Failure> {
                Ok(())
            }
            fn dump(&self, stream: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                write!(stream, $text)
            }
            fn dump_json(
                &self,
                stream: &mut ::rustycad_tkernel::standard::dump::DumpStream,
                depth: i32,
            ) -> ::std::io::Result<()> {
                $crate::tdata_std::dump_generic_empty(self, stream, depth)
            }
        }
    };
}

crate::tdata_std_empty_attribute!(
    /// `TDataStd_Tick`: a boolean flag by presence; `TDataStdTick::set(&label)`.
    TDataStdTick, "TDataStd_Tick", guid_from_text("40DC60CD-30B9-41be-B002-4169EFB34EA5"), "Tick"
);
crate::tdata_std_empty_attribute!(
    /// `TDataStd_Directory`: a label that hands out object labels.
    TDataStdDirectory, "TDataStd_Directory", guid_from_text("2a96b61f-ec8b-11d0-bee7-080009dc3333"), "Directory"
);
crate::tdata_std_empty_attribute!(
    /// `TDataStd_NoteBook`: a label that holds real and integer variables.
    TDataStdNoteBook, "TDataStd_NoteBook", guid_from_text("2a96b609-ec8b-11d0-bee7-080009dc3333"), "NoteBook"
);

impl TDataStdTick {
    /// `Set(label)`: finds or adds the tick.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataStdTick>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
}

/// The attribute of `id` on `current` or on its closest ancestor (the native `Find` loops).
fn find_upward<T: TdfAttribute + RuntimeTyped>(
    current: &TdfLabel,
    id: &Guid,
    found: &mut Handle<T>,
) -> Result<bool, Failure> {
    let mut label = current.clone();
    while !label.is_null() {
        let mut attribute: Handle<T> = Handle::default();
        if label.find_attribute_as(id, &mut attribute)? {
            *found = attribute;
            return Ok(true);
        }
        label = label.father();
    }
    Ok(false)
}

/// The native `New(label)`: `Standard_DomainError` for a label with attributes, else the
/// attribute plus a tag source.
fn new_on_empty_label<T: TdfAttribute + RuntimeTyped>(
    label: &TdfLabel,
    make: fn() -> T,
    who: &str,
) -> Result<Handle<T>, Failure> {
    if label.has_attribute()? {
        return Err(exceptions::domain_error(&format!(
            "{who} : not an empty label"
        )));
    }
    let created = Handle::new(make());
    label.add_attribute(&created.cast(), true)?;
    TdfTagSource::set(label)?;
    Ok(created)
}

impl TDataStdDirectory {
    /// `Find(current, D)`: the directory of the label or of its closest ancestor.
    pub fn find(
        current: &TdfLabel,
        directory: &mut Handle<TDataStdDirectory>,
    ) -> Result<bool, Failure> {
        find_upward(current, &Self::get_id(), directory)
    }
    /// `New(label)`: a directory (and a tag source) on an empty label.
    pub fn new_on(label: &TdfLabel) -> Result<Handle<TDataStdDirectory>, Failure> {
        new_on_empty_label(label, Self::new, "TDataStd_Directory::New")
    }
    /// `AddDirectory(dir)`: a new sub-directory on a new child label.
    pub fn add_directory(
        directory: &Handle<TDataStdDirectory>,
    ) -> Result<Handle<TDataStdDirectory>, Failure> {
        let label = crate::live(directory, "TDataStd_Directory::AddDirectory")?.label();
        Self::new_on(&TdfTagSource::new_child_of(&label)?)
    }
    /// `MakeObjectLabel(dir)`: a new child label of the directory.
    pub fn make_object_label(directory: &Handle<TDataStdDirectory>) -> Result<TdfLabel, Failure> {
        let label = crate::live(directory, "TDataStd_Directory::MakeObjectLabel")?.label();
        TdfTagSource::new_child_of(&label)
    }
}

impl TDataStdNoteBook {
    /// `Find(current, N)`: the notebook of the label or of its closest ancestor.
    pub fn find(
        current: &TdfLabel,
        notebook: &mut Handle<TDataStdNoteBook>,
    ) -> Result<bool, Failure> {
        find_upward(current, &Self::get_id(), notebook)
    }
    /// `New(label)`: a notebook (and a tag source) on an empty label.
    pub fn new_on(label: &TdfLabel) -> Result<Handle<TDataStdNoteBook>, Failure> {
        new_on_empty_label(label, Self::new, "TDataStd_NoteBook::New")
    }
    /// `Append(double, isExported)`: a real variable on a new child label.
    pub fn append_real(
        &self,
        value: f64,
        _is_exported: bool,
    ) -> Result<Handle<TDataStdReal>, Failure> {
        TDataStdReal::set(&TdfTagSource::new_child_of(&self.label())?, value)
    }
    /// `Append(int, isExported)`: an integer variable on a new child label.
    pub fn append_integer(
        &self,
        value: i32,
        _is_exported: bool,
    ) -> Result<Handle<TDataStdInteger>, Failure> {
        TDataStdInteger::set(&TdfTagSource::new_child_of(&self.label())?, value)
    }
}
