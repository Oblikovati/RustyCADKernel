// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.cxx (Clone, copyData,
// CopyChildren, CopyReferences, copyReferences and the file-static copyTagSources) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native bodies of the `TObj_Object` cloning members.
use super::TObjObjectBase;
use super::{
    TObjAssistant, TObjModel, TObjObject, TObjObjectState, TObjOcafObjectIterator, TObjPersistence,
    TObjTModel, TObjTObject,
};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::HExtendedString,
};
use rustycad_tklcaf::{
    tdf::{
        TdfAttributeIterator, TdfChildIterator, TdfCopyLabel, TdfLabel, TdfRelocationTable,
        TdfTagSource,
    },
    tdoc_std::TDocStdDocument,
};

/// `copyTagSources`: copies the `TDF_TagSource` of the label, then of the sub-labels
/// that exist in the target (where the data were copied).
fn copy_tag_sources(source: &TdfLabel, target: &TdfLabel) -> Result<(), Failure> {
    let mut tag_source: Handle<TdfTagSource> = Handle::default();
    if source.find_attribute_as(&TdfTagSource::get_id(), &mut tag_source)? {
        let target_source = TdfTagSource::set(target)?;
        target_source.set_value(TdfTagSource::get(&tag_source))?;
    }
    let mut children = TdfChildIterator::new(target, false)?;
    while children.more() {
        let child = children.value();
        let source_child = source.find_child(child.tag(), false)?;
        if !source_child.is_null() {
            copy_tag_sources(&source_child, &child)?;
        }
        children.next();
    }
    Ok(())
}

/// The model of the document of `label` (its `TObj_TModel`), null when none.
fn target_model(label: &TdfLabel) -> Result<Handle<dyn TObjModel>, Failure> {
    let document = TDocStdDocument::get(label)?;
    let main = rustycad_tkcdf::live(&document, "TObj_Object::Clone")?.main()?;
    let mut attribute: Handle<TObjTModel> = Handle::default();
    if main.find_attribute_as(&TObjTModel::get_id(), &mut attribute)? {
        return Ok(attribute.model());
    }
    Ok(Handle::default())
}

/// The body of `TObj_Object::Clone`: the target model is the current one of the
/// persistence while the new object is created, named, filled and given children.
pub(crate) fn object_clone<T: TObjObject + ?Sized>(
    object: &T,
    target_label: &TdfLabel,
    relocation_table: &Handle<TdfRelocationTable>,
) -> Result<Handle<dyn TObjObject>, Failure> {
    let table = if relocation_table.is_null() {
        Handle::new(TdfRelocationTable::new(false))
    } else {
        relocation_table.clone()
    };
    let current_model = TObjAssistant::get_current_model();
    let model = target_model(target_label)?;
    if current_model != model {
        TObjAssistant::set_current_model(&model);
    }
    let cloned = clone_into(object, target_label, &table, relocation_table.is_null());
    if current_model != model {
        TObjAssistant::set_current_model(&current_model);
    }
    cloned
}

/// Creates the object of the same type on the target label and copies the name, the
/// data, the children, their tag sources and (for a top-level clone) the references.
fn clone_into<T: TObjObject + ?Sized>(
    object: &T,
    target_label: &TdfLabel,
    table: &Handle<TdfRelocationTable>,
    copy_references: bool,
) -> Result<Handle<dyn TObjObject>, Failure> {
    let type_name = object.dynamic_type().name().to_owned();
    let created = TObjPersistence::create_new_object(&type_name, target_label)?;
    let Some(new_object) = created.get() else {
        return Ok(created);
    };
    TObjTObject::set(target_label, &created)?;
    rustycad_tkcdf::live(table, "TObj_Object::Clone")?
        .set_label_relocation(&object.get_label(), target_label);
    let clone_name = object.get_name_for_clone(&created)?;
    if let Some(name) = clone_name.get().filter(|name| !name.string().is_empty()) {
        new_object.set_name(&Handle::new(HExtendedString::from_h_extended_string(name)))?;
    }
    object.copy_data(&created)?;
    let target_children = new_object.get_child_label()?;
    object.copy_children(&target_children, table)?;
    copy_tag_sources(&object.get_child_label()?, &target_children)?;
    if copy_references {
        object.copy_references(&created, table)?;
    }
    Ok(created)
}

