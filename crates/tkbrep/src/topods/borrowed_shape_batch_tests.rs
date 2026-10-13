// SPDX-License-Identifier: GPL-2.0-only
//! The read-only batch is deliberately different from mapped Add's freezing behavior (ADR0014).
use super::{Builder, Compound, Shape, ShapeIterator, Vertex, borrowed_shape_batch};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};

#[test]
fn mapped_add_freezes_borrowed_components_while_batch_preserves_both_free_states() {
    for initial in [false, true] {
        let mut vertex = Vertex::default();
        crate::brep::BRepBuilder::default().make_vertex(&mut vertex);
        vertex.set_free(initial).unwrap();
        let batch = borrowed_shape_batch(&[&vertex]).unwrap();
        assert_eq!(vertex.free().unwrap(), initial);
        assert_eq!(batch.nb_children(), 1);
        let mut ordinary = Compound::default();
        Builder.make_compound(&mut ordinary);
        Builder.add(&mut ordinary, &vertex).unwrap();
        assert!(!vertex.free().unwrap());
    }
}
#[test]
fn batch_preserves_alias_orientation_location_and_input_order_without_freezing() {
    let mut vertex = Vertex::default();
    crate::brep::BRepBuilder::default().make_vertex(&mut vertex);
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(3., 4., 5.));
    let located = vertex.located(&Location::new(transform), true).unwrap();
    let reversed = located.oriented(Orientation::Reversed);
    let batch = borrowed_shape_batch(&[&located, &reversed]).unwrap();
    let mut children = ShapeIterator::new(&batch, true, true);
    assert!(children.value().unwrap().is_equal(&located));
    children.next();
    assert!(children.value().unwrap().is_equal(&reversed));
    assert!(located.is_same(&reversed) && vertex.free().unwrap());
}
#[test]
fn batch_null_rejection_and_empty_container_are_explicit() {
    let failure = borrowed_shape_batch(&[&Shape::default()]).unwrap_err();
    assert!(failure.message().contains("component=0"));
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    let empty = borrowed_shape_batch(&[]).unwrap();
    assert_eq!(empty.nb_children(), 0);
}
