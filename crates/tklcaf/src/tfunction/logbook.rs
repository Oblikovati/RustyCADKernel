// Rust port of ApplicationFramework/TKLCAF/TFunction/TFunction_Logbook (.hxx, .lxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TFunction_Logbook`: the touched, impacted and valid labels of a function scope, kept
//! on the root label.
use crate::guid_from_text;
use crate::live;
use crate::tdata_std::support::cast_attribute;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfChildIterator, TdfLabel, TdfLabelMap,
    TdfRelocationTable, TdfTool,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use std::io;
use std::sync::RwLockReadGuard;

/// `TFunction_Logbook::GetID()`.
const LOGBOOK_ID: Guid = guid_from_text("CF519724-5CA4-4B90-835F-8919BE1DDE4B");

/// `TFunction_Logbook`; `TFunctionLogbook::set(&label)` (on the root label). The map
/// getters lend the read guard of the map (natively a reference to it): a caller drops
/// it before it modifies the logbook.
///
/// ```
/// use rustycad_tklcaf::{tfunction::TFunctionLogbook, tdf::TdfData};
/// let data = TdfData::new();
/// let child = data.root().find_child(1, true).unwrap();
/// let logbook = TFunctionLogbook::set(&child).unwrap();
/// logbook.set_touched(&child).unwrap();
/// assert!(logbook.is_modified(&data.root(), true).unwrap());
/// ```
#[derive(Default)]
pub struct TFunctionLogbook {
    transient: StandardTransient,
    base: TdfAttributeBase,
    touched: SharedState<TdfLabelMap>,
    impacted: SharedState<TdfLabelMap>,
    valid: SharedState<TdfLabelMap>,
    is_done: SharedState<bool>,
}
impl_transient!(TFunctionLogbook, "TFunction_Logbook", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

/// Adds `label` and, with children, all its sublabels to `map`.
fn add_with_children(
    map: &mut TdfLabelMap,
    label: &TdfLabel,
    with_children: bool,
) -> Result<(), Failure> {
    map.add(label.clone());
    if !with_children {
        return Ok(());
    }
    let mut children = TdfChildIterator::new(label, true)?;
    while children.more() {
        map.add(children.value());
        children.next();
    }
    Ok(())
}

impl TFunctionLogbook {
    /// `Set(Access)`: the logbook of the root label, added when absent.
    pub fn set(access: &TdfLabel) -> Result<Handle<Self>, Failure> {
        let root = access.root()?;
        let mut found: Handle<Self> = Handle::default();
        if root.find_attribute_as(&LOGBOOK_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        root.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        LOGBOOK_ID
    }
    /// `TFunction_Logbook()`: empty, not done.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Clear`: unless empty, backs up and empties the three maps.
    pub fn clear(&self) -> Result<(), Failure> {
        if self.is_empty() {
            return Ok(());
        }
        self.backup()?;
        self.touched.write().clear(false);
        self.impacted.write().clear(false);
        self.valid.write().clear(false);
        Ok(())
    }
    /// `IsEmpty`: the three maps are empty.
    pub fn is_empty(&self) -> bool {
        self.touched.read().is_empty()
            && self.impacted.read().is_empty()
            && self.valid.read().is_empty()
    }
    /// `SetTouched(L)`: backed up when the label is new.
    pub fn set_touched(&self, label: &TdfLabel) -> Result<(), Failure> {
        if self.touched.read().contains(label) {
            return Ok(());
        }
        self.backup()?;
        self.touched.write().add(label.clone());
        Ok(())
    }
    /// `SetImpacted(L, WithChildren)`: backs up, then adds the label (and its sublabels).
    pub fn set_impacted(&self, label: &TdfLabel, with_children: bool) -> Result<(), Failure> {
        self.backup()?;
        add_with_children(&mut self.impacted.write(), label, with_children)
    }
    /// `SetValid(L, WithChildren)`: backs up, then adds the label (and its sublabels).
    pub fn set_valid(&self, label: &TdfLabel, with_children: bool) -> Result<(), Failure> {
        self.backup()?;
        add_with_children(&mut self.valid.write(), label, with_children)
    }
    /// `SetValid(Ls)`: backs up, then adds every label of the map.
    pub fn set_valid_labels(&self, labels: &TdfLabelMap) -> Result<(), Failure> {
        self.backup()?;
        let mut valid = self.valid.write();
        for label in labels.iter() {
            valid.add(label.clone());
        }
        Ok(())
    }
    /// `IsModified(L, WithChildren)`: the label (or, with children, a sublabel) is touched
    /// or impacted.
    pub fn is_modified(&self, label: &TdfLabel, with_children: bool) -> Result<bool, Failure> {
        if self.touched.read().contains(label) || self.impacted.read().contains(label) {
            return Ok(true);
        }
        if !with_children {
            return Ok(false);
        }
        let mut children = TdfChildIterator::new(label, false)?;
        while children.more() {
            if self.is_modified(&children.value(), true)? {
                return Ok(true);
            }
            children.next();
        }
        Ok(false)
    }
    /// `GetTouched`: the touched labels.
    pub fn get_touched(&self) -> RwLockReadGuard<'_, TdfLabelMap> {
        self.touched.read()
    }
    /// `GetImpacted`: the impacted labels.
    pub fn get_impacted(&self) -> RwLockReadGuard<'_, TdfLabelMap> {
        self.impacted.read()
    }
    /// `GetValid()`: the valid labels.
    pub fn get_valid(&self) -> RwLockReadGuard<'_, TdfLabelMap> {
        self.valid.read()
    }
    /// `GetValid(Ls)`: adds the valid labels to `labels`.
    pub fn get_valid_into(&self, labels: &mut TdfLabelMap) {
        for label in self.valid.read().iter() {
            labels.add(label.clone());
        }
    }
    /// `Done(status)`: backed up when it changes.
    pub fn done(&self, status: bool) -> Result<(), Failure> {
        if *self.is_done.read() == status {
            return Ok(());
        }
        self.backup()?;
        *self.is_done.write() = status;
        Ok(())
    }
    /// `IsDone`.
    pub fn is_done(&self) -> bool {
        *self.is_done.read()
    }
}

