//! 1:1 translation of src/ApplicationFramework/TKCAF/GTests/TNaming_Tool_Test.cxx.
//!
//! The cases that drive DNaming functions (TKDCAF, a DRAW toolkit), Boolean operations or
//! TNaming_Selector, TNaming_NamingTool and TNaming_Translator wait for those classes; the
//! ledger marks them `pending_dependency`.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkcaf::tnaming::{
    TNamingBuilder, TNamingIterator, TNamingNamedShape, TNamingNamedShapeMap,
    TNamingNewShapeIterator, TNamingOldShapeIterator, TNamingTool, TNamingTranslator,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tklcaf::tdf::{TdfData, TdfLabel, TdfTool};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use support::{find_named_shape, make_box};

/// The `TNaming_ToolTest` fixture: a bare data framework.
struct Fixture {
    _data: Handle<TdfData>,
    root: TdfLabel,
}

fn set_up() -> Fixture {
    let data = TdfData::new();
    let root = data.root();
    Fixture { _data: data, root }
}

fn make_sphere(radius: f64) -> Shape {
    BRepPrimAPIMakeSphere::new(radius)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}

fn child(label: &TdfLabel, tag: i32) -> TdfLabel {
    label.find_child(tag, true).unwrap()
}

fn named_shape_of(label: &TdfLabel) -> Handle<TNamingNamedShape> {
    find_named_shape(label).expect("the label has a named shape")
}

// Test TNaming_Tool::GetShape with primitive evolution.
#[test]
fn TNaming_ToolTest__GetShape_Primitive() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let retrieved = TNamingTool::get_shape(&named_shape_of(&label)).unwrap();
    assert!(!retrieved.is_null());
    assert!(retrieved.is_same(&shape));
}

// Test TNaming_Tool::GetShape with a null NamedShape: nothing is called on it.
#[test]
fn TNaming_ToolTest__GetShape_NullNamedShape() {
    let _fixture = set_up();
    let named: Handle<TNamingNamedShape> = Handle::default();
    if !named.is_null() {
        TNamingTool::get_shape(&named).unwrap();
    }
}

// Test TNaming_Tool::OriginalShape.
#[test]
fn TNaming_ToolTest__OriginalShape_Modification() {
    let fixture = set_up();
    let (box1, box2) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .modify(&box1, &box2)
        .unwrap();
    let original = TNamingTool::original_shape(&named_shape_of(&label)).unwrap();
    assert!(!original.is_null());
    assert!(original.is_same(&box1));
}

// Test TNaming_Tool::CurrentShape without modifications.
#[test]
fn TNaming_ToolTest__CurrentShape_NoModification() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let current = TNamingTool::current_shape(&named_shape_of(&label)).unwrap();
    assert!(!current.is_null());
    assert!(current.is_same(&shape));
}

// Test TNaming_Tool::CurrentShape with a modification.
#[test]
fn TNaming_ToolTest__CurrentShape_WithModification() {
    let fixture = set_up();
    let (box1, box2) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    let label1 = child(&fixture.root, 1);
    TNamingBuilder::new(&label1)
        .unwrap()
        .generated(&box1)
        .unwrap();
    let named1 = named_shape_of(&label1);
    let label2 = child(&fixture.root, 2);
    TNamingBuilder::new(&label2)
        .unwrap()
        .modify(&box1, &box2)
        .unwrap();
    // CurrentShape on the first named shape follows the modification.
    let current = TNamingTool::current_shape(&named1).unwrap();
    assert!(!current.is_null());
}

// Test TNaming_Tool::HasLabel.
#[test]
fn TNaming_ToolTest__HasLabel_ExistingShape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    assert!(TNamingTool::has_label(&label, &shape).unwrap());
}

// Test TNaming_Tool::HasLabel with a shape that is not stored.
#[test]
fn TNaming_ToolTest__HasLabel_NonExistingShape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let sphere = make_sphere(15.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    assert!(!TNamingTool::has_label(&label, &sphere).unwrap());
}

// Test TNaming_Tool::NamedShape.
#[test]
fn TNaming_ToolTest__NamedShape_RetrieveByShape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let named = TNamingTool::named_shape(&shape, &label).unwrap();
    assert!(!named.is_null());
    assert!(TNamingTool::get_shape(&named).unwrap().is_same(&shape));
}

// Test TNaming_Tool::NamedShape with a shape that is not stored.
#[test]
fn TNaming_ToolTest__NamedShape_NonExistingShape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let sphere = make_sphere(15.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    assert!(TNamingTool::named_shape(&sphere, &label).unwrap().is_null());
}

// Test TNaming_Tool::CurrentNamedShape.
#[test]
fn TNaming_ToolTest__CurrentNamedShape_Simple() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let current = TNamingTool::current_named_shape(&named_shape_of(&label)).unwrap();
    assert!(!current.is_null());
}

