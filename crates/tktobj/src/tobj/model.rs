// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Model.hxx (virtual members and fields)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_Model`: the base of the OCAF-based models. Its virtual members are the trait
//! [`TObjModel`] (the pure virtual `NewEmpty` is required); its non-virtual members are
//! the extension trait [`super::TObjModelState`] (tobj-design.md section 3).
use super::model_io;
use super::{
    TObjApplication, TObjCheckModel, TObjModelIterator, TObjModelState, TObjObject,
    TObjObjectIterator, TObjStandardApplication, TObjTNameContainer,
};
use rustycad_tkernel::{
    impl_transient_interface,
    message::Messenger,
    standard::{
        Failure, Guid, Handle, SharedState, StandardTransient, Transient, WeakHandle, exceptions,
    },
    tcollection::{ExtendedString, HExtendedString},
};
use rustycad_tklcaf::{
    guid_from_text,
    tdf::{TdfLabel, TdfRelocationTable},
    tdoc_std::TDocStdDocument,
};
use std::{
    io,
    sync::{Arc, OnceLock},
};

/// `TObj_Model::DataTag`: the data of the model on its main partition.
pub struct TObjModelDataTag;
impl TObjModelDataTag {
    /// `DataTag_FormatVersion`: `TObj_Partition::DataTag_Last`.
    pub const FORMAT_VERSION: i32 = super::TObjPartitionDataTag::LAST;
    /// `DataTag_Last`.
    pub const LAST: i32 = Self::FORMAT_VERSION + 1;
}

/// The private fields of `TObj_Model` (`myLabel`, `myMessenger`) and the self handle.
pub struct TObjModelBase {
    this: OnceLock<WeakHandle<dyn TObjModel>>,
    label: SharedState<TdfLabel>,
    messenger: SharedState<Arc<Messenger>>,
}

