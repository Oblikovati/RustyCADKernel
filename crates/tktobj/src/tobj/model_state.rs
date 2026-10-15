// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Model.hxx and .cxx (non-virtual
// members and statics) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-virtual members of `TObj_Model`: messenger, document, transactions, names,
//! partitions and the format version.
use super::TObjModelBase;
use super::TObjStandardPartition;
use super::{
    TObjHiddenPartition, TObjModel, TObjModelDataTag, TObjObject, TObjObjectState, TObjPartition,
    TObjTModel, TObjTNameContainer, TObjTObject,
};
use rustycad_tkernel::{
    message::Messenger,
    standard::{Failure, Handle},
    tcollection::{ExtendedString, HExtendedString},
};
use rustycad_tklcaf::{
    tdata_std::TDataStdInteger,
    tdf::TdfLabel,
    tdoc_std::{TDocStdDocument, TDocStdOwner},
};
use std::sync::Arc;

/// `TObj_Model`'s non-virtual members, for every [`TObjModel`].
pub trait TObjModelState: TObjModel {
    /// `SetMessenger(theMsgr)`.
    fn set_messenger(&self, messenger: Arc<Messenger>) {
        self.model_base().set_messenger(messenger);
    }
    /// `Messenger`: the application's messenger unless another was set.
    fn messenger(&self) -> Arc<Messenger> {
        self.model_base().messenger()
    }
    /// `Save`: saves to the document's path; true when it has none.
    fn save(&self) -> Result<bool, Failure> {
        let document = TDocStdDocument::get(&self.get_label())?;
        let Some(document) = document.get() else {
            return Ok(false);
        };
        let path = document.get_path()?;
        if path.is_empty() {
            return Ok(true);
        }
        self.save_as(&path)
    }
    /// `CloseDocument(theDoc)`: aborts an open command, forgets every attribute and closes
    /// the document in the application.
    fn close_document(&self, document: &Handle<TDocStdDocument>) -> Result<(), Failure> {
        let application = self.get_application()?;
        super::model_io::close_document_with(document, &application)
    }
    /// `GetLabel`: the main label of the model (null when closed).
    fn get_label(&self) -> TdfLabel {
        self.model_base().label()
    }
    /// `SetLabel(theLabel)`: used by persistence.
    fn set_label(&self, label: &TdfLabel) {
        self.model_base().set_label(label);
    }
    /// `GetMainPartition`: the partition on the model label.
    fn get_main_partition(&self) -> Result<Handle<dyn TObjPartition>, Failure> {
        self.get_partition(&self.get_label(), false)
    }
    /// `GetDocument`: the OCAF document of the model, null when closed.
    fn get_document(&self) -> Result<Handle<TDocStdDocument>, Failure> {
        let label = self.get_label();
        if label.is_null() {
            return Ok(Handle::default());
        }
        TDocStdDocument::get(&label)
    }
    /// `GetDataLabel` (protected): the data label of the main partition.
    fn get_model_data_label(&self) -> Result<TdfLabel, Failure> {
        let partition = self.get_main_partition()?;
        rustycad_tkcdf::live(&partition, "TObj_Model::GetDataLabel")?.get_data_label()
    }
    /// `IsRegisteredName(theName, theDictionary)`: a null dictionary is the model's own.
    fn is_registered_name(
        &self,
        name: &Handle<HExtendedString>,
        dictionary: &Handle<TObjTNameContainer>,
    ) -> Result<bool, Failure> {
        let dictionary = self.dictionary_or_own(dictionary)?;
        Ok(dictionary
            .get()
            .is_some_and(|dictionary| dictionary.is_registered(name)))
    }
    /// `RegisterName(theName, theLabel, theDictionary)`.
    fn register_name(
        &self,
        name: &Handle<HExtendedString>,
        label: &TdfLabel,
        dictionary: &Handle<TObjTNameContainer>,
    ) -> Result<(), Failure> {
        let dictionary = self.dictionary_or_own(dictionary)?;
        match dictionary.get() {
            Some(dictionary) => dictionary.record_name(name, label),
            None => Ok(()),
        }
    }
    /// `UnRegisterName(theName, theDictionary)`.
    fn unregister_name(
        &self,
        name: &Handle<HExtendedString>,
        dictionary: &Handle<TObjTNameContainer>,
    ) -> Result<(), Failure> {
        let dictionary = self.dictionary_or_own(dictionary)?;
        match dictionary.get() {
            Some(dictionary) => dictionary.remove_name(name),
            None => Ok(()),
        }
    }
    /// The dictionary, or the model's own one when it is null.
    fn dictionary_or_own(
        &self,
        dictionary: &Handle<TObjTNameContainer>,
    ) -> Result<Handle<TObjTNameContainer>, Failure> {
        if dictionary.is_null() {
            return self.get_dictionary();
        }
        Ok(dictionary.clone())
    }
    /// `GetDictionary`: the `TObj_TNameContainer` of the model label, null when none.
    fn get_dictionary(&self) -> Result<Handle<TObjTNameContainer>, Failure> {
        let mut dictionary = Handle::default();
        let label = self.get_label();
        if !label.is_null() {
            label.find_attribute_as(&TObjTNameContainer::get_id(), &mut dictionary)?;
        }
        Ok(dictionary)
    }
    /// The document of the model; `Standard_NullObject` without one (a null dereference
    /// natively).
    fn live_document(&self, who: &str) -> Result<Handle<TDocStdDocument>, Failure> {
        let document = self.get_document()?;
        rustycad_tkcdf::live(&document, who)?;
        Ok(document)
    }
    /// `HasOpenCommand`.
    fn has_open_command(&self) -> Result<bool, Failure> {
        Ok(self
            .live_document("TObj_Model::HasOpenCommand")?
            .has_open_command())
    }
    /// `OpenCommand`.
    fn open_command(&self) -> Result<(), Failure> {
        self.live_document("TObj_Model::OpenCommand")?
            .open_command()
    }
    /// `CommitCommand`.
    fn commit_command(&self) -> Result<(), Failure> {
        self.live_document("TObj_Model::CommitCommand")?
            .commit_command()
            .map(|_| ())
    }
    /// `AbortCommand`.
    fn abort_command(&self) -> Result<(), Failure> {
        self.live_document("TObj_Model::AbortCommand")?
            .abort_command()
    }
    /// `SetModified(theModified)`: the saved time is the framework time, minus one when
    /// modified.
    fn set_modified(&self, modified: bool) -> Result<(), Failure> {
        let document = self.get_document()?;
        let Some(document) = document.get() else {
            return Ok(());
        };
        let data = document.get_data();
        let mut saved_time = rustycad_tkcdf::live(&data, "TObj_Model::SetModified")?.time();
        if modified {
            saved_time -= 1;
        }
        document.set_saved_time(saved_time);
        Ok(())
    }
    /// `GetFormatVersion`: the stored version, -1 when none.
    fn get_format_version(&self) -> Result<i32, Failure> {
        let label = self
            .get_model_data_label()?
            .find_child(TObjModelDataTag::FORMAT_VERSION, false)?;
        if label.is_null() {
            return Ok(-1);
        }
        let mut version: Handle<TDataStdInteger> = Handle::default();
        if !label.find_attribute_as(&TDataStdInteger::get_id(), &mut version)? {
            return Ok(-1);
        }
        Ok(TDataStdInteger::get(&version))
    }
    /// `SetFormatVersion(theVersion)` (protected).
    fn set_format_version(&self, version: i32) -> Result<(), Failure> {
        let label = self
            .get_model_data_label()?
            .find_child(TObjModelDataTag::FORMAT_VERSION, true)?;
        TDataStdInteger::set(&label, version).map(|_| ())
    }
    /// `getPartition(theLabel, theHidden)` (protected): the partition of the label, created
    /// (hidden when asked) when the label holds no object.
    fn get_partition(
        &self,
        label: &TdfLabel,
        hidden: bool,
    ) -> Result<Handle<dyn TObjPartition>, Failure> {
        if label.is_null() {
            return Ok(Handle::default());
        }
        let mut attribute: Handle<TObjTObject> = Handle::default();
        if label.find_attribute_as(&TObjTObject::get_id(), &mut attribute)? {
            return Ok(attribute.get_object().cast());
        }
        if hidden {
            return Ok(TObjHiddenPartition::new(label)?.cast());
        }
        TObjStandardPartition::create(label, true)
    }
    /// `getPartition(theLabel, theIndex, theName, theHidden)` (protected): the partition on
    /// the child of that index, created and named when new.
    fn get_partition_at(
        &self,
        label: &TdfLabel,
        index: i32,
        name: &ExtendedString,
        hidden: bool,
    ) -> Result<Handle<dyn TObjPartition>, Failure> {
        if label.is_null() {
            return Ok(Handle::default());
        }
        let mut child = label.find_child(index, false)?;
        let is_new = child.is_null();
        if is_new {
            child = label.find_child(index, true)?;
        }
        let partition = self.get_partition(&child, hidden)?;
        if is_new {
            let named = Handle::new(HExtendedString::from(name.clone()));
            rustycad_tkcdf::live(&partition, "TObj_Model::getPartition")?.set_name(&named)?;
        }
        Ok(partition)
    }
    /// `getPartition(theIndex, theName, theHidden)` (protected): under the child label of
    /// the main partition.
    fn get_partition_of_main(
        &self,
        index: i32,
        name: &ExtendedString,
        hidden: bool,
    ) -> Result<Handle<dyn TObjPartition>, Failure> {
        let main = self.get_main_partition()?;
        let children =
            rustycad_tkcdf::live(&main, "TObj_Model::getPartition")?.get_child_label()?;
        self.get_partition_at(&children, index, name, hidden)
    }
    /// `CopyReferences(theTarget, theRelocTable)`: from the main partition to the target's.
    fn copy_model_references(
        &self,
        target: &Handle<dyn TObjModel>,
        relocation_table: &Handle<rustycad_tklcaf::tdf::TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let mine = self.get_main_partition()?;
        let target = rustycad_tkcdf::live(target, "TObj_Model::CopyReferences")?;
        let target_root: Handle<dyn TObjObject> = target.get_main_partition()?.cast();
        rustycad_tkcdf::live(&mine, "TObj_Model::CopyReferences")?
            .copy_references(&target_root, relocation_table)
    }
}
impl<T: TObjModel + ?Sized> TObjModelState for T {}

