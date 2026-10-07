//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDataStd_Attribute_Test.cxx.
//!
//! `CafBasic_K1_ShapeUndo` and `CafBasic_K2_ShapeReplacementUndo` need `TDataXtd_Shape`,
//! `TNaming` and `BRepPrimAPI_MakeBox` (TKCAF and TKPrim); the ledger marks them
//! `pending_dependency`.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::{Guid, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::{
    TDataStdAsciiString, TDataStdBooleanArray, TDataStdBooleanList, TDataStdByteArray,
    TDataStdComment, TDataStdExtStringArray, TDataStdExtStringList, TDataStdGenericExtString,
    TDataStdInteger, TDataStdIntegerArray, TDataStdIntegerList, TDataStdName, TDataStdReal,
    TDataStdRealArray, TDataStdRealList, TDataStdReferenceArray, TDataStdReferenceList,
    TDataStdUAttribute,
};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfLabel, TdfReference};
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};

fn guid(text: &str) -> Guid {
    guid_from_text(text)
}

fn text(value: &str) -> ExtendedString {
    ExtendedString::new(value)
}

fn new_document() -> Handle<TDocStdDocument> {
    let document = TDocStdDocument::new(&text("BinOcaf")).unwrap();
    document.set_undo_limit(10).unwrap();
    document
}

fn new_attribute_label(document: &Handle<TDocStdDocument>) -> TdfLabel {
    document.main().unwrap().find_child(2, true).unwrap()
}

fn expect_real_array(array: &Handle<TDataStdRealArray>, first: f64, second: f64) {
    assert!(!array.is_null());
    assert_eq!(array.lower(), 1);
    assert_eq!(array.upper(), 2);
    assert_eq!(array.length(), 2);
    assert_eq!(array.value(1).unwrap(), first);
    assert_eq!(array.value(2).unwrap(), second);
}

fn expect_integer_array(array: &Handle<TDataStdIntegerArray>, first: i32, second: i32) {
    assert!(!array.is_null());
    assert_eq!(array.lower(), 1);
    assert_eq!(array.upper(), 2);
    assert_eq!(array.length(), 2);
    assert_eq!(array.value(1).unwrap(), first);
    assert_eq!(array.value(2).unwrap(), second);
}

/// `NewCommand; ForgetAllAttributes; NewCommand; Undo; Redo; Undo` of the R5-V5 tests.
fn forget_all_then_undo_redo_undo(document: &Handle<TDocStdDocument>, label: &TdfLabel) {
    document.new_command().unwrap();
    label.forget_all_attributes(true).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    document.redo().unwrap();
    document.undo().unwrap();
}

/// `aLab.AddAttribute(att); EXPECT_NE(aNullGuid, att->ID())`.
fn expect_added_with_guid(label: &TdfLabel, attribute: Handle<dyn TdfAttribute>, null_guid: &Guid) {
    label.add_attribute(&attribute, false).unwrap();
    assert_ne!(*null_guid, attribute.id());
}

#[test]
fn TDataStd_Attribute_Test__OCC29371_AttributeGUIDsNotNull() {
    let application = TDocStdStandardApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&text("BinOcaf"), &mut document)
        .unwrap();
    let label = document.main().unwrap();
    let null_guid = guid("00000000-0000-0000-0000-000000000000");
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdAsciiString::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdBooleanArray::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdBooleanList::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdByteArray::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdExtStringArray::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdExtStringList::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdInteger::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdIntegerArray::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdIntegerList::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(&label, Handle::new(TDataStdName::new()).cast(), &null_guid);
    expect_added_with_guid(&label, Handle::new(TDataStdReal::new()).cast(), &null_guid);
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdRealArray::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdRealList::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdReferenceArray::new()).cast(),
        &null_guid,
    );
    expect_added_with_guid(
        &label,
        Handle::new(TDataStdReferenceList::new()).cast(),
        &null_guid,
    );
    application.close_tdoc_std_document(&document).unwrap();
}

// caf/basic/A1: setting and retrieving the default integer attribute.
#[test]
fn TDataStd_Attribute_Test__CafBasic_A1_IntegerSetGet() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let integer = TDataStdInteger::set(&label, 100).unwrap();
    assert!(!integer.is_null());
    assert_eq!(TDataStdInteger::get(&integer), 100);
}

