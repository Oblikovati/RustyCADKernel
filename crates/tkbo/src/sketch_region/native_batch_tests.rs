// SPDX-License-Identifier: GPL-2.0-only
//! Replay actual mapped Add/copy state independently of the intentionally nonfreezing project batch.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Shape, ShapeIterator, Vertex, vertex},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

fn native_vertex(free: bool) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(1., 2., 3.), 1e-7)
        .unwrap();
    vertex.set_free(free).unwrap();
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(3., 4., 5.));
    vertex.located(&Location::new(transform), true).unwrap()
}
fn native_batch(located: &Shape, reversed: &Shape) -> Shape {
    let builder = BRepBuilder::default();
    let mut batch = Compound::default();
    builder.make_compound(&mut batch);
    builder.add(&mut batch, located).unwrap();
    builder.add(&mut batch, reversed).unwrap();
    batch.into_shape()
}
fn copy_rows(batch: &Shape, located: &Shape, reversed: &Shape, free: bool) -> String {
    let mut copier = BRepBuilderAPICopy::with_shape(batch, true, true).unwrap();
    let mut copied = ShapeIterator::new(copier.shape().unwrap(), true, true);
    let first = copied.value().unwrap().clone();
    copied.next();
    let second = copied.value().unwrap();
    let row = format!(
        "copy {} {} {} {} {} {}\n",
        u8::from(free),
        u8::from(first.is_same(second)),
        u8::from(first.is_same(located)),
        u8::from(first.orientation() == located.orientation()),
        u8::from(second.orientation() == reversed.orientation()),
        u8::from(first.location() == located.location())
    );
    row + &world_rows(located, &first, free)
}
fn world_rows(source: &Shape, copied: &Shape, free: bool) -> String {
    let source_point = BRepTool::pnt(&vertex(source).unwrap()).unwrap();
    let copied_point = BRepTool::pnt(&vertex(copied).unwrap()).unwrap();
    let row = format!(
        "world {} {} {} {} {} {}\n",
        u8::from(free),
        source.location().transformation().value(1, 4).unwrap(),
        copied.location().transformation().value(1, 4).unwrap(),
        source_point.x(),
        copied_point.x(),
        source_point.distance(&copied_point)
    );
    row + &matrix_row(source, free, "source") + &matrix_row(copied, free, "copy")
}
fn matrix_row(shape: &Shape, free: bool, label: &str) -> String {
    let transform = shape.location().transformation();
    let mut row = format!("matrix {} {label}", u8::from(free));
    for i in 1..=3 {
        for j in 1..=4 {
            row.push_str(&format!(" {}", transform.value(i, j).unwrap()));
        }
    }
    row.push('\n');
    row
}
fn capture_native_batch(free: bool) -> String {
    let located = native_vertex(free);
    let reversed = located.oriented(Orientation::Reversed);
    let batch = native_batch(&located, &reversed);
    let row = format!(
        "add {} {} {}\n",
        u8::from(free),
        u8::from(located.free().unwrap()),
        u8::from(located.is_same(&reversed))
    );
    row + &copy_rows(&batch, &located, &reversed, free)
}
#[test]
fn mapped_flags_aliases_matrices_and_world_geometry_match_pinned_native() {
    let actual = capture_native_batch(false) + &capture_native_batch(true);
    let native = include_str!("../../tests/fixtures/borrowed-batch-native.txt");
    assert_eq!(actual, native);
}
