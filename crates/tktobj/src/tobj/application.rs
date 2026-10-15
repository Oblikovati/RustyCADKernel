// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Application.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_Application`: the OCAF application of TObj models (resources `TObj`).
use super::application_errors::{reader_error_key, store_error_key};
use super::messages::{load_tobj_messages, message_with};
use rustycad_tkcdf::{
    cdf::CdfApplication,
    cdm::CdmApplication,
    json_dump::{base_class, class_begin},
    pcdm::{PcdmReaderStatus, PcdmStoreStatus},
};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    message::{Gravity, Messenger, ProgressRange},
    standard::{Failure, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::ExtendedString,
};
use rustycad_tklcaf::tdoc_std::{
    TDocStdApplication, TDocStdApplicationBase, TDocStdApplicationIo, TDocStdDocument,
};
use std::{
    io,
    sync::{Arc, OnceLock},
};

/// The fields of `TObj_Application` (`myIsError`, `myIsVerbose`, `myMessenger`) over the
/// `TDocStd_Application` ones.
pub struct TObjApplicationBase {
    tdoc_std: TDocStdApplicationBase,
    is_error: SharedState<bool>,
    is_verbose: SharedState<bool>,
    messenger: SharedState<Arc<Messenger>>,
}

impl TObjApplicationBase {
    /// `TObj_Application()`: loads the TObj messages once (`Standard_ProgramError` when
    /// they are invalid), a new messenger, not verbose.
    pub fn new() -> Result<Self, Failure> {
        load_tobj_messages()?;
        Ok(Self {
            tdoc_std: TDocStdApplicationBase::default(),
            is_error: SharedState::new(false),
            is_verbose: SharedState::new(false),
            messenger: SharedState::new(Arc::new(Messenger::new())),
        })
    }
    /// The embedded `TDocStd_Application` fields.
    pub fn tdoc_std(&self) -> &TDocStdApplicationBase {
        &self.tdoc_std
    }
}

/// `TObj_Application`'s members over the embedded base; a concrete application fills its
/// `CdmApplication`, `CdfApplication` and `TDocStdApplication` impls like
/// [`TObjStandardApplication`].
pub trait TObjApplication: TDocStdApplication {
    /// The embedded base with the native fields.
    fn tobj_application_base(&self) -> &TObjApplicationBase;
    /// `SaveDocument(theSourceDoc, theTargetFile)`: `SaveAs`; an error is reported.
    fn save_document(
        &self,
        document: &Handle<TDocStdDocument>,
        target_file: &ExtendedString,
    ) -> Result<bool, Failure> {
        let status = self.save_as(document, target_file, &ProgressRange::default())?;
        self.store_result(status, target_file)
    }
    /// `SaveDocument(theSourceDoc, theOStream)`.
    fn save_document_stream(
        &self,
        document: &Handle<TDocStdDocument>,
        stream: &mut dyn io::Write,
    ) -> Result<bool, Failure> {
        let status = self.save_as_stream(document, stream, &ProgressRange::default());
        self.store_result(status, &ExtendedString::default())
    }
    /// `LoadDocument(theSourceFile, theTargetDoc)`: `Open`; a caught failure is
    /// `PCDM_RS_ReaderException`, an error is reported.
    fn load_document(
        &self,
        source_file: &ExtendedString,
        document: &mut Handle<TDocStdDocument>,
    ) -> Result<bool, Failure> {
        let status = self
            .open_path_without_filter(source_file, document, &ProgressRange::default())
            .unwrap_or(PcdmReaderStatus::ReaderException);
        self.reader_result(status, source_file)
    }
    /// `LoadDocument(theIStream, theTargetDoc)`.
    fn load_document_stream(
        &self,
        stream: &mut dyn io::Read,
        document: &mut Handle<TDocStdDocument>,
    ) -> Result<bool, Failure> {
        let status = self.open_stream_without_filter(stream, document, &ProgressRange::default());
        self.reader_result(status, &ExtendedString::default())
    }
    /// `CreateNewDocument(theDoc, theFormat)`: `NewDocument`.
    fn create_new_document(
        &self,
        document: &mut Handle<TDocStdDocument>,
        format: &ExtendedString,
    ) -> Result<bool, Failure> {
        *self.tobj_application_base().is_error.write() = false;
        self.new_tdoc_std_document(format, document)?;
        Ok(!*self.tobj_application_base().is_error.read())
    }
    /// `ErrorMessage(theMsg, theLevel)`: sends the message to the messenger.
    fn error_message(&self, message: &ExtendedString, level: Gravity) {
        self.messenger().send_extended(message, level);
    }
    /// `ErrorMessage(theMsg)`: with `Message_Alarm`.
    fn error_message_alarm(&self, message: &ExtendedString) {
        self.error_message(message, Gravity::Alarm);
    }
    /// `Messenger`: the messenger handle.
    fn messenger(&self) -> Arc<Messenger> {
        self.tobj_application_base().messenger.read().clone()
    }
    /// Assigns the messenger (the native `Messenger()` returns a modifiable reference).
    fn set_messenger(&self, messenger: Arc<Messenger>) {
        *self.tobj_application_base().messenger.write() = messenger;
    }
    /// `SetVerbose(isVerbose)`.
    fn set_verbose(&self, is_verbose: bool) {
        *self.tobj_application_base().is_verbose.write() = is_verbose;
    }
    /// `IsVerbose`.
    fn is_verbose(&self) -> bool {
        *self.tobj_application_base().is_verbose.read()
    }
    /// `DumpJson`: the class, then `TDocStd_Application` as the base.
    fn tobj_dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TObj_Application")?;
        base_class(stream, depth, &|s, d| {
            TDocStdApplication::dump_json(self, s, d)
        })
    }
    /// `SetError(PCDM_StoreStatus, theInfo)` (protected).
    fn set_store_error(
        &self,
        status: PcdmStoreStatus,
        info: &ExtendedString,
    ) -> Result<(), Failure> {
        self.error_message_alarm(&message_with(store_error_key(status), info)?);
        Ok(())
    }
    /// `SetError(PCDM_ReaderStatus, theInfo)` (protected).
    fn set_reader_error(
        &self,
        status: PcdmReaderStatus,
        info: &ExtendedString,
    ) -> Result<(), Failure> {
        self.error_message_alarm(&message_with(reader_error_key(status), info)?);
        Ok(())
    }
    /// Records the error flag of a store status and reports it (then `Standard::Purge`).
    fn store_result(
        &self,
        status: PcdmStoreStatus,
        info: &ExtendedString,
    ) -> Result<bool, Failure> {
        let is_error = status != PcdmStoreStatus::Ok;
        *self.tobj_application_base().is_error.write() = is_error;
        if is_error {
            self.set_store_error(status, info)?;
        }
        rustycad_tkernel::standard::memory::purge();
        Ok(!is_error)
    }
    /// Records the error flag of a reader status and reports it (then `Standard::Purge`).
    fn reader_result(
        &self,
        status: PcdmReaderStatus,
        info: &ExtendedString,
    ) -> Result<bool, Failure> {
        let is_error = status != PcdmReaderStatus::Ok;
        *self.tobj_application_base().is_error.write() = is_error;
        if is_error {
            self.set_reader_error(status, info)?;
        }
        rustycad_tkernel::standard::memory::purge();
        Ok(!is_error)
    }
}
impl_transient_interface!(
    dyn TObjApplication,
    "TObj_Application",
    dyn TDocStdApplication
);

