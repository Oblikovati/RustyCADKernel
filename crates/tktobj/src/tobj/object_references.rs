// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.cxx (HasReference,
// ReplaceReference, GetBadReference, RelocateReferences) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native bodies of the `TObj_Object` members that read and rewrite references.
use super::TObjObjectBase;
use super::{TObjLabelIterator, TObjObject, TObjObjectState, TObjTReference};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdf::{TdfChildIdIterator, TdfLabel, TdfTool};

/// The body of `TObj_Object::HasReference`.
pub(crate) fn object_has_reference<T: TObjObject + ?Sized>(
    object: &T,
    referred: &Handle<dyn TObjObject>,
) -> Result<bool, Failure> {
    let Some(kind) = referred.dynamic_type() else {
        return Ok(false);
    };
    let iterator = object.get_references(Some(kind))?;
    let Some(iterator) = iterator.get() else {
        return Ok(false);
    };
    while iterator.more() {
        if iterator.value() == *referred {
            return Ok(true);
        }
        iterator.next()?;
    }
    Ok(false)
}

/// The body of `TObj_Object::ReplaceReference`: the first reference to the old object is
/// forgotten (null new object) or pointed to the new one.
pub(crate) fn object_replace_reference<T: TObjObject + ?Sized>(
    object: &T,
    old_object: &Handle<dyn TObjObject>,
    new_object: &Handle<dyn TObjObject>,
) -> Result<(), Failure> {
    let iterator: Handle<dyn TObjLabelIterator> = object.get_references(None)?.cast();
    let Some(iterator) = iterator.get() else {
        return Ok(());
    };
    while iterator.more() {
        if iterator.value() != *old_object {
            iterator.next()?;
            continue;
        }
        let label = iterator.label_value();
        if new_object.is_null() {
            return label.forget_all_attributes(true);
        }
        TObjTReference::set(&label, new_object, &object.object_base().this()?)?;
        return Ok(());
    }
    Ok(())
}

/// The `TObj_TReference` attributes under the reference label, at all levels.
fn reference_attributes<T: TObjObject + ?Sized>(
    object: &T,
) -> Result<Vec<Handle<TObjTReference>>, Failure> {
    let mut iterator = TdfChildIdIterator::new(
        &object.get_reference_label()?,
        &TObjTReference::get_id(),
        true,
    )?;
    let mut references = Vec::new();
    while iterator.more() {
        references.push(iterator.value().cast());
        iterator.next()?;
    }
    Ok(references)
}

/// The body of `TObj_Object::GetBadReference`: the first referred label of the root's
/// framework that is outside the root.
pub(crate) fn object_get_bad_reference<T: TObjObject + ?Sized>(
    object: &T,
    root: &TdfLabel,
    bad_reference: &mut TdfLabel,
) -> Result<bool, Failure> {
    for reference in reference_attributes(object)? {
        let label = rustycad_tkcdf::live(&reference, "TObj_Object::GetBadReference")?.get_label();
        if label.data() == root.data() && !label.is_descendant(root) {
            *bad_reference = label;
            return Ok(true);
        }
    }
    Ok(false)
}

/// The body of `TObj_Object::RelocateReferences`: each reference into `from_root` points
/// to the equivalent label under `to_root`; false when that label holds no object.
pub(crate) fn object_relocate_references<T: TObjObject + ?Sized>(
    object: &T,
    from_root: &TdfLabel,
    to_root: &TdfLabel,
    update_back_references: bool,
) -> Result<bool, Failure> {
    for reference in reference_attributes(object)? {
        let reference = rustycad_tkcdf::live(&reference, "TObj_Object::RelocateReferences")?;
        let label = reference.get_label();
        if label.data() != from_root.data() || label.is_descendant(to_root) {
            continue; // need not to relocate
        }
        let mut new_label = TdfLabel::default();
        TdfTool::relocate_label(&label, from_root, to_root, &mut new_label, false)?;
        let mut new_object = Handle::default();
        if new_label.is_null() || !TObjObjectBase::get_obj(&new_label, &mut new_object, false)? {
            return Ok(false);
        }
        if update_back_references {
            let me = object.object_base().this()?;
            new_object.add_back_reference(&me);
            let old_object = reference.get_object()?;
            if let Some(old_object) = old_object.get() {
                old_object.remove_back_reference(&me, true);
            }
        }
        reference.set_label(&new_label, &reference.get_master_label())?;
    }
    Ok(true)
}
