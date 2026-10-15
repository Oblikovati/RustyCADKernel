//! 1:1 translation of src/DataExchange/TKXCAF/GTests/XCAFDoc_NotesTool_Test.cxx.
#![allow(non_snake_case)]

mod support;

use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Guid, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::tdata_std::TDataStdInteger;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTool};
use rustycad_tklcaf::tdoc_std::{TDocStdDocument, TDocStdStandardApplication};
use rustycad_tkxcaf::xcaf_doc::{
    XCAFDocAssemblyItemId, XCAFDocDocumentTool, XCAFDocNote, XCAFDocNotesTool,
};

struct NotesContext {
    _application: Handle<TDocStdStandardApplication>,
    _document: Handle<TDocStdDocument>,
    tool: Handle<XCAFDocNotesTool>,
    item1: TdfLabel,
    item2: TdfLabel,
    attribute_guid: Guid,
}

impl NotesContext {
    fn tool(&self) -> &XCAFDocNotesTool {
        self.tool.get().unwrap()
    }
}

fn new_context() -> NotesContext {
    let (application, document) = support::new_document("BinXCAF");
    let main = document.main().unwrap();
    let shape_tool = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let shapes = shape_tool.get().unwrap();
    let item1 = shapes
        .add_shape(&support::make_box(1.0, 1.0, 1.0), true, true)
        .unwrap();
    let item2 = shapes
        .add_shape(&support::make_box(2.0, 2.0, 2.0), true, true)
        .unwrap();
    let tool = XCAFDocDocumentTool::notes_tool(&main).unwrap();
    let attribute_guid = Guid::from_ascii(b"27927843-72A0-41ef-901D-621FAB01C27A").unwrap();
    TDataStdInteger::set_with_id(&item1, &attribute_guid, 1965).unwrap();
    NotesContext {
        _application: application,
        _document: document,
        tool,
        item1,
        item2,
        attribute_guid,
    }
}

fn text(value: &str) -> ExtendedString {
    ExtendedString::from_utf8(value)
}

fn comment(context: &NotesContext, value: &str) -> Handle<dyn XCAFDocNote> {
    context
        .tool()
        .create_comment(
            &text("The user"),
            &text("2026-08-07T00:00:00"),
            &text(value),
        )
        .unwrap()
}

fn balloon(context: &NotesContext, value: &str) -> Handle<dyn XCAFDocNote> {
    context
        .tool()
        .create_balloon(
            &text("The user"),
            &text("2026-08-07T00:00:00"),
            &text(value),
        )
        .unwrap()
}

fn binary_note(context: &NotesContext) -> Handle<dyn XCAFDocNote> {
    let data = HArray1::<u8>::with_bounds(1, 26).unwrap();
    for index in 1..=26 {
        data.change_array1()
            .set_value(index, b'a' + (index - 1) as u8)
            .unwrap();
    }
    let mime = AsciiString::from("application/octet-stream");
    context
        .tool()
        .create_bin_data(
            &text("The user"),
            &text("2026-08-07T00:00:00"),
            &text("Binary data"),
            &mime,
            &Handle::new(data),
        )
        .unwrap()
}

fn note_label(note: &Handle<dyn XCAFDocNote>) -> TdfLabel {
    note.get().map(|n| n.label()).unwrap_or_default()
}

fn item_id(item: &TdfLabel) -> XCAFDocAssemblyItemId {
    let mut entry = AsciiString::default();
    TdfTool::entry(item, &mut entry);
    XCAFDocAssemblyItemId::from_string(&entry)
}

fn add_base(context: &NotesContext, note: &Handle<dyn XCAFDocNote>, item: &TdfLabel) {
    assert!(!note.is_null());
    assert!(
        !context
            .tool()
            .add_note_label(&note_label(note), item)
            .unwrap()
            .is_null()
    );
}

fn add_attribute(context: &NotesContext, note: &Handle<dyn XCAFDocNote>, item: &TdfLabel) {
    assert!(!note.is_null());
    let reference = context
        .tool()
        .add_note_to_attr_label(&note_label(note), item, &context.attribute_guid)
        .unwrap();
    assert!(!reference.is_null());
}

fn add_subshape(context: &NotesContext, note: &Handle<dyn XCAFDocNote>, item: &TdfLabel) {
    assert!(!note.is_null());
    assert!(
        !context
            .tool()
            .add_note_to_subshape_label(&note_label(note), item, 1)
            .unwrap()
            .is_null()
    );
}

fn expect_counts(context: &NotesContext, annotations: i32, notes: i32, orphans: i32) {
    assert_eq!(context.tool().nb_annotated_items().unwrap(), annotations);
    assert_eq!(context.tool().nb_notes().unwrap(), notes);
    assert_eq!(context.tool().nb_orphan_notes().unwrap(), orphans);
}