// caf/basic/A2: undoing a replacement restores the previous integer value.
#[test]
fn TDataStd_Attribute_Test__CafBasic_A2_IntegerUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    TDataStdInteger::set(&label, 100).unwrap();
    document.new_command().unwrap();
    TDataStdInteger::set(&label, 200).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut integer: Handle<TDataStdInteger> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdInteger::get_id(), &mut integer)
            .unwrap()
    );
    assert_eq!(TDataStdInteger::get(&integer), 100);
}

// caf/basic/A4-A5: two user GUIDs identify independent integer attributes.
#[test]
fn TDataStd_Attribute_Test__CafBasic_A4_IntegerUserGuids() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94541-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94542-6dbc-11d4-b9c8-0060b0ee281b");
    TDataStdInteger::set_with_id(&label, &guid1, 100).unwrap();
    TDataStdInteger::set_with_id(&label, &guid2, 200).unwrap();
    let mut integer1: Handle<TDataStdInteger> = Handle::default();
    let mut integer2: Handle<TDataStdInteger> = Handle::default();
    assert!(label.find_attribute_as(&guid1, &mut integer1).unwrap());
    assert!(label.find_attribute_as(&guid2, &mut integer2).unwrap());
    assert_eq!(TDataStdInteger::get(&integer1), 100);
    assert_eq!(TDataStdInteger::get(&integer2), 200);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_A5_IntegerUserGuidUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94541-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94542-6dbc-11d4-b9c8-0060b0ee281b");
    TDataStdInteger::set_with_id(&label, &guid1, 100).unwrap();
    TDataStdInteger::set_with_id(&label, &guid2, 200).unwrap();
    document.new_command().unwrap();
    TDataStdInteger::set_with_id(&label, &guid1, 199).unwrap();
    TDataStdInteger::set_with_id(&label, &guid2, 299).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut integer1: Handle<TDataStdInteger> = Handle::default();
    let mut integer2: Handle<TDataStdInteger> = Handle::default();
    assert!(label.find_attribute_as(&guid1, &mut integer1).unwrap());
    assert!(label.find_attribute_as(&guid2, &mut integer2).unwrap());
    assert_eq!(TDataStdInteger::get(&integer1), 100);
    assert_eq!(TDataStdInteger::get(&integer2), 200);
}

// caf/basic/B1-B2: scalar real values support replacement and undo.
#[test]
fn TDataStd_Attribute_Test__CafBasic_B1_RealSetGet() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let real = TDataStdReal::set(&label, 100.0).unwrap();
    assert!(!real.is_null());
    assert_eq!(TDataStdReal::get(&real), 100.0);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_B2_RealUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    TDataStdReal::set(&label, 100.0).unwrap();
    document.new_command().unwrap();
    TDataStdReal::set(&label, 200.0).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut real: Handle<TDataStdReal> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdReal::get_id(), &mut real)
            .unwrap()
    );
    assert_eq!(TDataStdReal::get(&real), 100.0);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_B4_RealUserGuids() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94551-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94552-6dbc-11d4-b9c8-0060b0ee281b");
    TDataStdReal::set_with_id(&label, &guid1, 100.11).unwrap();
    TDataStdReal::set_with_id(&label, &guid2, 200.11).unwrap();
    let mut real1: Handle<TDataStdReal> = Handle::default();
    let mut real2: Handle<TDataStdReal> = Handle::default();
    assert!(label.find_attribute_as(&guid1, &mut real1).unwrap());
    assert!(label.find_attribute_as(&guid2, &mut real2).unwrap());
    assert_eq!(TDataStdReal::get(&real1), 100.11);
    assert_eq!(TDataStdReal::get(&real2), 200.11);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_B5_RealUserGuidUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94551-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94552-6dbc-11d4-b9c8-0060b0ee281b");
    TDataStdReal::set_with_id(&label, &guid1, 100.11).unwrap();
    TDataStdReal::set_with_id(&label, &guid2, 200.11).unwrap();
    document.new_command().unwrap();
    TDataStdReal::set_with_id(&label, &guid1, 199.11).unwrap();
    TDataStdReal::set_with_id(&label, &guid2, 299.11).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut real1: Handle<TDataStdReal> = Handle::default();
    let mut real2: Handle<TDataStdReal> = Handle::default();
    assert!(label.find_attribute_as(&guid1, &mut real1).unwrap());
    assert!(label.find_attribute_as(&guid2, &mut real2).unwrap());
    assert_eq!(TDataStdReal::get(&real1), 100.11);
    assert_eq!(TDataStdReal::get(&real2), 200.11);
}

