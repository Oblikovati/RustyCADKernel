// SPDX-License-Identifier: GPL-2.0-only
//! Exact structural replay of pinned native sweep roles; no OCCT dependency at test runtime.
use crate::sketch_region_candidates::tests::rectangle;
use rustycad_tkbrep::{
    topexp,
    topods::{Shape, edge},
    toptools::IndexedMapOfShape,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPICopy, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
fn rectangle_face() -> Shape {
    let mut wire = BRepBuilderAPIMakeWire::new();
    for shape in rectangle(0., 0., 4., 3.) {
        wire.add_edge(&edge(&shape).unwrap()).unwrap();
    }
    BRepBuilderAPIMakeFace::from_wire(&wire.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}
fn native_case(reverse: bool) -> String {
    let mut face = rectangle_face();
    if reverse {
        face.reverse();
    }
    let mut copier = BRepBuilderAPICopy::with_shape(&face, true, true).unwrap();
    let base = copier.shape().unwrap();
    let mut maker = BRepPrimAPIMakePrism::new(base, Vector3::new(0., 0., 2.), false, true).unwrap();
    let inputs = physical_map(base);
    let outputs = physical_map(maker.shape().unwrap());
    captured_rows(reverse, &inputs, &outputs, &mut maker)
}
fn captured_rows(
    reverse: bool,
    inputs: &IndexedMapOfShape,
    outputs: &IndexedMapOfShape,
    maker: &mut BRepPrimAPIMakePrism,
) -> String {
    let mut rows = case_header(reverse, inputs.extent(), outputs.extent());
    for index in 1..=inputs.extent() {
        rows.push_str(&native_member(
            maker,
            inputs.find_key(index).unwrap(),
            index,
        ));
    }
    rows
}

fn native_member(maker: &mut BRepPrimAPIMakePrism, base: &Shape, index: usize) -> String {
    let generated = maker.generated(base).unwrap().clone();
    let first = maker.first_shape_of(base).unwrap();
    let last = maker.last_shape_of(base).unwrap();
    let mut row = format!(
        "member {} {} {}",
        index,
        base.shape_type().unwrap() as i32,
        generated.len()
    );
    for shape in &generated {
        row.push_str(&format!(" {}", shape.shape_type().unwrap() as i32));
    }
    row.push_str(&cap_row(&first, &last, maker.is_deleted(base).unwrap()));
    row
}
#[test]
fn actual_native_generated_cap_and_wire_retirement_rows_replay_without_occt() {
    let actual = native_case(false) + &native_case(true);
    assert_eq!(
        actual,
        include_str!("../../tests/fixtures/prism-history-native.txt")
    );
    assert_eq!(Orientation::Reversed as i32, 1);
}

fn physical_map(shape: &Shape) -> IndexedMapOfShape {
    let mut found = IndexedMapOfShape::default();
    topexp::map_shapes(shape, &mut found, true, true);
    found
}
fn cap_row(first: &Shape, last: &Shape, deleted: bool) -> String {
    format!(
        " {} {} {}\n",
        first.shape_type().unwrap() as i32,
        last.shape_type().unwrap() as i32,
        u8::from(deleted)
    )
}

fn case_header(reverse: bool, inputs: usize, outputs: usize) -> String {
    format!("case {} {} {}\n", u8::from(reverse), inputs, outputs)
}