impl TObjModelBase {
    /// `TObj_Model()`: the messenger of the TObj application.
    pub fn new() -> Result<Self, Failure> {
        let application = TObjStandardApplication::get_instance()?;
        let messenger = rustycad_tkcdf::live(&application, "TObj_Model::TObj_Model")?.messenger();
        Ok(Self {
            this: OnceLock::new(),
            label: SharedState::new(TdfLabel::default()),
            messenger: SharedState::new(messenger),
        })
    }
    /// Records the owning handle once; a model constructor that returns a handle calls it.
    pub fn bind_self(&self, model: &Handle<dyn TObjModel>) {
        let _ = self.this.set(model.downgrade());
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before `bind_self`.
    pub fn this(&self) -> Result<Handle<dyn TObjModel>, Failure> {
        let this = self.this.get().map(WeakHandle::upgrade).unwrap_or_default();
        if this.is_null() {
            return Err(exceptions::null_object(
                "TObj_Model: the model has no owning handle; expected bind_self",
            ));
        }
        Ok(this)
    }
    /// `myLabel`.
    pub(crate) fn label(&self) -> TdfLabel {
        self.label.read().clone()
    }
    /// Sets `myLabel`.
    pub(crate) fn set_label(&self, label: &TdfLabel) {
        *self.label.write() = label.clone();
    }
    /// `myMessenger`.
    pub(crate) fn messenger(&self) -> Arc<Messenger> {
        self.messenger.read().clone()
    }
    /// Sets `myMessenger`.
    pub(crate) fn set_messenger(&self, messenger: Arc<Messenger>) {
        *self.messenger.write() = messenger;
    }
}

impl Drop for TObjModelBase {
    /// `~TObj_Model`: closes the model (the base `Close`, as in a native destructor).
    fn drop(&mut self) {
        let label = self.label();
        if !label.is_null() {
            // A destructor cannot report a failure.
            let _ = model_io::close_label(&label);
        }
    }
}

/// `TObj_Model`'s virtual members; a concrete model embeds a [`TObjModelBase`] and
/// registers `dyn TObjModel` as an interface of its `impl_transient!`.
pub trait TObjModel: Transient {
    /// The embedded base with the native private fields.
    fn model_base(&self) -> &TObjModelBase;
    /// `NewEmpty` (pure virtual): a new model of the same type.
    fn new_empty(&self) -> Result<Handle<dyn TObjModel>, Failure>;
    /// `checkDocumentEmpty(theFile)` (protected): true for an empty name, a missing or an
    /// empty file.
    fn check_document_empty(&self, file: &ExtendedString) -> bool {
        model_io::check_document_empty(file)
    }
    /// `Load(theFile)`: reads the model, or initializes an empty one when the file is
    /// empty or missing.
    fn load(&self, file: &ExtendedString) -> Result<bool, Failure> {
        model_io::load_file(self, file)
    }
    /// `Load(theIStream)`: reads the model, or initializes an empty one on failure.
    fn load_stream(&self, stream: &mut dyn io::Read) -> Result<bool, Failure> {
        model_io::load_stream(self, stream)
    }
    /// `SaveAs(theFile)`.
    fn save_as(&self, file: &ExtendedString) -> Result<bool, Failure> {
        model_io::save_as_file(self, file)
    }
    /// `SaveAs(theOStream)`.
    fn save_as_stream(&self, stream: &mut dyn io::Write) -> Result<bool, Failure> {
        model_io::save_as_stream(self, stream)
    }
    /// `Close`: closes the document; false when there is none.
    fn close(&self) -> Result<bool, Failure> {
        let label = self.get_label();
        if label.is_null() {
            return Ok(false);
        }
        let document = TDocStdDocument::get(&label)?;
        if document.is_null() {
            return Ok(false);
        }
        self.close_document(&document)?;
        self.model_base().set_label(&TdfLabel::default());
        Ok(true)
    }
    /// `GetFile`: the path of the stored document, null when not stored.
    fn get_file(&self) -> Result<Handle<HExtendedString>, Failure> {
        model_io::model_file(self)
    }
    /// `GetObjects`: a `TObj_ModelIterator` on every object.
    fn get_objects(&self) -> Result<Handle<dyn TObjObjectIterator>, Failure> {
        Ok(TObjModelIterator::new(&self.model_base().this()?)?.cast())
    }
    /// `GetChildren`: the children of the main partition, null without it.
    fn get_children(&self) -> Result<Handle<dyn TObjObjectIterator>, Failure> {
        let partition = self.get_main_partition()?;
        match partition.get() {
            Some(partition) => partition.get_children(None),
            None => Ok(Handle::default()),
        }
    }
    /// `FindObject(theName, theDictionary)`: the object of a registered name, null when
    /// none (a null dictionary is the model's own).
    fn find_object(
        &self,
        name: &Handle<HExtendedString>,
        dictionary: &Handle<TObjTNameContainer>,
    ) -> Result<Handle<dyn TObjObject>, Failure> {
        super::model_init::find_object(self, name, dictionary)
    }
    /// `GetChecker`: a new `TObj_CheckModel` of this model.
    fn get_checker(&self) -> Result<Handle<TObjCheckModel>, Failure> {
        Ok(Handle::new(TObjCheckModel::new(&self.model_base().this()?)))
    }
    /// `GetRoot`: the main partition.
    fn get_root(&self) -> Result<Handle<dyn TObjObject>, Failure> {
        Ok(self.get_partition(&self.get_label(), false)?.cast())
    }
    /// `GetModelName`: `TObj` by default.
    fn get_model_name(&self) -> Handle<HExtendedString> {
        Handle::new(HExtendedString::from_c_string(b"TObj"))
    }
    /// `IsModified`: the document changed since it was saved; false without document.
    fn is_modified(&self) -> Result<bool, Failure> {
        let document = self.get_document()?;
        match document.get() {
            Some(document) => document.is_changed(),
            None => Ok(false),
        }
    }
    /// `GetApplication`: the static instance of the TObj application.
    fn get_application(&self) -> Result<Handle<dyn TObjApplication>, Failure> {
        TObjStandardApplication::get_instance()
    }
    /// `GetFormat`: `TObjBin` by default.
    fn get_format(&self) -> ExtendedString {
        ExtendedString::new("TObjBin")
    }
    /// `Update`: true by default.
    fn update(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    /// `GetGUID`: the interface GUID of `TObj_Model`.
    fn get_guid(&self) -> Guid {
        guid_from_text("3bbefb49-e618-11d4-ba38-0060b0ee18ea")
    }
    /// `initNewModel(IsNew)` (protected): sets the names map; for a loaded model, the
    /// objects' `AfterRetrieval`, the back references and a consistency check.
    fn init_new_model(&self, is_new: bool) -> Result<bool, Failure> {
        super::model_init::init_new_model(self, is_new)
    }
    /// `updateBackReferences(theObject)` (protected): recursive.
    fn update_back_references(&self, object: &Handle<dyn TObjObject>) -> Result<(), Failure> {
        super::model_init::update_back_references(self, object)
    }
    /// `isToCheck` (protected): true.
    fn is_to_check(&self) -> bool {
        true
    }
    /// `Paste(theModel, theRelocTable)`: clones the main partition into the model.
    fn paste(
        &self,
        model: &Handle<dyn TObjModel>,
        relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<bool, Failure> {
        let Some(target) = model.get() else {
            return Ok(false);
        };
        TObjTNameContainer::set(&target.get_label())?;
        let partition = self.get_main_partition()?;
        rustycad_tkcdf::live(&partition, "TObj_Model::Paste")?
            .clone_object(&target.get_label(), relocation_table)?;
        Ok(true)
    }
}
impl_transient_interface!(dyn TObjModel, "TObj_Model", StandardTransient);
