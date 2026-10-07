// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Attribute.hxx and .cxx (virtual members)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Attribute`: the abstract application datum attached to a label. Its virtual members
//! are the trait [`TdfAttribute`]; its non-virtual members are the extension trait
//! [`super::TdfAttributeState`] (ocaf-design.md section 4).
use super::{
    TdfAttributeBase, TdfAttributeDelta, TdfAttributeState, TdfDataSet, TdfDefaultDeltaOnModification,
    TdfDefaultDeltaOnRemoval, TdfDeltaOnAddition, TdfDeltaOnForget, TdfDeltaOnModification,
    TdfDeltaOnRemoval, TdfDeltaOnResume, TdfIdFilter, TdfRelocationTable,
};
use crate::json::{attribute_dump_json, AttributeIndexMap};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{dump::DumpStream, Failure, Guid, Handle, StandardTransient, Transient},
};
use std::io;

/// `TDF_Attribute`'s virtual members; a concrete attribute embeds a [`TdfAttributeBase`]
/// and registers `dyn TdfAttribute` as an interface of its `impl_transient!`.
pub trait TdfAttribute: Transient {
    /// The embedded base with the native private fields.
    fn attribute_base(&self) -> &TdfAttributeBase;
    /// `ID` (pure virtual): the attribute identifier.
    fn id(&self) -> Guid;
    /// `SetID(theGuid)`: does nothing by default (attributes with a user ID override it).
    fn set_id(&self, _guid: &Guid) {}
    /// `SetID()`: does nothing by default.
    fn set_default_id(&self) {}
    /// `Restore` (pure virtual): takes the backed-up contents of `attribute`.
    fn restore(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure>;
    /// `NewEmpty` (pure virtual): a new empty attribute of the same end type.
    fn new_empty(&self) -> Handle<dyn TdfAttribute>;
    /// `Paste` (pure virtual): puts this attribute's contents into `into`.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure>;
    /// `AfterAddition`: does nothing by default.
    fn after_addition(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `BeforeRemoval`: does nothing by default.
    fn before_removal(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `BeforeForget`: does nothing by default.
    fn before_forget(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `AfterResume`: does nothing by default.
    fn after_resume(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `AfterRetrieval`: true (done) by default.
    fn after_retrieval(&self, _force_it: bool) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `BeforeUndo`: true (done) by default.
    fn before_undo(
        &self,
        _attribute_delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `AfterUndo`: true (done) by default.
    fn after_undo(
        &self,
        _attribute_delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `BeforeCommitTransaction`: does nothing by default.
    fn before_commit_transaction(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `BackupCopy`: a new empty attribute that restores this one.
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        let copy = self.new_empty();
        let this = self.attribute_base().this()?;
        crate::live(&copy, "TDF_Attribute::BackupCopy")?.restore(&this)?;
        Ok(copy)
    }
    /// `DeltaOnAddition`: a `TDF_DeltaOnAddition` on this attribute.
    fn delta_on_addition(&self) -> Result<Handle<TdfDeltaOnAddition>, Failure> {
        Ok(TdfDeltaOnAddition::new(&self.attribute_base().this()?))
    }
    /// `DeltaOnForget`: a `TDF_DeltaOnForget` on this attribute.
    fn delta_on_forget(&self) -> Result<Handle<TdfDeltaOnForget>, Failure> {
        Ok(TdfDeltaOnForget::new(&self.attribute_base().this()?))
    }
    /// `DeltaOnResume`: a `TDF_DeltaOnResume` on this attribute.
    fn delta_on_resume(&self) -> Result<Handle<TdfDeltaOnResume>, Failure> {
        Ok(TdfDeltaOnResume::new(&self.attribute_base().this()?))
    }
    /// `DeltaOnModification(anOldAttribute)`: a `TDF_DefaultDeltaOnModification` that
    /// restores `old_attribute`.
    fn delta_on_modification(
        &self,
        old_attribute: &Handle<dyn TdfAttribute>,
    ) -> Result<Handle<dyn TdfDeltaOnModification>, Failure> {
        Ok(TdfDefaultDeltaOnModification::new(old_attribute).cast())
    }
    /// `DeltaOnModification(aDelta)`: backs up, then restores the delta's attribute.
    fn apply_delta_on_modification(
        &self,
        delta: &Handle<dyn TdfDeltaOnModification>,
    ) -> Result<(), Failure> {
        self.backup()?;
        let attribute = crate::live(delta, "TDF_Attribute::DeltaOnModification")?.attribute();
        self.restore(&attribute)
    }
    /// `DeltaOnRemoval`: a `TDF_DefaultDeltaOnRemoval` on this attribute.
    fn delta_on_removal(&self) -> Result<Handle<dyn TdfDeltaOnRemoval>, Failure> {
        Ok(TdfDefaultDeltaOnRemoval::new(&self.attribute_base().this()?).cast())
    }
    /// `References`: adds nothing by default.
    fn references(&self, _data_set: &Handle<TdfDataSet>) {}
    /// `Dump`: `"\t<type>\tTrans. <n>; Valid Backuped Forgotten;\tID = <guid>"` and a newline.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)
    }
    /// `ExtendedDump`: [`TdfAttribute::dump`] by default.
    fn extended_dump(
        &self,
        stream: &mut dyn io::Write,
        _filter: &TdfIdFilter,
        _map: &mut AttributeIndexMap,
    ) -> io::Result<()> {
        self.dump(stream)
    }
    /// `DumpJson`: the label entry, ID, transaction and status flags.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        attribute_dump_json(self, stream, depth)
    }
}
impl_transient_interface!(dyn TdfAttribute, "TDF_Attribute", StandardTransient);

/// The native `TDF_Attribute::Dump` (non-virtual qualified calls `att->TDF_Attribute::Dump`
/// use it directly).
pub fn dump_attribute<A: TdfAttribute + ?Sized>(attribute: &A, stream: &mut dyn io::Write) -> io::Result<()> {
    let name = attribute.dynamic_type().name().to_owned();
    write!(stream, "\t{name}\tTrans. {};", attribute.transaction())?;
    if attribute.is_valid() {
        write!(stream, " Valid")?;
    }
    if attribute.is_backuped() {
        write!(stream, " Backuped")?;
    }
    if attribute.is_forgotten() {
        write!(stream, " Forgotten")?;
    }
    writeln!(stream, ";\tID = {}", attribute.id())
}
