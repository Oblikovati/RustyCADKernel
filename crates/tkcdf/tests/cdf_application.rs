//! 1:1 translation of src/ApplicationFramework/TKCDF/GTests/CDF_Application_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkcdf::{
    SharedResourceManager, cdf::CdfApplication, cdf::CdfApplicationBase,
    cdf::CdfApplicationServices, cdf_application_cdm_members, cdm::CdmApplication,
};
use rustycad_tkernel::{
    impl_transient,
    resource::{ResourceHost, ResourceManager},
    standard::{Handle, SharedState, StandardTransient},
    tcollection::ExtendedString,
};
use std::sync::Arc;

/// `CDF_TestApplication`: a CDF application with its own empty resource manager.
struct CdfTestApplication {
    transient: StandardTransient,
    base: CdfApplicationBase,
    resources: SharedResourceManager,
}
impl_transient!(CdfTestApplication, "CDF_TestApplication", dyn CdfApplication, transient,
    interfaces: [dyn CdmApplication, dyn CdfApplication]);
impl CdmApplication for CdfTestApplication {
    cdf_application_cdm_members!();
    fn resources(&self) -> SharedResourceManager {
        self.resources.clone()
    }
}
impl CdfApplication for CdfTestApplication {
    fn cdf_application_base(&self) -> &CdfApplicationBase {
        &self.base
    }
}

fn new_application() -> Handle<CdfTestApplication> {
    let manager = ResourceManager::empty(ResourceHost::native());
    let application = Handle::new(CdfTestApplication {
        transient: StandardTransient::default(),
        base: CdfApplicationBase::default(),
        resources: Arc::new(SharedState::new(manager)),
    });
    application
        .application_base()
        .bind_self(&application.cast());
    application
}

#[test]
fn CDF_ApplicationTest__FormatFromExtensionResource() {
    let application = new_application();
    application
        .resources()
        .write()
        .set_resource("MiXeD.FileFormat", b"Test format");
    let mut format = ExtendedString::default();
    assert!(
        application
            .format(&ExtendedString::new("document.MiXeD"), &mut format)
            .unwrap()
    );
    assert_eq!(format, ExtendedString::new("Test format"));
}

#[test]
fn CDF_ApplicationTest__ExtensionlessFileHasNoFormat() {
    let application = new_application();
    application
        .resources()
        .write()
        .set_resource(".FileFormat", b"Invalid fallback");
    let mut format = ExtendedString::default();
    assert!(
        !application
            .format(&ExtendedString::new("document"), &mut format)
            .unwrap()
    );
    assert!(format.is_empty());
}

#[test]
fn CDF_ApplicationTest__EmptyExtensionHasNoFormat() {
    let application = new_application();
    let mut format = ExtendedString::default();
    assert!(
        !application
            .format(&ExtendedString::new("document."), &mut format)
            .unwrap()
    );
    assert!(format.is_empty());
}
