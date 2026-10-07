// Rust port of the reference queries of ApplicationFramework/TKLCAF/TDF/TDF_Tool.cxx
// (IsSelfContained, OutReferers, OutReferences) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reference queries of `TDF_Tool`.
use super::{
    TdfAttributeIterator, TdfAttributeMap, TdfAttributeState, TdfChildIterator, TdfDataSet,
    TdfIdFilter, TdfLabel, TdfTool,
};
use rustycad_tkernel::standard::{Failure, Handle};

/// The two filters of the referer/reference queries.
struct ReferenceFilters<'a> {
    referers: &'a TdfIdFilter,
    references: &'a TdfIdFilter,
}

impl TdfTool {
    /// `IsSelfContained(aLabel)`: nothing in the sub-tree references outside it.
    pub fn is_self_contained(label: &TdfLabel) -> Result<bool, Failure> {
        Self::is_self_contained_filtered(label, &TdfIdFilter::new(false))
    }
    /// `IsSelfContained(aLabel, aFilter)`: only kept attributes count.
    pub fn is_self_contained_filtered(
        label: &TdfLabel,
        filter: &TdfIdFilter,
    ) -> Result<bool, Failure> {
        let data_set = Handle::new(TdfDataSet::new());
        if !descendant_references(label, label, filter, &data_set)? {
            return Ok(false);
        }
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            if !descendant_references(label, &children.value(), filter, &data_set)? {
                return Ok(false);
            }
            children.next();
        }
        Ok(true)
    }
    /// `OutReferers(aLabel, atts)`: the attributes of the sub-tree that reference outside.
    pub fn out_referers(label: &TdfLabel, attributes: &mut TdfAttributeMap) -> Result<(), Failure> {
        let filter = TdfIdFilter::new(false);
        Self::out_referers_filtered(label, &filter, &filter, attributes)
    }
    /// `OutReferers(aLabel, aFilterForReferers, aFilterForReferences, atts)`.
    pub fn out_referers_filtered(
        label: &TdfLabel,
        referers: &TdfIdFilter,
        references: &TdfIdFilter,
        attributes: &mut TdfAttributeMap,
    ) -> Result<(), Failure> {
        let filters = ReferenceFilters {
            referers,
            references,
        };
        let data_set = Handle::new(TdfDataSet::new());
        out_referers_of(label, label, attributes, &filters, &data_set)?;
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            out_referers_of(label, &children.value(), attributes, &filters, &data_set)?;
            children.next();
        }
        Ok(())
    }
    /// `OutReferences(aLabel, atts)`: the attributes outside the sub-tree it references.
    pub fn out_references(
        label: &TdfLabel,
        attributes: &mut TdfAttributeMap,
    ) -> Result<(), Failure> {
        let filter = TdfIdFilter::new(false);
        Self::out_references_filtered(label, &filter, &filter, attributes)
    }
    /// `OutReferences(aLabel, aFilterForReferers, aFilterForReferences, atts)`.
    pub fn out_references_filtered(
        label: &TdfLabel,
        referers: &TdfIdFilter,
        references: &TdfIdFilter,
        attributes: &mut TdfAttributeMap,
    ) -> Result<(), Failure> {
        let filters = ReferenceFilters {
            referers,
            references,
        };
        let data_set = Handle::new(TdfDataSet::new());
        out_references_of(label, label, attributes, &filters, &data_set)?;
        let mut children = TdfChildIterator::new(label, true)?;
        while children.more() {
            out_references_of(label, &children.value(), attributes, &filters, &data_set)?;
            children.next();
        }
        Ok(())
    }
}

/// The references collected in the data set by one attribute.
fn collected(
    data_set: &Handle<TdfDataSet>,
) -> (Vec<TdfLabel>, Vec<Handle<dyn super::TdfAttribute>>) {
    let Some(set) = data_set.get() else {
        return (Vec::new(), Vec::new());
    };
    let labels = set.labels().iter().cloned().collect();
    let attributes = set.attributes().iter().cloned().collect();
    (labels, attributes)
}

fn clear(data_set: &Handle<TdfDataSet>) {
    if let Some(set) = data_set.get() {
        set.clear();
    }
}

