//! Port tests of TNaming_Localizer and TNaming_Identifier on a named box (no upstream GTest
//! covers them directly; the naming GTests go through them).
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkcaf::tnaming::{
    TNamingBuilder, TNamingIdentifier, TNamingLocalizer, TNamingNameType, TNamingUsedShapes,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfData, TdfLabel};
use support::{explore, find_named_shape, make_box, shape_at};

fn child(label: &TdfLabel, tag: i32) -> TdfLabel {
    label.find_child(tag, true).unwrap()
}

fn used_shapes(data: &Handle<TdfData>) -> Handle<TNamingUsedShapes> {
    let mut used: Handle<TNamingUsedShapes> = Handle::default();
    data.root()
        .find_attribute_as(&TNamingUsedShapes::get_id(), &mut used)
        .unwrap();
    used
}

#[test]
fn a_primitive_is_named_by_modification_until_it() {
    let data = TdfData::new();
    let label = child(&data.root(), 1);
    let solid = make_box(10.0, 20.0, 30.0);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    let identifier = TNamingIdentifier::new(&label, &solid, &Shape::default(), false).unwrap();
    assert!(identifier.is_done() && identifier.is_feature());
    assert_eq!(identifier.name_type(), TNamingNameType::ModifUntil);
    assert!(identifier.feature().unwrap() == find_named_shape(&label).unwrap());
    assert!(identifier.shape_arg().is_err());
}

#[test]
fn an_unnamed_edge_is_the_intersection_of_its_named_faces() {
    let data = TdfData::new();
    let solid = make_box(10.0, 20.0, 30.0);
    let solid_label = child(&data.root(), 1);
    TNamingBuilder::new(&solid_label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    // Without named faces there is nothing to intersect.
    let edge = shape_at(&solid, ShapeEnum::Edge, 1);
    let unnamed = TNamingIdentifier::new(&solid_label, &edge, &solid, false).unwrap();
    assert!(!unnamed.is_done());
    let faces = explore(&solid, ShapeEnum::Face);
    let face_builder = TNamingBuilder::new(&child(&solid_label, 1)).unwrap();
    for face in &faces {
        face_builder.generated(face).unwrap();
    }
    let mut identifier = TNamingIdentifier::new(&solid_label, &edge, &solid, false).unwrap();
    assert!(identifier.is_done() && !identifier.is_feature());
    assert_eq!(identifier.name_type(), TNamingNameType::Intersection);
    let mut arguments = 0;
    while identifier.more_args() {
        assert!(!identifier.arg_is_feature());
        let face = identifier.shape_arg().unwrap();
        assert!(
            explore(&face, ShapeEnum::Edge)
                .iter()
                .any(|e| e.is_same(&edge))
        );
        identifier.next_arg();
        arguments += 1;
    }
    assert_eq!(arguments, 2);
}

#[test]
fn a_generated_shape_is_named_by_its_generators() {
    let data = TdfData::new();
    let solid = make_box(10.0, 20.0, 30.0);
    let other = make_box(1.0, 2.0, 3.0);
    let label = child(&data.root(), 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    let generation = child(&data.root(), 2);
    TNamingBuilder::new(&generation)
        .unwrap()
        .generated_from(&solid, &other)
        .unwrap();
    let identifier = TNamingIdentifier::new(&generation, &other, &Shape::default(), false).unwrap();
    assert!(identifier.is_done());
    assert_eq!(identifier.name_type(), TNamingNameType::Generation);
    assert!(identifier.shape_arg().unwrap().is_same(&solid));
    assert!(identifier.named_shape_of_generation() == find_named_shape(&generation).unwrap());
}

#[test]
fn the_localizer_caches_sub_shapes_and_finds_neighbours() {
    let data = TdfData::new();
    let solid = make_box(10.0, 20.0, 30.0);
    let label = child(&data.root(), 1);
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    let mut localizer = TNamingLocalizer::new();
    localizer.init(&used_shapes(&data), 0);
    assert_eq!(
        localizer
            .sub_shapes(&solid, ShapeEnum::Face)
            .unwrap()
            .extent(),
        6
    );
    // The cache is found again by IsSame.
    assert_eq!(
        localizer
            .sub_shapes(&solid.reversed(), ShapeEnum::Face)
            .unwrap()
            .extent(),
        6
    );
    let face = shape_at(&solid, ShapeEnum::Face, 1);
    let mut neighbours = MapOfShape::default();
    localizer
        .find_neighbourg(&solid, &face, &mut neighbours)
        .unwrap();
    assert_eq!(neighbours.extent(), 4);
    assert!(!neighbours.contains(&face));
    let ancestors = localizer.ancestors(&solid, ShapeEnum::Edge).unwrap();
    assert_eq!(ancestors.extent(), 12);
    assert!(TNamingLocalizer::is_new(&solid, &find_named_shape(&label).unwrap()).unwrap());
}
