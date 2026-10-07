//! Native replay of BOPAlgo_MakePeriodic; see tools/oracle/tkbo_make_periodic.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::BOPAlgoMakePeriodic;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakePrism};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};
use shape_dump::{dump_result, o};
use std::fmt::Write;

fn profile() -> Shape {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, z) in [
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 10.0),
        (5.0, 10.0),
        (5.0, 5.0),
        (0.0, 5.0),
    ] {
        polygon.add_point(Point3::new(x, 0.0, z)).unwrap();
    }
    polygon.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap();
    let mut prism =
        BRepPrimAPIMakePrism::new(&face.into_shape(), Vector3::new(0.0, 5.0, 0.0), false, true)
            .unwrap();
    prism.shape().unwrap().clone()
}

fn explore(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    found
}

/// Sorted by result index like the oracle: the merged history lists follow the bucket
/// order of TShape address hashes, an allocation artefact. The twins print no orientation:
/// a twin group keeps the first oriented copy met in that order, so it varies natively.
fn list<'a>(
    tag: &str,
    shapes: impl Iterator<Item = &'a Shape>,
    all: &IndexedMapOfShape,
    oriented: bool,
    rows: &mut String,
) {
    let orientation = |shape: &Shape| {
        if oriented {
            o(shape.orientation())
        } else {
            '-'
        }
    };
    let mut items: Vec<(usize, char)> = shapes
        .map(|shape| (all.find_index(shape), orientation(shape)))
        .collect();
    items.sort_unstable();
    write!(rows, "{tag} {}:", items.len()).unwrap();
    for (index, orientation) in items {
        write!(rows, " {index}{orientation}").unwrap();
    }
    writeln!(rows).unwrap();
}

fn faces(
    label: &str,
    maker: &BOPAlgoMakePeriodic,
    input: &Shape,
    result: &Shape,
    rows: &mut String,
) {
    let mut all = IndexedMapOfShape::default();
    topexp::map_shapes(result, &mut all, true, true);
    writeln!(rows, "faces {label}").unwrap();
    let history = maker.history().get().unwrap();
    for face in explore(input, ShapeEnum::Face) {
        list(
            "modified",
            history.modified(&face).unwrap().iter(),
            &all,
            true,
            rows,
        );
        list(
            "generated",
            history.generated(&face).unwrap().iter(),
            &all,
            true,
            rows,
        );
    }
    for face in explore(result, ShapeEnum::Face) {
        list("twins", maker.get_twins(&face).iter(), &all, false, rows);
    }
    // The native repeated edge twins follow the address order of the twin map (see the
    // oracle), so they are not compared.
    if label == "repeated" {
        return;
    }
    for edge in explore(result, ShapeEnum::Edge) {
        list(
            "edge-twins",
            maker.get_twins(&edge).iter(),
            &all,
            false,
            rows,
        );
    }
}

fn run(label: &str, maker: &mut BOPAlgoMakePeriodic, input: &Shape, rows: &mut String) {
    maker.set_shape(input);
    maker.options_mut().set_run_parallel(false);
    maker.perform().unwrap();
    let (errors, warnings) = (maker.options().has_errors(), maker.options().has_warnings());
    writeln!(
        rows,
        "case {label} errors {} warnings {}",
        i32::from(errors),
        i32::from(warnings)
    )
    .unwrap();
    if errors {
        return;
    }
    dump_result(maker.shape(), rows);
    faces("periodic", maker, input, &maker.shape().clone(), rows);
}

fn cube(size: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn profile_case(rows: &mut String) {
    let shape = profile();
    let mut maker = BOPAlgoMakePeriodic::new();
    maker.make_x_periodic(true, 10.0);
    maker.make_z_periodic(true, 10.0);
    maker.make_y_periodic(true, 5.0);
    run("profile", &mut maker, &shape, rows);
    maker.x_repeat(1).unwrap();
    maker.y_repeat(2).unwrap();
    writeln!(
        rows,
        "repeated warnings {}",
        i32::from(maker.options().has_warnings())
    )
    .unwrap();
    dump_result(maker.repeated_shape(), rows);
    faces(
        "repeated",
        &maker,
        &shape,
        &maker.repeated_shape().clone(),
        rows,
    );
    maker.clear_repetitions().unwrap();
    writeln!(
        rows,
        "cleared null {}",
        i32::from(maker.repeated_shape().is_null())
    )
    .unwrap();
    faces("cleared", &maker, &shape, &maker.shape().clone(), rows);
    maker.z_repeat(-1).unwrap();
    dump_result(maker.repeated_shape(), rows);
}

fn trimmed_case(rows: &mut String) {
    let mut trimmed = BOPAlgoMakePeriodic::new();
    trimmed.make_x_periodic(true, 5.0);
    trimmed.make_y_periodic(true, 8.0);
    trimmed.make_z_periodic(true, 12.0);
    trimmed.set_trimmed(0, false, 2.0);
    trimmed.set_trimmed(1, false, 1.0);
    trimmed.set_trimmed(2, false, -1.0);
    run("trimmed-box", &mut trimmed, &cube(10.0), rows);
    trimmed.x_repeat(2).unwrap();
    dump_result(trimmed.repeated_shape(), rows);
}

#[test]
fn make_periodic_matches_native_twins_and_history() {
    let mut rows = String::new();
    profile_case(&mut rows);
    trimmed_case(&mut rows);
    let mut none = BOPAlgoMakePeriodic::new();
    run("no-periodicity", &mut none, &cube(1.0), &mut rows);
    let ids = (
        BOPAlgoMakePeriodic::to_direction_id(-4),
        BOPAlgoMakePeriodic::to_direction_id(5),
    );
    writeln!(rows, "direction {} {}", ids.0, ids.1).unwrap();
    let expected = include_str!("../../../tools/oracle/tkbo_make_periodic.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(rows.lines().count(), expected.lines().count());
}
