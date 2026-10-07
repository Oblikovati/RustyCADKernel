// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_TagSource.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_TagSource`: the attribute that hands out new child tags.
use super::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable};
use crate::{guid_from_text, json::dump_attribute_json_base, live};
use rustycad_tkcdf::json_dump::{class_begin, field_integer};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream, exceptions,
    },
};
use std::io;

/// `TDF_TagSource::GetID()`.
const TAG_SOURCE_ID: Guid = guid_from_text("2a96b611-ec8b-11d0-bee7-080009dc3333");

/// `TDF_TagSource`; `TdfTagSource::set(&label)` finds or adds one.
///
/// ```
/// use rustycad_tklcaf::tdf::{TdfData, TdfTagSource};
/// let data = TdfData::new();
/// let first = TdfTagSource::new_child_of(&data.root()).unwrap();
/// let second = TdfTagSource::new_child_of(&data.root()).unwrap();
/// assert_eq!((first.tag(), second.tag()), (1, 2));
/// ```
#[derive(Default)]
pub struct TdfTagSource {
    transient: StandardTransient,
    base: TdfAttributeBase,
    tag: SharedState<i32>,
}
impl_transient!(TdfTagSource, "TDF_TagSource", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TdfTagSource {
    /// `GetID`; `TdfTagSource::get_id()`.
    pub fn get_id() -> Guid {
        TAG_SOURCE_ID
    }
    /// `TDF_TagSource()`: tag 0.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label)`: the tag source of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<TdfTagSource>, Failure> {
        let mut found: Handle<TdfTagSource> = Handle::default();
        if label.find_attribute_as(&TAG_SOURCE_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(TdfTagSource::new());
        label.add_attribute(&created.cast(), true)?;
        Ok(created)
    }
    /// `NewChild(L)` (static): a new child of `label` from its tag source, added when absent.
    pub fn new_child_of(label: &TdfLabel) -> Result<TdfLabel, Failure> {
        Self::set(label)?
            .get()
            .ok_or_else(|| exceptions::null_object("TDF_TagSource::NewChild"))?
            .new_child()
    }
    /// `NewTag`: backs up and increments the tag.
    pub fn new_tag(&self) -> Result<i32, Failure> {
        self.backup()?;
        let mut tag = self.tag.write();
        *tag += 1;
        Ok(*tag)
    }
    /// `NewChild()`: the child of the attribute's label with a new tag.
    pub fn new_child(&self) -> Result<TdfLabel, Failure> {
        let tag = self.new_tag()?;
        self.label().find_child(tag, true)
    }
    /// `Get`: the last tag given.
    pub fn get(&self) -> i32 {
        *self.tag.read()
    }
    /// `Set(T)`: the last tag given (backed up when it changes, OCC2932).
    pub fn set_value(&self, tag: i32) -> Result<(), Failure> {
        if self.get() == tag {
            return Ok(());
        }
        self.backup()?;
        *self.tag.write() = tag;
        Ok(())
    }
}

impl TdfAttribute for TdfTagSource {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        TAG_SOURCE_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let tag = live(&with.cast::<TdfTagSource>(), "TDF_TagSource::Restore")?.get();
        *self.tag.write() = tag;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(TdfTagSource::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        live(&into.cast::<TdfTagSource>(), "TDF_TagSource::Paste")?.set_value(self.get())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDF_TagSource")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_integer(stream, "myTag", self.get())
    }
}
