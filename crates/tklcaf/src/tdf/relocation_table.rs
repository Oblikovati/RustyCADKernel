// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_RelocationTable.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_RelocationTable`: source-to-target bindings of labels, attributes and transients
//! used by the copy algorithms.
use super::{TdfAttribute, TdfAttributeMap, TdfLabel, TdfLabelMap};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{DataMap, IndexedDataMap},
    standard::{Handle, RuntimeTyped, SharedState, StandardTransient},
};
use std::{io, sync::RwLockWriteGuard};

/// `NCollection_DataMap<TDF_Label, TDF_Label>`.
pub type TdfLabelTable = DataMap<TdfLabel, TdfLabel>;
/// `NCollection_DataMap<occ::handle<TDF_Attribute>, occ::handle<TDF_Attribute>>`.
pub type TdfAttributeTable = DataMap<Handle<dyn TdfAttribute>, Handle<dyn TdfAttribute>>;
/// `NCollection_IndexedDataMap<occ::handle<Standard_Transient>, occ::handle<Standard_Transient>>`.
pub type TdfTransientTable = IndexedDataMap<Handle<StandardTransient>, Handle<StandardTransient>>;

#[derive(Default, Clone, Copy)]
struct RelocationFlags {
    self_relocate: bool,
    after_relocate: bool,
}

/// `TDF_RelocationTable`. The members that return references natively lend write guards.
///
/// ```
/// use rustycad_tklcaf::tdf::{TdfData, TdfLabel, TdfRelocationTable};
/// let data = TdfData::new();
/// let (source, target) = (data.root().find_child(1, true).unwrap(), data.root().find_child(2, true).unwrap());
/// let table = TdfRelocationTable::new(false);
/// table.set_label_relocation(&source, &target);
/// let mut found = TdfLabel::default();
/// assert!(table.has_label_relocation(&source, &mut found) && found == target);
/// ```
#[derive(Default)]
pub struct TdfRelocationTable {
    transient: StandardTransient,
    flags: SharedState<RelocationFlags>,
    label_table: SharedState<TdfLabelTable>,
    attribute_table: SharedState<TdfAttributeTable>,
    transient_table: SharedState<TdfTransientTable>,
}
impl_transient!(
    TdfRelocationTable,
    "TDF_RelocationTable",
    StandardTransient,
    transient
);