/// Two notes on the base, attribute and subshape annotations of item 1.
fn annotate_all(context: &NotesContext) -> (Handle<dyn XCAFDocNote>, Handle<dyn XCAFDocNote>) {
    let note1 = comment(context, "Hello, World!");
    let note2 = comment(context, "Hello, Everyone!");
    add_base(context, &note1, &context.item1);
    add_base(context, &note2, &context.item1);
    add_attribute(context, &note1, &context.item1);
    add_attribute(context, &note2, &context.item1);
    add_subshape(context, &note1, &context.item1);
    add_subshape(context, &note2, &context.item1);
    (note1, note2)
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_A1_Empty() {
    let context = new_context();
    expect_counts(&context, 0, 0, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_A2_CreateOrphans() {
    let context = new_context();
    assert!(!comment(&context, "Hello, World!").is_null());
    assert!(!balloon(&context, "Hello, Everyone!").is_null());
    expect_counts(&context, 0, 2, 2);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_A3_CreateBinary() {
    let context = new_context();
    assert!(!binary_note(&context).is_null());
    expect_counts(&context, 0, 1, 1);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_A4_DeleteOne() {
    let context = new_context();
    let note = comment(&context, "Hello, World!");
    assert!(!note.is_null());
    assert!(!balloon(&context, "Hello, Everyone!").is_null());
    assert!(context.tool().delete_note(&note_label(&note)).unwrap());
    expect_counts(&context, 0, 1, 1);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_A5_DeleteAll() {
    let context = new_context();
    assert!(!comment(&context, "Hello, World!").is_null());
    assert!(!balloon(&context, "Hello, Everyone!").is_null());
    assert_eq!(context.tool().delete_all_notes().unwrap(), 2);
    expect_counts(&context, 0, 0, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_B1_TwoItemsOneNote() {
    let context = new_context();
    let note = comment(&context, "Hello, World!");
    add_base(&context, &note, &context.item1);
    add_base(&context, &note, &context.item2);
    expect_counts(&context, 2, 1, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_B2_TwoItemsTwoNotes() {
    let context = new_context();
    let note1 = comment(&context, "Hello, World!");
    let note2 = comment(&context, "Hello, Everyone!");
    add_base(&context, &note1, &context.item1);
    add_base(&context, &note2, &context.item2);
    expect_counts(&context, 2, 2, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_B3_DeleteOrphan() {
    let context = new_context();
    let note1 = comment(&context, "Hello, World!");
    let _note2 = comment(&context, "Hello, Everyone!");
    add_base(&context, &note1, &context.item1);
    add_base(&context, &note1, &context.item2);
    assert_eq!(context.tool().delete_orphan_notes().unwrap(), 1);
    expect_counts(&context, 2, 1, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_B4_TwoNotesOneItem() {
    let context = new_context();
    let note1 = comment(&context, "Hello, World!");
    let note2 = comment(&context, "Hello, Everyone!");
    add_base(&context, &note1, &context.item1);
    add_base(&context, &note2, &context.item1);
    expect_counts(&context, 1, 2, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_B5_AttributeAnnotation() {
    let context = new_context();
    let note = comment(&context, "Hello, World!");
    add_attribute(&context, &note, &context.item1);
    expect_counts(&context, 1, 1, 0);
    let found = context
        .tool()
        .find_annotated_item_attr_label(&context.item1, &context.attribute_guid)
        .unwrap();
    assert!(!found.is_null());
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_B6_BaseAndAttribute() {
    let context = new_context();
    let note = comment(&context, "Hello, World!");
    add_base(&context, &note, &context.item1);
    add_attribute(&context, &note, &context.item1);
    expect_counts(&context, 2, 1, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_B7_BaseAttributeAndSubshape() {
    let context = new_context();
    let note = comment(&context, "Hello, World!");
    add_base(&context, &note, &context.item1);
    add_attribute(&context, &note, &context.item1);
    add_subshape(&context, &note, &context.item1);
    expect_counts(&context, 3, 1, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C1_DeleteLinkedNote() {
    let context = new_context();
    let note = comment(&context, "Hello, World!");
    add_base(&context, &note, &context.item1);
    add_base(&context, &note, &context.item2);
    assert!(context.tool().delete_note(&note_label(&note)).unwrap());
    expect_counts(&context, 0, 0, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C2_DeleteOneLinkedNote() {
    let context = new_context();
    let note1 = comment(&context, "Hello, World!");
    let note2 = comment(&context, "Hello, World!");
    add_base(&context, &note1, &context.item1);
    add_base(&context, &note2, &context.item2);
    assert!(context.tool().delete_note(&note_label(&note1)).unwrap());
    expect_counts(&context, 1, 1, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C3_RemoveBaseAnnotation() {
    let context = new_context();
    let note1 = comment(&context, "Hello, World!");
    let note2 = comment(&context, "Hello, World!");
    add_base(&context, &note1, &context.item1);
    add_base(&context, &note2, &context.item2);
    assert!(
        context
            .tool()
            .remove_note_label(&note_label(&note1), &context.item1, false)
            .unwrap()
    );
    expect_counts(&context, 1, 2, 1);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C4_RemoveExtraAnnotations() {
    let context = new_context();
    let note = comment(&context, "Hello, World!");
    add_base(&context, &note, &context.item1);
    add_attribute(&context, &note, &context.item1);
    add_subshape(&context, &note, &context.item1);
    let tool = context.tool();
    assert!(
        tool.remove_attr_note_label(
            &note_label(&note),
            &context.item1,
            &context.attribute_guid,
            false
        )
        .unwrap()
    );
    assert!(
        tool.remove_subshape_note_label(&note_label(&note), &context.item1, 1, false)
            .unwrap()
    );
    expect_counts(&context, 1, 1, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C5_RemoveAllAttributeAnnotations() {
    let context = new_context();
    let note1 = comment(&context, "Hello, World!");
    let note2 = comment(&context, "Hello, Everyone!");
    add_base(&context, &note1, &context.item1);
    add_attribute(&context, &note1, &context.item1);
    add_attribute(&context, &note2, &context.item1);
    add_subshape(&context, &note1, &context.item1);
    assert!(
        context
            .tool()
            .remove_all_attr_notes_label(&context.item1, &context.attribute_guid, false)
            .unwrap()
    );
    expect_counts(&context, 2, 2, 1);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C6_RemoveAllSubshapeAnnotations() {
    let context = new_context();
    let note1 = comment(&context, "Hello, World!");
    let note2 = comment(&context, "Hello, Everyone!");
    add_base(&context, &note1, &context.item1);
    add_attribute(&context, &note1, &context.item1);
    add_subshape(&context, &note1, &context.item1);
    add_subshape(&context, &note2, &context.item1);
    assert!(
        context
            .tool()
            .remove_all_subshape_notes(&item_id(&context.item1), 1, false)
            .unwrap()
    );
    expect_counts(&context, 2, 2, 1);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C7_RemoveBaseAnnotations() {
    let context = new_context();
    annotate_all(&context);
    assert!(
        context
            .tool()
            .remove_all_notes_label(&context.item1, false)
            .unwrap()
    );
    expect_counts(&context, 2, 2, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C8_RemoveAllAnnotations() {
    let context = new_context();
    annotate_all(&context);
    let tool = context.tool();
    assert!(tool.remove_all_notes_label(&context.item1, false).unwrap());
    assert!(
        tool.remove_all_attr_notes_label(&context.item1, &context.attribute_guid, false)
            .unwrap()
    );
    assert!(
        tool.remove_all_subshape_notes(&item_id(&context.item1), 1, false)
            .unwrap()
    );
    expect_counts(&context, 0, 2, 2);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_C9_RemoveAllAndDeleteOrphans() {
    let context = new_context();
    annotate_all(&context);
    let tool = context.tool();
    assert!(tool.remove_all_notes_label(&context.item1, true).unwrap());
    assert!(
        tool.remove_all_attr_notes_label(&context.item1, &context.attribute_guid, true)
            .unwrap()
    );
    assert!(
        tool.remove_all_subshape_notes(&item_id(&context.item1), 1, true)
            .unwrap()
    );
    expect_counts(&context, 0, 0, 0);
}

#[test]
fn XCAFDoc_NotesTool_Test__GdtNotes_D1_ReuseAnnotatedItemLabels() {
    let context = new_context();
    let tool = context.tool();
    let note1 = comment(&context, "Hello, World!");
    let note2 = comment(&context, "Hello, Everyone!");
    add_base(&context, &note1, &context.item1);
    let base = tool.find_annotated_item_label(&context.item1).unwrap();
    add_base(&context, &note2, &context.item1);
    assert!(base == tool.find_annotated_item_label(&context.item1).unwrap());
    add_attribute(&context, &note1, &context.item1);
    let attribute = tool
        .find_annotated_item_attr_label(&context.item1, &context.attribute_guid)
        .unwrap();
    add_attribute(&context, &note2, &context.item1);
    assert!(
        attribute
            == tool
                .find_annotated_item_attr_label(&context.item1, &context.attribute_guid)
                .unwrap()
    );
    add_subshape(&context, &note1, &context.item1);
    let subshape = tool
        .find_annotated_item_subshape_label(&context.item1, 1)
        .unwrap();
    add_subshape(&context, &note2, &context.item1);
    assert!(
        subshape
            == tool
                .find_annotated_item_subshape_label(&context.item1, 1)
                .unwrap()
    );
    expect_counts(&context, 3, 2, 0);
}