/// Adds every label of `labels` to `map` (the native loops of `Restore`).
fn add_all(map: &mut TdfLabelMap, labels: &TdfLabelMap) {
    for label in labels.iter() {
        map.add(label.clone());
    }
}

/// One map of `Paste`: `target` is cleared (keeping its buckets) and takes the non-null
/// labels of `labels`, relocated when the table knows them.
fn paste_labels(
    target: &mut TdfLabelMap,
    labels: &TdfLabelMap,
    table: &Handle<TdfRelocationTable>,
) -> Result<(), Failure> {
    target.clear(false);
    for label in labels.iter().filter(|label| !label.is_null()) {
        let mut relocated = TdfLabel::default();
        if live(table, "TFunction_Logbook::Paste")?.has_label_relocation(label, &mut relocated) {
            target.add(relocated);
        } else {
            target.add(label.clone());
        }
    }
    Ok(())
}

impl TdfAttribute for TFunctionLogbook {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        LOGBOOK_ID
    }
    /// `Restore(other)`: the status, and the other's labels added to the three maps (the
    /// maps are not cleared first, as natively).
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TFunction_Logbook::Restore")?;
        let other = live(&other, "TFunction_Logbook::Restore")?;
        *self.is_done.write() = other.is_done();
        add_all(&mut self.valid.write(), &other.get_valid());
        add_all(&mut self.touched.write(), &other.get_touched());
        add_all(&mut self.impacted.write(), &other.get_impacted());
        Ok(())
    }
    /// `Paste(into, RT)`: `into` takes the status and the relocated labels.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<Self>(into, "TFunction_Logbook::Paste")?;
        let target = live(&target, "TFunction_Logbook::Paste")?;
        *target.is_done.write() = self.is_done();
        paste_labels(&mut target.touched.write(), &self.get_touched(), table)?;
        paste_labels(&mut target.impacted.write(), &self.get_impacted(), table)?;
        paste_labels(&mut target.valid.write(), &self.get_valid(), table)?;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Dump`: the status, then the touched, impacted and valid entries.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "Done = {}", i32::from(self.is_done()))?;
        let sections = [
            ("Touched labels: ", &self.touched),
            ("Impacted labels: ", &self.impacted),
            ("Valid labels: ", &self.valid),
        ];
        for (title, labels) in sections {
            let labels = labels.read();
            writeln!(stream, "{title}")?;
            for label in labels.iter() {
                let mut entry = AsciiString::default();
                TdfTool::entry(label, &mut entry);
                stream.write_all(entry.as_bytes())?;
                writeln!(stream)?;
            }
        }
        Ok(())
    }
}