// caf/basic/C1-C2: real arrays retain bounds, values, and undo state.
#[test]
fn TDataStd_Attribute_Test__CafBasic_C1_RealArraySetGet() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let array = TDataStdRealArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, 3.0).unwrap();
    array.set_value(2, 4.0).unwrap();
    expect_real_array(&array, 3.0, 4.0);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_C2_RealArrayUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let array = TDataStdRealArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, 3.0).unwrap();
    array.set_value(2, 4.0).unwrap();
    document.new_command().unwrap();
    array.set_value(1, 33.0).unwrap();
    array.set_value(2, 44.0).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    expect_real_array(&array, 3.0, 4.0);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_C4_RealArrayUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94511-6dbc-11d4-b9c8-0060b0ee281b");
    let array = TDataStdRealArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, 3.0).unwrap();
    array.set_value(2, 4.0).unwrap();
    let guid_array = TDataStdRealArray::set_with_id(&label, &user_guid, 1, 2, false).unwrap();
    guid_array.set_value(1, 3.0).unwrap();
    guid_array.set_value(2, 4.0).unwrap();
    expect_real_array(&array, 3.0, 4.0);
    expect_real_array(&guid_array, 3.0, 4.0);
    assert_eq!(guid_array.id(), user_guid);
}

// caf/basic/D1-D2: integer arrays mirror real-array behavior.
#[test]
fn TDataStd_Attribute_Test__CafBasic_D1_IntegerArraySetGet() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let array = TDataStdIntegerArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, 3).unwrap();
    array.set_value(2, 4).unwrap();
    expect_integer_array(&array, 3, 4);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_D2_IntegerArrayUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let array = TDataStdIntegerArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, 3).unwrap();
    array.set_value(2, 4).unwrap();
    document.new_command().unwrap();
    array.set_value(1, 33).unwrap();
    array.set_value(2, 44).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    expect_integer_array(&array, 3, 4);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_D4_IntegerArrayDefaultGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let array = TDataStdIntegerArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, 3).unwrap();
    array.set_value(2, 4).unwrap();
    assert_eq!(array.id(), TDataStdIntegerArray::get_id());
    expect_integer_array(&array, 3, 4);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_D5_IntegerArrayUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94511-6dbc-11d4-b9c8-0060b0ee281b");
    let array = TDataStdIntegerArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, 3).unwrap();
    array.set_value(2, 4).unwrap();
    let guid_array = TDataStdIntegerArray::set_with_id(&label, &user_guid, 1, 2, false).unwrap();
    guid_array.set_value(1, 3).unwrap();
    guid_array.set_value(2, 4).unwrap();
    expect_integer_array(&array, 3, 4);
    expect_integer_array(&guid_array, 3, 4);
    assert_eq!(guid_array.id(), user_guid);
}

