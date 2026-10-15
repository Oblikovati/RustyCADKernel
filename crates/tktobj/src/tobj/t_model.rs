// Rust port of ApplicationFramework/TKTObj/TObj/TObj_TModel.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_TModel`: the OCAF attribute that stores a TObj model on the main label.
use super::TObjModel;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
};
use rustycad_tklcaf::{
    guid_from_text,
    tdata_std::support::cast_attribute,
    tdf::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfRelocationTable},
};

/// `TObj_TModel::GetID()`.
const TMODEL_ID: Guid = guid_from_text("bbdab6a6-dca9-11d4-ba37-0060b0ee18ea");

/// `TObj_TModel`; `label.add_attribute(&Handle::new(TObjTModel::new()).cast(), true)`.
#[derive(Default)]
pub struct TObjTModel {
    transient: StandardTransient,
    base: TdfAttributeBase,
    model: SharedState<Handle<dyn TObjModel>>,
}
impl_transient!(TObjTModel, "TObj_TModel", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TObjTModel {
    /// `GetID`.
    pub fn get_id() -> Guid {
        TMODEL_ID
    }
    /// `TObj_TModel()`: no model.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(theModel)`: backed up, then set.
    pub fn set(&self, model: &Handle<dyn TObjModel>) -> Result<(), Failure> {
        self.backup()?;
        *self.model.write() = model.clone();
        Ok(())
    }
    /// `Model`: the stored model.
    pub fn model(&self) -> Handle<dyn TObjModel> {
        self.model.read().clone()
    }
}

impl TdfAttribute for TObjTModel {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        TMODEL_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TObjTModel>(with, "TObj_TModel::Restore")?;
        *self.model.write() = other.model();
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TObjTModel>(into, "TObj_TModel::Paste")?;
        target.set(&self.model())
    }
}
