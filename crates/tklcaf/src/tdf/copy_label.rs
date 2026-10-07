// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_CopyLabel.hxx, .lxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_CopyLabel`: copies a label, its descendants and what they reference to a target
//! label, sharing (not copying) the attributes referenced from outside.
use super::{
    TdfAttributeIterator, TdfAttributeMap, TdfAttributeState, TdfChildIterator, TdfClosureMode,
    TdfClosureTool, TdfCopyTool, TdfDataSet, TdfIdFilter, TdfLabel, TdfRelocationTable, TdfTool,
};
use crate::live;
use rustycad_tkernel::standard::{Failure, Handle};

/// `TDF_CopyLabel`.
///
/// ```
/// use rustycad_tklcaf::tdf::{TdfCopyLabel, TdfData, TdfTagSource};
/// let data = TdfData::new();
/// let source = data.root().find_child(1, true).unwrap();
/// TdfTagSource::set(&source).unwrap();
/// let target = data.root().find_child(2, true).unwrap();
/// let mut copy = TdfCopyLabel::with_labels(&source, &target);
/// copy.perform().unwrap();
/// assert!(copy.is_done() && target.is_attribute(&TdfTagSource::get_id()).unwrap());
/// ```
pub struct TdfCopyLabel {
    relocation_table: Handle<TdfRelocationTable>,
    source: TdfLabel,
    target: TdfLabel,
    filter: TdfIdFilter,
    externals: TdfAttributeMap,
    is_done: bool,
}
impl Default for TdfCopyLabel {
    /// `TDF_CopyLabel()`: null labels, keep-all filter.
    fn default() -> Self {
        Self::with_labels(&TdfLabel::default(), &TdfLabel::default())
    }
}
impl TdfCopyLabel {
    /// `TDF_CopyLabel()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `TDF_CopyLabel(aSource, aTarget)`.
    pub fn with_labels(source: &TdfLabel, target: &TdfLabel) -> Self {
        Self {
            relocation_table: Handle::default(),
            source: source.clone(),
            target: target.clone(),
            filter: TdfIdFilter::new(false),
            externals: TdfAttributeMap::default(),
            is_done: false,
        }
    }
    /// `Load(aSource, aTarget)`.
    pub fn load(&mut self, source: &TdfLabel, target: &TdfLabel) {
        self.source = source.clone();
        self.target = target.clone();
    }
    /// `UseFilter(aFilter)`: assigns the filter.
    pub fn use_filter(&mut self, filter: &TdfIdFilter) {
        self.filter.assign(filter);
    }
    /// `ExternalReferences(L, aExternals, aFilter)`: the kept attributes that the sub-tree
    /// of `label` references outside itself; true when there is one.
    pub fn external_references(
        label: &TdfLabel,
        externals: &mut TdfAttributeMap,
        filter: &TdfIdFilter,
    ) -> Result<bool, Failure> {
        let mut data_set = Handle::new(TdfDataSet::new());
        Self::external_references_of(label, label, externals, filter, &mut data_set)?;
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            Self::external_references_of(
                label,
                &children.value(),
                externals,
                filter,
                &mut data_set,
            )?;
            children.next();
        }
        Ok(externals.extent() != 0)
    }
    /// `ExternalReferences(aRefLab, Lab, aExternals, aFilter, aDataSet)`: the kept attributes
    /// referenced by the attributes of `label` outside `reference_label`.
    pub fn external_references_of(
        reference_label: &TdfLabel,
        label: &TdfLabel,
        externals: &mut TdfAttributeMap,
        filter: &TdfIdFilter,
        data_set: &mut Handle<TdfDataSet>,
    ) -> Result<(), Failure> {
        let set = live(data_set, "TDF_CopyLabel::ExternalReferences")?;
        let mut iterator = TdfAttributeIterator::new(label, true)?;
        while iterator.more() {
            if let Some(attribute) = iterator.value().get() {
                attribute.references(data_set);
            }
            let referenced: Vec<_> = set.attributes().iter().cloned().collect();
            for other in referenced {
                let other_label = other.get().map(|other| other.label()).unwrap_or_default();
                if other_label.is_null() {
                    continue;
                }
                if filter.is_kept_attribute(&other)
                    && other_label.is_different(reference_label)
                    && !other_label.is_descendant(reference_label)
                {
                    externals.add(other.clone());
                }
            }
            set.clear();
            iterator.next();
        }
        Ok(())
    }
    /// `Perform`: copies the source to the target; does nothing (not done) when they are in
    /// different frameworks and the source is not self-contained.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.is_done = false;
        let other_framework = self.source.data() != self.target.data();
        if other_framework && !TdfTool::is_self_contained_filtered(&self.source, &self.filter)? {
            return Ok(());
        }
        let has_externals =
            Self::external_references(&self.source, &mut self.externals, &self.filter)?;
        self.relocation_table = Handle::new(TdfRelocationTable::new(true));
        let table = live(&self.relocation_table, "TDF_CopyLabel::Perform")?;
        let data_set = Handle::new(TdfDataSet::new());
        live(&data_set, "TDF_CopyLabel::Perform")?.add_label(&self.source);
        table.set_label_relocation(&self.source, &self.target);
        TdfClosureTool::closure_filtered(&data_set, &self.filter, &TdfClosureMode::new(true))?;
        if has_externals {
            for external in self.externals.iter() {
                table.set_attribute_relocation(external, external);
            }
        }
        TdfCopyTool::copy(&data_set, &self.relocation_table)?;
        self.is_done = true;
        Ok(())
    }
    /// `IsDone`; `copy.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }
    /// `RelocationTable`: the table of the last `Perform`.
    pub fn relocation_table(&self) -> &Handle<TdfRelocationTable> {
        &self.relocation_table
    }
}
