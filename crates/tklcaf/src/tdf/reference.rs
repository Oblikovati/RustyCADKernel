// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Reference.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Reference`: an attribute that points to another label.
use super::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfDataSet, TdfLabel, TdfRelocationTable,
};
use crate::{
    guid_from_text,
    json::{dump_attribute_json_base, dump_label_entry},
    live,
};
use rustycad_tkcdf::json_dump::class_begin;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
};
use std::io;

/// `TDF_Reference::GetID()`.
const REFERENCE_ID: Guid = guid_from_text("2a96b610-ec8b-11d0-bee7-080009dc3333");

/// `TDF_Reference`; `TdfReference::set(&label, &origin)`.
#[derive(Default)]
pub struct TdfReference {
    transient: StandardTransient,
    base: TdfAttributeBase,
    origin: SharedState<TdfLabel>,
}
impl_transient!(TdfReference, "TDF_Reference", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TdfReference {
    /// `GetID`; `TdfReference::get_id()`.
    pub fn get_id() -> Guid {
        REFERENCE_ID
    }
    /// `TDF_Reference()`: a null origin.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(I, Origin)` (static): the reference of `label`, added when absent, set to `origin`.
    ///
    /// ```
    /// use rustycad_tklcaf::tdf::{TdfData, TdfReference};
    /// let data = TdfData::new();
    /// let target = data.root().find_child(2, true).unwrap();
    /// let reference = TdfReference::set(&data.root(), &target).unwrap();
    /// assert!(TdfReference::get(&reference) == target);
    /// ```
    pub fn set(label: &TdfLabel, origin: &TdfLabel) -> Result<Handle<TdfReference>, Failure> {
        let mut found: Handle<TdfReference> = Handle::default();
        if !label.find_attribute_as(&REFERENCE_ID, &mut found)? {
            found = Handle::new(TdfReference::new());
            label.add_attribute(&found.cast(), true)?;
        }
        live(&found, "TDF_Reference::Set")?.set_origin(origin)?;
        Ok(found)
    }
    /// `Set(Origin)`: backed up when it changes (OCC2932).
    pub fn set_origin(&self, origin: &TdfLabel) -> Result<(), Failure> {
        if self.get() == *origin {
            return Ok(());
        }
        self.backup()?;
        *self.origin.write() = origin.clone();
        Ok(())
    }
    /// `Get`: the referenced label.
    pub fn get(&self) -> TdfLabel {
        self.origin.read().clone()
    }
}

impl TdfAttribute for TdfReference {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        REFERENCE_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let origin = live(&with.cast::<TdfReference>(), "TDF_Reference::Restore")?.get();
        *self.origin.write() = origin;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(TdfReference::new()).cast()
    }
    /// `Paste`: the relocated origin, else the origin itself.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let origin = self.get();
        let mut target = TdfLabel::default();
        if !origin.is_null() {
            let table = live(relocation_table, "TDF_Reference::Paste")?;
            if !table.has_label_relocation(&origin, &mut target) {
                target = origin;
            }
        }
        live(&into.cast::<TdfReference>(), "TDF_Reference::Paste")?.set_origin(&target)
    }
    /// `References`: the origin, unless this label is imported.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        if self.label().is_imported() {
            return;
        }
        if let Some(data_set) = data_set.get() {
            data_set.add_label(&self.get());
        }
    }
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Reference")
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDF_Reference")?;
        dump_attribute_json_base(self, stream, depth)?;
        dump_label_entry(stream, "aLabel", &self.get())
    }
}
