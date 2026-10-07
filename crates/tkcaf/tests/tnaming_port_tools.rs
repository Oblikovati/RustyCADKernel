//! Port tests of TNaming_NamingTool, TNaming_Scope and TNaming_ShapesSet on a modification
//! chain (no upstream GTest covers them directly).
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingNamingTool, TNamingScope};
use rustycad_tkernel::standard::Handle;
use rustycad_tklcaf::tdf::{TdfData, TdfLabel, TdfLabelMap};
use support::{find_named_shape, make_box};

/// Three labels: a primitive box, then two successive modifications.
struct Chain {
    _data: Handle<TdfData>,
    labels: [TdfLabel; 3],
    shapes: [Shape; 3],
}

fn chain() -> Chain {
    let data = TdfData::new();
    let labels = [1, 2, 3].map(|tag| data.root().find_child(tag, true).unwrap());
    let shapes = [
        make_box(1.0, 1.0, 1.0),
        make_box(2.0, 2.0, 2.0),
        make_box(3.0, 3.0, 3.0),
    ];
    TNamingBuilder::new(&labels[0])
        .unwrap()
        .generated(&shapes[0])
        .unwrap();
    TNamingBuilder::new(&labels[1])
        .unwrap()
        .modify(&shapes[0], &shapes[1])
        .unwrap();
    TNamingBuilder::new(&labels[2])
        .unwrap()
        .modify(&shapes[1], &shapes[2])
        .unwrap();
    Chain {
        _data: data,
        labels,
        shapes,
    }
}

fn single(shapes: &IndexedMapOfShape) -> Shape {
    assert_eq!(shapes.extent(), 1);
    shapes.find_key(1).unwrap().clone()
}

#[test]
fn descendants_follow_the_modifications() {
    let chain = chain();
    let named = find_named_shape(&chain.labels[0]).unwrap();
    let mut descendants = TdfLabelMap::default();
    TNamingNamingTool::build_descendants(&named, &mut descendants).unwrap();
    assert_eq!(descendants.extent(), 3);
    for label in &chain.labels {
        assert!(descendants.contains(label));
    }
    // From the middle: the old shape's named shape adds its other descendants only.
    let middle = find_named_shape(&chain.labels[1]).unwrap();
    let mut from_middle = TdfLabelMap::default();
    TNamingNamingTool::build_descendants(&middle, &mut from_middle).unwrap();
    assert!(from_middle.contains(&chain.labels[1]) && from_middle.contains(&chain.labels[2]));
}

#[test]
fn current_shapes_stop_at_forbidden_and_invalid_labels() {
    let chain = chain();
    let named = find_named_shape(&chain.labels[0]).unwrap();
    let none = TdfLabelMap::default();
    let mut current = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape(&none, &none, &named, &mut current).unwrap();
    assert!(single(&current).is_same(&chain.shapes[2]));

    let mut forbidden = TdfLabelMap::default();
    forbidden.add(chain.labels[2].clone());
    let mut stopped = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape(&none, &forbidden, &named, &mut stopped).unwrap();
    assert!(single(&stopped).is_same(&chain.shapes[1]));

    let mut valid = TdfLabelMap::default();
    valid.add(chain.labels[1].clone());
    let mut outside = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape(&valid, &none, &named, &mut outside).unwrap();
    assert!(outside.is_empty());

    let mut from_shape = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape_from_shape(
        &none,
        &none,
        &chain.labels[0],
        &chain.shapes[0],
        &mut from_shape,
    )
    .unwrap();
    assert!(single(&from_shape).is_same(&chain.shapes[2]));
    // The native code asks for the named shape of an iterator at its end.
    let error = TNamingNamingTool::current_shape_from_shape(
        &none,
        &none,
        &chain.labels[0],
        &chain.shapes[2],
        &mut IndexedMapOfShape::default(),
    )
    .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
}

#[test]
fn a_scope_restricts_the_current_shape() {
    let chain = chain();
    let named = find_named_shape(&chain.labels[0]).unwrap();
    let whole = TNamingScope::new();
    assert!(
        whole
            .current_shape(&named)
            .unwrap()
            .is_same(&chain.shapes[2])
    );
    let mut valid = TdfLabelMap::default();
    valid.add(chain.labels[0].clone());
    valid.add(chain.labels[1].clone());
    let scoped = TNamingScope::with_map(&valid);
    assert!(
        scoped
            .current_shape(&named)
            .unwrap()
            .is_same(&chain.shapes[1])
    );
}
