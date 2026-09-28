//! TopoDS behaviour beyond the upstream suite: native quirks, errors and identity
//! (docs/topology-design.md). The native kernel comparison is the br_topods probe.
mod support;
use rustycad_tkbrep::topods::{
    self, AlertWithShape, Builder, Compound, Edge, HShape, Shape, ShapeIterator, Solid, TEdge,
    TShape, TSolid, Wire,
};
use rustycad_tkernel::message::Alert;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, dump::DumpStream};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use support::{make_edge, make_vertex};

fn compound() -> Compound {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    compound
}

fn wire() -> Wire {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    wire
}

fn translation(x: f64) -> Location {
    Location::new(Transform3::translation([x, 0.0, 0.0]))
}

#[test]
fn null_shape_members_raise_null_object_instead_of_crashing() {
    let null = Shape::default();
    assert!(null.is_null());
    assert_eq!(null.orientation(), Orientation::External);
    assert_eq!(null.nb_children(), 0);
    let failures = [
        null.shape_type().map(|_| ()).unwrap_err(),
        null.free().map(|_| ()).unwrap_err(),
        null.set_convex(true).unwrap_err(),
        null.empty_copied().map(|_| ()).unwrap_err(),
    ];
    for failure in failures {
        assert_eq!(failure.exception_type(), "Standard_NullObject");
    }
}

#[test]
fn nullify_resets_location_and_orientation() {
    let mut shape = make_vertex(Point3::default())
        .moved(&translation(1.0), false)
        .unwrap();
    shape.set_orientation(Orientation::Internal);
    shape.nullify();
    assert!(shape.is_null() && shape.location().is_identity());
    assert_eq!(shape.orientation(), Orientation::External);
}

#[test]
fn builder_refuses_incompatible_and_frozen_parents() {
    let mut wire = wire();
    let vertex = make_vertex(Point3::default());
    let error = Builder.add(&mut wire, &vertex).unwrap_err();
    assert_eq!(error.exception_type(), "TopoDS_UnCompatibleShapes");
    // The component is frozen before the parent is checked, even when Add fails.
    assert!(!vertex.free().unwrap());
    let mut holder = compound();
    Builder.add(&mut holder, &wire).unwrap();
    let edge = make_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0));
    let frozen = Builder.add(&mut wire, &edge).unwrap_err();
    assert_eq!(frozen.exception_type(), "TopoDS_FrozenShape");
    let removal = Builder.remove(&mut wire, &edge).unwrap_err();
    assert_eq!(removal.exception_type(), "TopoDS_FrozenShape");
}

#[test]
fn self_insertion_is_refused_by_the_freeze() {
    let mut shape = compound();
    let same = shape.clone();
    let error = Builder.add(&mut shape, &same).unwrap_err();
    assert_eq!(error.exception_type(), "TopoDS_FrozenShape");
    assert_eq!(shape.nb_children(), 0);
}

