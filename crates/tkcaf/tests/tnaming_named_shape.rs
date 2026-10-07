//! 1:1 translation of src/ApplicationFramework/TKCAF/GTests/TNaming_NamedShape_Test.cxx.
//!
//! The fixture creates its document through `TDocStd_Application::NewDocument("BinOcaf")`;
//! the Bin drivers are not ported (native file formats, user decision 2026-09) and no case
//! reads or writes a file.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkcaf::tnaming::{
    TNamingBuilder, TNamingEvolution, TNamingIterator, TNamingNamedShape, TNamingNewShapeIterator,
    TNamingSameShapeIterator, TNamingTool,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::{TDocStdDocument, TDocStdStandardApplication};
use support::{find_named_shape, make_box, make_box_between, shape_at};

struct Fixture {
    _application: Handle<TDocStdStandardApplication>,
    document: Handle<TDocStdDocument>,
    label: TdfLabel,
}

fn set_up() -> Fixture {
    let (application, document) = support::new_application_document();
    document.set_undo_limit(10).unwrap();
    let label = support::main_child(&document, 1);
    Fixture {
        _application: application,
        document,
        label,
    }
}

fn build_primitive(label: &TdfLabel, shapes: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for shape in shapes {
        builder.generated(shape).unwrap();
    }
}

fn build_generated(label: &TdfLabel, olds: &[Shape], news: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (old, new) in olds.iter().zip(news) {
        builder.generated_from(old, new).unwrap();
    }
}

fn build_modified(label: &TdfLabel, olds: &[Shape], news: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for (old, new) in olds.iter().zip(news) {
        builder.modify(old, new).unwrap();
    }
}

fn build_deleted(label: &TdfLabel, olds: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for old in olds {
        builder.delete(old).unwrap();
    }
}

fn build_selected(label: &TdfLabel, shapes: &[Shape]) {
    let builder = TNamingBuilder::new(label).unwrap();
    for shape in shapes {
        builder.select(shape, shape).unwrap();
    }
}

/// The (old, new) pairs of a label's named shape.
fn pairs(label: &TdfLabel) -> Vec<(Shape, Shape)> {
    let Some(named) = find_named_shape(label) else {
        return Vec::new();
    };
    let mut iterator = TNamingIterator::new(&named).unwrap();
    let mut pairs = Vec::new();
    while iterator.more() {
        pairs.push((iterator.old_shape().unwrap(), iterator.new_shape().unwrap()));
        iterator.next().unwrap();
    }
    pairs
}

fn count_new_shapes(label: &TdfLabel) -> usize {
    pairs(label)
        .iter()
        .filter(|(_, new)| !new.is_null())
        .count()
}

fn count_old_shapes(label: &TdfLabel) -> usize {
    pairs(label)
        .iter()
        .filter(|(old, _)| !old.is_null())
        .count()
}

fn collect_all_new_shapes(shape: &Shape, access: &TdfLabel, shapes: &mut Vec<Shape>) {
    let mut iterator = TNamingNewShapeIterator::from_label(shape, access).unwrap();
    while iterator.more() {
        let new_shape = iterator.shape().unwrap();
        if !new_shape.is_null() {
            shapes.push(new_shape.clone());
            collect_all_new_shapes(&new_shape, access, shapes);
        }
        iterator.next().unwrap();
    }
}

fn collect_all_old_shapes(shape: &Shape, access: &TdfLabel, shapes: &mut Vec<Shape>) {
    let named = TNamingTool::named_shape(shape, access).unwrap();
    if named.is_null() {
        return;
    }
    let mut iterator = TNamingIterator::new(&named).unwrap();
    while iterator.more() {
        let (old, new) = (iterator.old_shape().unwrap(), iterator.new_shape().unwrap());
        if !old.is_null() && !new.is_null() && new.is_same(shape) {
            shapes.push(old.clone());
            collect_all_old_shapes(&old, access, shapes);
        }
        iterator.next().unwrap();
    }
}

fn collect_all_new_shapes_at_label(label: &TdfLabel, access: &TdfLabel, shapes: &mut Vec<Shape>) {
    for (_, new) in pairs(label) {
        if !new.is_null() {
            collect_all_new_shapes(&new, access, shapes);
        }
    }
}

