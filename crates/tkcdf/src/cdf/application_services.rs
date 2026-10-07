// Rust port of ApplicationFramework/TKCDF/CDF/CDF_Application.cxx (non-virtual members)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-virtual members of `CDF_Application`, provided for every application.
use super::application_drivers::{format, resource_value};
use super::{CdfApplication, CdfMetaDataDriver, CdfTypeOfActivation};
use crate::cdm::{CdmCanCloseStatus, CdmDocument, CdmDocumentLifecycle, CdmMetaData};
use crate::live;
use crate::pcdm::{
    PcdmReadWriterBase, PcdmReaderFilter, PcdmReaderStatus, PcdmRetrievalDriverBase,
};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Failure, Handle, exceptions},
    storage::Data,
    tcollection::ExtendedString,
};
use std::io;

/// The non-virtual members of `CDF_Application` (ocaf-design.md section 6).
pub trait CdfApplicationServices: CdfApplication {
    /// `Open(aDocument)`: adds it to the directory, opens it here and activates it as new.
    fn open_document(&self, document: &Handle<dyn CdmDocument>) -> Result<(), Failure> {
        live(
            &self.cdf_application_base().directory(),
            "CDF_Application::Open",
        )?
        .add(document);
        let this = self.application_base().this()?;
        live(document, "CDF_Application::Open")?.open(&this);
        self.activate(document, CdfTypeOfActivation::New)
    }
    /// `CanClose(aDocument)`: the document's own answer.
    fn can_close_document(
        &self,
        document: &Handle<dyn CdmDocument>,
    ) -> Result<CdmCanCloseStatus, Failure> {
        live(document, "CDF_Application::CanClose")?.can_close()
    }
    /// `Close(aDocument)`: removes it from the directory, then closes it.
    fn close_document(&self, document: &Handle<dyn CdmDocument>) -> Result<(), Failure> {
        live(
            &self.cdf_application_base().directory(),
            "CDF_Application::Close",
        )?
        .remove(document);
        live(document, "CDF_Application::Close")?.close()
    }
    /// `Retrieve(aFolder, aName, UseStorageConfiguration, theFilter, theRange)`.
    fn retrieve_document(
        &self,
        folder: &ExtendedString,
        name: &ExtendedString,
        use_storage_configuration: bool,
        filter: &Handle<PcdmReaderFilter>,
        range: &ProgressRange<'_>,
    ) -> Result<Handle<dyn CdmDocument>, Failure> {
        self.retrieve_version(
            folder,
            name,
            &ExtendedString::default(),
            use_storage_configuration,
            filter,
            range,
        )
    }
    /// `Retrieve(aFolder, aName, aVersion, ...)`: retrieves the document of the meta-data
    /// (the version is ignored when empty), adds it to the directory, activates and opens it.
    fn retrieve_version(
        &self,
        folder: &ExtendedString,
        name: &ExtendedString,
        version: &ExtendedString,
        use_storage_configuration: bool,
        filter: &Handle<PcdmReaderFilter>,
        range: &ProgressRange<'_>,
    ) -> Result<Handle<dyn CdmDocument>, Failure> {
        let driver = self.cdf_application_base().meta_data_driver_field();
        let driver = live(&driver, "CDF_Application::Retrieve")?;
        let meta_data = if version.length() == 0 {
            driver.meta_data_without_version(folder, name)?
        } else {
            driver.meta_data(folder, name, version)?
        };
        let activation = self.type_of_activation(&meta_data)?;
        let document =
            self.retrieve_meta_data(&meta_data, use_storage_configuration, false, filter, range)?;
        live(
            &self.cdf_application_base().directory(),
            "CDF_Application::Retrieve",
        )?
        .add(&document);
        self.activate(&document, activation)?;
        live(&document, "CDF_Application::Retrieve")?.open(&self.application_base().this()?);
        Ok(document)
    }
    /// `CanRetrieve(theFolder, theName, theAppendMode)`.
    fn can_retrieve(
        &self,
        folder: &ExtendedString,
        name: &ExtendedString,
        append_mode: bool,
    ) -> Result<PcdmReaderStatus, Failure> {
        self.can_retrieve_version(folder, name, &ExtendedString::default(), append_mode)
    }
    /// `CanRetrieve(theFolder, theName, theVersion, theAppendMode)`: whether a document can
    /// be retrieved, and why not.
    fn can_retrieve_version(
        &self,
        folder: &ExtendedString,
        name: &ExtendedString,
        version: &ExtendedString,
        append_mode: bool,
    ) -> Result<PcdmReaderStatus, Failure> {
        let driver = self.cdf_application_base().meta_data_driver_field();
        let driver = live(&driver, "CDF_Application::CanRetrieve")?;
        if !driver.find(folder, name, version) {
            return Ok(PcdmReaderStatus::UnknownDocument);
        }
        if !driver.has_read_permission(folder, name, version) {
            return Ok(PcdmReaderStatus::PermissionDenied);
        }
        let meta_data = driver.meta_data(folder, name, version)?;
        let meta = live(&meta_data, "CDF_Application::CanRetrieve")?;
        if !append_mode && meta.is_retrieved() {
            let modified = live(&meta.document(), "CDF_Application::CanRetrieve")?.is_modified();
            return Ok(if modified {
                PcdmReaderStatus::AlreadyRetrievedAndModified
            } else {
                PcdmReaderStatus::AlreadyRetrieved
            });
        }
        if append_mode && !meta.is_retrieved() {
            return Ok(PcdmReaderStatus::NoDocument);
        }
        self.check_reader_availability(&meta.file_name())
    }
    /// The last branch of `CanRetrieve`: the format is known and a reader exists for it
    /// (a failing reader look-up is only an availability check).
    fn check_reader_availability(
        &self,
        file_name: &ExtendedString,
    ) -> Result<PcdmReaderStatus, Failure> {
        let mut file_format = PcdmReadWriterBase::file_format(file_name);
        if file_format.length() == 0 {
            let extension = super::application_drivers::file_extension(file_name);
            if extension.is_empty() {
                return Ok(PcdmReaderStatus::UnrecognizedFileFormat);
            }
            let resource = format!(
                "{}.FileFormat",
                String::from_utf8_lossy(extension.as_bytes())
            );
            match resource_value(&self.resources(), &resource)? {
                Some(value) => file_format = value,
                None => return Ok(PcdmReaderStatus::UnrecognizedFileFormat),
            }
        }
        match self.reader_from_format(&file_format) {
            Ok(reader) if reader.is_null() => Ok(PcdmReaderStatus::NoDriver),
            _ => Ok(PcdmReaderStatus::Ok),
        }
    }
    /// `GetRetrieveStatus`: the status of the last retrieval.
    fn get_retrieve_status(&self) -> PcdmReaderStatus {
        self.cdf_application_base().retrievable_status()
    }
    /// `Read(theIStream, theDocument, theFilter, theRange)`: the stream's native format is
    /// not readable by the port (native formats are not ported), so the status becomes
    /// `PCDM_RS_FormatFailure` without reading, as natively for a stream in no native format.
    fn read_stream(
        &self,
        stream: &mut dyn io::Read,
        _document: &mut Handle<dyn CdmDocument>,
        _filter: &Handle<PcdmReaderFilter>,
        _range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let mut data: Handle<Data> = Handle::default();
        let format = PcdmReadWriterBase::file_format_stream(stream, &mut data);
        if format.is_empty() {
            self.cdf_application_base()
                .set_retrievable_status(PcdmReaderStatus::FormatFailure);
        }
        Ok(())
    }
    /// `Format(aFileName, theFormat)`: the format of the file; false when unknown.
    fn format(
        &self,
        file_name: &ExtendedString,
        found: &mut ExtendedString,
    ) -> Result<bool, Failure> {
        format(&self.resources(), file_name, found)
    }
    /// `DefaultFolder`: the driver's default folder, kept once read.
    fn default_folder(&self) -> Result<ExtendedString, Failure> {
        let base = self.cdf_application_base();
        if base.default_folder.read().length() == 0 {
            let driver = base.meta_data_driver_field();
            *base.default_folder.write() =
                live(&driver, "CDF_Application::DefaultFolder")?.default_folder();
        }
        Ok(ExtendedString::from_utf16(
            base.default_folder.read().as_utf16(),
        ))
    }
    /// `SetDefaultFolder(aFolder)`: only to a folder the driver finds.
    fn set_default_folder(&self, folder: &[u16]) -> Result<bool, Failure> {
        let folder = ExtendedString::from_utf16(folder);
        let driver = self.cdf_application_base().meta_data_driver_field();
        let found = live(&driver, "CDF_Application::SetDefaultFolder")?.find_folder(&folder);
        if found {
            *self.cdf_application_base().default_folder.write() = folder;
        }
        Ok(found)
    }
    /// `MetaDataDriver`: `Standard_NoSuchObject` when the application has none.
    fn meta_data_driver(&self) -> Result<Handle<dyn CdfMetaDataDriver>, Failure> {
        let driver = self.cdf_application_base().meta_data_driver_field();
        if driver.is_null() {
            return Err(exceptions::no_such_object(
                "no metadatadriver has been provided; this application is not able to store or retrieve files.",
            ));
        }
        Ok(driver)
    }
    /// Private `TypeOfActivation`: new unless the document is retrieved and opened.
    fn type_of_activation(
        &self,
        meta_data: &Handle<CdmMetaData>,
    ) -> Result<CdfTypeOfActivation, Failure> {
        let meta = live(meta_data, "CDF_Application::TypeOfActivation")?;
        if !meta.is_retrieved() {
            return Ok(CdfTypeOfActivation::New);
        }
        let document = meta.document();
        let document = live(&document, "CDF_Application::TypeOfActivation")?;
        if !document.is_opened() {
            return Ok(CdfTypeOfActivation::New);
        }
        Ok(if document.is_modified() {
            CdfTypeOfActivation::Modified
        } else {
            CdfTypeOfActivation::Unchanged
        })
    }
    /// Private `CanRetrieve(aMetaData, theAppendMode)`: with or without the version.
    fn can_retrieve_meta_data(
        &self,
        meta_data: &Handle<CdmMetaData>,
        append_mode: bool,
    ) -> Result<PcdmReaderStatus, Failure> {
        let meta = live(meta_data, "CDF_Application::CanRetrieve")?;
        if meta.has_version() {
            return self.can_retrieve_version(
                &meta.folder(),
                &meta.name(),
                &meta.version()?,
                append_mode,
            );
        }
        self.can_retrieve(&meta.folder(), &meta.name(), append_mode)
    }
    /// Private `DocumentVersion`: the version stored in the file (-1 for a file in no
    /// native format).
    fn cdf_document_version(&self, meta_data: &Handle<CdmMetaData>) -> Result<i32, Failure> {
        let file_name = live(meta_data, "CDF_Application::DocumentVersion")?.file_name();
        Ok(PcdmRetrievalDriverBase::document_version(
            &file_name,
            &self.message_driver(),
        ))
    }
    /// Private `Retrieve(aMetaData, UseStorageConfiguration, IsComponent, ...)`.
    fn retrieve_meta_data(
        &self,
        meta_data: &Handle<CdmMetaData>,
        use_storage_configuration: bool,
        is_component: bool,
        filter: &Handle<PcdmReaderFilter>,
        range: &ProgressRange<'_>,
    ) -> Result<Handle<dyn CdmDocument>, Failure> {
        super::application_retrieve::retrieve(
            self,
            meta_data,
            use_storage_configuration,
            is_component,
            filter,
            range,
        )
    }
}
impl<T: CdfApplication + ?Sized> CdfApplicationServices for T {}
