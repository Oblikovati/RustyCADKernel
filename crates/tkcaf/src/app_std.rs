// Rust port of ApplicationFramework/TKCAF/AppStd/AppStd_Application.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppStd_Application`: the standard OCAF application, whose resources are `Standard`.
use rustycad_tkcdf::cdf::CdfApplication;
use rustycad_tkcdf::cdm::CdmApplication;
use rustycad_tkcdf::json_dump::{base_class, class_begin};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, StandardTransient, dump::DumpStream};
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdApplicationBase};
use std::io;

/// `AppStd_Application`.
///
/// ```
/// use rustycad_tkcaf::app_std::AppStdApplication;
/// use rustycad_tklcaf::tdoc_std::TDocStdApplication;
/// let application = AppStdApplication::new();
/// assert_eq!(application.resources_name(), "Standard");
/// ```
#[derive(Default)]
pub struct AppStdApplication {
    transient: StandardTransient,
    base: TDocStdApplicationBase,
}
impl_transient!(AppStdApplication, "AppStd_Application", dyn TDocStdApplication, transient,
    interfaces: [dyn CdmApplication, dyn CdfApplication, dyn TDocStdApplication]);

impl AppStdApplication {
    /// `new AppStd_Application()`, already in its bound handle.
    pub fn new() -> Handle<AppStdApplication> {
        let application = Handle::new(Self::default());
        if let Some(own) = application.get() {
            CdmApplication::application_base(own).bind_self(&application.cast());
        }
        application
    }
    /// `DumpJson` (it hides the non-virtual `TDocStd_Application::DumpJson`): the class name,
    /// then the `TDocStd_Application` dump as its base.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "AppStd_Application")?;
        base_class(stream, depth, &|s, d| {
            TDocStdApplication::dump_json(self, s, d)
        })
    }
}

impl CdmApplication for AppStdApplication {
    rustycad_tklcaf::tdoc_std_application_cdm_members!();
}
impl CdfApplication for AppStdApplication {
    rustycad_tklcaf::tdoc_std_application_cdf_members!();
}
impl TDocStdApplication for AppStdApplication {
    fn tdoc_std_application_base(&self) -> &TDocStdApplicationBase {
        &self.base
    }
    /// `ResourcesName`: `Standard`.
    fn resources_name(&self) -> String {
        String::from("Standard")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::tcollection::ExtendedString;
    use rustycad_tklcaf::tdoc_std::TDocStdDocument;

    #[test]
    fn documents_are_created_and_dumped_with_the_base() {
        let application = AppStdApplication::new();
        let mut document: Handle<TDocStdDocument> = Handle::default();
        application
            .new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document)
            .unwrap();
        assert_eq!(application.nb_documents().unwrap(), 1);
        let mut stream = DumpStream::default();
        application.dump_json(&mut stream, -1).unwrap();
        let text = String::from_utf8_lossy(stream.bytes()).into_owned();
        let expected = concat!(
            "\"className\": \"AppStd_Application\", ",
            "\"className\": \"TDocStd_Application\", \"IsDriverLoaded\": 1"
        );
        assert_eq!(text, expected);
    }
}