// Test TNaming_Tool::Label.
#[test]
fn TNaming_ToolTest__Label_RetrieveLabel() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let mut transaction = 0;
    let retrieved = TNamingTool::label(&label, &shape, &mut transaction).unwrap();
    assert!(!retrieved.is_null());
    assert!(retrieved.is_equal(&label));
}

// Test TNaming_Tool::GeneratedShape.
#[test]
fn TNaming_ToolTest__GeneratedShape_Simple() {
    let fixture = set_up();
    let (old, new) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated_from(&old, &new)
        .unwrap();
    let generated = TNamingTool::generated_shape(&old, &named_shape_of(&label)).unwrap();
    assert!(!generated.is_null());
    assert!(generated.is_same(&new));
}

// Test TNaming_Tool with multiple shapes.
#[test]
fn TNaming_ToolTest__MultipleShapes_GetShape() {
    let fixture = set_up();
    let boxes = [
        make_box(10.0, 10.0, 10.0),
        make_box(20.0, 20.0, 20.0),
        make_box(30.0, 30.0, 30.0),
    ];
    let labels: Vec<TdfLabel> = (1..=3).map(|tag| child(&fixture.root, tag)).collect();
    for (label, shape) in labels.iter().zip(&boxes) {
        TNamingBuilder::new(label)
            .unwrap()
            .generated(shape)
            .unwrap();
    }
    for (label, shape) in labels.iter().zip(&boxes) {
        assert!(
            TNamingTool::get_shape(&named_shape_of(label))
                .unwrap()
                .is_same(shape)
        );
    }
}

// Test TNaming_Tool::ValidUntil.
#[test]
fn TNaming_ToolTest__ValidUntil_SimpleCase() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    assert!(TNamingTool::valid_until(&label, &shape).unwrap() >= 0);
}

// Test TNaming_Tool with a deleted shape.
#[test]
fn TNaming_ToolTest__DeletedShape_CurrentShape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label1 = child(&fixture.root, 1);
    TNamingBuilder::new(&label1)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let label2 = child(&fixture.root, 2);
    TNamingBuilder::new(&label2)
        .unwrap()
        .delete(&shape)
        .unwrap();
    let named1 = named_shape_of(&label1);
    let _current = TNamingTool::current_shape(&named1).unwrap();
    // The behaviour depends on the implementation, but it does not fail.
    assert!(TNamingTool::current_shape(&named1).is_ok());
}

// Test edge case: an empty label.
#[test]
fn TNaming_ToolTest__EmptyLabel_HasLabel() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let empty = child(&fixture.root, 99);
    assert!(!TNamingTool::has_label(&empty, &shape).unwrap());
}

// Test TNaming_Tool::Collect.
#[test]
fn TNaming_ToolTest__Collect_SimpleCase() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let label = child(&fixture.root, 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let mut collected = TNamingNamedShapeMap::default();
    assert!(TNamingTool::collect(&named_shape_of(&label), &mut collected, true).is_ok());
}

// Regression: NamedShape on a framework without TNaming_UsedShapes.
#[test]
fn TNaming_ToolTest__NamedShape_MissingUsedShapes() {
    let _fixture = set_up();
    let data = TdfData::new();
    let root = data.root();
    let shape = make_box(10.0, 20.0, 30.0);
    assert!(TNamingTool::named_shape(&shape, &root).unwrap().is_null());
}

/// The file-static `CollectNamingSubShapes`: the distinct sub-shapes of a type (the shape
/// itself excluded).
fn collect_naming_sub_shapes(shape: &Shape, shape_type: ShapeEnum) -> Vec<Shape> {
    let mut seen = MapOfShape::default();
    seen.add(shape.clone());
    support::explore(shape, shape_type)
        .into_iter()
        .filter(|current| seen.add(current.clone()))
        .collect()
}

fn add_naming_primitives(label: &TdfLabel, shapes: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for shape in shapes {
        builder.generated(shape).unwrap();
    }
}

fn add_naming_generated(label: &TdfLabel, olds: &[Shape], news: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (old, new) in olds.iter().zip(news) {
        builder.generated_from(old, new).unwrap();
    }
}

fn add_naming_modified(label: &TdfLabel, olds: &[Shape], news: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (old, new) in olds.iter().zip(news) {
        builder.modify(old, new).unwrap();
    }
}

fn add_naming_deleted(label: &TdfLabel, shapes: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for shape in shapes {
        builder.delete(shape).unwrap();
    }
}

fn add_naming_selected(label: &TdfLabel, shapes: &[Shape], contexts: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (shape, context) in shapes.iter().zip(contexts) {
        builder.select(shape, context).unwrap();
    }
}

