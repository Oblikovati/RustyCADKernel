// Rust port of ApplicationFramework/TKCDF/CDM/CDM_Document.cxx (storage members)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The storage members of `CDM_Document`: meta-data, requested storage place and the
//! format resources.
use super::{CdmDocument, CdmDocumentLifecycle, CdmMetaData, CdmReference};
use crate::{SharedResourceManager, live};
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::ExtendedString,
};

/// The storage members of `CDM_Document`, provided for every [`CdmDocument`].
pub trait CdmDocumentStorage: CdmDocument {
    /// `StorageVersion`: the modification counter at the last storage (0 by default).
    fn storage_version(&self) -> i32 {
        self.document_base().read().storage_version
    }
    /// `SetMetaData`: attaches the document to its storage description, updates the
    /// references of the other retrieved documents that wait for it, and takes the folder
    /// and version as the requested ones. `Standard_Failure` when the document was not
    /// opened and must update the application's table.
    fn set_meta_data(&self, meta_data: &Handle<CdmMetaData>) -> Result<(), Failure> {
        let base = self.document_base();
        let this = base.this()?;
        let meta = live(meta_data, "CDM_Document::SetMetaData")?;
        if !meta.is_retrieved() || meta.document() != this {
            meta.set_document(&this);
            update_waiting_references(self, meta_data)?;
            let previous = base.read().meta_data.clone();
            if let Some(previous) = previous.get() {
                previous.unset_document();
            }
        }
        {
            let mut state = base.write();
            state.storage_version = state.version;
            state.meta_data = meta_data.clone();
        }
        self.set_requested_folder(&meta.folder());
        if meta.has_version() {
            self.set_requested_previous_version(&meta.version()?);
        }
        Ok(())
    }
    /// `UnsetIsStored`: the meta-data no longer has a retrieved document (the document
    /// keeps its meta-data, as natively).
    fn unset_is_stored(&self) {
        let meta_data = self.document_base().read().meta_data.clone();
        if let Some(meta_data) = meta_data.get() {
            meta_data.unset_document();
        }
    }
    /// `MetaData`: `Standard_NoSuchObject` for a document that is not stored.
    fn meta_data(&self) -> Result<Handle<CdmMetaData>, Failure> {
        let meta_data = self.document_base().read().meta_data.clone();
        if meta_data.is_null() {
            return Err(exceptions::no_such_object(
                "cannot furnish the MetaData of an object which is not stored",
            ));
        }
        Ok(meta_data)
    }
    /// `Folder`: the meta-data folder; `Standard_NoSuchObject` when not stored.
    fn folder(&self) -> Result<ExtendedString, Failure> {
        let meta_data = self.document_base().read().meta_data.clone();
        match meta_data.get() {
            Some(meta_data) => Ok(meta_data.folder()),
            None => Err(exceptions::no_such_object(
                "cannot furnish the folder of an object which is not stored",
            )),
        }
    }
    /// `SetRequestedFolder`: ignored for an empty folder.
    fn set_requested_folder(&self, folder: &ExtendedString) {
        if folder.length() == 0 {
            return;
        }
        let mut state = self.document_base().write();
        state.requested_folder_is_defined = true;
        state.requested_folder = folder.clone();
    }
    /// `RequestedFolder`: `Standard_NoSuchObject` when none was requested.
    fn requested_folder(&self) -> Result<ExtendedString, Failure> {
        let state = self.document_base().read();
        if !state.requested_folder_is_defined {
            return Err(exceptions::no_such_object(
                "storage folder is undefined for this document",
            ));
        }
        Ok(state.requested_folder.clone())
    }
    /// `HasRequestedFolder`; `document.has_requested_folder()`.
    fn has_requested_folder(&self) -> bool {
        self.document_base().read().requested_folder_is_defined
    }
    /// `SetRequestedName`; `document.set_requested_name(&name)`.
    fn set_requested_name(&self, name: &ExtendedString) {
        let mut state = self.document_base().write();
        state.requested_name = name.clone();
        state.requested_name_is_defined = true;
    }
    /// `RequestedName`: the requested name; else the meta-data name, else `"Document_"`
    /// (which then counts as requested).
    fn requested_name(&self) -> ExtendedString {
        let mut state = self.document_base().write();
        if !state.requested_name_is_defined {
            state.requested_name = match state.meta_data.get() {
                Some(meta_data) => meta_data.name(),
                None => ExtendedString::new("Document_"),
            };
        }
        state.requested_name_is_defined = true;
        state.requested_name.clone()
    }
    /// `SetRequestedPreviousVersion`; `document.set_requested_previous_version(&v)`.
    fn set_requested_previous_version(&self, previous_version: &ExtendedString) {
        let mut state = self.document_base().write();
        state.requested_previous_version_is_defined = true;
        state.requested_previous_version = previous_version.clone();
    }
    /// `UnsetRequestedPreviousVersion`; `document.unset_requested_previous_version()`.
    fn unset_requested_previous_version(&self) {
        self.document_base()
            .write()
            .requested_previous_version_is_defined = false;
    }
    /// `HasRequestedPreviousVersion`; `document.has_requested_previous_version()`.
    fn has_requested_previous_version(&self) -> bool {
        self.document_base()
            .read()
            .requested_previous_version_is_defined
    }
    /// `RequestedPreviousVersion`: `Standard_NoSuchObject` when none was requested.
    fn requested_previous_version(&self) -> Result<ExtendedString, Failure> {
        let state = self.document_base().read();
        if !state.requested_previous_version_is_defined {
            return Err(exceptions::no_such_object(
                "PreviousVersion is undefined fro this document",
            ));
        }
        Ok(state.requested_previous_version.clone())
    }
    /// `SetRequestedComment`; `document.set_requested_comment(&comment)`.
    fn set_requested_comment(&self, comment: &ExtendedString) {
        self.document_base().write().requested_comment = comment.clone();
    }
    /// `RequestedComment`: a copy through `ToExtString`, so it ends at the first NUL.
    fn requested_comment(&self) -> ExtendedString {
        ExtendedString::from_utf16(self.document_base().read().requested_comment.as_utf16())
    }
    /// `LoadResources`: reads `<format>.FileExtension` and `<format>.Description` once from
    /// the application resources; `Standard_Failure` when the document is not opened.
    fn load_resources(&self) -> Result<(), Failure> {
        if self.document_base().resources_are_loaded() {
            return Ok(());
        }
        let resources = storage_resource(self)?;
        let mut format = self.storage_format();
        format.assign_cat(ExtendedString::new("."));
        let extension = find_resource(&resources, &format, "FileExtension")?;
        let description = find_resource(&resources, &format, "Description")?;
        let mut state = self.document_base().write();
        state.file_extension_was_found = extension.is_some();
        if let Some(extension) = extension {
            state.file_extension = extension;
        }
        state.description_was_found = description.is_some();
        if let Some(description) = description {
            state.description = description;
        }
        state.resources_are_loaded = true;
        Ok(())
    }
    /// `FindFileExtension`: loads the resources, then whether the extension was found.
    fn find_file_extension(&self) -> Result<bool, Failure> {
        self.load_resources()?;
        Ok(self.document_base().read().file_extension_was_found)
    }
    /// `FileExtension`: the `<format>.FileExtension` resource.
    fn file_extension(&self) -> Result<ExtendedString, Failure> {
        self.load_resources()?;
        Ok(self.document_base().read().file_extension.clone())
    }
    /// `FindDescription`: loads the resources, then whether the description was found.
    fn find_description(&self) -> Result<bool, Failure> {
        self.load_resources()?;
        Ok(self.document_base().read().description_was_found)
    }
    /// `Description`: the `<format>.Description` resource.
    fn description(&self) -> Result<ExtendedString, Failure> {
        self.load_resources()?;
        Ok(self.document_base().read().description.clone())
    }
}
impl<T: CdmDocument + ?Sized> CdmDocumentStorage for T {}