#[test]
fn null_components_and_parents_raise_null_object() {
    let mut shape = compound();
    let error = Builder.add(&mut shape, &Shape::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    let vertex = make_vertex(Point3::default());
    let mut null = Shape::default();
    assert_eq!(
        Builder
            .add(&mut null, &vertex)
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
    assert!(!vertex.free().unwrap());
}

#[test]
fn children_are_stored_relative_to_a_reversed_located_parent() {
    let mut parent = compound();
    parent.set_orientation(Orientation::Reversed);
    parent.set_location(&translation(2.0), false).unwrap();
    let child = make_vertex(Point3::default())
        .moved(&translation(5.0), false)
        .unwrap();
    Builder.add(&mut parent, &child).unwrap();
    let stored = ShapeIterator::new(&parent, false, false)
        .value()
        .unwrap()
        .clone();
    assert_eq!(stored.orientation(), Orientation::Reversed);
    assert!(!stored.location().is_equal(child.location()));
    let seen = ShapeIterator::new(&parent, true, true)
        .value()
        .unwrap()
        .clone();
    assert!(seen.is_equal(&child));
    Builder.remove(&mut parent, &child).unwrap();
    assert_eq!(parent.nb_children(), 0);
}

#[test]
fn removing_an_absent_child_keeps_the_modified_flag() {
    let mut shape = compound();
    shape.set_modified(false).unwrap();
    Builder
        .remove(&mut shape, &make_vertex(Point3::default()))
        .unwrap();
    assert!(!shape.modified().unwrap());
}

#[test]
fn raising_setters_refuse_scaling_and_leave_the_shape_unchanged() {
    let mut scale = Transform3::default();
    scale.set_scale(Point3::default(), 2.0).unwrap();
    let scaled = Location::new(scale);
    let mut shape: Shape = make_vertex(Point3::default()).into();
    let error = shape.set_location(&scaled, true).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert!(shape.location().is_identity());
    assert_eq!(
        shape.move_by(&scaled, true).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    shape.move_by(&scaled, false).unwrap();
    assert!(!shape.location().is_identity());
}

#[test]
fn tshape_constructors_set_the_native_default_flags() {
    let solid = TSolid::new();
    assert!(solid.free() && solid.modified() && !solid.orientable() && !solid.closed());
    let vertex = make_vertex(Point3::default());
    assert!(vertex.closed().unwrap() && vertex.convex().unwrap());
    let compound = compound();
    assert!(!compound.orientable().unwrap());
    assert_eq!(solid.shape_type(), ShapeEnum::Solid);
}

#[test]
fn modified_clears_checked_only_when_set() {
    let shape = compound();
    shape.set_checked(true).unwrap();
    shape.set_modified(false).unwrap();
    assert!(shape.checked().unwrap());
    shape.set_modified(true).unwrap();
    assert!(!shape.checked().unwrap());
    shape.set_locked(true).unwrap();
    assert!(shape.locked().unwrap() && shape.free().unwrap());
}

#[test]
fn empty_copies_keep_the_overriding_class() {
    let edge = make_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0));
    let copy = edge.empty_copied().unwrap();
    assert!(!copy.is_partner(&edge));
    assert_eq!(copy.nb_children(), 0);
    // The fake overrides EmptyCopy, as BRep_TEdge does; TopoDS_TEdge's own copy is plain.
    assert_eq!(copy.tshape().dynamic_type().unwrap().name(), "FakeTEdge");
    assert!(
        copy.tshape()
            .is_kind(<dyn TShape as RuntimeTyped>::static_type())
    );
    let plain = TEdge::new().empty_copy();
    assert_eq!(plain.dynamic_type().unwrap().name(), "TopoDS_TEdge");
    let mut solid = Solid::default();
    Builder.make_solid(&mut solid);
    assert_eq!(
        solid
            .empty_copied()
            .unwrap()
            .tshape()
            .dynamic_type()
            .unwrap()
            .name(),
        "TopoDS_TSolid"
    );
}

#[test]
fn casts_accept_null_shapes_and_refuse_other_types() {
    assert!(topods::edge(&Shape::default()).unwrap().is_null());
    let vertex: Shape = make_vertex(Point3::default()).into();
    assert!(topods::vertex(&vertex).is_ok());
    assert_eq!(
        topods::edge(&vertex).unwrap_err().exception_type(),
        "Standard_TypeMismatch"
    );
    let unchecked = Edge::unchecked(vertex.clone());
    assert_eq!(unchecked.shape_type().unwrap(), ShapeEnum::Vertex);
}

#[test]
fn identity_levels_and_hash_follow_the_native_formula() {
    let vertex: Shape = make_vertex(Point3::default()).into();
    let moved = vertex.moved(&translation(1.0), false).unwrap();
    let reversed = vertex.reversed();
    assert!(vertex.is_partner(&moved) && !vertex.is_same(&moved));
    assert!(vertex.is_same(&reversed) && !vertex.is_equal(&reversed) && vertex != reversed);
    let expected = Shape::hash_from_parts(vertex.tshape_address(), 0);
    assert_eq!(vertex.hash_code(), expected);
    assert_eq!(reversed.hash_code(), expected);
    let located = Shape::hash_from_parts(moved.tshape_address(), moved.location().hash_code());
    assert_eq!(moved.hash_code(), located);
    assert_eq!(Shape::default().hash_code(), Shape::hash_from_parts(0, 0));
}

#[test]
fn iterator_protocol_and_native_range() {
    let mut shape = compound();
    for x in 0..3 {
        Builder
            .add(
                &mut shape,
                &make_vertex(Point3::new(f64::from(x), 0.0, 0.0)),
            )
            .unwrap();
    }
    let mut iterator = ShapeIterator::new(&shape, true, true);
    assert_eq!(iterator.clone().count(), 3);
    for _ in 0..3 {
        iterator.next();
    }
    iterator.next();
    assert!(!iterator.more());
    assert_eq!(
        iterator.value().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert!(!ShapeIterator::default().more());
}

#[test]
fn iterator_composes_orientation_only_when_asked() {
    let mut shape = compound();
    Builder
        .add(
            &mut shape,
            &make_vertex(Point3::default()).oriented(Orientation::Reversed),
        )
        .unwrap();
    shape.set_orientation(Orientation::Reversed);
    let composed = ShapeIterator::new(&shape, true, false)
        .value()
        .unwrap()
        .orientation();
    let raw = ShapeIterator::new(&shape, false, false)
        .value()
        .unwrap()
        .orientation();
    assert_eq!(
        (composed, raw),
        (Orientation::Forward, Orientation::Reversed)
    );
}

#[test]
fn dump_json_omits_nested_objects_at_depth_zero() {
    let shape = compound();
    let mut full = DumpStream::default();
    shape.dump_json(&mut full, -1).unwrap();
    assert!(full.text().starts_with(
        "\"className\": \"TopoDS_Shape\", \"TShape\": {\"className\": \"TopoDS_TShape\""
    ));
    let mut shallow = DumpStream::default();
    shape.dump_json(&mut shallow, 0).unwrap();
    assert_eq!(
        shallow.text(),
        "\"className\": \"TopoDS_Shape\", \"Orient\": 0"
    );
}

#[test]
fn hshape_and_alert_share_their_shape() {
    let shape: Shape = compound().into();
    let holder = Handle::new(HShape::new(&shape));
    holder.change_shape(|held| held.reverse());
    assert!(holder.shape().is_same(&shape) && !holder.shape().is_equal(&shape));
    let alert = AlertWithShape::new(&shape);
    assert!(alert.get_shape().is_equal(&shape) && !alert.supports_merge());
    alert.set_shape(&Shape::default());
    assert!(alert.get_shape().is_null());
}

#[test]
fn typed_shapes_convert_to_shapes() {
    let tshape: Handle<dyn TShape> = Handle::new(TSolid::new()).into_cast();
    let mut solid = Solid::default();
    Builder.make_shape(&mut solid, &tshape);
    assert_eq!(solid.orientation(), Orientation::Forward);
    let shape: Shape = solid.clone().into();
    assert!(shape.is_equal(&solid) && solid.into_shape().is_equal(&shape));
}