// caf/basic/E1-E2: names support setting and undoing a replacement.
#[test]
fn TDataStd_Attribute_Test__CafBasic_E1_NameSetGet() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let name = TDataStdName::set(&label, &text("New Attribute")).unwrap();
    assert!(!name.is_null());
    assert!(TDataStdGenericExtString::get(&*name).is_equal(text("New Attribute")));
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_E2_NameUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let name = TDataStdName::set(&label, &text("New Attribute")).unwrap();
    document.new_command().unwrap();
    name.set_value(&text("Old Attribute")).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert!(TDataStdGenericExtString::get(&*name).is_equal(text("New Attribute")));
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_E4_NameUserGuids() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94561-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94562-6dbc-11d4-b9c8-0060b0ee281b");
    let name1 = TDataStdName::set_with_id(&label, &guid1, &text("New Attribute1")).unwrap();
    let name2 = TDataStdName::set_with_id(&label, &guid2, &text("New Attribute2")).unwrap();
    assert!(!name1.is_null());
    assert!(!name2.is_null());
    assert!(TDataStdGenericExtString::get(&*name1).is_equal(text("New Attribute1")));
    assert!(TDataStdGenericExtString::get(&*name2).is_equal(text("New Attribute2")));
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_E5_NameUserGuidUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94561-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94562-6dbc-11d4-b9c8-0060b0ee281b");
    let name1 = TDataStdName::set_with_id(&label, &guid1, &text("New Attribute1")).unwrap();
    let name2 = TDataStdName::set_with_id(&label, &guid2, &text("New Attribute2")).unwrap();
    document.new_command().unwrap();
    name1.set_value(&text("Modified Attribute1")).unwrap();
    name2.set_value(&text("Modified Attribute2")).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert!(TDataStdGenericExtString::get(&*name1).is_equal(text("New Attribute1")));
    assert!(TDataStdGenericExtString::get(&*name2).is_equal(text("New Attribute2")));
}

// caf/basic/F1-F2: comments use the same generic extended-string behavior.
#[test]
fn TDataStd_Attribute_Test__CafBasic_F1_CommentSetGet() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let comment = TDataStdComment::set(&label, &text("New Attribute")).unwrap();
    assert!(!comment.is_null());
    assert!(TDataStdGenericExtString::get(&*comment).is_equal(text("New Attribute")));
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_F2_CommentUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let comment = TDataStdComment::set(&label, &text("New Attribute")).unwrap();
    document.new_command().unwrap();
    comment.set_value(&text("Old Attribute")).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert!(TDataStdGenericExtString::get(&*comment).is_equal(text("New Attribute")));
}

// caf/basic/J1-J5: UAttributes are identified by their local GUID.
#[test]
fn TDataStd_Attribute_Test__CafBasic_J1_UAttributeSetGet() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let local_guid = guid("c73bd075-22ee-11d2-acde-080009dc4422");
    let attribute = TDataStdUAttribute::set(&label, &local_guid).unwrap();
    assert!(!attribute.is_null());
    assert_eq!(attribute.id(), local_guid);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_J2_UAttributeUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("c73bd075-22ee-11d2-acde-080009dc4422");
    let guid2 = guid("425a8e02-8934-11d2-b3f0-0060b0f1d69e");
    TDataStdUAttribute::set(&label, &guid1).unwrap();
    document.new_command().unwrap();
    label.forget_attribute_id(&guid1).unwrap();
    TDataStdUAttribute::set(&label, &guid2).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut attribute: Handle<TDataStdUAttribute> = Handle::default();
    assert!(label.find_attribute_as(&guid1, &mut attribute).unwrap());
    assert_eq!(attribute.id(), guid1);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_J4_UAttributeMultipleGuids() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("c73bd075-22ee-11d2-acde-080009dc4422");
    let guid2 = guid("c73bd076-22ee-11d2-acde-080009dc4422");
    TDataStdUAttribute::set(&label, &guid1).unwrap();
    TDataStdUAttribute::set(&label, &guid2).unwrap();
    assert!(label.is_attribute(&guid1).unwrap());
    assert!(label.is_attribute(&guid2).unwrap());
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_J5_UAttributeCopy() {
    let document = new_document();
    let source_label = new_attribute_label(&document);
    let target_label = document.main().unwrap().find_child(3, true).unwrap();
    let guid1 = guid("c73bd075-22ee-11d2-acde-080009dc4422");
    let guid2 = guid("c73bd076-22ee-11d2-acde-080009dc4422");
    TDataStdUAttribute::set(&source_label, &guid1).unwrap();
    TDataStdUAttribute::set(&source_label, &guid2).unwrap();
    let mut source1: Handle<TDataStdUAttribute> = Handle::default();
    let mut source2: Handle<TDataStdUAttribute> = Handle::default();
    assert!(
        source_label
            .find_attribute_as(&guid1, &mut source1)
            .unwrap()
    );
    assert!(
        source_label
            .find_attribute_as(&guid2, &mut source2)
            .unwrap()
    );
    target_label
        .add_attribute(&source1.new_empty(), false)
        .unwrap();
    target_label
        .add_attribute(&source2.new_empty(), false)
        .unwrap();
    assert!(target_label.is_attribute(&guid1).unwrap());
    assert!(target_label.is_attribute(&guid2).unwrap());
}

