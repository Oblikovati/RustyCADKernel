//! Native replay of BOPAlgo_Splitter and BRepAlgoAPI_Splitter; see
//! tools/oracle/tkbo_splitter.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::BOPAlgoSplitter;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPISplitter;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use shape_dump::dump_result;
use std::fmt::Write;

fn run(label: &str, arguments: &List<Shape>, tools: &List<Shape>, rows: &mut String) {
    let mut splitter = BOPAlgoSplitter::new();
    splitter.set_arguments(arguments);
    splitter.set_tools(tools);
    splitter.perform();
    let shape = splitter.shape().clone();
    writeln!(
        rows,
        "case {label} errors {} null {}",
        i32::from(splitter.options().has_errors()),
        i32::from(shape.is_null())
    )
    .unwrap();
    if shape.is_null() {
        return;
    }
    writeln!(rows, "type {}", shape.shape_type().unwrap() as i32).unwrap();
    dump_result(&shape, rows);
    let mut api = BRepAlgoAPISplitter::new();
    api.set_arguments(arguments);
    api.set_tools(tools);
    api.build().unwrap();
    writeln!(rows, "api done {}", i32::from(api.is_done())).unwrap();
    dump_result(api.shape(), rows);
}

fn list(shapes: &[&Shape]) -> List<Shape> {
    shapes.iter().map(|shape| (*shape).clone()).collect()
}

fn make_box(origin: Point3, size: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

#[test]
fn splitters_match_native_oracle() {
    let mut rows = String::new();
    let target = make_box(Point3::default(), 10.);
    let plane = Plane3::new(
        Point3::new(0., 0., 4.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-5., 15., -5., 15.))
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    run("box-plane", &list(&[&target]), &list(&[&face]), &mut rows);
    let other = make_box(Point3::new(5., 5., 5.), 10.);
    run("box-box", &list(&[&target]), &list(&[&other]), &mut rows);
    let second = make_box(Point3::new(20., 0., 0.), 10.);
    let axes = Axis2::new(
        Point3::new(15., 5., -5.),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    let cylinder = BRepPrimAPIMakeCylinder::with_axes(axes, 2., 20.)
        .shape()
        .unwrap()
        .clone();
    run(
        "two-cylinder",
        &list(&[&target, &second]),
        &list(&[&cylinder]),
        &mut rows,
    );
    run("lonely", &list(&[&target]), &List::default(), &mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_splitter.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, expected);
}
