//! Port tests of the TNaming package functions (no upstream GTest covers them directly).
mod support;

use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Shell, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkcaf::tnaming::{
    TNaming, TNamingBuilder, TNamingEvolution, TNamingIterator, TNamingNameType,
    TNamingShapeShapeMap, TNamingTool,
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfData, TdfLabel};
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::{explore, find_named_shape, make_box, shape_at};

fn child(label: &TdfLabel, tag: i32) -> TdfLabel {
    label.find_child(tag, true).unwrap()
}

fn first_new_shape(label: &TdfLabel) -> Shape {
    TNamingIterator::from_label(label)
        .unwrap()
        .new_shape()
        .unwrap()
}

fn translation(dx: f64) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(dx, 0.0, 0.0));
    transform
}

#[test]
fn displace_moves_the_shapes_of_the_tree() {
    let data = TdfData::new();
    let label = child(&data.root(), 1);
    let sub_label = child(&label, 1);
    let solid = make_box(1.0, 2.0, 3.0);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    let face = shape_at(&solid, ShapeEnum::Face, 1);
    TNamingBuilder::new(&sub_label)
        .unwrap()
        .generated(&face)
        .unwrap();
    let location = Location::new(translation(5.0));
    TNaming::displace(&label, &location, true).unwrap();
    for (tagged, original) in [(&label, &solid), (&sub_label, &face)] {
        let moved = first_new_shape(tagged);
        assert!(moved.is_partner(original) && !moved.is_same(original));
        assert!(moved.location().is_equal(&location));
    }
    assert_eq!(find_named_shape(&label).unwrap().version(), 1);
}

#[test]
fn transform_replaces_shapes_by_their_images() {
    // Transform iterates a label while its builder clears it: the named shape must be backed
    // up (a later transaction), else the native iterator reads deleted nodes.
    let document = support::new_document();
    document.set_undo_limit(10).unwrap();
    let label = support::main_child(&document, 1);
    let solid = make_box(1.0, 2.0, 3.0);
    document.new_command().unwrap();
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    document.commit_command().unwrap();
    document.new_command().unwrap();
    TNaming::transform(&label, &translation(2.0)).unwrap();
    document.commit_command().unwrap();
    let image = first_new_shape(&label);
    assert!(image.is_partner(&solid) && !image.is_same(&solid));
    assert_eq!(
        find_named_shape(&label).unwrap().evolution(),
        TNamingEvolution::Primitive
    );
    // The undo restores the original shape.
    document.undo().unwrap();
    assert!(first_new_shape(&label).is_same(&solid));
}

#[test]
fn change_shapes_copies_every_tshape() {
    let data = TdfData::new();
    let label = child(&data.root(), 1);
    let solid = make_box(1.0, 2.0, 3.0);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    let mut map = TNamingShapeShapeMap::default();
    TNaming::change_shapes(&label, &mut map).unwrap();
    let copy = first_new_shape(&label);
    assert!(!copy.is_partner(&solid));
    assert!(map.find(&solid).unwrap().is_same(&copy));
    assert_eq!(explore(&copy, ShapeEnum::Face).len(), 6);
    assert_eq!(copy.closed().unwrap(), solid.closed().unwrap());
    // The faces of the copy are copies too, shared with the map.
    let face = shape_at(&solid, ShapeEnum::Face, 1);
    assert!(!map.find(&face).unwrap().is_partner(&face));
}

#[test]
fn replicate_generates_the_transformed_shape_and_faces() {
    let data = TdfData::new();
    let source = child(&data.root(), 1);
    let solid = make_box(1.0, 2.0, 3.0);
    TNamingBuilder::new(&source)
        .unwrap()
        .generated(&solid)
        .unwrap();
    let target = child(&data.root(), 2);
    TNaming::replicate(
        &find_named_shape(&source).unwrap(),
        &translation(4.0),
        &target,
    )
    .unwrap();
    let named = find_named_shape(&target).unwrap();
    assert_eq!(named.evolution(), TNamingEvolution::Generated);
    let mut iterator = TNamingIterator::new(&named).unwrap();
    assert!(iterator.old_shape().unwrap().is_same(&solid));
    iterator.next().unwrap();
    assert!(!iterator.more());
    let mut faces = 0;
    let mut sub_iterator = TNamingIterator::from_label(&child(&target, 1)).unwrap();
    while sub_iterator.more() {
        faces += 1;
        sub_iterator.next().unwrap();
    }
    assert_eq!(faces, 6);
}

