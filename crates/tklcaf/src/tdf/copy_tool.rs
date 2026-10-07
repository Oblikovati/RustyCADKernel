// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_CopyTool.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_CopyTool`: copies a closed data set to the labels its relocation table names.
use super::{
    TdfAttribute, TdfAttributeIterator, TdfAttributeMap, TdfAttributeTable, TdfChildIterator,
    TdfDataSet, TdfIdFilter, TdfLabel, TdfLabelMap, TdfLabelTable, TdfRelocationTable,
};
use crate::live;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};

/// `TDF_CopyTool`.
pub struct TdfCopyTool;
impl TdfCopyTool {
    /// `Copy(aSourceDataSet, aRelocationTable)`: the target attributes' privilege is ignored.
    pub fn copy(
        source: &Handle<TdfDataSet>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Self::copy_full(
            source,
            relocation_table,
            &TdfIdFilter::default(),
            &TdfIdFilter::default(),
            false,
        )
    }
    /// `Copy(aSourceDataSet, aRelocationTable, aPrivilegeFilter)`.
    pub fn copy_with_privilege(
        source: &Handle<TdfDataSet>,
        relocation_table: &Handle<TdfRelocationTable>,
        privilege_filter: &TdfIdFilter,
    ) -> Result<(), Failure> {
        Self::copy_full(
            source,
            relocation_table,
            privilege_filter,
            &TdfIdFilter::default(),
            false,
        )
    }
    /// `Copy(aSourceDataSet, aRelocationTable, aPrivilegeFilter, aRefFilter, setSelfContained)`:
    /// builds the target labels and attributes under each bound root, then pastes every
    /// bound source attribute whose target ID the privilege filter ignores. The reference
    /// filter and the self-contained flag are unused natively.
    pub fn copy_full(
        source: &Handle<TdfDataSet>,
        relocation_table: &Handle<TdfRelocationTable>,
        privilege_filter: &TdfIdFilter,
        _reference_filter: &TdfIdFilter,
        _set_self_contained: bool,
    ) -> Result<(), Failure> {
        let set = live(source, "TDF_CopyTool::Copy")?;
        if set.is_empty() {
            return Ok(());
        }
        let table = live(relocation_table, "TDF_CopyTool::Copy")?;
        let source_labels = set.labels().clone();
        let source_attributes = set.attributes().clone();
        let roots = set.roots().clone();
        let mut label_table = std::mem::take(&mut *table.label_table());
        let mut attribute_table = std::mem::take(&mut *table.attribute_table());
        let maps = SourceMaps {
            labels: &source_labels,
            attributes: &source_attributes,
        };
        let built = build_roots(&roots, &mut label_table, &mut attribute_table, &maps);
        *table.label_table() = label_table;
        *table.attribute_table() = attribute_table;
        built?;
        paste_all(relocation_table, privilege_filter)
    }
}

/// The source label and attribute maps of a copy.
struct SourceMaps<'a> {
    labels: &'a TdfLabelMap,
    attributes: &'a TdfAttributeMap,
}

/// The parallel exploration of the bound roots (an unbound root is skipped).
fn build_roots(
    roots: &[TdfLabel],
    labels: &mut TdfLabelTable,
    attributes: &mut TdfAttributeTable,
    maps: &SourceMaps<'_>,
) -> Result<(), Failure> {
    for root in roots {
        let Ok(target) = labels.find(root).cloned() else {
            continue;
        };
        copy_labels(root, &target, labels, attributes, maps)?;
    }
    Ok(())
}

/// The paste phase, in the iteration order of the attribute table.
fn paste_all(
    relocation_table: &Handle<TdfRelocationTable>,
    privilege_filter: &TdfIdFilter,
) -> Result<(), Failure> {
    let table = live(relocation_table, "TDF_CopyTool::Copy")?;
    let pairs: Vec<_> = table
        .attribute_table()
        .items()
        .map(|(s, t)| (s.clone(), t.clone()))
        .collect();
    for (source, target) in pairs {
        let Some(source_attribute) = source.get() else {
            continue;
        };
        let target_id = live(&target, "TDF_CopyTool::Copy")?.id();
        if source != target && privilege_filter.is_ignored(&target_id) {
            source_attribute.paste(&target, relocation_table)?;
        }
    }
    Ok(())
}

/// Private `CopyLabels`: the attributes of the label, then the children in the set.
fn copy_labels(
    source: &TdfLabel,
    target: &TdfLabel,
    labels: &mut TdfLabelTable,
    attributes: &mut TdfAttributeTable,
    maps: &SourceMaps<'_>,
) -> Result<(), Failure> {
    copy_attributes(source, target, attributes, maps.attributes)?;
    let mut children = TdfChildIterator::new(source, false)?;
    while children.more() {
        let child = children.value();
        if maps.labels.contains(&child) {
            let child_target = match labels.find(&child) {
                Ok(bound) => bound.clone(),
                Err(_) => {
                    let created = target.find_child(child.tag(), true)?;
                    labels.bind(child.clone(), created.clone());
                    created
                }
            };
            copy_labels(&child, &child_target, labels, attributes, maps)?;
        }
        children.next();
    }
    Ok(())
}

/// Private `CopyAttributes`: binds each source attribute of the set to the target's
/// attribute of the same ID, created empty (with the source ID) when absent;
/// `Standard_TypeMismatch` when the target's is of another type.
fn copy_attributes(
    source: &TdfLabel,
    target: &TdfLabel,
    attributes: &mut TdfAttributeTable,
    set: &TdfAttributeMap,
) -> Result<(), Failure> {
    let mut iterator = TdfAttributeIterator::new(source, true)?;
    while iterator.more() {
        let attribute = iterator.value();
        iterator.next();
        if !set.contains(&attribute) {
            continue;
        }
        let source_attribute = live(&attribute, "TDF_CopyTool::CopyAttributes")?;
        let id = source_attribute.id();
        let mut existing: Handle<dyn TdfAttribute> = Handle::default();
        if !target.find_attribute(&id, &mut existing)? {
            let created = source_attribute.new_empty();
            let created_attribute = live(&created, "TDF_CopyTool::CopyAttributes")?;
            if created_attribute.id() != id {
                created_attribute.set_id(&id);
            }
            target.add_attribute(&created, true)?;
            attributes.bind(attribute.clone(), created);
            continue;
        }
        if !existing.is_instance(source_attribute.dynamic_type()) {
            return Err(exceptions::type_mismatch(&format!(
                "TDF_CopyTool: Cannot paste to a different type attribute ({} into {}).",
                source_attribute.dynamic_type().name(),
                existing.dynamic_type().map_or("", |t| t.name())
            )));
        }
        attributes.bind(attribute.clone(), existing);
    }
    Ok(())
}