fn collect_all_old_shapes_at_label(label: &TdfLabel, access: &TdfLabel, shapes: &mut Vec<Shape>) {
    for (_, new) in pairs(label) {
        if !new.is_null() {
            collect_all_old_shapes(&new, access, shapes);
        }
    }
}

fn count_all_new_shapes_at_label(label: &TdfLabel, access: &TdfLabel) -> usize {
    let mut shapes = Vec::new();
    collect_all_new_shapes_at_label(label, access, &mut shapes);
    shapes.len()
}

fn count_all_old_shapes_at_label(label: &TdfLabel, access: &TdfLabel) -> usize {
    let mut shapes = Vec::new();
    collect_all_old_shapes_at_label(label, access, &mut shapes);
    shapes.len()
}

fn count_same_shapes(shape: &Shape, access: &TdfLabel) -> usize {
    let mut count = 0;
    let mut iterator = TNamingSameShapeIterator::from_label(shape, access).unwrap();
    while iterator.more() {
        if !iterator.label().unwrap().is_null() {
            count += 1;
        }
        iterator.next().unwrap();
    }
    count
}

fn count_same_shapes_in_draw_result(shape: &Shape, access: &TdfLabel) -> usize {
    let count = count_same_shapes(shape, access);
    // GetSameShapes concatenates its first two labels without a separator; Tcl's llength
    // consequently reports one fewer item whenever two or more labels exist.
    if count > 1 { count - 1 } else { count }
}

fn named_shape_of(label: &TdfLabel) -> Handle<TNamingNamedShape> {
    find_named_shape(label).expect("the label has a named shape")
}

// ===================== TNaming_Builder tests =====================

#[test]
fn TNaming_NamedShapeTest__Generated_Primitive() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    assert_eq!(
        named_shape_of(&fixture.label).evolution(),
        TNamingEvolution::Primitive
    );
}

#[test]
fn TNaming_NamedShapeTest__Generated_FromOld() {
    let fixture = set_up();
    let (old, new) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated_from(&old, &new)
        .unwrap();
    assert_eq!(
        named_shape_of(&fixture.label).evolution(),
        TNamingEvolution::Generated
    );
}

#[test]
fn TNaming_NamedShapeTest__Modify_Shape() {
    let fixture = set_up();
    let (old, new) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .modify(&old, &new)
        .unwrap();
    assert_eq!(
        named_shape_of(&fixture.label).evolution(),
        TNamingEvolution::Modify
    );
}

#[test]
fn TNaming_NamedShapeTest__Delete_Shape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .delete(&shape)
        .unwrap();
    assert_eq!(
        named_shape_of(&fixture.label).evolution(),
        TNamingEvolution::Delete
    );
}

#[test]
fn TNaming_NamedShapeTest__Select_Shape() {
    let fixture = set_up();
    let (shape, context) = (make_box(10.0, 20.0, 30.0), make_box(50.0, 50.0, 50.0));
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .select(&shape, &context)
        .unwrap();
    assert_eq!(
        named_shape_of(&fixture.label).evolution(),
        TNamingEvolution::Selected
    );
}

#[test]
fn TNaming_NamedShapeTest__NamedShape_ReturnsAttribute() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    let builder = TNamingBuilder::new(&fixture.label).unwrap();
    builder.generated(&shape).unwrap();
    let named = builder.named_shape();
    assert!(!named.is_null());
    // Verify it is the same attribute stored on the label.
    assert!(named == named_shape_of(&fixture.label));
}

// ===================== TNaming_NamedShape tests =====================

#[test]
fn TNaming_NamedShapeTest__IsEmpty_NewAttribute() {
    let fixture = set_up();
    let shape = make_box(10.0, 10.0, 10.0);
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let named = named_shape_of(&fixture.label);
    assert!(!named.is_empty());
    named.clear().unwrap();
    assert!(named.is_empty());
}

