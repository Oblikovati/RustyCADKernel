// Rust port of ApplicationFramework/TKCDF/CDF/CDF_Store.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDF_Store`: prepares the storage of a document (folder, name, comment, version) and
//! stores it with the documents it needs.
use super::{
    CdfApplication, CdfApplicationServices, CdfMetaDataDriver, CdfStoreList, CdfStoreSetNameStatus,
};
use crate::cdm::{CdmDocument, CdmDocumentLifecycle, CdmDocumentStorage, CdmMetaData};
use crate::live;
use crate::pcdm::PcdmStoreStatus;
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Failure, Handle, exceptions},
    tcollection::{ExtendedString, HExtendedString},
};

/// `CDF_Store`; `CdfStore::new(&document)?`.
pub struct CdfStore {
    list: Handle<CdfStoreList>,
    main_document: Handle<dyn CdmDocument>,
    current_document: Handle<dyn CdmDocument>,
    has_sub_components: bool,
    is_main_document: bool,
    path: ExtendedString,
    text: ExtendedString,
    status: PcdmStoreStatus,
}

/// The file-static `blank`: a shared empty string.
fn blank() -> Handle<HExtendedString> {
    Handle::new(HExtendedString::from(&ExtendedString::new("")))
}

/// `new TCollection_HExtendedString(text)`.
fn shared(text: &ExtendedString) -> Handle<HExtendedString> {
    Handle::new(HExtendedString::from(text))
}