fn expect_naming_current_shape(label: &TdfLabel, expected: &Shape) {
    let current = TNamingTool::current_shape(&named_shape_of(label)).unwrap();
    assert!(current.is_same(expected), "CurrentShape mismatch");
}

fn expect_naming_get_shape(label: &TdfLabel, expected: &Shape) {
    let shape = TNamingTool::get_shape(&named_shape_of(label)).unwrap();
    assert!(shape.is_same(expected), "GetShape mismatch");
}

fn expect_naming_shape(label: &TdfLabel, expected: &Shape) {
    expect_naming_current_shape(label, expected);
    expect_naming_get_shape(label, expected);
}

/// Copies of some shapes of a slice, by index.
fn pick(shapes: &[Shape], indexes: &[usize]) -> Vec<Shape> {
    indexes.iter().map(|&index| shapes[index].clone()).collect()
}

// caf/nam/A1: TNaming_Builder accepts primitive, generated, modified, deleted, replacement,
// and selected evolutions without DRAW command wrappers.
#[test]
fn TNaming_Tool_Test__CafNamedShape_A1_BuilderEvolutions() {
    let data = TdfData::new();
    let test_label = child(&data.root(), 1);
    let (label1, label2) = (child(&test_label, 1), child(&test_label, 2));
    let label3 = child(&test_label, 3);
    let (label11, label12) = (child(&label1, 1), child(&label1, 2));
    let (label21, label22) = (child(&label2, 1), child(&label2, 2));
    let (label31, label32) = (child(&label3, 1), child(&label3, 2));
    let label41 = child(&child(&test_label, 4), 1);
    let label5 = child(&test_label, 5);
    let label51 = child(&label5, 1);
    let label6 = child(&test_label, 6);
    let label61 = child(&label6, 1);

    let box1 = make_box(100.0, 200.0, 300.0);
    let box2 = make_box(200.0, 300.0, 400.0);
    let box3 = make_box(300.0, 400.0, 500.0);
    let box4 = make_box(300.0, 400.0, 500.0);
    let e = collect_naming_sub_shapes(&box1, ShapeEnum::Edge);
    let f2 = collect_naming_sub_shapes(&box2, ShapeEnum::Face);
    let f3 = collect_naming_sub_shapes(&box3, ShapeEnum::Face);
    let f4 = collect_naming_sub_shapes(&box4, ShapeEnum::Face);
    assert_eq!((e.len(), f2.len(), f3.len(), f4.len()), (12, 6, 6, 6));

    add_naming_primitives(&label1, std::slice::from_ref(&box1));
    add_naming_primitives(&label11, &e[0..1]);
    add_naming_primitives(&label11, &e[0..3]);
    add_naming_primitives(&label12, &e[3..6]);
    add_naming_generated(
        &label2,
        std::slice::from_ref(&box1),
        std::slice::from_ref(&box2),
    );
    add_naming_generated(&label21, &e[0..1], &f2[0..1]);
    add_naming_generated(&label22, &pick(&e, &[1, 1, 2]), &f2[1..4]);
    add_naming_generated(&label21, &pick(&e, &[2, 2, 3]), &pick(&f2, &[0, 2, 3]));
    add_naming_modified(
        &label3,
        std::slice::from_ref(&box2),
        std::slice::from_ref(&box3),
    );
    add_naming_modified(&label31, &f2[0..1], &f3[0..1]);
    add_naming_modified(&label32, &pick(&f2, &[4, 1, 2]), &f3[1..4]);
    let old31 = [f2[2].clone(), e[2].clone(), f2[3].clone()];
    add_naming_modified(&label31, &old31, &pick(&f3, &[0, 2, 3]));
    add_naming_deleted(&label41, &f3[0..1]);
    add_naming_deleted(&label41, &pick(&f3, &[0, 3]));
    add_naming_modified(
        &label5,
        std::slice::from_ref(&box3),
        std::slice::from_ref(&box4),
    );
    add_naming_modified(&label51, &f3[1..3], &f4[1..3]);
    add_naming_selected(
        &label6,
        std::slice::from_ref(&box4),
        std::slice::from_ref(&box4),
    );
    let selected61 = [f4[1].clone(), f4[1].clone(), f2[4].clone(), f2[4].clone()];
    add_naming_selected(&label61, &selected61, &selected61);

    let labels = [
        &label1, &label11, &label12, &label2, &label21, &label22, &label3, &label31, &label32,
        &label41, &label5, &label51, &label6, &label61,
    ];
    for label in labels {
        assert!(!named_shape_of(label).is_null());
    }
    expect_naming_shape(&label1, &box1);
    expect_naming_get_shape(&label2, &box2);
    expect_naming_get_shape(&label3, &box3);
    expect_naming_shape(&label5, &box4);
    expect_naming_shape(&label6, &box4);
}

