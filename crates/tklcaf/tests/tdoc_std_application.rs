//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDocStd_Application_Test.cxx.
//!
//! The native fixture first calls `BinDrivers::DefineFormat(anApplication)`, which registers
//! the drivers of the native Bin format; native file formats are not ported (user decision
//! 2026-09) and the test never reads or writes a file, so the translation omits that call.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdoc_std::{
    TDocStdApplication, TDocStdDocument, TDocStdOwner, TDocStdStandardApplication,
};

// bugs/caf/bug159: closing an OCAF document clears its owner link and releases the
// application's reference to the document.
#[test]
fn TDocStd_Application_Test__CafBug_159_OwnerClearedOnClose() {
    let application = TDocStdStandardApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document)
        .unwrap();
    assert!(!document.is_null());
    let reference_count_before_close = document.ref_count();
    let mut owner: Handle<TDocStdOwner> = Handle::default();
    assert!(
        document
            .main()
            .unwrap()
            .root()
            .unwrap()
            .find_attribute_as(&TDocStdOwner::get_id(), &mut owner)
            .unwrap()
    );
    let mut owned_document = owner.get_document();
    assert!(!owned_document.is_null());
    assert_eq!(owned_document.object_address(), document.object_address());
    owned_document = Handle::default();
    drop(owned_document);
    assert!(application.close_tdoc_std_document(&document).is_ok());
    assert!(owner.get_document().is_null());
    assert!(reference_count_before_close - document.ref_count() >= 1);
}
