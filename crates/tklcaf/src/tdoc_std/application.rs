// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_Application.hxx and .cxx (fields,
// virtual members and document management) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDocStd_Application`: the OCAF application (formats, documents, open and save).
use super::TDocStdDocument;
use crate::live;
use rustycad_tkcdf::{
    SharedResourceManager,
    cdf::{CdfApplication, CdfApplicationBase, CdfApplicationServices, CdfDirectoryIterator},
    cdm::CdmDocument,
    json_dump::{class_begin, field_integer},
    pcdm::{PcdmRetrievalDriver, PcdmStorageDriver},
};
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    resource::{ResourceHost, ResourceManager},
    standard::{Failure, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::{AsciiString, ExtendedString},
};
use std::{io, sync::Arc};

/// The protected fields of `TDocStd_Application`, embedding the CDF ones;
/// `TDocStdApplicationBase::default()` is the constructor.
pub struct TDocStdApplicationBase {
    cdf: CdfApplicationBase,
    resources: SharedState<Option<SharedResourceManager>>,
    is_driver_loaded: bool,
}
impl Default for TDocStdApplicationBase {
    /// `TDocStd_Application()`: the driver is loaded when the CDF meta-data driver exists.
    fn default() -> Self {
        let cdf = CdfApplicationBase::default();
        let is_driver_loaded = cdf.meta_data_driver_field().is_some();
        Self {
            cdf,
            resources: SharedState::default(),
            is_driver_loaded,
        }
    }
}
impl TDocStdApplicationBase {
    /// The embedded `CDF_Application` fields.
    pub fn cdf(&self) -> &CdfApplicationBase {
        &self.cdf
    }
    /// The protected `myResources` (null until `Resources` loads it).
    pub fn resources_field(&self) -> Option<SharedResourceManager> {
        self.resources.read().clone()
    }
    /// The protected `myIsDriverLoaded`.
    pub fn is_driver_loaded_field(&self) -> bool {
        self.is_driver_loaded
    }
}