// bugs/caf/bug26428: both named-shape iterators can be constructed when the named shape
// contains only a generated (new) shape.
#[test]
fn TNaming_Tool_Test__CafBug_26428_NewAndOldShapeIterators() {
    let data = TdfData::new();
    let label = child(&data.root(), 1);
    let shape = make_box(10.0, 20.0, 30.0);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let names = TNamingIterator::from_label(&label).unwrap();
    let new_iterator = TNamingNewShapeIterator::from_iterator(&names).unwrap();
    let old_iterator = TNamingOldShapeIterator::from_iterator(&names).unwrap();
    let _ = (new_iterator.more(), old_iterator.more());
    let named = named_shape_of(&label);
    assert!(TNamingTool::get_shape(&named).unwrap().is_same(&shape));
}

/// `TDF_Tool::Label(aDocument->GetData(), anEntry, aLabel, true)`.
fn label_at(document: &Handle<TDocStdDocument>, entry: &str) -> TdfLabel {
    let mut label = TdfLabel::default();
    let entry = AsciiString::from_bytes(entry.as_bytes());
    TdfTool::label_from_entry(&document.get_data(), &entry, &mut label, true).unwrap();
    label
}

/// The X of the centre of mass of the first solid of the current shape of a label.
fn first_solid_centre_x(label: &TdfLabel) -> f64 {
    let current = TNamingTool::current_shape(&named_shape_of(label)).unwrap();
    assert!(!current.is_null());
    let first_solid = ShapeIterator::new(&current, true, true)
        .find(|sub| sub.shape_type().unwrap() == ShapeEnum::Solid)
        .expect("a solid in the current shape");
    let mut properties = GProps::new();
    BRepGProp::volume_properties(&first_solid, &mut properties, false, false, false).unwrap();
    properties.centre_of_mass().x()
}

// bugs/caf/bug26155: the order of a modified compound remains stable when the same naming
// sequence is evaluated repeatedly.
#[test]
fn TNaming_Tool_Test__CafBug_26155_ModificationOrderStable() {
    let document = support::new_document();
    assert!(!document.is_null());
    let tolerance = 0.1;
    let mut reference: Option<f64> = None;
    for index in 1..=50 {
        let primitive_entry = format!("{index}:1:1");
        let primitive_label = label_at(&document, &primitive_entry);
        assert!(!primitive_label.is_null());
        let shape = make_box(10.0, 10.0, 10.0);
        TNamingBuilder::new(&primitive_label)
            .unwrap()
            .generated(&shape)
            .unwrap();
        let first_half = support::make_box_between([0.0, 0.0, 0.0], [5.0, 10.0, 10.0]);
        let second_half = support::make_box_between([5.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        let first_label = label_at(&document, &format!("{primitive_entry}:1"));
        TNamingBuilder::new(&first_label)
            .unwrap()
            .modify(&shape, &first_half)
            .unwrap();
        let second_label = label_at(&document, &format!("{primitive_entry}:2"));
        TNamingBuilder::new(&second_label)
            .unwrap()
            .modify(&shape, &second_half)
            .unwrap();
        let current_x = first_solid_centre_x(&primitive_label);
        match reference {
            None => reference = Some(current_x),
            Some(reference) => assert!((current_x - reference).abs() <= tolerance),
        }
    }
}

// bugs/caf/bug24263: TNaming_CopyShape copies a solid and its faces while preserving the
// orientation carried by each source shape.
#[test]
fn TNaming_Tool_Test__CafBug_24263_CopyShapePreservesOrientation() {
    let solid = make_box(100.0, 100.0, 100.0);
    let mut translator = TNamingTranslator::new();
    translator.add(&solid);
    let faces = support::explore(&solid, ShapeEnum::Face);
    for face in &faces {
        translator.add(face);
    }
    let reversed = |shape: &Shape| shape.orientation() == Orientation::Reversed;
    let nb_reversed = faces.iter().filter(|face| reversed(face)).count();
    assert_eq!(faces.len(), 6);
    assert!(nb_reversed > 0);

    translator.perform().unwrap();
    assert!(translator.is_done());
    let copied = translator.copied_shape(&solid);
    assert!(!copied.is_null());
    assert_eq!(copied.orientation(), Orientation::Forward);
    let mut copied_reversed = 0;
    for face in &faces {
        let copied_face = translator.copied_shape(face);
        assert!(!copied_face.is_null());
        assert_eq!(copied_face.orientation(), face.orientation());
        copied_reversed += usize::from(reversed(&copied_face));
    }
    assert_eq!(copied_reversed, nb_reversed);
}
