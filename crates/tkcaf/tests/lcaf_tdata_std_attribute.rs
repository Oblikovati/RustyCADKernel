//! 1:1 translation of the cases of src/ApplicationFramework/TKLCAF/GTests/TDataStd_Attribute_Test.cxx
//! that use `TDataXtd_Shape`, which the TKCAF crate provides (the other cases are in the
//! TKLCAF crate).
#![allow(non_snake_case)]
mod support;

use rustycad_tkcaf::tdata_xtd::TDataXtdShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

/// `NewDocument()`: a `BinOcaf` document with an undo limit of 10.
fn new_document() -> Handle<TDocStdDocument> {
    let document = support::new_document();
    document.set_undo_limit(10).unwrap();
    document
}

/// `NewAttributeLabel(theDocument)`.
fn new_attribute_label(document: &Handle<TDocStdDocument>) -> TdfLabel {
    support::main_child(document, 2)
}

/// The checks shared by K1 and K2: the shape label exists after the undo and its shape
/// attribute holds `expected`.
fn expect_restored_shape(
    document: &Handle<TDocStdDocument>,
    expected: &rustycad_tkbrep::topods::Shape,
) {
    let label = document.main().unwrap().find_child(2, false).unwrap();
    assert!(!label.is_null(), "Shape label was not restored after undo");
    let mut attribute: Handle<TDataXtdShape> = Handle::default();
    assert!(TDataXtdShape::find(&label, &mut attribute).unwrap());
    assert!(TDataXtdShape::get(&label).unwrap().is_same(expected));
    document.clear_undos();
    document.clear_redos();
}

// caf/basic/K1: a named shape survives ForgetAll/Undo/Redo/Undo.
#[test]
fn TDataStd_Attribute_Test__CafBasic_K1_ShapeUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let shape = support::make_box(100.0, 200.0, 300.0);
    TDataXtdShape::set(&label, &shape).unwrap();

    document.new_command().unwrap();
    label.forget_all_attributes(true).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    document.redo().unwrap();
    document.undo().unwrap();

    expect_restored_shape(&document, &shape);
}

// caf/basic/K2: undoing a replacement restores the first named shape.
#[test]
fn TDataStd_Attribute_Test__CafBasic_K2_ShapeReplacementUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let first = support::make_box(100.0, 200.0, 300.0);
    let second = support::make_box_at([-10.0, -20.0, -30.0], 100.0, 200.0, 300.0);
    TDataXtdShape::set(&label, &first).unwrap();
    document.new_command().unwrap();
    TDataXtdShape::set(&label, &second).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();

    expect_restored_shape(&document, &first);
}
