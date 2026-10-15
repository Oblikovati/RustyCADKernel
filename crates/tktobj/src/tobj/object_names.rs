// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.cxx (GetModel, GetDictionary,
// GetName, SetName, AfterRetrieval) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native bodies of the `TObj_Object` members on the model, the dictionary and names.
use super::{
    TObjModel, TObjModelState, TObjObject, TObjObjectState, TObjTModel, TObjTNameContainer,
};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::HExtendedString,
};
use rustycad_tklcaf::{
    tdata_std::{TDataStdGenericExtString, TDataStdName},
    tdf::TdfLabel,
    tdoc_std::{TDocStdDocument, TDocStdOwner},
};

/// The body of `TObj_Object::GetModel`: the document is found through the
/// `TDocStd_Owner` of the root, the model through the `TObj_TModel` of its main label.
pub(crate) fn model_of_label(label: &TdfLabel) -> Result<Handle<dyn TObjModel>, Failure> {
    let document = document_of_label(label)?;
    let Some(document) = document.get() else {
        return Ok(Handle::default());
    };
    let main = document.main()?;
    let mut attribute: Handle<TObjTModel> = Handle::default();
    if !main.is_null() && main.find_attribute_as(&TObjTModel::get_id(), &mut attribute)? {
        return Ok(attribute.model());
    }
    Ok(Handle::default())
}

/// The document of the label's framework through the owner attribute of the root; null
/// when the label, its data or the owner is missing.
fn document_of_label(label: &TdfLabel) -> Result<Handle<TDocStdDocument>, Failure> {
    let data = label.data();
    let Some(data) = data.get() else {
        return Ok(Handle::default());
    };
    let root = data.root();
    let mut owner: Handle<TDocStdOwner> = Handle::default();
    if root.is_null() || !root.find_attribute_as(&TDocStdOwner::get_id(), &mut owner)? {
        return Ok(Handle::default());
    }
    Ok(owner.get_document())
}

/// The body of `TObj_Object::GetDictionary`: the model's dictionary, null without model.
pub(crate) fn object_get_dictionary<T: TObjObject + ?Sized>(
    object: &T,
) -> Result<Handle<TObjTNameContainer>, Failure> {
    let model = object.get_model()?;
    match model.get() {
        Some(model) => model.get_dictionary(),
        None => Ok(Handle::default()),
    }
}

/// The body of `TObj_Object::GetName`: the `TDataStd_Name`, else an empty string.
pub(crate) fn object_get_name<T: TObjObject + ?Sized>(
    object: &T,
) -> Result<Handle<HExtendedString>, Failure> {
    let mut name: Handle<TDataStdName> = Handle::default();
    if object
        .get_label()
        .find_attribute_as(&TDataStdName::get_id(), &mut name)?
    {
        return Ok(Handle::new(HExtendedString::from(
            TDataStdGenericExtString::get(&*name),
        )));
    }
    Ok(Handle::new(HExtendedString::from_c_string(b"")))
}

/// The body of `TObj_Object::SetName`: false when the name is registered already, else
/// the old name is unregistered and the new one registered and stored.
pub(crate) fn object_set_name<T: TObjObject + ?Sized>(
    object: &T,
    name: &Handle<HExtendedString>,
) -> Result<bool, Failure> {
    let old_name = object.get_name()?;
    if same_name(&old_name, name)? {
        return Ok(true);
    }
    let dictionary = object.get_dictionary()?;
    let model = object.get_model()?;
    let model = rustycad_tkcdf::live(&model, "TObj_Object::SetName")?;
    if model.is_registered_name(name, &dictionary)? {
        return Ok(false);
    }
    if !old_name.is_null() {
        model.unregister_name(&old_name, &dictionary)?;
    }
    let Some(text) = name.get() else {
        object
            .get_label()
            .forget_attribute_id(&TDataStdName::get_id())?;
        return Ok(true);
    };
    model.register_name(name, &object.get_label(), &dictionary)?;
    TDataStdName::set(&object.get_label(), &text.string())?;
    Ok(true)
}

/// `!anOldName.IsNull() && theName->String().IsEqual(anOldName->String())`; a null new
/// name is `Standard_NullObject` there, as the native dereference.
pub(crate) fn same_name(
    old_name: &Handle<HExtendedString>,
    name: &Handle<HExtendedString>,
) -> Result<bool, Failure> {
    let Some(old_name) = old_name.get() else {
        return Ok(false);
    };
    let name = rustycad_tkcdf::live(name, "TObj_Object::SetName")?;
    Ok(name.string().is_equal(&*old_name.string()))
}

/// The body of `TObj_Object::AfterRetrieval`: registers the name in the model.
pub(crate) fn object_after_retrieval<T: TObjObject + ?Sized>(object: &T) -> Result<(), Failure> {
    let model = object.get_model()?;
    let Some(model) = model.get() else {
        return Ok(());
    };
    model.register_name(
        &object.get_name()?,
        &object.get_label(),
        &object.get_dictionary()?,
    )
}