impl CdfStore {
    /// `CDF_Store(aDocument)`: the store list of the document; the referenced documents
    /// that are not stored get default folders and names.
    pub fn new(document: &Handle<dyn CdmDocument>) -> Result<Self, Failure> {
        let mut store = Self {
            list: Handle::default(),
            main_document: document.clone(),
            current_document: document.clone(),
            has_sub_components: false,
            is_main_document: false,
            path: ExtendedString::default(),
            text: ExtendedString::default(),
            status: PcdmStoreStatus::NoObj,
        };
        store.init()?;
        Ok(store)
    }
    /// Private `Init`.
    fn init(&mut self) -> Result<(), Failure> {
        self.current_document = self.main_document.clone();
        self.list = Handle::new(CdfStoreList::new(&self.current_document)?);
        self.is_main_document = false;
        let list = live(&self.list, "CDF_Store::Init")?;
        list.init();
        while list.more() {
            self.current_document = list.value();
            if self.current_document != self.main_document {
                self.has_sub_components = true;
                self.find_default()?;
            }
            list.next();
        }
        self.is_main_document = true;
        self.current_document = self.main_document.clone();
        Ok(())
    }
    fn current(&self) -> Result<&dyn CdmDocument, Failure> {
        live(&self.current_document, "CDF_Store")
    }
    /// The native `theMetaDataDriver` macro: the driver of the current document's application.
    fn meta_data_driver(&self) -> Result<Handle<dyn CdfMetaDataDriver>, Failure> {
        let application: Handle<dyn CdfApplication> = self.current()?.application()?.cast();
        live(
            &application,
            "CDF_Store: the application is not a CDF_Application",
        )?
        .meta_data_driver()
    }
    /// `Folder`: the requested folder, else blank.
    pub fn folder(&self) -> Result<Handle<HExtendedString>, Failure> {
        let current = self.current()?;
        if current.has_requested_folder() {
            return Ok(shared(&current.requested_folder()?));
        }
        Ok(blank())
    }
    /// `Name`: the requested name.
    pub fn name(&self) -> Result<Handle<HExtendedString>, Failure> {
        Ok(shared(&self.current()?.requested_name()))
    }
    /// `SetFolder(aFolder)` with a C string; see [`CdfStore::set_folder`].
    pub fn set_folder_units(&self, folder: &[u16]) -> Result<bool, Failure> {
        self.set_folder(&ExtendedString::from_utf16(folder))
    }
    /// `SetFolder(aFolder)`: an existing folder (without a trailing separator) becomes the
    /// requested one.
    pub fn set_folder(&self, folder: &ExtendedString) -> Result<bool, Failure> {
        let mut folder = folder.clone();
        let length = folder.length();
        let last = folder.value(length).unwrap_or(0);
        if length > 1 && (last == u16::from(b'/') || last == u16::from(b'\\')) {
            folder.trunc(length - 1)?;
        }
        if !live(&self.meta_data_driver()?, "CDF_Store::SetFolder")?.find_folder(&folder) {
            return Ok(false);
        }
        self.current()?.set_requested_folder(&folder);
        Ok(true)
    }
    /// `RecheckName`: `SetName` with the requested name.
    pub fn recheck_name(&self) -> Result<CdfStoreSetNameStatus, Failure> {
        let name = self.current()?.requested_name();
        self.set_name(&name)
    }
    /// `SetName(aName)`: the driver's spelling of the name; tells whether it replaces a
    /// stored document or names an opened one.
    pub fn set_name(&self, name: &ExtendedString) -> Result<CdfStoreSetNameStatus, Failure> {
        let driver = self.meta_data_driver()?;
        let driver = live(&driver, "CDF_Store::SetName")?;
        let current = self.current()?;
        let name = driver.set_name(&self.current_document, name)?;
        if current.is_stored() {
            let meta = current.meta_data()?;
            let meta = live(&meta, "CDF_Store::SetName")?;
            if meta.folder() == current.requested_folder()? && meta.name() == name {
                return Ok(CdfStoreSetNameStatus::Ok);
            }
        }
        if current.has_requested_folder()
            && driver.find_without_version(&current.requested_folder()?, &name)
        {
            let existing = driver.meta_data_without_version(&current.requested_folder()?, &name)?;
            if live(&existing, "CDF_Store::SetName")?.is_retrieved() {
                return Ok(CdfStoreSetNameStatus::OpenDocument);
            }
            current.set_requested_name(&name);
            return Ok(CdfStoreSetNameStatus::ReplacingAnExistentDocument);
        }
        current.set_requested_name(&name);
        Ok(CdfStoreSetNameStatus::Ok)
    }
    /// `SetName(aName)` with a C string.
    pub fn set_name_units(&self, name: &[u16]) -> Result<CdfStoreSetNameStatus, Failure> {
        self.set_name(&ExtendedString::from_utf16(name))
    }
    /// `Realize`: stores the list; `Standard_ProgramError` when information is missing.
    pub fn realize(&mut self, range: &ProgressRange<'_>) -> Result<(), Failure> {
        let list = live(&self.list, "CDF_Store::Realize")?;
        if !list.is_consistent()? {
            return Err(exceptions::program_error("information are missing"));
        }
        let mut meta_data: Handle<CdmMetaData> = Handle::default();
        self.text = ExtendedString::new("");
        self.status = list.store(&mut meta_data, &mut self.text, range)?;
        if self.status == PcdmStoreStatus::Ok {
            self.path = live(&meta_data, "CDF_Store::Realize")?.path();
        }
        Ok(())
    }
    /// `Path`: the stored path of the last successful `Realize`.
    pub fn path(&self) -> ExtendedString {
        self.path.clone()
    }
    /// `MetaDataPath`: the current document's meta-data path, else blank.
    pub fn meta_data_path(&self) -> Result<Handle<HExtendedString>, Failure> {
        let current = self.current()?;
        if !current.is_stored() {
            return Ok(blank());
        }
        Ok(shared(
            &live(&current.meta_data()?, "CDF_Store::MetaDataPath")?.path(),
        ))
    }
    /// `Description`: the main document's format description, else blank.
    pub fn description(&self) -> Result<Handle<HExtendedString>, Failure> {
        let main = live(&self.main_document, "CDF_Store::Description")?;
        if main.find_description()? {
            return Ok(shared(&main.description()?));
        }
        Ok(blank())
    }
    /// `IsStored`.
    pub fn is_stored(&self) -> Result<bool, Failure> {
        Ok(self.current()?.is_stored())
    }
    /// `IsModified`.
    pub fn is_modified(&self) -> Result<bool, Failure> {
        Ok(self.current()?.is_modified())
    }
    /// `CurrentIsConsistent`: stored, or a folder is requested.
    pub fn current_is_consistent(&self) -> Result<bool, Failure> {
        let current = self.current()?;
        Ok(current.is_stored() || current.has_requested_folder())
    }
    /// `IsConsistent`: the store list is consistent.
    pub fn is_consistent(&self) -> Result<bool, Failure> {
        live(&self.list, "CDF_Store::IsConsistent")?.is_consistent()
    }
    /// `HasAPreviousVersion`.
    pub fn has_a_previous_version(&self) -> Result<bool, Failure> {
        Ok(self.current()?.has_requested_previous_version())
    }
    /// `PreviousVersion`: the requested previous version, else blank.
    pub fn previous_version(&self) -> Result<Handle<HExtendedString>, Failure> {
        let current = self.current()?;
        if current.has_requested_previous_version() {
            return Ok(shared(&current.requested_previous_version()?));
        }
        Ok(blank())
    }
    /// `SetPreviousVersion`: accepted when the driver has no versions; else only for a
    /// version it finds in the requested folder.
    pub fn set_previous_version(&self, previous_version: &[u16]) -> Result<bool, Failure> {
        let driver = self.meta_data_driver()?;
        let driver = live(&driver, "CDF_Store::SetPreviousVersion")?;
        if !driver.has_version_capability() {
            return Ok(true);
        }
        let current = self.current()?;
        if !current.has_requested_folder() {
            return Ok(false);
        }
        let version = ExtendedString::from_utf16(previous_version);
        if !driver.find(
            &current.requested_folder()?,
            &current.requested_name(),
            &version,
        ) {
            return Ok(false);
        }
        current.set_requested_previous_version(&version);
        Ok(true)
    }
    /// `SetCurrent(aPresentation)`: only recomputes whether the current is the main document.
    pub fn set_current(&mut self, _presentation: &[u16]) {
        self.is_main_document = self.current_document == self.main_document;
    }
    /// `SetMain`: the main document becomes the current one.
    pub fn set_main(&mut self) {
        self.current_document = self.main_document.clone();
        self.is_main_document = true;
    }
    /// `IsMainDocument`.
    pub fn is_main_document(&self) -> bool {
        self.is_main_document
    }
    /// `StoreStatus`: the status of the last `Realize` (`PCDM_SS_No_Obj` before).
    pub fn store_status(&self) -> PcdmStoreStatus {
        self.status
    }
    /// `AssociatedStatusText`.
    pub fn associated_status_text(&self) -> ExtendedString {
        self.text.clone()
    }
    /// `SetComment`: the requested comment of the current document.
    pub fn set_comment(&self, comment: &[u16]) -> Result<(), Failure> {
        self.current()?
            .set_requested_comment(&ExtendedString::from_utf16(comment));
        Ok(())
    }
    /// `Comment`: the requested comment.
    pub fn comment(&self) -> Result<Handle<HExtendedString>, Failure> {
        Ok(shared(&self.current()?.requested_comment()))
    }
    /// Private `FindDefault`: an unstored document gets the default folder and the driver's
    /// spelling of its name.
    fn find_default(&self) -> Result<(), Failure> {
        let current = self.current()?;
        if current.is_stored() {
            return Ok(());
        }
        let application: Handle<dyn CdfApplication> = current.application()?.cast();
        let folder = live(&application, "CDF_Store::FindDefault")?.default_folder()?;
        current.set_requested_folder(&folder);
        let driver = self.meta_data_driver()?;
        let name = live(&driver, "CDF_Store::FindDefault")?
            .set_name(&self.current_document, &current.requested_name())?;
        current.set_requested_name(&name);
        Ok(())
    }
}
