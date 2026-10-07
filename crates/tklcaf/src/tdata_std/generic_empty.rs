// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_GenericEmpty, TDataStd_Tick,
// TDataStd_Directory and TDataStd_NoteBook (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_GenericEmpty` and the content-free attributes derived from it.
use super::support::find_or_add;
use super::{TDataStdInteger, TDataStdReal};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable, TdfTagSource,
};
use rustycad_tkcdf::json_dump::class_begin;
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{
        Failure, Guid, Handle, RuntimeTyped, StandardTransient, dump::DumpStream, exceptions,
    },
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

/// Declares one content-free attribute with its fixed ID and dump text.
macro_rules! empty_attribute {
    ($(#[$doc:meta])* $name:ident, $occt:literal, $id:expr, $text:literal) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            transient: StandardTransient,
            base: TdfAttributeBase,
        }
        impl_transient!($name, $occt, dyn TDataStdGenericEmpty, transient,
            interfaces: [dyn TdfAttribute, dyn TDataStdGenericEmpty]);
        impl TDataStdGenericEmpty for $name {}
        impl $name {
            /// `GetID`: the fixed ID.
            pub fn get_id() -> Guid {
                $id
            }
            /// The default constructor.
            pub fn new() -> Self {
                Self::default()
            }
        }
        impl TdfAttribute for $name {
            fn attribute_base(&self) -> &TdfAttributeBase {
                &self.base
            }
            fn id(&self) -> Guid {
                $id
            }
            /// `Restore` of `TDataStd_GenericEmpty`: nothing to restore.
            fn restore(&self, _with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
                Ok(())
            }
            /// `NewEmpty` (from `IMPLEMENT_DERIVED_ATTRIBUTE`).
            fn new_empty(&self) -> Handle<dyn TdfAttribute> {
                Handle::new(Self::new()).cast()
            }
            /// `Paste` of `TDataStd_GenericEmpty`: nothing to paste.
            fn paste(&self, _into: &Handle<dyn TdfAttribute>, _table: &Handle<TdfRelocationTable>) -> Result<(), Failure> {
                Ok(())
            }
            fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
                write!(stream, $text)
            }
            fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
                dump_generic_empty(self, stream, depth)
            }
        }
    };
}

empty_attribute!(
    /// `TDataStd_Tick`: a boolean flag by presence; `TDataStdTick::set(&label)`.
    TDataStdTick, "TDataStd_Tick", guid_from_text("40DC60CD-30B9-41be-B002-4169EFB34EA5"), "Tick"
);
empty_attribute!(
    /// `TDataStd_Directory`: a label that hands out object labels.
    TDataStdDirectory, "TDataStd_Directory", guid_from_text("2a96b61f-ec8b-11d0-bee7-080009dc3333"), "Directory"
);
empty_attribute!(
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