/// The loop of `SetMetaData` over the application table: every other retrieved document's
/// references that wait for `meta_data` get its document.
fn update_waiting_references<D: CdmDocument + ?Sized>(
    document: &D,
    meta_data: &Handle<CdmMetaData>,
) -> Result<(), Failure> {
    let application = document.application()?;
    let application = live(&application, "CDM_Document::SetMetaData")?;
    let table: Vec<Handle<CdmMetaData>> = application
        .meta_data_look_up_table()
        .values()
        .cloned()
        .collect();
    for other in table.iter().filter(|other| *other != meta_data) {
        let Some(other) = other.get().filter(|other| other.is_retrieved()) else {
            continue;
        };
        let other_document = other.document();
        let references = live(&other_document, "CDM_Document::SetMetaData")?
            .document_base()
            .references_from_this();
        for reference in &references {
            CdmReference::update(reference, meta_data)?;
        }
    }
    Ok(())
}

/// Private `StorageResource`: the resources of the opening application.
fn storage_resource<D: CdmDocument + ?Sized>(
    document: &D,
) -> Result<SharedResourceManager, Failure> {
    let application = document.document_base().read().application.clone();
    match application.get() {
        Some(application) => application.resources(),
        None => Err(Failure::new(&format!(
            "this document of format {} has not yet been opened by any application. \n",
            document.storage_format().to_utf8()
        ))),
    }
}

/// The file-static `FIND`: the value of `<format.><item>` read as UTF-8, if defined.
fn find_resource(
    resources: &SharedResourceManager,
    format: &ExtendedString,
    item: &str,
) -> Result<Option<ExtendedString>, Failure> {
    let mut name = format.clone();
    name.assign_cat(ExtendedString::new(item));
    let name = name.to_utf8();
    let manager = resources.read();
    if !manager.find(&name) {
        return Ok(None);
    }
    let value = manager.value(&name)?;
    Ok(Some(ExtendedString::from_ascii_string(value, true)))
}