#[test]
fn TNaming_NamedShapeTest__Evolution_ReflectsBuilder() {
    let fixture = set_up();
    let (old, new) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .modify(&old, &new)
        .unwrap();
    assert_eq!(
        named_shape_of(&fixture.label).evolution(),
        TNamingEvolution::Modify
    );
    // Rebuild with GENERATED on a new label.
    let label2 = support::main_child(&fixture.document, 2);
    TNamingBuilder::new(&label2)
        .unwrap()
        .generated_from(&old, &new)
        .unwrap();
    assert_eq!(
        named_shape_of(&label2).evolution(),
        TNamingEvolution::Generated
    );
}

#[test]
fn TNaming_NamedShapeTest__Get_ReturnsPrimitiveShape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let result = TNamingNamedShape::get(&named_shape_of(&fixture.label)).unwrap();
    assert!(!result.is_null());
    assert!(result.is_same(&shape));
}

#[test]
fn TNaming_NamedShapeTest__Version_SetAndGet() {
    let fixture = set_up();
    let shape = make_box(10.0, 10.0, 10.0);
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let named = named_shape_of(&fixture.label);
    named.set_version(42);
    assert_eq!(named.version(), 42);
    named.set_version(0);
    assert_eq!(named.version(), 0);
}

#[test]
fn TNaming_NamedShapeTest__Clear_EmptiesAttribute() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    let named = named_shape_of(&fixture.label);
    assert!(!named.is_empty());
    named.clear().unwrap();
    assert!(named.is_empty());
    // Get() on a cleared attribute returns a null shape.
    assert!(TNamingNamedShape::get(&named).unwrap().is_null());
}

// ===================== TNaming_Iterator tests =====================

#[test]
fn TNaming_NamedShapeTest__Iterator_TraversesPairs() {
    let fixture = set_up();
    let (old, new) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated_from(&old, &new)
        .unwrap();
    let found = pairs(&fixture.label);
    for (old_shape, new_shape) in &found {
        assert!(old_shape.is_same(&old));
        assert!(new_shape.is_same(&new));
    }
    assert_eq!(found.len(), 1);
}

#[test]
fn TNaming_NamedShapeTest__Iterator_IsModification() {
    let fixture = set_up();
    let (old, new) = (make_box(10.0, 10.0, 10.0), make_box(20.0, 20.0, 20.0));
    // MODIFY evolution reports IsModification = true.
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .modify(&old, &new)
        .unwrap();
    let mut iterator = TNamingIterator::new(&named_shape_of(&fixture.label)).unwrap();
    while iterator.more() {
        assert!(iterator.is_modification().unwrap());
        assert_eq!(iterator.evolution().unwrap(), TNamingEvolution::Modify);
        iterator.next().unwrap();
    }
    // GENERATED evolution reports IsModification = false.
    let label2 = support::main_child(&fixture.document, 2);
    TNamingBuilder::new(&label2)
        .unwrap()
        .generated_from(&old, &new)
        .unwrap();
    let mut iterator2 = TNamingIterator::new(&named_shape_of(&label2)).unwrap();
    while iterator2.more() {
        assert!(!iterator2.is_modification().unwrap());
        assert_eq!(iterator2.evolution().unwrap(), TNamingEvolution::Generated);
        iterator2.next().unwrap();
    }
}

#[test]
fn TNaming_NamedShapeTest__Iterator_MultipleShapes() {
    let fixture = set_up();
    let boxes = [
        make_box(10.0, 10.0, 10.0),
        make_box(20.0, 20.0, 20.0),
        make_box(30.0, 30.0, 30.0),
    ];
    build_primitive(&fixture.label, &boxes);
    let found = pairs(&fixture.label);
    for (old, new) in &found {
        assert!(old.is_null());
        assert!(!new.is_null());
    }
    assert_eq!(found.len(), 3);
}

// ===================== TNaming_NewShapeIterator tests =====================

