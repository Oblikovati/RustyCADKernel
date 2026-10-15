// Rust port of ApplicationFramework/TKTObj/TObj/TObj_TObject.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_TObject`: the OCAF attribute that stores an object of a TObj model on its label.
use super::{TObjDeletingMode, TObjObject};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
};
use rustycad_tklcaf::{
    guid_from_text,
    tdata_std::support::{cast_attribute, find_or_add},
    tdf::{
        TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeState, TdfChildIterator,
        TdfLabel, TdfRelocationTable,
    },
};

/// `TObj_TObject::GetID()`.
const TOBJECT_ID: Guid = guid_from_text("bbdab6a7-dca9-11d4-ba37-0060b0ee18ea");

/// `TObj_TObject`; `TObjTObject::set(&label, &object)`.
#[derive(Default)]
pub struct TObjTObject {
    transient: StandardTransient,
    base: TdfAttributeBase,
    element: SharedState<Handle<dyn TObjObject>>,
}
impl_transient!(TObjTObject, "TObj_TObject", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TObjTObject {
    /// `GetID`.
    pub fn get_id() -> Guid {
        TOBJECT_ID
    }
    /// `TObj_TObject()`: no object.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(theLabel, theElem)` (static): the attribute of the label, added when absent,
    /// set to the object.
    pub fn set(
        label: &TdfLabel,
        element: &Handle<dyn TObjObject>,
    ) -> Result<Handle<TObjTObject>, Failure> {
        let attribute = find_or_add(label, &TOBJECT_ID, false, Self::new)?;
        rustycad_tkcdf::live(&attribute, "TObj_TObject::Set")?.set_object(element)?;
        Ok(attribute)
    }
    /// `Set(theElem)`: backed up, then set.
    pub fn set_object(&self, element: &Handle<dyn TObjObject>) -> Result<(), Failure> {
        self.backup()?;
        *self.element.write() = element.clone();
        Ok(())
    }
    /// `Get`: the stored object.
    pub fn get_object(&self) -> Handle<dyn TObjObject> {
        self.element.read().clone()
    }
}

/// Forgets the attributes of every direct sub-label of the object's label.
fn forget_sub_labels(label: &TdfLabel) -> Result<(), Failure> {
    let mut children = TdfChildIterator::new(label, false)?;
    while children.more() {
        let child = children.value();
        if !child.is_null() {
            child.forget_all_attributes(true)?;
        }
        children.next();
    }
    Ok(())
}

impl TdfAttribute for TObjTObject {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        TOBJECT_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TObjTObject>(with, "TObj_TObject::Restore")?;
        *self.element.write() = other.get_object();
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
        let target = cast_attribute::<TObjTObject>(into, "TObj_TObject::Paste")?;
        target.set_object(&self.get_object())
    }
    /// `BeforeForget`: the object dies (its label becomes null) after its sub-labels are
    /// cleared and its back references removed.
    fn before_forget(&self) -> Result<(), Failure> {
        let element = self.get_object();
        let Some(object) = element.get() else {
            return Ok(());
        };
        let label = object.object_base().label();
        if !label.is_null() {
            forget_sub_labels(&label)?;
        }
        object.remove_back_references(TObjDeletingMode::Forced)?;
        object.object_base().set_label(&TdfLabel::default());
        Ok(())
    }
    /// `AfterUndo`: the object rises again when its label holds this attribute's object.
    fn after_undo(
        &self,
        attribute_delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        let element = self.get_object();
        let Some(object) = element.get() else {
            return Ok(true);
        };
        let label = rustycad_tkcdf::live(attribute_delta, "TObj_TObject::AfterUndo")?.label();
        let mut attribute: Handle<TObjTObject> = Handle::default();
        let holds_me = !label.is_null()
            && label.find_attribute_as(&TOBJECT_ID, &mut attribute)?
            && attribute.get_object() == element;
        let alive = if holds_me { label } else { TdfLabel::default() };
        object.object_base().set_label(&alive);
        Ok(true)
    }
}