// caf/basic/L1-L2: references store and restore target labels.
#[test]
fn TDataStd_Attribute_Test__CafBasic_L1_ReferenceSetGet() {
    let document = new_document();
    let target = document.main().unwrap().find_child(3, true).unwrap();
    let reference_label = document.main().unwrap().find_child(5, true).unwrap();
    let reference = TdfReference::set(&reference_label, &target).unwrap();
    assert!(!reference.is_null());
    assert_eq!(TdfReference::get(&reference), target);
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_L2_ReferenceUndo() {
    let document = new_document();
    let target1 = document.main().unwrap().find_child(3, true).unwrap();
    let target2 = document.main().unwrap().find_child(4, true).unwrap();
    let reference_label = document.main().unwrap().find_child(5, true).unwrap();
    let reference = TdfReference::set(&reference_label, &target1).unwrap();
    document.new_command().unwrap();
    reference.set_origin(&target2).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert_eq!(TdfReference::get(&reference), target1);
}

// caf/basic/M5: extension-string arrays support a second user-defined GUID.
#[test]
fn TDataStd_Attribute_Test__CafBasic_M5_ExtStringArrayUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94515-6dbc-11d4-b9c8-0060b0ee281b");
    let array = TDataStdExtStringArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, text("aaaaa")).unwrap();
    array.set_value(2, text("bbbbbbb")).unwrap();
    let guid_array = TDataStdExtStringArray::set_with_id(&label, &user_guid, 1, 2, false).unwrap();
    guid_array.set_value(1, text("aaaaa")).unwrap();
    guid_array.set_value(2, text("bbbbbbb")).unwrap();
    assert_eq!(array.length(), 2);
    assert_eq!(guid_array.length(), 2);
    assert!(
        array
            .value(1)
            .unwrap()
            .is_equal(guid_array.value(1).unwrap())
    );
    assert!(
        array
            .value(2)
            .unwrap()
            .is_equal(guid_array.value(2).unwrap())
    );
    assert_eq!(guid_array.id(), user_guid);
}

// caf/basic/M1-M2: extension-string arrays preserve values and undo changes.
#[test]
fn TDataStd_Attribute_Test__CafBasic_M1_ExtStringArrayUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let array = TDataStdExtStringArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, text("TDataStd")).unwrap();
    array.set_value(2, text("ExtStringArray")).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(array.length(), 2);
    assert!(array.value(1).unwrap().is_equal(text("TDataStd")));
    assert!(array.value(2).unwrap().is_equal(text("ExtStringArray")));
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_M2_ExtStringArrayReplacementUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let array = TDataStdExtStringArray::set(&label, 1, 2, false).unwrap();
    array.set_value(1, text("TDataStd")).unwrap();
    array.set_value(2, text("ExtStringArray")).unwrap();
    document.new_command().unwrap();
    array.set_value(1, text("NEW TDataStd")).unwrap();
    array.set_value(2, text("NEW ExtStringArray")).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert!(array.value(1).unwrap().is_equal(text("TDataStd")));
    assert!(array.value(2).unwrap().is_equal(text("ExtStringArray")));
}

