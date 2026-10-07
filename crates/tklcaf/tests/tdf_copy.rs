//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDF_Copy_Test.cxx.
#![allow(non_snake_case)]
// The native test writes the literal 3.1415, not PI.
#![allow(clippy::approx_constant)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName, TDataStdReal};
use rustycad_tklcaf::tdf::{
    TdfClosureTool, TdfCopyLabel, TdfCopyTool, TdfDataSet, TdfLabel, TdfRelocationTable,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

fn new_copy_document() -> Handle<TDocStdDocument> {
    TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap()
}

fn root_child(document: &Handle<TDocStdDocument>, tag: i32) -> TdfLabel {
    document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(tag, true)
        .unwrap()
}

fn expect_copied_label(label: &TdfLabel, expected_name: &ExtendedString, expected_real: f64) {
    let mut name: Handle<TDataStdName> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdName::get_id(), &mut name)
            .unwrap(),
        "Name attribute is missing"
    );
    assert!(TDataStdGenericExtString::get(&*name).is_equal(expected_name));
    let mut real: Handle<TDataStdReal> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdReal::get_id(), &mut real)
            .unwrap(),
        "Real attribute is missing"
    );
    assert_eq!(TDataStdReal::get(&real), expected_real);
}

// caf/bugs/buc60813: TDF_CopyLabel can copy a label repeatedly without losing the copied
// name or real attribute.
#[test]
fn TDF_Copy_Test__Buc_CafBug_60813_CopyLabelTwice() {
    let document = new_copy_document();
    document.set_undo_limit(100).unwrap();
    document.new_command().unwrap();
    let source = document.main().unwrap();
    let target1 = root_child(&document, 2);
    let target2 = root_child(&document, 3);
    TDataStdName::set(&source, &ExtendedString::new("SourceLabel")).unwrap();
    TDataStdReal::set(&source, 3.1415).unwrap();
    let mut first_copy = TdfCopyLabel::new();
    first_copy.load(&source, &target1);
    first_copy.perform().unwrap();
    assert!(first_copy.is_done());
    expect_copied_label(&target1, &ExtendedString::new("SourceLabel"), 3.1415);
    let mut second_copy = TdfCopyLabel::new();
    second_copy.load(&target1, &target2);
    second_copy.perform().unwrap();
    assert!(second_copy.is_done());
    expect_copied_label(&target2, &ExtendedString::new("SourceLabel"), 3.1415);
}

// caf/bugs/bug22976: CopyDF copies the content of a label, not only the label itself.
#[test]
fn TDF_Copy_Test__CafBug_22976_CopyDataFrameworkLabel() {
    let document = new_copy_document();
    document.set_undo_limit(100).unwrap();
    document.new_command().unwrap();
    let source = document.main().unwrap();
    let target = root_child(&document, 2);
    TDataStdReal::set(&source, 1.2345).unwrap();
    let data_set = Handle::new(TdfDataSet::new());
    data_set.add_label(&source);
    TdfClosureTool::closure(&data_set).unwrap();
    let relocation = Handle::new(TdfRelocationTable::new(false));
    relocation.set_label_relocation(&source, &target);
    TdfCopyTool::copy(&data_set, &relocation).unwrap();
    let mut copied: Handle<TDataStdReal> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TDataStdReal::get_id(), &mut copied)
            .unwrap()
    );
    assert_eq!(TDataStdReal::get(&copied), 1.2345);
}
