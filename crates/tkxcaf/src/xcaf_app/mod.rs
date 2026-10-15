// Rust port of DataExchange/TKXCAF/XCAFApp/XCAFApp_Application.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The XCAFApp package: `XCAFApp_Application`, the application of XDE documents.
use crate::xcaf_doc::XCAFDocDocumentTool;
use crate::xcaf_prs::XCAFPrsDriver;
use rustycad_tkcdf::cdf::CdfApplication;
use rustycad_tkcdf::cdm::CdmDocument;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdApplicationBase, TDocStdDocument};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_integer};
use rustycad_tkvcaf::tprs_std::TPrsStdDriverTable;
use std::io;
use std::sync::OnceLock;

/// `XCAFApp_Application`: a `TDocStd_Application` whose new documents get the XDE tools.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkernel::tcollection::ExtendedString;
/// use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument};
/// use rustycad_tkxcaf::xcaf_app::XCAFAppApplication;
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocDocumentTool;
/// let application = XCAFAppApplication::new();
/// let mut document: Handle<TDocStdDocument> = Handle::default();
/// application.new_tdoc_std_document(&ExtendedString::from_utf8("BinXCAF"), &mut document)?;
/// assert!(XCAFDocDocumentTool::is_xcaf_document(&document)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFAppApplication {
    transient: StandardTransient,
    base: TDocStdApplicationBase,
}
rustycad_tkernel::impl_transient!(XCAFAppApplication, "XCAFApp_Application", dyn TDocStdApplication, transient,
    interfaces: [dyn rustycad_tkcdf::cdm::CdmApplication, dyn CdfApplication, dyn TDocStdApplication]);

impl XCAFAppApplication {
    /// `XCAFApp_Application()`, in its bound handle: registers the `XCAFPrs_Driver`.
    pub fn new() -> Handle<XCAFAppApplication> {
        let application = Handle::new(Self::default());
        if let Some(own) = application.get() {
            rustycad_tkcdf::cdm::CdmApplication::application_base(own)
                .bind_self(&application.cast());
        }
        if let Some(table) = TPrsStdDriverTable::get().get() {
            table.add_driver(
                &XCAFPrsDriver::get_id(),
                &Handle::new(XCAFPrsDriver::new()).cast(),
            );
        }
        application
    }
    /// `GetApplication`: the process-wide application, created once.
    pub fn get_application() -> Handle<XCAFAppApplication> {
        static APPLICATION: OnceLock<Handle<XCAFAppApplication>> = OnceLock::new();
        APPLICATION.get_or_init(Self::new).clone()
    }
}

impl rustycad_tkcdf::cdm::CdmApplication for XCAFAppApplication {
    rustycad_tklcaf::tdoc_std_application_cdm_members!();
}

impl CdfApplication for XCAFAppApplication {
    rustycad_tklcaf::tdoc_std_application_cdf_members!();
    /// `InitDocument(aDoc)`: the document tool of the main label (`Standard_NullObject` for
    /// a document that is not a `TDocStd_Document`).
    fn init_document(&self, document: &Handle<dyn CdmDocument>) -> Result<(), Failure> {
        let document = document.cast::<TDocStdDocument>();
        let main = rustycad_tklcaf::live(&document, "XCAFApp_Application::InitDocument")?.main()?;
        XCAFDocDocumentTool::set(&main, true)?;
        Ok(())
    }
}

impl TDocStdApplication for XCAFAppApplication {
    fn tdoc_std_application_base(&self) -> &TDocStdApplicationBase {
        &self.base
    }
    /// `ResourcesName`: `XCAF`.
    fn resources_name(&self) -> String {
        "XCAF".to_owned()
    }
    /// `DumpJson`: the class, then the `TDocStd_Application` dump.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFApp_Application")?;
        base_class(stream, depth, &|s, _| {
            class_begin(s, "TDocStd_Application")?;
            field_integer(s, "myIsDriverLoaded", i32::from(self.is_driver_loaded()))
        })
    }
}
