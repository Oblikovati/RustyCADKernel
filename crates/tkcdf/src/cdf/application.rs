// Rust port of ApplicationFramework/TKCDF/CDF/CDF_Application.hxx and .cxx (fields,
// constructor and virtual members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CDF_Application`: an application that opens, retrieves and stores documents through
//! format drivers (ocaf-design.md section 6).
use super::{CdfDirectory, CdfFwosDriver, CdfMetaDataDriver, CdfTypeOfActivation};
use crate::cdm::{CdmApplication, CdmApplicationBase, CdmDocument};
use crate::pcdm::{PcdmReader, PcdmReaderStatus, PcdmRetrievalDriver, PcdmStorageDriver};
use rustycad_tkernel::{
    impl_transient_interface,
    ncollection::IndexedDataMap,
    plugin::{PluginLoader, StaticPluginLibraries},
    resource::{NativeResourceEnvironment, NativeResourceIo},
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
    tcollection::ExtendedString,
};
use std::sync::{Arc, RwLockWriteGuard};

/// `Plugin::Load` as the application calls it: the transient a GUID's plugin makes.
pub type CdfPluginLoad =
    Arc<dyn Fn(&Guid) -> Result<Handle<StandardTransient>, Failure> + Send + Sync>;

/// The native `Plugin::Load`: the "Plugin" resources of the host, libraries the port
/// cannot open natively (none is registered), messages on standard output.
pub fn native_plugin_load(guid: &Guid) -> Result<Handle<StandardTransient>, Failure> {
    let mut loader: PluginLoader<Handle<StandardTransient>> = PluginLoader::new(
        Arc::new(NativeResourceIo),
        Arc::new(NativeResourceEnvironment),
        Box::new(StaticPluginLibraries::default()),
        Box::new(std::io::stdout()),
    );
    loader.load(guid, false)
}

/// `NCollection_IndexedDataMap<TCollection_ExtendedString, occ::handle<PCDM_RetrievalDriver>>`.
pub type CdfReaderMap = IndexedDataMap<ExtendedString, Handle<dyn PcdmRetrievalDriver>>;
/// `NCollection_IndexedDataMap<TCollection_ExtendedString, occ::handle<PCDM_StorageDriver>>`.
pub type CdfWriterMap = IndexedDataMap<ExtendedString, Handle<dyn PcdmStorageDriver>>;

/// The fields of `CDF_Application`, embedding the `CDM_Application` ones;
/// `CdfApplicationBase::default()` is the protected constructor.
pub struct CdfApplicationBase {
    cdm: CdmApplicationBase,
    pub(crate) retrievable_status: SharedState<PcdmReaderStatus>,
    pub(crate) default_folder: SharedState<ExtendedString>,
    meta_data_driver: SharedState<Handle<dyn CdfMetaDataDriver>>,
    directory: SharedState<Handle<CdfDirectory>>,
    readers: SharedState<CdfReaderMap>,
    writers: SharedState<CdfWriterMap>,
    plugin_load: SharedState<CdfPluginLoad>,
}
impl Default for CdfApplicationBase {
    /// `CDF_Application()`: status OK, an empty directory and a `CDF_FWOSDriver` over the
    /// application's meta-data table.
    fn default() -> Self {
        let cdm = CdmApplicationBase::default();
        let driver: Handle<dyn CdfMetaDataDriver> =
            Handle::new(CdfFwosDriver::new(&cdm.shared_meta_data_look_up_table())).cast();
        Self {
            cdm,
            retrievable_status: SharedState::new(PcdmReaderStatus::Ok),
            default_folder: SharedState::default(),
            meta_data_driver: SharedState::new(driver),
            directory: SharedState::new(Handle::new(CdfDirectory::new())),
            readers: SharedState::default(),
            writers: SharedState::default(),
            plugin_load: SharedState::new(Arc::new(native_plugin_load)),
        }
    }
}
impl CdfApplicationBase {
    /// The embedded `CDM_Application` fields.
    pub fn cdm(&self) -> &CdmApplicationBase {
        &self.cdm
    }
    /// The public field `myMetaDataDriver`; `base.meta_data_driver_field()`.
    pub fn meta_data_driver_field(&self) -> Handle<dyn CdfMetaDataDriver> {
        self.meta_data_driver.read().clone()
    }
    /// Assigns the public field `myMetaDataDriver`.
    pub fn set_meta_data_driver(&self, driver: &Handle<dyn CdfMetaDataDriver>) {
        *self.meta_data_driver.write() = driver.clone();
    }
    /// The public field `myDirectory`.
    pub fn directory(&self) -> Handle<CdfDirectory> {
        self.directory.read().clone()
    }
    /// Assigns the public field `myDirectory`.
    pub fn set_directory(&self, directory: &Handle<CdfDirectory>) {
        *self.directory.write() = directory.clone();
    }
    /// The protected `myRetrievableStatus`.
    pub fn retrievable_status(&self) -> PcdmReaderStatus {
        *self.retrievable_status.read()
    }
    /// Assigns the protected `myRetrievableStatus`.
    pub fn set_retrievable_status(&self, status: PcdmReaderStatus) {
        *self.retrievable_status.write() = status;
    }
    /// The protected `myReaders`.
    pub fn readers(&self) -> RwLockWriteGuard<'_, CdfReaderMap> {
        self.readers.write()
    }
    /// The protected `myWriters`.
    pub fn writers(&self) -> RwLockWriteGuard<'_, CdfWriterMap> {
        self.writers.write()
    }
    /// The `Plugin::Load` the application uses (injected for tests and hosts).
    pub fn plugin_load(&self) -> CdfPluginLoad {
        self.plugin_load.read().clone()
    }
    /// Replaces the plugin loader; `base.set_plugin_load(Arc::new(my_loader))`.
    pub fn set_plugin_load(&self, load: CdfPluginLoad) {
        *self.plugin_load.write() = load;
    }
    /// `Load(aGUID)`: the application a plugin makes, null when it is not one.
    pub fn load(guid: &Guid) -> Result<Handle<dyn CdfApplication>, Failure> {
        Ok(native_plugin_load(guid)?.cast())
    }
}