/// The body of `TObj_Object::copyData`: copies the data label when the target is of
/// this type or a subtype.
pub(crate) fn object_copy_data<T: TObjObject + ?Sized>(
    object: &T,
    target: &Handle<dyn TObjObject>,
) -> Result<bool, Failure> {
    let target = rustycad_tkcdf::live(target, "TObj_Object::copyData")?;
    if !target.dynamic_type().is_subtype(object.dynamic_type()) {
        return Ok(false);
    }
    let source_label = object.get_data_label()?;
    let target_label = target.get_data_label()?;
    if source_label.is_null() || target_label.is_null() {
        return Ok(false);
    }
    let mut copier = TdfCopyLabel::with_labels(&source_label, &target_label);
    copier.perform()?;
    Ok(copier.is_done())
}

/// The child objects at any depth under the child label (children on sub-labels of the
/// child label are supported).
fn deep_children<T: TObjObject + ?Sized>(
    object: &T,
) -> Result<Vec<Handle<dyn TObjObject>>, Failure> {
    let iterator = TObjOcafObjectIterator::new(&object.get_child_label()?, None, true, false)?;
    let mut children = Vec::new();
    while super::TObjObjectIterator::more(&*iterator) {
        children.push(super::TObjObjectIterator::value(&*iterator));
        super::TObjObjectIterator::next(&*iterator)?;
    }
    Ok(children)
}

/// The body of `TObj_Object::CopyChildren`: each child is cloned on the label of the
/// same tag path under the target label.
pub(crate) fn object_copy_children<T: TObjObject + ?Sized>(
    object: &T,
    target_label: &TdfLabel,
    relocation_table: &Handle<TdfRelocationTable>,
) -> Result<(), Failure> {
    let source_child_label = object.get_child_label()?;
    for child in deep_children(object)? {
        let Some(child) = child.get() else {
            continue;
        };
        let mut tags = Vec::new();
        let mut current = child.get_label();
        while !current.is_null() && current != source_child_label {
            tags.push(current.tag());
            current = current.father();
        }
        let mut child_label = target_label.clone();
        for tag in tags.iter().rev() {
            child_label = child_label.find_child(*tag, true)?;
        }
        child.clone_object(&child_label, relocation_table)?;
    }
    Ok(())
}

/// The body of `TObj_Object::CopyReferences`: the children's references (through the
/// relocation table), then the own references.
pub(crate) fn object_copy_references<T: TObjObject + ?Sized>(
    object: &T,
    target: &Handle<dyn TObjObject>,
    relocation_table: &Handle<TdfRelocationTable>,
) -> Result<(), Failure> {
    let table = rustycad_tkcdf::live(relocation_table, "TObj_Object::CopyReferences")?;
    for source_child in deep_children(object)? {
        let source = rustycad_tkcdf::live(&source_child, "TObj_Object::CopyReferences")?;
        let mut destination_label = TdfLabel::default();
        if !table.has_label_relocation(&source.get_label(), &mut destination_label) {
            continue;
        }
        let mut destination = Handle::default();
        if !TObjObjectBase::get_obj(&destination_label, &mut destination, false)? {
            continue;
        }
        let same_type = destination
            .dynamic_type()
            .is_some_and(|kind| std::ptr::eq(kind, source.dynamic_type()));
        if !destination.is_alive()? || !same_type {
            continue; // should not be with relocation table
        }
        source.copy_references(&destination, relocation_table)?;
    }
    let target = rustycad_tkcdf::live(target, "TObj_Object::CopyReferences")?;
    target.get_reference_label()?.forget_all_attributes(true)?;
    copy_label_references(
        &object.get_reference_label()?,
        &target.get_reference_label()?,
        relocation_table,
    )
}

/// `copyReferences(theSourceLabel, theTargetLabel, theRelocTable)` (protected): a new
/// empty attribute pasted from each source attribute, recursively on the sub-labels.
pub fn copy_label_references(
    source: &TdfLabel,
    target: &TdfLabel,
    relocation_table: &Handle<TdfRelocationTable>,
) -> Result<(), Failure> {
    let mut attributes = TdfAttributeIterator::new(source, true)?;
    while attributes.more() {
        let attribute = attributes.value();
        let live = rustycad_tkcdf::live(&attribute, "TObj_Object::copyReferences")?;
        let copy = live.new_empty();
        target.add_attribute(&copy, true)?;
        live.paste(&copy, relocation_table)?;
        attributes.next();
    }
    let mut children = TdfChildIterator::new(source, false)?;
    while children.more() {
        let child = children.value();
        let target_child = target.find_child(child.tag(), true)?;
        copy_label_references(&child, &target_child, relocation_table)?;
        children.next();
    }
    Ok(())
}