// caf/basic/N4-N5: user-defined ASCII-string attributes are independent and undoable.
#[test]
fn TDataStd_Attribute_Test__CafBasic_N4_AsciiStringUserGuids() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94571-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94572-6dbc-11d4-b9c8-0060b0ee281b");
    TDataStdAsciiString::set_with_id(&label, &guid1, &AsciiString::new("New Attribute1")).unwrap();
    TDataStdAsciiString::set_with_id(&label, &guid2, &AsciiString::new("New Attribute2")).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    let mut attribute1: Handle<TDataStdAsciiString> = Handle::default();
    let mut attribute2: Handle<TDataStdAsciiString> = Handle::default();
    assert!(label.find_attribute_as(&guid1, &mut attribute1).unwrap());
    assert!(label.find_attribute_as(&guid2, &mut attribute2).unwrap());
    assert_eq!(
        TDataStdAsciiString::get(&attribute1),
        AsciiString::new("New Attribute1")
    );
    assert_eq!(
        TDataStdAsciiString::get(&attribute2),
        AsciiString::new("New Attribute2")
    );
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_N5_AsciiStringReplacementUndo() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let guid1 = guid("12e94571-6dbc-11d4-b9c8-0060b0ee281b");
    let guid2 = guid("12e94572-6dbc-11d4-b9c8-0060b0ee281b");
    TDataStdAsciiString::set_with_id(&label, &guid1, &AsciiString::new("Attribute_1")).unwrap();
    TDataStdAsciiString::set_with_id(&label, &guid2, &AsciiString::new("Attribute_2")).unwrap();
    document.new_command().unwrap();
    TDataStdAsciiString::set_with_id(&label, &guid1, &AsciiString::new("Modified_1")).unwrap();
    TDataStdAsciiString::set_with_id(&label, &guid2, &AsciiString::new("Modified_2")).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut attribute1: Handle<TDataStdAsciiString> = Handle::default();
    let mut attribute2: Handle<TDataStdAsciiString> = Handle::default();
    assert!(label.find_attribute_as(&guid1, &mut attribute1).unwrap());
    assert!(label.find_attribute_as(&guid2, &mut attribute2).unwrap());
    assert_eq!(
        TDataStdAsciiString::get(&attribute1),
        AsciiString::new("Attribute_1")
    );
    assert_eq!(
        TDataStdAsciiString::get(&attribute2),
        AsciiString::new("Attribute_2")
    );
}

// caf/basic/O5-P5: array attributes can be restored under a second GUID.
#[test]
fn TDataStd_Attribute_Test__CafBasic_O5_BooleanArrayUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94516-6dbc-11d4-b9c8-0060b0ee281b");
    let default = TDataStdBooleanArray::set(&label, 1, 2).unwrap();
    let custom = TDataStdBooleanArray::set_with_id(&label, &user_guid, 1, 2).unwrap();
    default.set_value(1, false).unwrap();
    default.set_value(2, true).unwrap();
    custom.set_value(1, false).unwrap();
    custom.set_value(2, true).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.length(), 2);
    assert_eq!(custom.length(), 2);
    assert_eq!(default.value(1).unwrap(), custom.value(1).unwrap());
    assert_eq!(default.value(2).unwrap(), custom.value(2).unwrap());
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_P5_ByteArrayUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94517-6dbc-11d4-b9c8-0060b0ee281b");
    let default = TDataStdByteArray::set(&label, 1, 2, false).unwrap();
    let custom = TDataStdByteArray::set_with_id(&label, &user_guid, 1, 2, false).unwrap();
    default.set_value(1, 10).unwrap();
    default.set_value(2, 12).unwrap();
    custom.set_value(1, 10).unwrap();
    custom.set_value(2, 12).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.value(1).unwrap(), custom.value(1).unwrap());
    assert_eq!(default.value(2).unwrap(), custom.value(2).unwrap());
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_Q5_ReferenceArrayUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94518-6dbc-11d4-b9c8-0060b0ee281b");
    let target1 = document.main().unwrap().find_child(3, true).unwrap();
    let target2 = document.main().unwrap().find_child(4, true).unwrap();
    let default = TDataStdReferenceArray::set(&label, 1, 2).unwrap();
    let custom = TDataStdReferenceArray::set_with_id(&label, &user_guid, 1, 2).unwrap();
    default.set_value(1, target1.clone()).unwrap();
    default.set_value(2, target2.clone()).unwrap();
    custom.set_value(1, target1).unwrap();
    custom.set_value(2, target2).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.value(1).unwrap(), custom.value(1).unwrap());
    assert_eq!(default.value(2).unwrap(), custom.value(2).unwrap());
}