/// `CDF_Application`'s virtual members; a concrete application embeds a
/// [`CdfApplicationBase`] and implements `CdmApplication` with [`crate::cdf_application_cdm_members`].
pub trait CdfApplication: CdmApplication {
    /// The embedded base with the native fields.
    fn cdf_application_base(&self) -> &CdfApplicationBase;
    /// `NewDocument`: does nothing by default.
    fn new_document(
        &self,
        _format: &ExtendedString,
        _document: &mut Handle<dyn CdmDocument>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `InitDocument`: does nothing by default.
    fn init_document(&self, _document: &Handle<dyn CdmDocument>) -> Result<(), Failure> {
        Ok(())
    }
    /// `ReaderFromFormat`: the registered reader, else the `<format>.RetrievalPlugin` plugin.
    fn reader_from_format(
        &self,
        format: &ExtendedString,
    ) -> Result<Handle<dyn PcdmReader>, Failure> {
        super::application_drivers::reader_from_format(
            self.cdf_application_base(),
            &self.resources(),
            format,
        )
    }
    /// `WriterFromFormat`: the registered writer, else the `<format>.StoragePlugin` plugin.
    fn writer_from_format(
        &self,
        format: &ExtendedString,
    ) -> Result<Handle<dyn PcdmStorageDriver>, Failure> {
        super::application_drivers::writer_from_format(
            self.cdf_application_base(),
            &self.resources(),
            format,
        )
    }
    /// `Activate` (private virtual): does nothing by default.
    fn activate(
        &self,
        _document: &Handle<dyn CdmDocument>,
        _type_of_activation: CdfTypeOfActivation,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
impl_transient_interface!(dyn CdfApplication, "CDF_Application", dyn CdmApplication);

/// Fills the `CdmApplication` impl of a concrete `CDF_Application`: the base accessor and
/// the `Retrieve`/`DocumentVersion` overrides of CDF.
#[macro_export]
macro_rules! cdf_application_cdm_members {
    () => {
        fn application_base(&self) -> &$crate::cdm::CdmApplicationBase {
            $crate::cdf::CdfApplication::cdf_application_base(self).cdm()
        }
        fn retrieve(
            &self,
            meta_data: &rustycad_tkernel::standard::Handle<$crate::cdm::CdmMetaData>,
            use_storage_configuration: bool,
            filter: &rustycad_tkernel::standard::Handle<$crate::pcdm::PcdmReaderFilter>,
            range: &rustycad_tkernel::message::ProgressRange<'_>,
        ) -> Result<
            rustycad_tkernel::standard::Handle<dyn $crate::cdm::CdmDocument>,
            rustycad_tkernel::standard::Failure,
        > {
            $crate::cdf::CdfApplicationServices::retrieve_meta_data(
                self,
                meta_data,
                use_storage_configuration,
                true,
                filter,
                range,
            )
        }
        fn document_version(
            &self,
            meta_data: &rustycad_tkernel::standard::Handle<$crate::cdm::CdmMetaData>,
        ) -> Result<i32, rustycad_tkernel::standard::Failure> {
            $crate::cdf::CdfApplicationServices::cdf_document_version(self, meta_data)
        }
    };
}