#[test]
fn TNaming_NamedShapeTest__NewShapeIterator_FollowsModification() {
    let fixture = set_up();
    let box1 = make_box(10.0, 10.0, 10.0);
    let box2 = make_box(20.0, 20.0, 20.0);
    let box3 = make_box(30.0, 30.0, 30.0);
    build_primitive(
        &support::main_child(&fixture.document, 1),
        std::slice::from_ref(&box1),
    );
    build_modified(
        &support::main_child(&fixture.document, 2),
        std::slice::from_ref(&box1),
        std::slice::from_ref(&box2),
    );
    build_modified(
        &support::main_child(&fixture.document, 3),
        std::slice::from_ref(&box2),
        std::slice::from_ref(&box3),
    );
    let main = fixture.document.main().unwrap();
    // Follow the modification chain from box1.
    let iterator = TNamingNewShapeIterator::from_label(&box1, &main).unwrap();
    assert!(iterator.more());
    assert!(iterator.shape().unwrap().is_same(&box2));
    assert!(iterator.is_modification().unwrap());
    // Follow from box2.
    let iterator2 = TNamingNewShapeIterator::from_label(&box2, &main).unwrap();
    assert!(iterator2.more());
    assert!(iterator2.shape().unwrap().is_same(&box3));
}

#[test]
fn TNaming_NamedShapeTest__NewShapeIterator_MultipleBranches() {
    let fixture = set_up();
    let box1 = make_box(10.0, 10.0, 10.0);
    let modified1 = make_box(20.0, 20.0, 20.0);
    let modified2 = make_box(30.0, 30.0, 30.0);
    build_primitive(
        &support::main_child(&fixture.document, 1),
        std::slice::from_ref(&box1),
    );
    build_modified(
        &support::main_child(&fixture.document, 2),
        std::slice::from_ref(&box1),
        std::slice::from_ref(&modified1),
    );
    // Label 3 also modifies box1 (split scenario).
    build_modified(
        &support::main_child(&fixture.document, 3),
        std::slice::from_ref(&box1),
        std::slice::from_ref(&modified2),
    );
    let mut count = 0;
    let mut iterator =
        TNamingNewShapeIterator::from_label(&box1, &fixture.document.main().unwrap()).unwrap();
    while iterator.more() {
        let new_shape = iterator.shape().unwrap();
        assert!(new_shape.is_same(&modified1) || new_shape.is_same(&modified2));
        count += 1;
        iterator.next().unwrap();
    }
    assert_eq!(count, 2);
}

// ===================== Undo/Redo tests =====================

#[test]
fn TNaming_NamedShapeTest__UndoRedo_RestoresNamedShape() {
    let fixture = set_up();
    let shape = make_box(10.0, 20.0, 30.0);
    fixture.document.new_command().unwrap();
    TNamingBuilder::new(&fixture.label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    fixture.document.commit_command().unwrap();
    let named = named_shape_of(&fixture.label);
    assert!(!named.is_empty());
    assert!(TNamingNamedShape::get(&named).unwrap().is_same(&shape));
    // Undo removes the attribute.
    fixture.document.undo().unwrap();
    assert!(find_named_shape(&fixture.label).is_none());
    // Redo restores it.
    fixture.document.redo().unwrap();
    let restored = named_shape_of(&fixture.label);
    assert!(!restored.is_empty());
    assert!(TNamingNamedShape::get(&restored).unwrap().is_same(&shape));
}

// caf/named_shape/A1: SetShape is restored by redo after an undo.
#[test]
fn TNaming_NamedShape_Test__CafNamedShape_A1_SetShapeUndoRedo() {
    let document = support::new_document();
    document.set_undo_limit(10).unwrap();
    let label = support::main_child(&document, 2);
    let shape = make_box(100.0, 200.0, 300.0);
    document.new_command().unwrap();
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&shape)
        .unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let after_undo = document.main().unwrap().find_child(2, false).unwrap();
    if !after_undo.is_null() {
        assert!(find_named_shape(&after_undo).is_none());
    }
    document.redo().unwrap();
    let label = support::main_child(&document, 2);
    assert!(!label.is_null());
    let named = named_shape_of(&label);
    assert!(TNamingNamedShape::get(&named).unwrap().is_same(&shape));
    document.clear_undos();
    document.clear_redos();
}

/// The shapes of caf/nam/A2.
struct GraphShapes {
    box1: Shape,
    box2: Shape,
    box3: Shape,
    box4: Shape,
    edges: Vec<Shape>,
    faces2: Vec<Shape>,
    faces3: Vec<Shape>,
    faces4: Vec<Shape>,
}