/// `TObj_Application` itself; [`TObjStandardApplication::get_instance`] is the process
/// instance.
pub struct TObjStandardApplication {
    transient: StandardTransient,
    base: TObjApplicationBase,
}
impl_transient!(TObjStandardApplication, "TObj_Application", dyn TObjApplication, transient,
    interfaces: [dyn CdmApplication, dyn CdfApplication, dyn TDocStdApplication, dyn TObjApplication]);

/// The process instance of `GetInstance` (a function-local static natively).
static INSTANCE: OnceLock<Result<Handle<TObjStandardApplication>, Failure>> = OnceLock::new();

impl TObjStandardApplication {
    /// `TObj_Application()`, already in its bound handle.
    pub fn new() -> Result<Handle<TObjStandardApplication>, Failure> {
        let application = Handle::new(Self {
            transient: StandardTransient::default(),
            base: TObjApplicationBase::new()?,
        });
        if let Some(own) = application.get() {
            CdmApplication::application_base(own).bind_self(&application.cast());
        }
        Ok(application)
    }
    /// `GetInstance`: the static instance of the application.
    ///
    /// ```
    /// use rustycad_tktobj::tobj::TObjStandardApplication;
    /// use rustycad_tklcaf::tdoc_std::TDocStdApplication;
    /// let application = TObjStandardApplication::get_instance().unwrap();
    /// assert_eq!(application.resources_name(), "TObj");
    /// ```
    pub fn get_instance() -> Result<Handle<dyn TObjApplication>, Failure> {
        let instance = INSTANCE.get_or_init(Self::new);
        instance
            .as_ref()
            .map(|application| application.cast())
            .map_err(Clone::clone)
    }
}

impl CdmApplication for TObjStandardApplication {
    rustycad_tklcaf::tdoc_std_application_cdm_members!();
}
impl CdfApplication for TObjStandardApplication {
    rustycad_tklcaf::tdoc_std_application_cdf_members!();
}
impl TDocStdApplication for TObjStandardApplication {
    fn tdoc_std_application_base(&self) -> &TDocStdApplicationBase {
        self.base.tdoc_std()
    }
    /// `ResourcesName`: `TObj`.
    fn resources_name(&self) -> String {
        String::from("TObj")
    }
}
impl TObjApplication for TObjStandardApplication {
    fn tobj_application_base(&self) -> &TObjApplicationBase {
        &self.base
    }
}
