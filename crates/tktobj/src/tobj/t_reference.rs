// Rust port of ApplicationFramework/TKTObj/TObj/TObj_TReference.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_TReference`: the OCAF attribute of a reference from one TObj object (the
//! master) to another; each reference registers a back reference in the referred object.
use super::TObjObjectBase;
use super::{TObjObject, TObjObjectState, TObjTObject};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, RuntimeTyped, SharedState, StandardTransient},
};
use rustycad_tklcaf::{
    guid_from_text,
    tdata_std::support::cast_attribute,
    tdf::{
        TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeState, TdfDeltaOnAddition,
        TdfDeltaOnRemoval, TdfLabel, TdfRelocationTable,
    },
};

/// `TObj_TReference::GetID()`.
const REFERENCE_ID: Guid = guid_from_text("3bbefb44-e618-11d4-ba38-0060b0ee18ea");

/// The referred label (`myLabel`) and the master label (`myMasterLabel`).
#[derive(Clone, Default)]
struct ReferenceLabels {
    referred: TdfLabel,
    master: TdfLabel,
}

/// The master object and the referred object of a reference.
type MasterAndReferred = (Handle<dyn TObjObject>, Handle<dyn TObjObject>);

/// `TObj_TReference`; `TObjTReference::set(&label, &referred, &master)`.
#[derive(Default)]
pub struct TObjTReference {
    transient: StandardTransient,
    base: TdfAttributeBase,
    labels: SharedState<ReferenceLabels>,
}
impl_transient!(TObjTReference, "TObj_TReference", dyn TdfAttribute, transient,
    interfaces: [dyn TdfAttribute]);

/// The object stored on `label` by a `TObj_TObject`, null when none.
fn object_on(label: &TdfLabel) -> Result<Handle<dyn TObjObject>, Failure> {
    let mut attribute: Handle<TObjTObject> = Handle::default();
    if label.is_null() || !label.find_attribute_as(&TObjTObject::get_id(), &mut attribute)? {
        return Ok(Handle::default());
    }
    Ok(attribute.get_object())
}

impl TObjTReference {
    /// `GetID`.
    pub fn get_id() -> Guid {
        REFERENCE_ID
    }
    /// `TObj_TReference()`: null labels.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(theLabel, theObject, theMaster)` (static): the reference of the label (the old
    /// referred object loses its back reference) pointing to `object`, which gets a back
    /// reference to `master`.
    pub fn set(
        label: &TdfLabel,
        object: &Handle<dyn TObjObject>,
        master: &Handle<dyn TObjObject>,
    ) -> Result<Handle<TObjTReference>, Failure> {
        let mut reference: Handle<TObjTReference> = Handle::default();
        if !label.find_attribute_as(&REFERENCE_ID, &mut reference)? {
            reference = Handle::new(TObjTReference::new());
            label.add_attribute(&reference.cast(), true)?;
        } else if let Some(old) = reference.get_object()?.get() {
            old.remove_back_reference(master, true);
        }
        let master_label = rustycad_tkcdf::live(master, "TObj_TReference::Set")?.get_label();
        reference.set_object(object, &master_label)?;
        if let Some(referred) = object.get() {
            referred.add_back_reference(master);
        }
        Ok(reference)
    }
    /// `Set(theObject, theMasterLabel)`: backed up, then the object's label and the master.
    pub fn set_object(
        &self,
        object: &Handle<dyn TObjObject>,
        master_label: &TdfLabel,
    ) -> Result<(), Failure> {
        let referred = object
            .get()
            .map(|object| object.get_label())
            .unwrap_or_default();
        self.set_label(&referred, master_label)
    }
    /// `Set(theLabel, theMasterLabel)`: backed up, then both labels (for persistence).
    pub fn set_label(&self, label: &TdfLabel, master_label: &TdfLabel) -> Result<(), Failure> {
        self.backup()?;
        *self.labels.write() = ReferenceLabels {
            referred: label.clone(),
            master: master_label.clone(),
        };
        Ok(())
    }
    /// `Get`: the object on the referred label, null when none.
    pub fn get_object(&self) -> Result<Handle<dyn TObjObject>, Failure> {
        object_on(&self.get_label())
    }
    /// `GetMasterLabel`.
    pub fn get_master_label(&self) -> TdfLabel {
        self.labels.read().master.clone()
    }
    /// `GetLabel`: the referred label.
    pub fn get_label(&self) -> TdfLabel {
        self.labels.read().referred.clone()
    }
    /// The master object (`myMasterLabel`'s `TObj_TObject`) and the referred object, when
    /// the master label holds an attribute; the shared prologue of the callbacks.
    fn master_and_referred(&self) -> Result<Option<MasterAndReferred>, Failure> {
        let master_label = self.get_master_label();
        let mut master: Handle<TObjTObject> = Handle::default();
        if master_label.is_null()
            || !master_label.find_attribute_as(&TObjTObject::get_id(), &mut master)?
        {
            return Ok(None);
        }
        Ok(Some((master.get_object(), self.get_object()?)))
    }
    /// Adds a back reference from the referred object to the master, when both exist.
    fn restore_back_reference(&self) -> Result<(), Failure> {
        if let Some((master, referred)) = self.master_and_referred()?
            && let Some(referred) = referred.get()
        {
            referred.add_back_reference(&master);
        }
        Ok(())
    }
}

