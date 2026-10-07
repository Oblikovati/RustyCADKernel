//! Native replay of the BRepAlgoAPI Boolean classes; see tools/oracle/tkbo_brep_algo_api.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbo::brep_algo_api::{
    BRepAlgoAPIBuilderAlgo, BRepAlgoAPICommon, BRepAlgoAPICut, BRepAlgoAPIFuse, BRepAlgoAPISection,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use shape_dump::{dump_result, index_of};
use std::fmt::Write;

fn report(label: &str, api: &BRepAlgoAPIBuilderAlgo, rows: &mut String) {
    writeln!(
        rows,
        "case {label} done {} errors {} warnings {} modified {} generated {} deleted {}",
        i32::from(api.is_done()),
        i32::from(api.has_errors()),
        i32::from(api.has_warnings()),
        i32::from(api.has_modified()),
        i32::from(api.has_generated()),
        i32::from(api.has_deleted())
    )
    .unwrap();
    let all = dump_result(api.shape(), rows);
    rows.push_str("section");
    for edge in api.section_edges().unwrap().iter() {
        write!(rows, " {}", index_of(&all, edge)).unwrap();
    }
    rows.push('\n');
}

fn make_box(origin: Point3, size: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn shapes() -> (Shape, Shape, Shape) {
    let axes = Axis2::new(
        Point3::new(5., 5., -5.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let cylinder = BRepPrimAPIMakeCylinder::with_axes(axes, 2., 20.)
        .shape()
        .unwrap()
        .clone();
    (
        make_box(Point3::default(), 10.),
        make_box(Point3::new(5., 5., 5.), 10.),
        cylinder,
    )
}

fn operations(rows: &mut String) {
    let (target, other, cylinder) = shapes();
    report(
        "cut",
        &BRepAlgoAPICut::from_shapes(&target, &other).unwrap(),
        rows,
    );
    report(
        "fuse",
        &BRepAlgoAPIFuse::from_shapes(&target, &cylinder).unwrap(),
        rows,
    );
    report(
        "common",
        &BRepAlgoAPICommon::from_shapes(&target, &cylinder).unwrap(),
        rows,
    );
    let section = BRepAlgoAPISection::from_shapes(&target, &cylinder, None, true).unwrap();
    report("section", &section, rows);
    let plane = Plane3::new(
        Point3::new(0., 0., 4.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let section = BRepAlgoAPISection::from_shape_plane(&target, plane, true).unwrap();
    report("section-plane", &section, rows);
    let arguments: List<Shape> = [target, cylinder].into_iter().collect();
    let mut general = BRepAlgoAPIBuilderAlgo::new();
    general.set_arguments(&arguments);
    general.build().unwrap();
    report("general-fuse", &general, rows);
}

fn filler_operations(rows: &mut String) {
    let (target, other, _) = shapes();
    let mut filler = BOPAlgoPaveFiller::new();
    let arguments: List<Shape> = [target.clone(), other.clone()].into_iter().collect();
    filler.set_arguments(&arguments);
    filler.perform();
    let mut forward =
        BRepAlgoAPICut::from_shapes_with_filler(&target, &other, filler, true).unwrap();
    report("filler-cut", &forward, rows);
    let filler = forward.take_ds_filler().unwrap();
    let mut backward =
        BRepAlgoAPICut::from_shapes_with_filler(&target, &other, filler, false).unwrap();
    report("filler-cut21", &backward, rows);
    let filler = backward.take_ds_filler().unwrap();
    let common = BRepAlgoAPICommon::from_shapes_with_filler(&target, &other, filler).unwrap();
    report("filler-common", &common, rows);
    let mut empty = BRepAlgoAPICut::new();
    empty.build().unwrap();
    writeln!(
        rows,
        "case empty done {} errors {}",
        i32::from(empty.is_done()),
        i32::from(empty.has_errors())
    )
    .unwrap();
}

#[test]
fn brep_algo_api_booleans_match_native_oracle() {
    let mut rows = String::new();
    operations(&mut rows);
    filler_operations(&mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_brep_algo_api.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, expected);
}
