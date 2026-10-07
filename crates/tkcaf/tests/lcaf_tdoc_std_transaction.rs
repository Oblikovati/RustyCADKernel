//! 1:1 translation of the cases of src/ApplicationFramework/TKLCAF/GTests/TDocStd_Transaction_Test.cxx
//! that use TNaming named shapes and TDataXtd attributes, which the TKCAF crate provides (the
//! other cases are in the TKLCAF crate). The fixture's `BinDrivers::DefineFormat` registers
//! the native Bin format, which is not ported (user decision 2026-09); no case reads or writes
//! a file, so the translation omits that call.
#![allow(non_snake_case)]
mod support;

use rustycad_tkcaf::tnaming::TNamingBuilder;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::TDataStdReal;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};

/// `NewApplicationTransactionDocument(anApplication)`.
fn new_application_transaction_document()
-> (Handle<TDocStdStandardApplication>, Handle<TDocStdDocument>) {
    let application = TDocStdStandardApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document)
        .unwrap();
    (application, document)
}

/// `RootChild(theDocument, theTag)`.
fn root_child(document: &Handle<TDocStdDocument>, tag: i32) -> TdfLabel {
    document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(tag, true)
        .unwrap()
}

/// `DeltaHasAttribute(theDelta, theAttributeName)` on the last undo.
fn last_delta_has(document: &Handle<TDocStdDocument>, attribute_name: &str) -> bool {
    let undos = document.get_undos();
    let delta = undos.last().expect("an undo");
    delta.attribute_deltas().iter().any(|attribute_delta| {
        attribute_delta.attribute().dynamic_type().unwrap().name() == attribute_name
    })
}

/// `ExpectLastDeltaHas(theDocument, first, second)`.
fn expect_last_delta_has(document: &Handle<TDocStdDocument>, first: &str, second: &str) {
    assert!(last_delta_has(document, first), "{first}");
    assert!(last_delta_has(document, second), "{second}");
}

// bugs/caf/bug2932_18: an unchanged shape keeps both deltas.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_18_UnchangedShapeKeepsBothDeltas() {
    let (_application, document) = new_application_transaction_document();
    assert!(!document.is_null());
    document.set_undo_limit(100).unwrap();
    let label = root_child(&document, 2);
    let solid = support::make_box_at([10.0, 20.0, 30.0], 100.0, 200.0, 300.0);
    document.new_command().unwrap();
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    TDataStdReal::set(&label, 300.0).unwrap();
    document.new_command().unwrap();
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    TDataStdReal::set(&label, 400.0).unwrap();
    document.new_command().unwrap();
    expect_last_delta_has(&document, "TDataStd_Real", "TNaming_NamedShape");
}