/// `TDocStd_Application`'s members over the embedded base. A concrete application fills
/// its `CdmApplication` and `CdfApplication` impls with
/// [`crate::tdoc_std_application_cdm_members`] and [`crate::tdoc_std_application_cdf_members`].
pub trait TDocStdApplication: CdfApplication {
    /// The embedded base with the native fields.
    fn tdoc_std_application_base(&self) -> &TDocStdApplicationBase;
    /// `ResourcesName` (virtual): the resource file name, empty by default.
    fn resources_name(&self) -> String {
        String::new()
    }
    /// `OnOpenTransaction` (virtual): nothing at this level.
    fn on_open_transaction(&self, _document: &Handle<TDocStdDocument>) -> Result<(), Failure> {
        Ok(())
    }
    /// `OnCommitTransaction` (virtual): nothing at this level.
    fn on_commit_transaction(&self, _document: &Handle<TDocStdDocument>) -> Result<(), Failure> {
        Ok(())
    }
    /// `OnAbortTransaction` (virtual): nothing at this level.
    fn on_abort_transaction(&self, _document: &Handle<TDocStdDocument>) -> Result<(), Failure> {
        Ok(())
    }
    /// `Resources` override: loads `Resource_Manager(ResourcesName())` once.
    fn tdoc_std_resources(&self) -> Result<SharedResourceManager, Failure> {
        let base = self.tdoc_std_application_base();
        if let Some(resources) = base.resources_field() {
            return Ok(resources);
        }
        let manager = ResourceManager::new(&self.resources_name(), false, ResourceHost::native())?;
        let shared: SharedResourceManager = Arc::new(SharedState::new(manager));
        *base.resources.write() = Some(shared.clone());
        Ok(shared)
    }
    /// `NewDocument(format, aDoc)` override: a new `TDocStd_Document`, initialized and
    /// opened in the session.
    fn tdoc_std_new_document(
        &self,
        format: &ExtendedString,
        document: &mut Handle<dyn CdmDocument>,
    ) -> Result<(), Failure> {
        let created: Handle<dyn CdmDocument> = TDocStdDocument::new(format)?.cast();
        self.init_document(&created)?;
        self.open_document(&created)?;
        *document = created;
        Ok(())
    }
    /// `IsDriverLoaded`.
    fn is_driver_loaded(&self) -> bool {
        self.tdoc_std_application_base().is_driver_loaded
    }
    /// `DefineFormat`: registers the format resources and its reader and writer.
    fn define_format(
        &self,
        format: &AsciiString,
        description: &AsciiString,
        extension: &AsciiString,
        reader: &Handle<dyn PcdmRetrievalDriver>,
        writer: &Handle<dyn PcdmStorageDriver>,
    ) -> Result<(), Failure> {
        let resources = self.resources()?;
        {
            let mut manager = resources.write();
            let text = |value: &AsciiString| String::from_utf8_lossy(value.as_bytes()).into_owned();
            manager.set_resource(
                &format!("{}.Description", text(format)),
                description.as_bytes(),
            );
            manager.set_resource(
                &format!("{}.FileExtension", text(format)),
                extension.as_bytes(),
            );
            manager.set_resource(
                &format!("{}.FileFormat", text(extension)),
                format.as_bytes(),
            );
        }
        let key = ExtendedString::from_ascii_string(format, true);
        if let Some(reader) = reader.get() {
            reader.set_format(&key);
        }
        if let Some(writer) = writer.get() {
            writer.set_format(&key);
        }
        let base = self.cdf_application_base();
        base.readers().add(key.clone(), reader.clone());
        base.writers().add(key, writer.clone());
        Ok(())
    }
    /// `ReadingFormats`: the formats with a reader, in registration order.
    fn reading_formats(&self, formats: &mut Vec<AsciiString>) {
        formats.clear();
        registered_formats(&self.cdf_application_base().readers(), formats);
    }
    /// `WritingFormats`: the formats with a writer, in registration order.
    fn writing_formats(&self, formats: &mut Vec<AsciiString>) {
        formats.clear();
        registered_formats(&self.cdf_application_base().writers(), formats);
    }
    /// `NbDocuments`: the documents of the session.
    fn nb_documents(&self) -> Result<i32, Failure> {
        Ok(live(
            &self.cdf_application_base().directory(),
            "TDocStd_Application::NbDocuments",
        )?
        .length())
    }
    /// `GetDocument(index)`: the index-th document of the session (1-based), null past the end.
    fn get_document(&self, index: i32) -> Result<Handle<TDocStdDocument>, Failure> {
        let mut iterator =
            CdfDirectoryIterator::with_directory(&self.cdf_application_base().directory())?;
        let mut current = 0;
        while iterator.more_document() {
            current += 1;
            if current == index {
                return Ok(iterator.document().cast());
            }
            iterator.next_document();
        }
        Ok(Handle::default())
    }
    /// `GetDocument(index, aDoc)` (deprecated natively).
    fn get_document_into(
        &self,
        index: i32,
        document: &mut Handle<TDocStdDocument>,
    ) -> Result<(), Failure> {
        *document = self.get_document(index)?;
        Ok(())
    }
    /// `NewDocument(format, TDocStd_Document&)`: the virtual `NewDocument`, down cast.
    fn new_tdoc_std_document(
        &self,
        format: &ExtendedString,
        document: &mut Handle<TDocStdDocument>,
    ) -> Result<(), Failure> {
        let mut created: Handle<dyn CdmDocument> = Handle::default();
        self.new_document(format, &mut created)?;
        *document = created.cast();
        Ok(())
    }
    /// `Close(aDoc)`: detaches the owner from the document, then closes it.
    fn close_tdoc_std_document(&self, document: &Handle<TDocStdDocument>) -> Result<(), Failure> {
        let Some(target) = document.get() else {
            return Ok(());
        };
        let mut owner: Handle<super::TDocStdOwner> = Handle::default();
        if target
            .main()?
            .root()?
            .find_attribute_as(&super::TDocStdOwner::get_id(), &mut owner)?
        {
            live(&owner, "TDocStd_Application::Close")?.set_document(&Handle::default());
        }
        target.before_close()?;
        self.close_document(&document.cast())
    }
    /// `IsInSession(path)`: the 1-based index of the saved document with that path
    /// (separators unified), 0 when none.
    fn is_in_session(&self, path: &ExtendedString) -> Result<i32, Failure> {
        let unified = unify_separators(path);
        for index in 1..=self.nb_documents()? {
            let document = self.get_document(index)?;
            let document = live(&document, "TDocStd_Application::IsInSession")?;
            if document.is_saved() && unify_separators(&document.get_path()?) == unified {
                return Ok(index);
            }
        }
        Ok(0)
    }
    /// `DumpJson`: the class name and the driver flag.
    fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        class_begin(stream, "TDocStd_Application")?;
        field_integer(
            stream,
            "myIsDriverLoaded",
            i32::from(self.is_driver_loaded()),
        )
    }
}
impl_transient_interface!(
    dyn TDocStdApplication,
    "TDocStd_Application",
    dyn CdfApplication
);

