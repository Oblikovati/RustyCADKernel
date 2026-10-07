// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_DataSet.hxx, .lxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_DataSet`: a set of labels, attributes and root labels (the input of the closure and
//! copy tools).
use super::{TdfAttribute, TdfAttributeState, TdfLabel};
use rustycad_tkernel::{
    impl_transient,
    ncollection::Map,
    standard::{Handle, SharedState, StandardTransient},
};
use std::{io, sync::RwLockWriteGuard};

/// `NCollection_Map<TDF_Label>`.
pub type TdfLabelMap = Map<TdfLabel>;
/// `NCollection_Map<occ::handle<TDF_Attribute>>`.
pub type TdfAttributeMap = Map<Handle<dyn TdfAttribute>>;

/// `TDF_DataSet`. The members that return references natively lend write guards; drop a
/// guard before calling another member of the same set.
///
/// ```
/// use rustycad_tklcaf::tdf::{TdfData, TdfDataSet};
/// let data = TdfData::new();
/// let set = TdfDataSet::new();
/// set.add_label(&data.root());
/// set.add_root(&data.root());
/// assert!(set.contains_label(&data.root()) && set.roots().len() == 1);
/// ```
#[derive(Default)]
pub struct TdfDataSet {
    transient: StandardTransient,
    root_labels: SharedState<Vec<TdfLabel>>,
    label_map: SharedState<TdfLabelMap>,
    attribute_map: SharedState<TdfAttributeMap>,
}
impl_transient!(TdfDataSet, "TDF_DataSet", StandardTransient, transient);

impl TdfDataSet {
    /// `TDF_DataSet()`: empty.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Clear`: forgets the roots, labels and attributes.
    pub fn clear(&self) {
        self.root_labels.write().clear();
        self.label_map.write().clear(true);
        self.attribute_map.write().clear(true);
    }
    /// `IsEmpty`: no label and no attribute.
    pub fn is_empty(&self) -> bool {
        self.label_map.read().is_empty() && self.attribute_map.read().is_empty()
    }
    /// `AddLabel`: adds a non-null label.
    pub fn add_label(&self, label: &TdfLabel) {
        if !label.is_null() {
            self.label_map.write().add(label.clone());
        }
    }
    /// `ContainsLabel`.
    pub fn contains_label(&self, label: &TdfLabel) -> bool {
        self.label_map.read().contains(label)
    }
    /// `Labels`: the label map.
    pub fn labels(&self) -> RwLockWriteGuard<'_, TdfLabelMap> {
        self.label_map.write()
    }
    /// `AddAttribute`: adds a non-null attribute.
    pub fn add_attribute(&self, attribute: &Handle<dyn TdfAttribute>) {
        if attribute.is_some() {
            self.attribute_map.write().add(attribute.clone());
        }
    }
    /// `ContainsAttribute`.
    pub fn contains_attribute(&self, attribute: &Handle<dyn TdfAttribute>) -> bool {
        self.attribute_map.read().contains(attribute)
    }
    /// `Attributes`: the attribute map.
    pub fn attributes(&self) -> RwLockWriteGuard<'_, TdfAttributeMap> {
        self.attribute_map.write()
    }
    /// `AddRoot`: appends the label to the roots when the set contains it.
    pub fn add_root(&self, label: &TdfLabel) {
        if self.contains_label(label) {
            self.root_labels.write().push(label.clone());
        }
    }
    /// `Roots`: the root list.
    pub fn roots(&self) -> RwLockWriteGuard<'_, Vec<TdfLabel>> {
        self.root_labels.write()
    }
    /// `Dump` (and `operator<<`): the roots, the labels and the attributes.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "\t\t=====< TDF_DataSet dump >=====")?;
        writeln!(stream, "Root Labels :\n=============")?;
        for label in self.root_labels.read().iter() {
            label.entry_dump(stream)?;
            write!(stream, " | ")?;
        }
        writeln!(stream, "\nLabels :\n========")?;
        for label in self.label_map.read().iter() {
            label.entry_dump(stream)?;
            write!(stream, " | ")?;
        }
        writeln!(stream, "\nAttributes :\n============\n")?;
        let attributes: Vec<_> = self.attribute_map.read().iter().cloned().collect();
        for attribute in attributes.iter().filter_map(Handle::get) {
            attribute.label().entry_dump(stream)?;
            write!(stream, " \t")?;
            attribute.dump(stream)?;
            writeln!(stream)?;
        }
        writeln!(stream)
    }
}
