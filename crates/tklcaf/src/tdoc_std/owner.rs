// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_Owner (.hxx, .cxx) and
// TDocStd_Modified (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDocStd_Owner` (the document of a framework, on its root) and `TDocStd_Modified` (the
//! labels marked modified, on the root).
use super::TDocStdDocument;
use crate::guid_from_text;
use crate::tdata_std::support::cast_attribute;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfData, TdfLabel, TdfLabelMap,
    TdfRelocationTable,
};
use rustycad_tkcdf::json_dump::{class_begin, field_dumped};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Guid, Handle, SharedState, StandardTransient, WeakHandle, dump::DumpStream,
        exceptions,
    },
};
use std::io;

/// `TDocStd_Owner::GetID()`.
const OWNER_ID: Guid = guid_from_text("2a96b617-ec8b-11d0-bee7-080009dc3333");
/// `TDocStd_Modified::GetID()`.
const MODIFIED_ID: Guid = guid_from_text("2a96b622-ec8b-11d0-bee7-080009dc3333");

/// `TDocStd_Owner`: keeps a raw pointer to its document natively, a weak handle here.
#[derive(Default)]
pub struct TDocStdOwner {
    transient: StandardTransient,
    base: TdfAttributeBase,
    document: SharedState<WeakHandle<TDocStdDocument>>,
}
impl_transient!(TDocStdOwner, "TDocStd_Owner", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TDocStdOwner {
    /// `GetID`.
    pub fn get_id() -> Guid {
        OWNER_ID
    }
    /// `TDocStd_Owner()`: no document.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetDocument(indata, doc)` (static): adds the owner to the root;
    /// `Standard_DomainError` when it is already there.
    pub fn set_document_of(
        data: &Handle<TdfData>,
        document: &Handle<TDocStdDocument>,
    ) -> Result<(), Failure> {
        let root = crate::live(data, "TDocStd_Owner::SetDocument")?.root();
        if root.is_attribute(&OWNER_ID)? {
            return Err(exceptions::domain_error(
                "TDocStd_Owner::SetDocument : already called",
            ));
        }
        let owner = Handle::new(Self::new());
        crate::live(&owner, "TDocStd_Owner::SetDocument")?.set_document(document);
        root.add_attribute(&owner.cast(), true)
    }
    /// `GetDocument(ofdata)` (static): `Standard_DomainError` when the root has no owner.
    pub fn get_document_of(data: &Handle<TdfData>) -> Result<Handle<TDocStdDocument>, Failure> {
        let root = crate::live(data, "TDocStd_Owner::GetDocument")?.root();
        let mut owner: Handle<TDocStdOwner> = Handle::default();
        if !root.find_attribute_as(&OWNER_ID, &mut owner)? {
            return Err(exceptions::domain_error(
                "TDocStd_Owner::GetDocument : document not found",
            ));
        }
        Ok(crate::live(&owner, "TDocStd_Owner::GetDocument")?.get_document())
    }
    /// `SetDocument(document)` (both the handle and the pointer overloads).
    pub fn set_document(&self, document: &Handle<TDocStdDocument>) {
        *self.document.write() = document.downgrade();
    }
    /// `GetDocument`: null once the document is gone.
    pub fn get_document(&self) -> Handle<TDocStdDocument> {
        self.document.read().upgrade()
    }
}

impl TdfAttribute for TDocStdOwner {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        OWNER_ID
    }
    fn restore(&self, _with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        Ok(())
    }
    /// `NewEmpty`: a null handle, as natively.
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::default()
    }
    fn paste(
        &self,
        _into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Owner")
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDocStd_Owner")?;
        let document = self.get_document();
        let Some(document) = document.get() else {
            return Ok(());
        };
        let child = |s: &mut DumpStream, d: i32| document.dump_json(s, d);
        field_dumped(stream, depth, "myDocument", Some(&child))
    }
}

/// `TDocStd_Modified`: the labels marked modified in a framework.
#[derive(Default)]
pub struct TDocStdModified {
    transient: StandardTransient,
    base: TdfAttributeBase,
    modified: SharedState<TdfLabelMap>,
}
impl_transient!(TDocStdModified, "TDocStd_Modified", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TDocStdModified {
    /// `GetID`.
    pub fn get_id() -> Guid {
        MODIFIED_ID
    }
    /// `TDocStd_Modified()`: no label.
    pub fn new() -> Self {
        Self::default()
    }
    /// The attribute on the root of the label's framework, if any.
    fn of(label: &TdfLabel) -> Result<Handle<TDocStdModified>, Failure> {
        let mut found: Handle<TDocStdModified> = Handle::default();
        label.root()?.find_attribute_as(&MODIFIED_ID, &mut found)?;
        Ok(found)
    }
    /// `IsEmpty(access)` (static): no attribute, or no label in it.
    pub fn is_empty_at(access: &TdfLabel) -> Result<bool, Failure> {
        Ok(Self::of(access)?.get().is_none_or(Self::is_empty))
    }
    /// `Add(alabel)` (static): marks the label (adding the attribute on the root).
    pub fn add(label: &TdfLabel) -> Result<bool, Failure> {
        let mut modified = Self::of(label)?;
        if modified.is_null() {
            modified = Handle::new(Self::new());
            label.root()?.add_attribute(&modified.cast(), true)?;
        }
        crate::live(&modified, "TDocStd_Modified::Add")?.add_label(label)
    }
    /// `Remove(alabel)` (static): true when there is no attribute.
    pub fn remove(label: &TdfLabel) -> Result<bool, Failure> {
        match Self::of(label)?.get() {
            Some(modified) => modified.remove_label(label),
            None => Ok(true),
        }
    }
    /// `Contains(alabel)` (static).
    pub fn contains(label: &TdfLabel) -> Result<bool, Failure> {
        Ok(Self::of(label)?
            .get()
            .is_some_and(|modified| modified.modified.read().contains(label)))
    }
    /// `Get(access)` (static): `Standard_DomainError` when there is no attribute.
    pub fn get_at(access: &TdfLabel) -> Result<TdfLabelMap, Failure> {
        match Self::of(access)?.get() {
            Some(modified) => Ok(modified.get()),
            None => Err(exceptions::domain_error("TDocStd_Modified::Get : IsEmpty")),
        }
    }
    /// `Clear(access)` (static).
    pub fn clear_at(access: &TdfLabel) -> Result<(), Failure> {
        match Self::of(access)?.get() {
            Some(modified) => modified.clear(),
            None => Ok(()),
        }
    }
    /// `AddLabel`: backs up, then adds.
    pub fn add_label(&self, label: &TdfLabel) -> Result<bool, Failure> {
        self.backup()?;
        Ok(self.modified.write().add(label.clone()))
    }
    /// `RemoveLabel`: backs up, then removes.
    pub fn remove_label(&self, label: &TdfLabel) -> Result<bool, Failure> {
        self.backup()?;
        Ok(self.modified.write().remove(label))
    }
    /// `IsEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.modified.read().is_empty()
    }
    /// `Clear()`: backs up, then empties.
    pub fn clear(&self) -> Result<(), Failure> {
        self.backup()?;
        self.modified.write().clear(true);
        Ok(())
    }
    /// `Get()`: a copy of the label map.
    pub fn get(&self) -> TdfLabelMap {
        self.modified.read().clone()
    }
}

impl TdfAttribute for TDocStdModified {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        MODIFIED_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TDocStdModified>(with, "TDocStd_Modified::Restore")?;
        let labels = crate::live(&other, "TDocStd_Modified::Restore")?.get();
        *self.modified.write() = labels;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        _into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Modified")
    }
}