/// `/` and `\` read as `|`, as `IsInSession` compares paths.
fn unify_separators(path: &ExtendedString) -> ExtendedString {
    let mut unified = path.clone();
    unified.change_all(u16::from(b'/'), u16::from(b'|'));
    unified.change_all(u16::from(b'\\'), u16::from(b'|'));
    unified
}

/// Fills the `CdmApplication` impl of a `TDocStd_Application`.
#[macro_export]
macro_rules! tdoc_std_application_cdm_members {
    () => {
        rustycad_tkcdf::cdf_application_cdm_members!();
        fn resources(
            &self,
        ) -> Result<rustycad_tkcdf::SharedResourceManager, rustycad_tkernel::standard::Failure> {
            $crate::tdoc_std::TDocStdApplication::tdoc_std_resources(self)
        }
    };
}

/// Fills the `CdfApplication` impl of a `TDocStd_Application`.
#[macro_export]
macro_rules! tdoc_std_application_cdf_members {
    () => {
        fn cdf_application_base(&self) -> &rustycad_tkcdf::cdf::CdfApplicationBase {
            $crate::tdoc_std::TDocStdApplication::tdoc_std_application_base(self).cdf()
        }
        fn new_document(
            &self,
            format: &rustycad_tkernel::tcollection::ExtendedString,
            document: &mut rustycad_tkernel::standard::Handle<dyn rustycad_tkcdf::cdm::CdmDocument>,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::tdoc_std::TDocStdApplication::tdoc_std_new_document(self, format, document)
        }
    };
}

/// `TDocStd_Application` itself (`new TDocStd_Application()`).
#[derive(Default)]
pub struct TDocStdStandardApplication {
    transient: StandardTransient,
    base: TDocStdApplicationBase,
}
impl_transient!(TDocStdStandardApplication, "TDocStd_Application", dyn TDocStdApplication, transient,
    interfaces: [dyn rustycad_tkcdf::cdm::CdmApplication, dyn CdfApplication, dyn TDocStdApplication]);
impl TDocStdStandardApplication {
    /// `TDocStd_Application()`, already in its bound handle.
    ///
    /// ```
    /// use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};
    /// use rustycad_tkernel::{standard::Handle, tcollection::ExtendedString};
    /// let application = TDocStdStandardApplication::new();
    /// let mut document: Handle<TDocStdDocument> = Handle::default();
    /// application.new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document).unwrap();
    /// assert_eq!(application.nb_documents().unwrap(), 1);
    /// ```
    pub fn new() -> Handle<TDocStdStandardApplication> {
        let application = Handle::new(Self::default());
        if let Some(own) = application.get() {
            rustycad_tkcdf::cdm::CdmApplication::application_base(own)
                .bind_self(&application.cast());
        }
        application
    }
}
impl rustycad_tkcdf::cdm::CdmApplication for TDocStdStandardApplication {
    crate::tdoc_std_application_cdm_members!();
}
impl CdfApplication for TDocStdStandardApplication {
    crate::tdoc_std_application_cdf_members!();
}
impl TDocStdApplication for TDocStdStandardApplication {
    fn tdoc_std_application_base(&self) -> &TDocStdApplicationBase {
        &self.base
    }
}

/// The formats of a driver map whose driver is not null, in registration order (the
/// shared loop of `ReadingFormats` and `WritingFormats`).
fn registered_formats<D: rustycad_tkernel::standard::RuntimeTyped + ?Sized>(
    drivers: &IndexedDataMap<ExtendedString, Handle<D>>,
    formats: &mut Vec<AsciiString>,
) {
    for index in 1..=drivers.extent() {
        let (Ok(format), Ok(driver)) = (drivers.find_key(index), drivers.find_from_index(index))
        else {
            continue;
        };
        if driver.is_some() {
            formats.push(AsciiString::from_extended(format, None));
        }
    }
}