// caf/basic/R5-V5: list attributes preserve values for default and custom GUIDs.
#[test]
fn TDataStd_Attribute_Test__CafBasic_R5_RealListUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94521-6dbc-11d4-b9c8-0060b0ee281b");
    let default = TDataStdRealList::set(&label).unwrap();
    let custom = TDataStdRealList::set_with_id(&label, &user_guid).unwrap();
    default.append(3.0).unwrap();
    default.append(4.0).unwrap();
    custom.append(3.0).unwrap();
    custom.append(4.0).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.extent(), custom.extent());
    assert_eq!(default.first().unwrap(), custom.first().unwrap());
    assert_eq!(default.last().unwrap(), custom.last().unwrap());
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_S5_IntegerListUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94531-6dbc-11d4-b9c8-0060b0ee281b");
    let default = TDataStdIntegerList::set(&label).unwrap();
    let custom = TDataStdIntegerList::set_with_id(&label, &user_guid).unwrap();
    default.append(33).unwrap();
    default.append(44).unwrap();
    custom.append(33).unwrap();
    custom.append(44).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.extent(), custom.extent());
    assert_eq!(default.first().unwrap(), custom.first().unwrap());
    assert_eq!(default.last().unwrap(), custom.last().unwrap());
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_T5_ExtStringListUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94541-6dbc-11d4-b9c8-0060b0ee281b");
    let default = TDataStdExtStringList::set(&label).unwrap();
    let custom = TDataStdExtStringList::set_with_id(&label, &user_guid).unwrap();
    default.append(text("aaaaa")).unwrap();
    default.append(text("bbbbbb")).unwrap();
    custom.append(text("aaaaa")).unwrap();
    custom.append(text("bbbbbb")).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.extent(), custom.extent());
    assert!(default.first().unwrap().is_equal(custom.first().unwrap()));
    assert!(default.last().unwrap().is_equal(custom.last().unwrap()));
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_U5_BooleanListUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94551-6dbc-11d4-b9c8-0060b0ee281b");
    let default = TDataStdBooleanList::set(&label).unwrap();
    let custom = TDataStdBooleanList::set_with_id(&label, &user_guid).unwrap();
    default.append(false).unwrap();
    default.append(true).unwrap();
    custom.append(false).unwrap();
    custom.append(true).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.extent(), custom.extent());
    assert_eq!(default.first().unwrap(), custom.first().unwrap());
    assert_eq!(default.last().unwrap(), custom.last().unwrap());
}

#[test]
fn TDataStd_Attribute_Test__CafBasic_V5_ReferenceListUserGuid() {
    let document = new_document();
    let label = new_attribute_label(&document);
    let user_guid = guid("12e94561-6dbc-11d4-b9c8-0060b0ee281b");
    let target1 = document.main().unwrap().find_child(3, true).unwrap();
    let target2 = document.main().unwrap().find_child(4, true).unwrap();
    let default = TDataStdReferenceList::set(&label).unwrap();
    let custom = TDataStdReferenceList::set_with_id(&label, &user_guid).unwrap();
    default.append(target1.clone()).unwrap();
    default.append(target2.clone()).unwrap();
    custom.append(target1).unwrap();
    custom.append(target2).unwrap();
    forget_all_then_undo_redo_undo(&document, &label);
    assert_eq!(default.extent(), custom.extent());
    assert_eq!(default.first().unwrap(), custom.first().unwrap());
    assert_eq!(default.last().unwrap(), custom.last().unwrap());
}

// caf/bugs/E1: forgetting and replacing an integer must remain undoable.
#[test]
fn TDataStd_Attribute_Test__CafBug_E1_UndoAfterForgetAndReplace() {
    let document = new_document();
    let label = document.main().unwrap().find_child(1, true).unwrap();
    TDataStdInteger::set(&label, 1).unwrap();
    TDataStdReal::set(&label, 1.5).unwrap();
    document.new_command().unwrap();
    label
        .forget_attribute_id(&TDataStdInteger::get_id())
        .unwrap();
    TDataStdInteger::set(&label, 3).unwrap();
    document.new_command().unwrap();
    assert!(document.undo().is_ok());
}