impl TObjModelBase {
    /// `GetDocumentModel(theLabel)` (static): the model of the document holding the label,
    /// null for a null label or without model.
    pub fn get_document_model(label: &TdfLabel) -> Result<Handle<dyn TObjModel>, Failure> {
        if label.is_null() {
            return Ok(Handle::default());
        }
        let data = label.data();
        let root = rustycad_tkcdf::live(&data, "TObj_Model::GetDocumentModel")?.root();
        let mut owner: Handle<TDocStdOwner> = Handle::default();
        if root.is_null() || !root.find_attribute_as(&TDocStdOwner::get_id(), &mut owner)? {
            return Ok(Handle::default());
        }
        let document = owner.get_document();
        let Some(document) = document.get() else {
            return Ok(Handle::default());
        };
        let mut attribute: Handle<TObjTModel> = Handle::default();
        if document
            .main()?
            .find_attribute_as(&TObjTModel::get_id(), &mut attribute)?
        {
            return Ok(attribute.model());
        }
        Ok(Handle::default())
    }
    /// `SetNewName(theObject)` (static): a new name from the object's partition.
    pub fn set_new_name(object: &Handle<dyn TObjObject>) -> Result<(), Failure> {
        let partition = TObjStandardPartition::get_partition(object)?;
        let Some(partition) = partition.get() else {
            return Ok(());
        };
        let name = partition.get_new_name(true)?;
        if !name.is_null() {
            rustycad_tkcdf::live(object, "TObj_Model::SetNewName")?.set_name(&name)?;
        }
        Ok(())
    }
}