impl TdfRelocationTable {
    /// `TDF_RelocationTable(selfRelocate)`.
    pub fn new(self_relocate: bool) -> Self {
        let table = Self::default();
        table.flags.write().self_relocate = self_relocate;
        table
    }
    /// `SelfRelocate(bool)`.
    pub fn set_self_relocate(&self, self_relocate: bool) {
        self.flags.write().self_relocate = self_relocate;
    }
    /// `SelfRelocate()`: an unbound item relocates to itself.
    pub fn self_relocate(&self) -> bool {
        self.flags.read().self_relocate
    }
    /// `AfterRelocate(bool)`.
    pub fn set_after_relocate(&self, after_relocate: bool) {
        self.flags.write().after_relocate = after_relocate;
    }
    /// `AfterRelocate()`: a self-relocated item then reports no relocation.
    pub fn after_relocate(&self) -> bool {
        self.flags.read().after_relocate
    }
    /// The answer for an unbound item: itself under self relocation.
    fn unbound<T: Clone>(&self, source: &T, target: &mut T) -> bool {
        let flags = *self.flags.read();
        if flags.self_relocate {
            *target = source.clone();
            return !flags.after_relocate;
        }
        false
    }
    /// `SetRelocation(label, label)`: binds once; a second binding is ignored.
    pub fn set_label_relocation(&self, source: &TdfLabel, target: &TdfLabel) {
        let mut table = self.label_table.write();
        if !table.is_bound(source) {
            table.bind(source.clone(), target.clone());
        }
    }
    /// `HasRelocation(label, label&)`: the bound target, else see `SelfRelocate`.
    pub fn has_label_relocation(&self, source: &TdfLabel, target: &mut TdfLabel) -> bool {
        target.nullify();
        if let Ok(found) = self.label_table.read().find(source) {
            *target = found.clone();
            return true;
        }
        self.unbound(source, target)
    }
    /// `SetRelocation(attribute, attribute)`: binds once.
    pub fn set_attribute_relocation(
        &self,
        source: &Handle<dyn TdfAttribute>,
        target: &Handle<dyn TdfAttribute>,
    ) {
        let mut table = self.attribute_table.write();
        if !table.is_bound(source) {
            table.bind(source.clone(), target.clone());
        }
    }
    /// `HasRelocation(attribute, attribute&)`.
    pub fn has_attribute_relocation(
        &self,
        source: &Handle<dyn TdfAttribute>,
        target: &mut Handle<dyn TdfAttribute>,
    ) -> bool {
        *target = Handle::default();
        if let Ok(found) = self.attribute_table.read().find(source) {
            *target = found.clone();
            return true;
        }
        self.unbound(source, target)
    }
    /// The template `HasRelocation<T>`: relocated and of type `T`.
    pub fn has_attribute_relocation_as<T: RuntimeTyped + ?Sized>(
        &self,
        source: &Handle<dyn TdfAttribute>,
        target: &mut Handle<T>,
    ) -> bool {
        let mut found = Handle::default();
        if !self.has_attribute_relocation(source, &mut found) {
            return false;
        }
        *target = found.cast::<T>();
        target.is_some()
    }
    /// `SetTransientRelocation`: binds once.
    pub fn set_transient_relocation(
        &self,
        source: &Handle<StandardTransient>,
        target: &Handle<StandardTransient>,
    ) {
        let mut table = self.transient_table.write();
        if !table.contains(source) {
            table.add(source.clone(), target.clone());
        }
    }
    /// `HasTransientRelocation`.
    pub fn has_transient_relocation(
        &self,
        source: &Handle<StandardTransient>,
        target: &mut Handle<StandardTransient>,
    ) -> bool {
        *target = Handle::default();
        if let Ok(found) = self.transient_table.read().find_from_key(source) {
            *target = found.clone();
            return true;
        }
        self.unbound(source, target)
    }
    /// `Clear`: forgets every binding (not the flags).
    pub fn clear(&self) {
        self.label_table.write().clear(true);
        self.attribute_table.write().clear(true);
        self.transient_table.write().clear(true);
    }
    /// `TargetLabelMap`: adds every bound target label.
    pub fn target_label_map(&self, labels: &mut TdfLabelMap) {
        for (_, target) in self.label_table.read().items() {
            labels.add(target.clone());
        }
    }
    /// `TargetAttributeMap`: adds every bound target attribute.
    pub fn target_attribute_map(&self, attributes: &mut TdfAttributeMap) {
        for (_, target) in self.attribute_table.read().items() {
            attributes.add(target.clone());
        }
    }
    /// `LabelTable`.
    pub fn label_table(&self) -> RwLockWriteGuard<'_, TdfLabelTable> {
        self.label_table.write()
    }
    /// `AttributeTable`.
    pub fn attribute_table(&self) -> RwLockWriteGuard<'_, TdfAttributeTable> {
        self.attribute_table.write()
    }
    /// `TransientTable`.
    pub fn transient_table(&self) -> RwLockWriteGuard<'_, TdfTransientTable> {
        self.transient_table.write()
    }
    /// `Dump(dumpLabels, dumpAttributes, dumpTransients, anOS)`.
    ///
    /// Native quirk kept: the newline after the label table goes to standard output, not
    /// to the stream.
    pub fn dump(
        &self,
        dump_labels: bool,
        dump_attributes: bool,
        dump_transients: bool,
        stream: &mut dyn io::Write,
    ) -> io::Result<()> {
        let flags = *self.flags.read();
        let self_word = if flags.self_relocate { "IS" } else { "NOT" };
        let after_word = if flags.after_relocate {
            "WITH"
        } else {
            "WITHOUT"
        };
        writeln!(
            stream,
            "Relocation Table  {self_word} self relocate {after_word} after relocate"
        )?;
        write!(stream, "Nb labels={}", self.label_table.read().extent())?;
        write!(
            stream,
            "  Nb attributes={}",
            self.attribute_table.read().extent()
        )?;
        writeln!(
            stream,
            "  Nb transients={}",
            self.transient_table.read().extent()
        )?;
        if dump_labels {
            self.dump_labels(stream)?;
            println!();
        }
        if dump_attributes {
            self.dump_attributes(stream)?;
        }
        if dump_transients {
            let count = self.transient_table.read().extent();
            writeln!(stream, "Transient Table:{count} transient(s) in table.")?;
        }
        Ok(())
    }
    fn dump_labels(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "Label Table:")?;
        let pairs: Vec<_> = self
            .label_table
            .read()
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (index, (source, target)) in pairs.iter().enumerate() {
            write!(stream, "{} ", index + 1)?;
            source.entry_dump(stream)?;
            write!(stream, "<=>")?;
            target.entry_dump(stream)?;
            write!(stream, "| ")?;
        }
        Ok(())
    }
    fn dump_attributes(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "Attribute Table:")?;
        let pairs: Vec<_> = self
            .attribute_table
            .read()
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (index, (source, target)) in pairs.iter().enumerate() {
            write!(stream, "{} ", index + 1)?;
            if let Some(source) = source.get() {
                source.dump(stream)?;
            }
            write!(stream, "<=>")?;
            if let Some(target) = target.get() {
                target.dump(stream)?;
            }
            writeln!(stream, "| ")?;
        }
        Ok(())
    }
}