fn graph_shapes() -> GraphShapes {
    let box1 = make_box(100.0, 200.0, 300.0);
    let box2 = make_box(200.0, 300.0, 400.0);
    let box3 = make_box(300.0, 400.0, 500.0);
    let box4 = make_box_between([100.0, 100.0, 100.0], [300.0, 400.0, 500.0]);
    let edges: Vec<Shape> = (1..=12)
        .map(|i| shape_at(&box1, ShapeEnum::Edge, i))
        .collect();
    let faces = |shape: &Shape| -> Vec<Shape> {
        (1..=6)
            .map(|i| shape_at(shape, ShapeEnum::Face, i))
            .collect()
    };
    let (faces2, faces3, faces4) = (faces(&box2), faces(&box3), faces(&box4));
    for shape in edges.iter().chain(&faces2).chain(&faces3).chain(&faces4) {
        assert!(!shape.is_null());
    }
    GraphShapes {
        box1,
        box2,
        box3,
        box4,
        edges,
        faces2,
        faces3,
        faces4,
    }
}

/// The 14 labels of caf/nam/A2, built as the DRAW case does.
fn build_graph(test_label: &TdfLabel, s: &GraphShapes) -> Vec<TdfLabel> {
    let child = |label: &TdfLabel, tag: i32| label.find_child(tag, true).unwrap();
    let label1 = child(test_label, 1);
    let (label11, label12) = (child(&label1, 1), child(&label1, 2));
    let label2 = child(test_label, 2);
    let (label21, label22) = (child(&label2, 1), child(&label2, 2));
    let label3 = child(test_label, 3);
    let (label31, label32) = (child(&label3, 1), child(&label3, 2));
    let label41 = child(&child(test_label, 4), 1);
    let label5 = child(test_label, 5);
    let label51 = child(&label5, 1);
    let label6 = child(test_label, 6);
    let label61 = child(&label6, 1);
    let (e, f2, f3, f4) = (&s.edges, &s.faces2, &s.faces3, &s.faces4);
    build_primitive(&label1, std::slice::from_ref(&s.box1));
    build_primitive(&label11, &e[0..1]);
    build_primitive(&label11, &e[0..3]);
    build_primitive(&label12, &e[3..6]);
    build_generated(
        &label2,
        std::slice::from_ref(&s.box1),
        std::slice::from_ref(&s.box2),
    );
    build_generated(&label21, &e[0..1], &f2[0..1]);
    build_generated(
        &label22,
        &[e[1].clone(), e[1].clone(), e[2].clone()],
        &f2[1..4],
    );
    build_generated(
        &label21,
        &[e[2].clone(), e[2].clone(), e[3].clone()],
        &[f2[0].clone(), f2[2].clone(), f2[3].clone()],
    );
    build_modified(
        &label3,
        std::slice::from_ref(&s.box2),
        std::slice::from_ref(&s.box3),
    );
    build_modified(&label31, &f2[0..1], &f3[0..1]);
    build_modified(
        &label32,
        &[f2[4].clone(), f2[1].clone(), f2[2].clone()],
        &f3[1..4],
    );
    build_modified(
        &label31,
        &[f2[2].clone(), e[2].clone(), f2[3].clone()],
        &[f3[0].clone(), f3[2].clone(), f3[3].clone()],
    );
    build_deleted(&label41, &f3[0..1]);
    build_deleted(&label41, &[f3[0].clone(), f3[3].clone()]);
    build_modified(
        &label5,
        std::slice::from_ref(&s.box3),
        std::slice::from_ref(&s.box4),
    );
    build_modified(&label51, &f3[1..3], &f4[1..3]);
    build_selected(&label6, std::slice::from_ref(&s.box4));
    build_selected(&label61, &[f4[1].clone(), f2[4].clone()]);
    vec![
        label1, label11, label12, label2, label21, label22, label3, label31, label32, label41,
        label5, label51, label6, label61,
    ]
}

