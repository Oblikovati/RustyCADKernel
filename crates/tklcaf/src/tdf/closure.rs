// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_ClosureMode (.hxx, .lxx, .cxx) and
// TDF_ClosureTool (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_ClosureMode` and `TDF_ClosureTool`: completes a data set with descendants and
//! references.
use super::{
    TdfAttributeIterator, TdfAttributeMap, TdfAttributeState, TdfChildIterator, TdfDataSet,
    TdfIdFilter, TdfLabel, TdfLabelMap,
};
use rustycad_tkernel::standard::{Failure, Handle};

/// `DescendantsFlag`.
const DESCENDANTS_FLAG: i32 = 1;
/// `ReferencesFlag`.
const REFERENCES_FLAG: i32 = 2;

/// `TDF_ClosureMode`: which closures to compute; `TdfClosureMode::new(true)` sets all.
#[derive(Clone, Copy, Debug)]
pub struct TdfClosureMode {
    flags: i32,
}
impl Default for TdfClosureMode {
    fn default() -> Self {
        Self::new(true)
    }
}
impl TdfClosureMode {
    /// `TDF_ClosureMode(aMode)`: every flag set (`~0`) or none.
    pub fn new(mode: bool) -> Self {
        Self {
            flags: if mode { !0 } else { 0 },
        }
    }
    /// `Descendants(aStatus)`.
    pub fn set_descendants(&mut self, status: bool) {
        self.set(DESCENDANTS_FLAG, status);
    }
    /// `Descendants()`.
    pub fn descendants(&self) -> bool {
        self.flags & DESCENDANTS_FLAG != 0
    }
    /// `References(aStatus)`.
    pub fn set_references(&mut self, status: bool) {
        self.set(REFERENCES_FLAG, status);
    }
    /// `References()`.
    pub fn references(&self) -> bool {
        self.flags & REFERENCES_FLAG != 0
    }
    fn set(&mut self, flag: i32, status: bool) {
        self.flags = if status {
            self.flags | flag
        } else {
            self.flags & !flag
        };
    }
}

/// `TDF_ClosureTool`.
pub struct TdfClosureTool;
impl TdfClosureTool {
    /// `Closure(aDataSet)`: keep-all filter, every mode.
    pub fn closure(data_set: &Handle<TdfDataSet>) -> Result<(), Failure> {
        Self::closure_filtered(
            data_set,
            &TdfIdFilter::new(false),
            &TdfClosureMode::new(true),
        )
    }
    /// `Closure(aDataSet, aFilter, aMode)`: the labels of the set become its roots, then
    /// their attributes, descendants and references are added.
    pub fn closure_filtered(
        data_set: &Handle<TdfDataSet>,
        filter: &TdfIdFilter,
        mode: &TdfClosureMode,
    ) -> Result<(), Failure> {
        let Some(set) = data_set.get() else {
            return Err(rustycad_tkernel::standard::exceptions::null_object(
                "TDF_ClosureTool::Closure: null data set",
            ));
        };
        let roots: Vec<TdfLabel> = set.labels().iter().cloned().collect();
        *set.roots() = roots.clone();
        let mut labels = std::mem::take(&mut *set.labels());
        let mut attributes = std::mem::take(&mut *set.attributes());
        let outcome = close_roots(&roots, &mut labels, &mut attributes, filter, mode);
        *set.labels() = labels;
        *set.attributes() = attributes;
        outcome
    }
    /// `Closure(aLabel, aLabMap, anAttMap, aFilter, aMode)`: the descendants with
    /// attributes, their ancestors, and their attributes' closure.
    pub fn closure_label(
        label: &TdfLabel,
        labels: &mut TdfLabelMap,
        attributes: &mut TdfAttributeMap,
        filter: &TdfIdFilter,
        mode: &TdfClosureMode,
    ) -> Result<(), Failure> {
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            let child = children.value();
            if child.has_attribute()? {
                labels.add(child.clone());
                let mut up = child.father();
                while labels.add(up.clone()) {
                    up = up.father();
                }
                label_attributes(&child, labels, attributes, filter, mode)?;
            }
            children.next();
        }
        Ok(())
    }
}

/// The loop over the roots of `Closure(aDataSet, ...)`.
fn close_roots(
    roots: &[TdfLabel],
    labels: &mut TdfLabelMap,
    attributes: &mut TdfAttributeMap,
    filter: &TdfIdFilter,
    mode: &TdfClosureMode,
) -> Result<(), Failure> {
    for root in roots {
        if root.has_attribute()? {
            label_attributes(root, labels, attributes, filter, mode)?;
        }
        if mode.descendants() {
            TdfClosureTool::closure_label(root, labels, attributes, filter, mode)?;
        }
    }
    Ok(())
}

/// Private `LabelAttributes`: the kept attributes of the label and, in reference mode,
/// what they reference.
fn label_attributes(
    label: &TdfLabel,
    labels: &mut TdfLabelMap,
    attributes: &mut TdfAttributeMap,
    filter: &TdfIdFilter,
    mode: &TdfClosureMode,
) -> Result<(), Failure> {
    let mut iterator = TdfAttributeIterator::new(label, true)?;
    while iterator.more() {
        let attribute = iterator.value();
        iterator.next();
        if !filter.is_kept_attribute(&attribute) || !attributes.add(attribute.clone()) {
            continue;
        }
        if mode.references() {
            close_references(&attribute, labels, attributes, filter, mode)?;
        }
    }
    Ok(())
}

/// The references of one attribute: referenced attributes (by their label, or alone when
/// they have none), then referenced labels.
fn close_references(
    attribute: &Handle<dyn super::TdfAttribute>,
    labels: &mut TdfLabelMap,
    attributes: &mut TdfAttributeMap,
    filter: &TdfIdFilter,
    mode: &TdfClosureMode,
) -> Result<(), Failure> {
    let references = Handle::new(TdfDataSet::new());
    if let Some(attribute) = attribute.get() {
        attribute.references(&references);
    }
    let referenced: Vec<_> = references.attributes().iter().cloned().collect();
    for other in referenced {
        let Some(other_attribute) = other.get() else {
            continue;
        };
        let other_label = other_attribute.label();
        if other_label.is_null() {
            // Native assumption: an attribute without label references nothing itself.
            attributes.add(other.clone());
        } else if labels.add(other_label.clone()) {
            TdfClosureTool::closure_label(&other_label, labels, attributes, filter, mode)?;
        }
    }
    let referenced_labels: Vec<_> = references.labels().iter().cloned().collect();
    for other_label in referenced_labels {
        if labels.add(other_label.clone()) {
            TdfClosureTool::closure_label(&other_label, labels, attributes, filter, mode)?;
        }
    }
    Ok(())
}