#[test]
fn make_shape_and_substitution() {
    let data = TdfData::new();
    let (first, second) = (make_box(1.0, 1.0, 1.0), make_box(2.0, 2.0, 2.0));
    let mut shapes = MapOfShape::default();
    assert!(TNaming::make_shape(&shapes).unwrap().is_null());
    shapes.add(first.clone());
    assert!(TNaming::make_shape(&shapes).unwrap().is_same(&first));
    shapes.add(second.clone());
    let compound = TNaming::make_shape(&shapes).unwrap();
    assert_eq!(compound.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(compound.nb_children(), 2);

    let label = child(&data.root(), 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&first)
        .unwrap();
    let mut reversed = first.reversed();
    assert!(TNaming::substitute_s_shape(&label, &first, &mut reversed).unwrap());
    assert_eq!(reversed.orientation(), first.orientation());
    let mut missing = second.clone();
    assert!(!TNaming::substitute_s_shape(&label, &first, &mut missing).unwrap());
}

#[test]
fn update_keeps_the_old_key_bound_as_natively() {
    let data = TdfData::new();
    let label = child(&data.root(), 1);
    let (first, second) = (make_box(1.0, 1.0, 1.0), make_box(2.0, 2.0, 2.0));
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&first)
        .unwrap();
    let mut map = TNamingShapeShapeMap::default();
    map.bind(first.clone(), second.clone());
    TNaming::update(&label, &mut map).unwrap();
    // The ref shape now holds the new shape, both keys reach it.
    assert!(first_new_shape(&label).is_same(&second));
    assert!(TNamingTool::has_label(&label, &first).unwrap());
    assert!(TNamingTool::has_label(&label, &second).unwrap());
}

#[test]
fn substitute_rebinds_the_parallel_tree() {
    let data = TdfData::new();
    let source = child(&data.root(), 1);
    let target = child(&data.root(), 2);
    let (first, second) = (make_box(1.0, 1.0, 1.0), make_box(2.0, 2.0, 2.0));
    TNamingBuilder::new(&source)
        .unwrap()
        .generated(&first)
        .unwrap();
    TNamingBuilder::new(&target)
        .unwrap()
        .generated(&second)
        .unwrap();
    let mut map = TNamingShapeShapeMap::default();
    TNaming::substitute(&source, &target, &mut map).unwrap();
    assert!(map.find(&second).unwrap().is_same(&first));
    assert!(!TNamingTool::has_label(&target, &second).unwrap());
    assert!(first_new_shape(&target).is_same(&first));
}

#[test]
fn contexts_and_outer_boundaries_of_a_box() {
    let solid = make_box(1.0, 2.0, 3.0);
    let shell = Iterator::next(&mut ShapeIterator::new(&solid, true, true)).unwrap();
    let face = shape_at(&solid, ShapeEnum::Face, 1);
    assert!(
        TNaming::find_unique_context(&face, &solid)
            .unwrap()
            .is_same(&shell)
    );
    let mut arrays: Handle<HArray1<Shape>> = Handle::default();
    let single = TNaming::find_unique_context_set(&face, &solid, &mut arrays).unwrap();
    assert!(single.is_null() && arrays.is_null());

    let mut wire = Wire::default();
    assert!(TNaming::outer_wire(&topods::face(&face).unwrap(), &mut wire).unwrap());
    assert!(!wire.is_null());
    let mut outer = Shell::default();
    assert!(TNaming::outer_shell(&topods::solid(&solid).unwrap(), &mut outer).unwrap());
    assert!(outer.is_same(&shell));
}

#[test]
fn the_print_functions_name_the_enumerators() {
    let mut text = Vec::new();
    TNaming::print_evolution(TNamingEvolution::Replace, &mut text).unwrap();
    TNaming::print_name_type(TNamingNameType::FilterByNeighbourgs, &mut text).unwrap();
    assert_eq!(text, b"UNKNOWN_EvolutionFILTERBYNEIGHBOURGS");
    let data = TdfData::new();
    let mut empty = Vec::new();
    TNaming::print_label(&data.root(), &mut empty).unwrap();
    assert!(empty.is_empty());
    let label = child(&data.root(), 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&make_box(1.0, 1.0, 1.0))
        .unwrap();
    let mut dump = Vec::new();
    TNaming::print_label(&label, &mut dump).unwrap();
    let dump = String::from_utf8(dump).unwrap();
    assert!(
        dump.starts_with("The content of UsedShapes attribute:\n  SOLID  0:1 Key_TShape"),
        "{dump}"
    );
    let mut ids = Vec::new();
    TNaming::id_list(&mut ids);
    assert_eq!(ids.len(), 1);
}
