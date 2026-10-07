// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_UAttribute.hxx, .cxx and
// TDataStd_Current.hxx, .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_UAttribute` (an attribute that is only an ID) and `TDataStd_Current` (the
//! current label of a framework).
use super::support::{cast_attribute, find_or_add, set_backed_up, set_user_id};
use crate::guid_from_text;
use crate::json::{dump_attribute_json_base, dump_label_entry};
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfDataSet, TdfLabel, TdfRelocationTable, dump_attribute,
};
use rustycad_tkcdf::json_dump::{class_begin, field_guid};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream, exceptions,
    },
};
use std::io;

/// `TDataStd_UAttribute`: identified by a user ID only; `TDataStdUAttribute::set(&label, &id)`.
#[derive(Default)]
pub struct TDataStdUAttribute {
    transient: StandardTransient,
    base: TdfAttributeBase,
    id: SharedState<Guid>,
}
impl_transient!(TDataStdUAttribute, "TDataStd_UAttribute", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TDataStdUAttribute {
    /// `TDataStd_UAttribute()`: the null ID.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label, guid)`: finds or adds the attribute of that ID.
    pub fn set(label: &TdfLabel, guid: &Guid) -> Result<Handle<TDataStdUAttribute>, Failure> {
        find_or_add(label, guid, true, Self::new)
    }
}

impl TdfAttribute for TDataStdUAttribute {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        *self.id.read()
    }
    /// `SetID(LocalID)`: backed up when it changes (OCC2932).
    fn set_id(&self, guid: &Guid) -> Result<(), Failure> {
        set_user_id(self, &self.id, guid)
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TDataStdUAttribute>(with, "TDataStd_UAttribute::Restore")?;
        let id = crate::live(&other, "TDataStd_UAttribute::Restore")?.id();
        self.set_id(&id)
    }
    /// `NewEmpty`: a new attribute with the same ID.
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        let created = Self::new();
        *created.id.write() = self.id();
        Handle::new(created).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TDataStdUAttribute>(into, "TDataStd_UAttribute::Paste")?;
        crate::live(&target, "TDataStd_UAttribute::Paste")?.set_id(&self.id())
    }
    fn references(&self, _data_set: &Handle<TdfDataSet>) {}
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "UAttribute")?;
        dump_attribute(self, stream)
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_UAttribute")?;
        field_guid(stream, "myID", self.id())?;
        dump_attribute_json_base(self, stream, depth)
    }
}

/// `TDataStd_Current::GetID()`.
const CURRENT_ID: Guid = guid_from_text("2a96b623-ec8b-11d0-bee7-080009dc3333");

/// `TDataStd_Current`: kept on the root label; `TDataStdCurrent::set(&label)`.
#[derive(Default)]
pub struct TDataStdCurrent {
    transient: StandardTransient,
    base: TdfAttributeBase,
    label: SharedState<TdfLabel>,
}
impl_transient!(TDataStdCurrent, "TDataStd_Current", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TDataStdCurrent {
    /// `GetID`.
    pub fn get_id() -> Guid {
        CURRENT_ID
    }
    /// `TDataStd_Current()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The root label of the framework of `label` (`L.Data()->Root()`).
    fn root_of(label: &TdfLabel) -> Result<TdfLabel, Failure> {
        let data = label.data();
        Ok(crate::live(&data, "TDataStd_Current: the label has no framework")?.root())
    }
    /// `Set(L)`: makes `current` the current label of its framework.
    pub fn set(current: &TdfLabel) -> Result<(), Failure> {
        let attribute = find_or_add(&Self::root_of(current)?, &CURRENT_ID, false, Self::new)?;
        crate::live(&attribute, "TDataStd_Current::Set")?.set_label(current)
    }
    /// `Get(acces)`: the current label; `Standard_DomainError` when it was never set.
    pub fn get(access: &TdfLabel) -> Result<TdfLabel, Failure> {
        let mut attribute: Handle<TDataStdCurrent> = Handle::default();
        if !Self::root_of(access)?.find_attribute_as(&CURRENT_ID, &mut attribute)? {
            return Err(exceptions::domain_error("TDataStd_Current::Get : not set"));
        }
        Ok(crate::live(&attribute, "TDataStd_Current::Get")?.get_label())
    }
    /// `Has(acces)`: a current label is set.
    pub fn has(access: &TdfLabel) -> Result<bool, Failure> {
        Self::root_of(access)?.is_attribute(&CURRENT_ID)
    }
    /// `SetLabel`: backed up when it changes (OCC2932).
    pub fn set_label(&self, current: &TdfLabel) -> Result<(), Failure> {
        set_backed_up(self, &self.label, current)
    }
    /// `GetLabel`.
    pub fn get_label(&self) -> TdfLabel {
        self.label.read().clone()
    }
}

impl TdfAttribute for TDataStdCurrent {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        CURRENT_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TDataStdCurrent>(with, "TDataStd_Current::Restore")?;
        let label = crate::live(&other, "TDataStd_Current::Restore")?.get_label();
        *self.label.write() = label;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the relocated label, else the label itself.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let own = self.get_label();
        let mut target_label = TdfLabel::default();
        if !own.is_null() {
            let table = crate::live(relocation_table, "TDataStd_Current::Paste")?;
            if !table.has_label_relocation(&own, &mut target_label) {
                target_label = own;
            }
        }
        let target = cast_attribute::<TDataStdCurrent>(into, "TDataStd_Current::Paste")?;
        crate::live(&target, "TDataStd_Current::Paste")?.set_label(&target_label)
    }
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Current")
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_Current")?;
        dump_attribute_json_base(self, stream, depth)?;
        dump_label_entry(stream, "aLabel", &self.get_label())
    }
}