// caf/nam/A2: the complete TNaming iterator graph preserves direct, recursive, and
// same-shape relationships from the DRAW case.
#[test]
fn TNaming_NamedShape_Test__CafNamedShape_A2_IteratorGraph() {
    use TNamingEvolution::{Delete, Generated, Modify, Primitive, Selected};
    let document = support::new_document();
    let test_label = support::main_child(&document, 1);
    let access = document.get_data().root();
    let s = graph_shapes();
    let labels = build_graph(&test_label, &s);

    let expected_evolution = [
        Primitive, Primitive, Primitive, Generated, Generated, Generated, Modify, Modify, Modify,
        Delete, Modify, Modify, Selected, Selected,
    ];
    let expected_new = [1, 3, 3, 1, 3, 3, 1, 3, 3, 0, 1, 2, 1, 2];
    let expected_old = [0, 0, 0, 1, 3, 3, 1, 3, 3, 2, 1, 2, 1, 2];
    let expected_all_new = [3, 14, 2, 2, 3, 5, 1, 1, 2, 0, 0, 0, 0, 2];
    let expected_all_old = [0, 0, 0, 1, 3, 3, 2, 6, 5, 0, 3, 5, 3, 2];
    for (index, label) in labels.iter().enumerate() {
        assert_eq!(
            named_shape_of(label).evolution(),
            expected_evolution[index],
            "label {index}"
        );
        assert_eq!(
            count_new_shapes(label),
            expected_new[index],
            "label {index}"
        );
        assert_eq!(
            count_old_shapes(label),
            expected_old[index],
            "label {index}"
        );
        assert_eq!(
            count_all_new_shapes_at_label(label, &access),
            expected_all_new[index],
            "label {index}"
        );
        assert_eq!(
            count_all_old_shapes_at_label(label, &access),
            expected_all_old[index],
            "label {index}"
        );
    }

    let mut iterator_shapes = Vec::new();
    collect_all_new_shapes_at_label(&labels[1], &access, &mut iterator_shapes);
    let mut old_iterator_shapes = Vec::new();
    collect_all_old_shapes_at_label(&labels[7], &access, &mut old_iterator_shapes);
    assert_eq!(iterator_shapes.len(), 14);
    assert_eq!(old_iterator_shapes.len(), 6);
    let old_shape_order = [5, 6, 1, 2, 3, 4];
    let mut shapes = vec![s.box1.clone()];
    shapes.extend(iterator_shapes.iter().cloned());
    shapes.extend(
        old_shape_order
            .iter()
            .map(|&i| old_iterator_shapes[i - 1].clone()),
    );
    assert_eq!(shapes.len(), 21);

    let expected_same = [
        1, 2, 2, 1, 3, 2, 1, 2, 1, 1, 2, 1, 3, 2, 1, 3, 4, 3, 4, 1, 2,
    ];
    let expected_same_labels = [
        2, 3, 3, 1, 4, 3, 2, 3, 1, 2, 3, 1, 4, 3, 2, 4, 5, 4, 5, 2, 3,
    ];
    for (index, shape) in shapes.iter().enumerate() {
        assert_eq!(
            count_same_shapes(shape, &access),
            expected_same_labels[index],
            "shape {}",
            index + 1
        );
        assert_eq!(
            count_same_shapes_in_draw_result(shape, &access),
            expected_same[index],
            "shape {}",
            index + 1
        );
    }

    let expected_same_pairs: [(usize, usize); 14] = [
        (20, 10),
        (19, 17),
        (18, 16),
        (18, 13),
        (18, 5),
        (16, 13),
        (16, 5),
        (15, 7),
        (14, 6),
        (14, 3),
        (13, 5),
        (12, 9),
        (11, 8),
        (6, 3),
    ];
    for (first, second) in expected_same_pairs {
        assert!(shapes[first - 1].is_same(&shapes[second - 1]));
    }
    for first in 1..=shapes.len() {
        for second in 1..first {
            if !shapes[first - 1].is_same(&shapes[second - 1]) {
                continue;
            }
            let is_expected = expected_same_pairs
                .iter()
                .any(|&(a, b)| (a == first && b == second) || (a == second && b == first));
            assert!(is_expected, "unexpected same pair ({first}, {second})");
        }
    }
}
