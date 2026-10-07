//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDocStd_XLink_Test.cxx.
//!
//! The suite is a TKLCAF GTest that copies TNaming named shapes of BRepPrimAPI boxes, so
//! its translation lives in the TKCAF crate, which provides TNaming. The fixture's
//! application creates "BinOcaf" documents; the Bin drivers are not ported (native file
//! formats, user decision 2026-09) and no case reads or writes a file.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingTool};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdf::{TdfLabel, TdfReference};
use rustycad_tklcaf::tdoc_std::{
    TDocStdApplication, TDocStdDocument, TDocStdStandardApplication, TDocStdXLink, TDocStdXLinkTool,
};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::sync::OnceLock;
use support::{find_named_shape, make_box};

/// The fixture's static application.
fn application() -> &'static Handle<TDocStdStandardApplication> {
    static APPLICATION: OnceLock<Handle<TDocStdStandardApplication>> = OnceLock::new();
    APPLICATION.get_or_init(TDocStdStandardApplication::new)
}

fn new_document() -> Handle<TDocStdDocument> {
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application()
        .new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document)
        .unwrap();
    document
}

fn new_label(document: &Handle<TDocStdDocument>, tag: i32) -> TdfLabel {
    support::main_child(document, tag)
}

fn make_sphere(radius: f64) -> Shape {
    BRepPrimAPIMakeSphere::new(radius)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}

fn set_shape(label: &TdfLabel, shape: &Shape) {
    TNamingBuilder::new(label)
        .unwrap()
        .generated(shape)
        .unwrap();
}

fn shape_of(label: &TdfLabel) -> Shape {
    find_named_shape(label).map_or_else(Shape::default, |named| {
        TNamingTool::get_shape(&named).unwrap()
    })
}

fn bounds_of(shape: &Shape) -> [f64; 6] {
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(shape, &mut bounds, true).unwrap();
    bounds.get().unwrap()
}

fn same_bounds(first: &Shape, second: &Shape) -> bool {
    if first.is_null() || second.is_null() {
        return first.is_null() && second.is_null();
    }
    let (first, second) = (bounds_of(first), bounds_of(second));
    first
        .iter()
        .zip(&second)
        .all(|(a, b)| (a - b).abs() <= 1.0e-12)
}

fn enable_undo(document: &Handle<TDocStdDocument>) {
    document.set_undo_limit(100).unwrap();
}

fn commit_and_undo_redo(document: &Handle<TDocStdDocument>) {
    document.new_command().unwrap();
    assert!(document.undo().unwrap());
    assert!(document.redo().unwrap());
}

fn expect_linked(target: &TdfLabel, source: &TdfLabel) {
    let mut link: Handle<TDocStdXLink> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TDocStdXLink::get_id(), &mut link)
            .unwrap()
    );
    assert!(!link.label_entry().is_empty());
    let mut reference: Handle<TdfReference> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TdfReference::get_id(), &mut reference)
            .unwrap()
    );
    assert!(TdfReference::get(&reference) == *source);
}

/// The common start of the link cases: a box on the source and `tool` run on the target.
fn copy_box(
    source_document: &Handle<TDocStdDocument>,
    target_document: &Handle<TDocStdDocument>,
    with_link: bool,
) -> (TdfLabel, TdfLabel, Shape, TDocStdXLinkTool) {
    let source = new_label(source_document, 2);
    let target = new_label(target_document, 3);
    let solid = make_box(100.0, 200.0, 300.0);
    set_shape(&source, &solid);
    target_document.new_command().unwrap();
    let mut tool = TDocStdXLinkTool::new();
    if with_link {
        tool.copy_with_link(&target, &source).unwrap();
    } else {
        tool.copy(&target, &source).unwrap();
    }
    assert!(tool.is_done());
    (source, target, solid, tool)
}

// caf/xlink/A1: copy a named shape without a link inside one document.
#[test]
fn TDocStd_XLink_Test__CafXLink_A1_CopySameDocument() {
    let document = new_document();
    enable_undo(&document);
    let (_, target, solid, _) = copy_box(&document, &document, false);
    commit_and_undo_redo(&document);
    let copied = shape_of(&target);
    assert!(!copied.is_null());
    assert!(same_bounds(&solid, &copied));
}

// caf/xlink/A2: copy a self-contained named shape between documents.
#[test]
fn TDocStd_XLink_Test__CafXLink_A2_CopyOtherDocument() {
    let source_document = new_document();
    let target_document = new_document();
    enable_undo(&target_document);
    let (_, target, solid, _) = copy_box(&source_document, &target_document, false);
    commit_and_undo_redo(&target_document);
    let copied = shape_of(&target);
    assert!(!copied.is_null());
    assert!(same_bounds(&solid, &copied));
}

// caf/xlink/B1: create a link inside one document and preserve it through undo/redo.
#[test]
fn TDocStd_XLink_Test__CafXLink_B1_CopyWithLinkSameDocument() {
    let document = new_document();
    enable_undo(&document);
    let (source, target, solid, _) = copy_box(&document, &document, true);
    commit_and_undo_redo(&document);
    expect_linked(&target, &source);
    assert!(same_bounds(&solid, &shape_of(&target)));
}

