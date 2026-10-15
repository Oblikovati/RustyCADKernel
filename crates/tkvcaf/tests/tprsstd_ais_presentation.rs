//! 1:1 translation of src/ApplicationFramework/TKVCAF/GTests/TPrsStd_AISPresentation_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkcaf::tdata_xtd::TDataXtdConstraint;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkvcaf::tprs_std::TPrsStdAISPresentation;

// Migrated from tests/bugs/caf/buc60910.
// The DRAW case checks presentation mode state only; it does not create a
// viewer or otherwise exercise visualization output.
#[test]
fn TPrsStd_AISPresentationTest__CafBug_60910_ModeState() {
    let document = TDocStdDocument::new(&ExtendedString::from_utf8("BinOcaf")).unwrap();
    let label = document.main().unwrap().find_child(2, true).unwrap();
    let presentation = TPrsStdAISPresentation::set(&label, &TDataXtdConstraint::get_id()).unwrap();
    assert!(!presentation.is_null());

    assert!(!presentation.has_own_mode().unwrap());
    presentation.set_mode(3).unwrap();
    assert_eq!(presentation.mode().unwrap(), 3);
    assert!(presentation.has_own_mode().unwrap());
    presentation.unset_mode().unwrap();
    assert!(!presentation.has_own_mode().unwrap());
}