impl TdfAttribute for TObjTReference {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        REFERENCE_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TObjTReference>(with, "TObj_TReference::Restore")?;
        *self.labels.write() = other.labels.read().clone();
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the relocated referred object, with the master found above the target.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TObjTReference>(into, "TObj_TReference::Paste")?;
        let referred = self.get_label();
        if referred.is_null() {
            target.labels.write().referred = TdfLabel::default();
            return Ok(());
        }
        let table = rustycad_tkcdf::live(relocation_table, "TObj_TReference::Paste")?;
        let mut relocated = referred.clone();
        if !table.has_label_relocation(&referred, &mut relocated) {
            relocated = referred;
        }
        paste_into(&target, &object_on(&relocated)?)
    }
    /// `BeforeForget`: the referred object loses its back reference to the master.
    fn before_forget(&self) -> Result<(), Failure> {
        let Some((master, referred)) = self.master_and_referred()? else {
            return Ok(());
        };
        let Some(referred) = referred.get() else {
            return Ok(());
        };
        rustycad_tkcdf::live(&master, "TObj_TReference::BeforeForget")?
            .before_forget_reference(&self.get_label())?;
        referred.remove_back_reference(&master, true);
        Ok(())
    }
    /// `BeforeUndo`: undoing an addition removes the back reference.
    fn before_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if !delta.is_kind(TdfDeltaOnAddition::static_type()) {
            return Ok(true);
        }
        if let Some((master, referred)) = self.master_and_referred()?
            && let Some(referred) = referred.get()
        {
            referred.remove_back_reference(&master, true);
        }
        Ok(true)
    }
    /// `AfterUndo`: undoing a removal restores the back reference.
    fn after_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if delta.is_kind(<dyn TdfDeltaOnRemoval>::static_type()) {
            self.restore_back_reference()?;
        }
        Ok(true)
    }
    /// `AfterResume`: restores the back reference.
    fn after_resume(&self) -> Result<(), Failure> {
        self.restore_back_reference()
    }
    /// `AfterRetrieval`: restores the back reference; false when the master label holds
    /// no object attribute.
    fn after_retrieval(&self, _force_it: bool) -> Result<bool, Failure> {
        if self.get_master_label().is_null() {
            return Ok(true);
        }
        if self.master_and_referred()?.is_none() {
            return Ok(false);
        }
        self.restore_back_reference()?;
        Ok(true)
    }
}

/// The tail of `Paste`: the master is the object above the target attribute's label;
/// nothing is set when it is missing.
fn paste_into(target: &TObjTReference, referred: &Handle<dyn TObjObject>) -> Result<(), Failure> {
    let mut master = Handle::default();
    TObjObjectBase::get_obj(&target.label(), &mut master, true)?;
    let master_label = master
        .get()
        .map(|master| master.get_label())
        .unwrap_or_default();
    let mut master_attribute: Handle<TObjTObject> = Handle::default();
    if master_label.is_null()
        || !master_label.find_attribute_as(&TObjTObject::get_id(), &mut master_attribute)?
    {
        return Ok(());
    }
    target.set_object(referred, &master_label)?;
    if let Some(referred) = referred.get() {
        referred.add_back_reference(&master_attribute.get_object());
    }
    Ok(())
}