// caf/xlink/B2: a link follows source history through source undo/redo.
#[test]
fn TDocStd_XLink_Test__CafXLink_B2_SourceUndoRedo() {
    let document = new_document();
    enable_undo(&document);
    let (source, target, solid, mut tool) = copy_box(&document, &document, true);
    document.new_command().unwrap();
    set_shape(&source, &make_sphere(60.0));
    document.new_command().unwrap();
    assert!(document.undo().unwrap());
    assert!(document.redo().unwrap());
    tool.update_link(&target).unwrap();
    assert!(tool.is_done());
    expect_linked(&target, &source);
    assert!(!same_bounds(&solid, &shape_of(&target)));
}

// caf/xlink/B4: create a link between two documents and preserve the copied shape.
#[test]
fn TDocStd_XLink_Test__CafXLink_B4_CopyWithLinkOtherDocument() {
    let source_document = new_document();
    let target_document = new_document();
    enable_undo(&target_document);
    let (source, target, solid, _) = copy_box(&source_document, &target_document, true);
    commit_and_undo_redo(&target_document);
    expect_linked(&target, &source);
    assert!(same_bounds(&solid, &shape_of(&target)));
}

// caf/xlink/B5: the target remains a valid external link after source history changes.
#[test]
fn TDocStd_XLink_Test__CafXLink_B5_SourceHistoryOtherDocument() {
    let source_document = new_document();
    let target_document = new_document();
    enable_undo(&source_document);
    enable_undo(&target_document);
    let (source, target, solid, mut tool) = copy_box(&source_document, &target_document, true);
    target_document.new_command().unwrap();
    source_document.new_command().unwrap();
    set_shape(&source, &make_sphere(70.0));
    source_document.new_command().unwrap();
    assert!(source_document.undo().unwrap());
    assert!(source_document.redo().unwrap());
    tool.update_link(&target).unwrap();
    assert!(tool.is_done());
    expect_linked(&target, &source);
    assert!(!same_bounds(&solid, &shape_of(&target)));
}

// caf/xlink/C1: updating references keeps the explicitly changed target shape.
#[test]
fn TDocStd_XLink_Test__CafXLink_C1_UpdateReferences() {
    let document = new_document();
    enable_undo(&document);
    let (source, target, solid, _) = copy_box(&document, &document, true);
    document.new_command().unwrap();
    document.new_command().unwrap();
    set_shape(&target, &make_sphere(70.0));
    let mut link: Handle<TDocStdXLink> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TDocStdXLink::get_id(), &mut link)
            .unwrap()
    );
    document.update_references(&link.document_entry()).unwrap();
    document.new_command().unwrap();
    assert!(document.undo().unwrap());
    assert!(document.redo().unwrap());
    expect_linked(&target, &source);
    assert!(!same_bounds(&solid, &shape_of(&target)));
}

// caf/xlink/D1: explicitly update a link after changing its source document.
#[test]
fn TDocStd_XLink_Test__CafXLink_D1_UpdateOtherDocument() {
    let source_document = new_document();
    let target_document = new_document();
    enable_undo(&source_document);
    enable_undo(&target_document);
    let (source, target, solid, mut tool) = copy_box(&source_document, &target_document, true);
    target_document.new_command().unwrap();
    source_document.new_command().unwrap();
    set_shape(&source, &make_sphere(70.0));
    source_document.new_command().unwrap();
    target_document.new_command().unwrap();
    tool.update_link(&target).unwrap();
    assert!(tool.is_done());
    target_document.new_command().unwrap();
    assert!(target_document.undo().unwrap());
    assert!(target_document.redo().unwrap());
    expect_linked(&target, &source);
    assert!(!same_bounds(&solid, &shape_of(&target)));
}

// caf/xlink/D2: update two cross-document links independently.
#[test]
fn TDocStd_XLink_Test__CafXLink_D2_UpdateAllOtherDocumentLinks() {
    let source_document = new_document();
    let target_document = new_document();
    enable_undo(&source_document);
    enable_undo(&target_document);
    let (source1, source2) = (
        new_label(&source_document, 2),
        new_label(&source_document, 22),
    );
    let (target1, target2) = (
        new_label(&target_document, 3),
        new_label(&target_document, 32),
    );
    let (box1, box2) = (
        make_box(100.0, 200.0, 300.0),
        make_box(2100.0, 2200.0, 2300.0),
    );
    set_shape(&source1, &box1);
    set_shape(&source2, &box2);
    target_document.new_command().unwrap();
    let mut tool = TDocStdXLinkTool::new();
    tool.copy_with_link(&target1, &source1).unwrap();
    assert!(tool.is_done());
    tool.copy_with_link(&target2, &source2).unwrap();
    assert!(tool.is_done());
    target_document.new_command().unwrap();
    source_document.new_command().unwrap();
    set_shape(&source1, &make_sphere(70.0));
    set_shape(&source2, &make_sphere(270.0));
    source_document.new_command().unwrap();
    target_document.new_command().unwrap();
    tool.update_link(&target1).unwrap();
    assert!(tool.is_done());
    tool.update_link(&target2).unwrap();
    assert!(tool.is_done());
    target_document.new_command().unwrap();
    assert!(target_document.undo().unwrap());
    assert!(target_document.redo().unwrap());
    expect_linked(&target1, &source1);
    expect_linked(&target2, &source2);
    let (updated1, updated2) = (shape_of(&target1), shape_of(&target2));
    assert!(!same_bounds(&box1, &updated1));
    assert!(!same_bounds(&box2, &updated2));
    assert!(!same_bounds(&updated1, &updated2));
}
