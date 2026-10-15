// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Model.cxx (FindObject, initNewModel,
// updateBackReferences) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native bodies of the `TObj_Model` members that find objects by name and set up a
//! new or loaded model.
use super::{TObjLabelIterator, TObjModel, TObjModelState, TObjObject, TObjTNameContainer};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::HExtendedString,
};

/// The body of `TObj_Model::FindObject`: the object of a registered name.
pub(crate) fn find_object<M: TObjModel + ?Sized>(
    model: &M,
    name: &Handle<HExtendedString>,
    dictionary: &Handle<TObjTNameContainer>,
) -> Result<Handle<dyn TObjObject>, Failure> {
    let dictionary = model.dictionary_or_own(dictionary)?;
    let mut result = Handle::default();
    if model.is_registered_name(name, &dictionary)? {
        let label =
            rustycad_tkcdf::live(&dictionary, "TObj_Model::FindObject")?.find_label(name)?;
        <dyn TObjObject>::get_obj(&label, &mut result, false)?;
    }
    Ok(result)
}

/// The body of `TObj_Model::initNewModel`.
pub(crate) fn init_new_model<M: TObjModel + ?Sized>(
    model: &M,
    is_new: bool,
) -> Result<bool, Failure> {
    TObjTNameContainer::set(&model.get_label())?;
    if is_new {
        return Ok(true);
    }
    let objects = model.get_objects()?;
    let objects = rustycad_tkcdf::live(&objects, "TObj_Model::initNewModel")?;
    while objects.more() {
        if let Some(object) = objects.value().get() {
            object.after_retrieval()?;
        }
        objects.next()?;
    }
    model.update_back_references(&model.get_main_partition()?.cast())?;
    if model.is_to_check() {
        let checker = model.get_checker()?;
        let checker = rustycad_tkcdf::live(&checker, "TObj_Model::initNewModel")?;
        checker.perform()?;
        checker.send_messages()?;
        model.set_modified(true)?;
    }
    Ok(true)
}

/// The referred objects of `object`, when its references come as a label iterator.
fn referred_objects(
    object: &dyn TObjObject,
) -> Result<Option<Vec<Handle<dyn TObjObject>>>, Failure> {
    let iterator: Handle<dyn TObjLabelIterator> = object.get_references(None)?.cast();
    let Some(iterator) = iterator.get() else {
        return Ok(None);
    };
    let mut referred = Vec::new();
    while iterator.more() {
        referred.push(iterator.value());
        iterator.next()?;
    }
    Ok(Some(referred))
}

/// The body of `TObj_Model::updateBackReferences`: recursive on the children; the old
/// back references to the object are removed first (LH3D15722), then created again.
pub(crate) fn update_back_references<M: TObjModel + ?Sized>(
    model: &M,
    object: &Handle<dyn TObjObject>,
) -> Result<(), Failure> {
    let Some(live) = object.get() else {
        return Ok(());
    };
    let children = live.get_children(None)?;
    let children = rustycad_tkcdf::live(&children, "TObj_Model::updateBackReferences")?;
    while children.more() {
        model.update_back_references(&children.value())?;
        children.next()?;
    }
    let Some(referred) = referred_objects(live)? else {
        return Ok(()); // to avoid exception
    };
    for target in referred.iter().filter_map(Handle::get) {
        target.remove_back_reference(object, false);
    }
    let Some(referred) = referred_objects(live)? else {
        return Ok(());
    };
    for target in referred.iter().filter_map(Handle::get) {
        target.add_back_reference(object);
    }
    Ok(())
}