/// The file-static `TDF_Tool_DescendantRef`. Native quirk kept: the data set is cleared
/// only after an attribute whose references all stay inside.
fn descendant_references(
    reference_label: &TdfLabel,
    label: &TdfLabel,
    filter: &TdfIdFilter,
    data_set: &Handle<TdfDataSet>,
) -> Result<bool, Failure> {
    let mut iterator = TdfAttributeIterator::new(label, true)?;
    while iterator.more() {
        let attribute = iterator.value();
        iterator.next();
        if !filter.is_kept_attribute(&attribute) {
            continue;
        }
        if let Some(attribute) = attribute.get() {
            attribute.references(data_set);
        }
        let (labels, attributes) = collected(data_set);
        if labels
            .iter()
            .any(|label| !label.is_descendant(reference_label))
        {
            return Ok(false);
        }
        for other in attributes {
            let other_label = other.get().map(|other| other.label()).unwrap_or_default();
            if !other_label.is_null()
                && filter.is_kept_attribute(&other)
                && !other_label.is_descendant(reference_label)
            {
                return Ok(false);
            }
        }
        clear(data_set);
    }
    Ok(true)
}

/// The file-static `TDF_Tool_OutReferers`.
fn out_referers_of(
    reference_label: &TdfLabel,
    label: &TdfLabel,
    found: &mut TdfAttributeMap,
    filters: &ReferenceFilters<'_>,
    data_set: &Handle<TdfDataSet>,
) -> Result<(), Failure> {
    let mut iterator = TdfAttributeIterator::new(label, true)?;
    while iterator.more() {
        let attribute = iterator.value();
        iterator.next();
        if !filters.referers.is_kept_attribute(&attribute) {
            continue;
        }
        if let Some(referer) = attribute.get() {
            referer.references(data_set);
        }
        let (labels, attributes) = collected(data_set);
        let attribute_outside = attributes.iter().any(|other| {
            let other_label = other.get().map(|other| other.label()).unwrap_or_default();
            filters.references.is_kept_attribute(other)
                && !other_label.is_null()
                && !other_label.is_descendant(reference_label)
        });
        if attribute_outside
            || labels
                .iter()
                .any(|other| !other.is_descendant(reference_label))
        {
            found.add(attribute.clone());
        }
        clear(data_set);
    }
    Ok(())
}

/// The file-static `TDF_Tool_OutReferences`. Native quirk kept: the data set is cleared once
/// per label, so the references of earlier attributes are seen again.
fn out_references_of(
    reference_label: &TdfLabel,
    label: &TdfLabel,
    found: &mut TdfAttributeMap,
    filters: &ReferenceFilters<'_>,
    data_set: &Handle<TdfDataSet>,
) -> Result<(), Failure> {
    let mut iterator = TdfAttributeIterator::new(label, true)?;
    while iterator.more() {
        let attribute = iterator.value();
        iterator.next();
        if !filters.referers.is_kept_attribute(&attribute) {
            continue;
        }
        if let Some(referer) = attribute.get() {
            referer.references(data_set);
        }
        let (labels, attributes) = collected(data_set);
        for other in attributes {
            let other_label = other.get().map(|other| other.label()).unwrap_or_default();
            if filters.references.is_kept_attribute(&other)
                && !other_label.is_null()
                && !other_label.is_descendant(reference_label)
            {
                found.add(other.clone());
            }
        }
        for other_label in labels
            .iter()
            .filter(|other| !other.is_descendant(reference_label))
        {
            add_kept_attributes(other_label, filters.references, found)?;
        }
    }
    clear(data_set);
    Ok(())
}

/// Every attribute of the label that the filter keeps.
fn add_kept_attributes(
    label: &TdfLabel,
    filter: &TdfIdFilter,
    found: &mut TdfAttributeMap,
) -> Result<(), Failure> {
    let mut iterator = TdfAttributeIterator::new(label, true)?;
    while iterator.more() {
        if filter.is_kept_attribute(&iterator.value()) {
            found.add(iterator.value());
        }
        iterator.next();
    }
    Ok(())
}
