// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.cxx (RemoveBackReference,
// CanDetach, Detach, RemoveBackReferences) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native bodies of the `TObj_Object` members that delete objects and unlink the
//! objects that refer to them.
use super::{TObjDeletingMode, TObjObject, TObjObjectState};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdf::TdfData;

/// The body of `TObj_Object::RemoveBackReference`: removes the first (or every) entry of
/// the object; an emptied sequence is released.
pub(crate) fn object_remove_back_reference<T: TObjObject + ?Sized>(
    object: &T,
    referring: &Handle<dyn TObjObject>,
    single_only: bool,
) {
    let base = object.object_base();
    let sequence = base.back_references();
    let Some(entries) = sequence.get() else {
        return;
    };
    let is_empty = entries.change_sequence(|items| {
        let mut index = 1;
        while index <= items.len() {
            if items.value(index).is_ok_and(|item| item == referring) {
                let _ = items.remove(index);
                if single_only {
                    break;
                }
                continue;
            }
            index += 1;
        }
        items.is_empty()
    });
    if is_empty {
        base.set_back_references(Handle::default());
    }
}

/// The referring objects of `object` that are alive.
fn live_back_references<T: TObjObject + ?Sized>(
    object: &T,
) -> Result<Vec<Handle<dyn TObjObject>>, Failure> {
    let references = object.get_back_references(None);
    let mut objects = Vec::new();
    let Some(references) = references.get() else {
        return Ok(objects);
    };
    while references.more() {
        objects.push(references.value());
        references.next()?;
    }
    Ok(objects)
}

/// The body of `TObj_Object::CanDetach`.
pub(crate) fn object_can_detach<T: TObjObject + ?Sized>(
    object: &T,
    mode: TObjDeletingMode,
) -> Result<bool, Failure> {
    if !object.is_alive()? {
        return Ok(false);
    }
    let references = live_back_references(object)?;
    // A free object can be deleted in any mode.
    if references.is_empty() || mode == TObjDeletingMode::Forced {
        return Ok(true);
    }
    if mode == TObjDeletingMode::FreeOnly {
        return Ok(false);
    }
    let me = object.object_base().this()?;
    for referring in &references {
        if !rustycad_tkcdf::live(referring, "TObj_Object::CanDetach")?.can_remove_reference(&me)? {
            return Ok(false); // one of objects could not be unlinked
        }
    }
    Ok(true)
}

/// The body of `TObj_Object::Detach`: unlinks the referring objects, detaches the
/// children, clears the data and the name, then forgets the object.
pub(crate) fn object_detach<T: TObjObject + ?Sized>(
    object: &T,
    mode: TObjDeletingMode,
) -> Result<bool, Failure> {
    if !object.is_alive()? || !object.remove_back_references(mode)? {
        return Ok(false);
    }
    let old_name = object.get_name()?;
    let children = object.get_children(None)?;
    let children = rustycad_tkcdf::live(&children, "TObj_Object::Detach")?;
    while children.more() {
        rustycad_tkcdf::live(&children.value(), "TObj_Object::Detach")?.detach(mode)?;
        children.next()?;
    }
    object.get_reference_label()?.forget_all_attributes(true)?;
    object.clear_back_references();
    object.get_data_label()?.forget_all_attributes(true)?;
    unregister_own_name(object, &old_name)?;
    object.get_label().forget_all_attributes(true)?;
    Ok(true)
}

/// Unregisters the name only when the dictionary maps it to this object's label.
fn unregister_own_name<T: TObjObject + ?Sized>(
    object: &T,
    old_name: &Handle<rustycad_tkernel::tcollection::HExtendedString>,
) -> Result<(), Failure> {
    if old_name.is_null() {
        return Ok(());
    }
    let dictionary = object.get_dictionary()?;
    let Some(dictionary) = dictionary.get() else {
        return Ok(());
    };
    if !dictionary.is_registered(old_name) {
        return Ok(());
    }
    let registered = dictionary.find_label(old_name)?;
    if !registered.is_null() && registered == object.get_label() {
        dictionary.remove_name(old_name)?;
    }
    Ok(())
}

/// The body of `TObj_Object::RemoveBackReferences`: the referring containers drop their
/// reference, the strong referrers are detached (not in `KeepDepending` mode).
pub(crate) fn object_remove_back_references<T: TObjObject + ?Sized>(
    object: &T,
    mode: TObjDeletingMode,
) -> Result<bool, Failure> {
    let references = live_back_references(object)?;
    if references.is_empty() {
        return Ok(true); // a free object can be deleted in any mode
    }
    if mode == TObjDeletingMode::FreeOnly {
        return Ok(false);
    }
    let me = object.object_base().this()?;
    let (containers, strongs) = sort_referring(&references, &me)?;
    if mode == TObjDeletingMode::KeepDepending && !strongs.is_empty() {
        return Ok(false);
    }
    let own_data = object.get_label().data();
    for container in &containers {
        with_modification_allowed(container, &own_data, |referring| {
            referring.remove_reference(&me)
        })?;
    }
    for strong in &strongs {
        with_modification_allowed(strong, &own_data, |referring| {
            referring.detach(mode).map(|_| ())
        })?;
    }
    Ok(true)
}

/// Splits the alive referring objects into those that can drop their reference
/// (containers) and those that cannot (strong relations).
type ReferringSplit = (Vec<Handle<dyn TObjObject>>, Vec<Handle<dyn TObjObject>>);
fn sort_referring(
    references: &[Handle<dyn TObjObject>],
    me: &Handle<dyn TObjObject>,
) -> Result<ReferringSplit, Failure> {
    let (mut containers, mut strongs) = (Vec::new(), Vec::new());
    for referring in references {
        let Some(object) = referring.get() else {
            continue;
        };
        if !object.is_alive()? {
            continue;
        }
        if object.can_remove_reference(me)? {
            containers.push(referring.clone());
        } else {
            strongs.push(referring.clone());
        }
    }
    Ok((containers, strongs))
}

/// Runs `edit` on a referring object, allowing modifications of its framework while
/// it is not the own one (an object of another document), as natively.
fn with_modification_allowed(
    referring: &Handle<dyn TObjObject>,
    own_data: &Handle<TdfData>,
    edit: impl FnOnce(&dyn TObjObject) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let Some(object) = referring.get() else {
        return Ok(());
    };
    let label = object.get_label();
    if label.is_null() {
        return Ok(()); // undead object on dead label
    }
    let data = label.data();
    let data = rustycad_tkcdf::live(&data, "TObj_Object::RemoveBackReferences")?;
    let modification_mode = data.is_modification_allowed();
    let foreign = label.data() != *own_data;
    if foreign {
        data.allow_modification(true);
    }
    edit(object)?;
    if foreign {
        data.allow_modification(modification_mode);
    }
    Ok(())
}
