//! Native replay of BOPAlgo_BOP (common, fuse, cut, cut21) on solid pairs; see
//! tools/oracle/tkbo_bop.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::{BOPAlgoBOP, Operation};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use shape_dump::{dump_result, index_of, o};
use std::fmt::Write;

fn history(bop: &BOPAlgoBOP, argument: &Shape, all: &IndexedMapOfShape, rows: &mut String) {
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(argument, ShapeEnum::Face, &mut faces).unwrap();
    for (position, face) in faces.iter().enumerate() {
        write!(rows, "h {} m", position + 1).unwrap();
        for modified in bop.modified(face).unwrap() {
            write!(
                rows,
                " {}{}",
                index_of(all, &modified),
                o(modified.orientation())
            )
            .unwrap();
        }
        writeln!(
            rows,
            " g {} d {}",
            bop.generated(face).unwrap().len(),
            i32::from(bop.is_deleted(face).unwrap())
        )
        .unwrap();
    }
}

fn run(label: &str, first: &Shape, second: &Shape, rows: &mut String) {
    for operation in [
        Operation::Common,
        Operation::Fuse,
        Operation::Cut,
        Operation::Cut21,
    ] {
        let mut bop = BOPAlgoBOP::new();
        bop.add_argument(first);
        bop.add_tool(second);
        bop.set_operation(operation);
        bop.perform();
        writeln!(
            rows,
            "case {label} {} errors {} warnings {}",
            operation as i32,
            i32::from(bop.options().has_errors()),
            i32::from(bop.options().has_warnings())
        )
        .unwrap();
        let all = dump_result(bop.shape(), rows);
        history(&bop, first, &all, rows);
        history(&bop, second, &all, rows);
    }
}

fn solid_box(origin: Point3, size: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

#[test]
fn boolean_operations_match_native_oracle() {
    let mut rows = String::new();
    let unit = solid_box(Point3::default(), 10.);
    run(
        "box-box",
        &unit,
        &solid_box(Point3::new(5., 5., 5.), 10.),
        &mut rows,
    );
    let face_box = BRepPrimAPIMakeBox::at_point(Point3::new(10., 2., 2.), 5., 5., 5.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    run("box-box-face", &unit, &face_box, &mut rows);
    let axes = Axis2::new(
        Point3::new(5., 5., -5.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let cylinder = BRepPrimAPIMakeCylinder::with_axes(axes, 2., 20.)
        .shape()
        .unwrap()
        .clone();
    run("box-cylinder", &unit, &cylinder, &mut rows);
    let sphere = BRepPrimAPIMakeSphere::at_center(Point3::new(10., 10., 10.), 4.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    run("box-sphere", &unit, &sphere, &mut rows);
    run(
        "box-inner",
        &unit,
        &solid_box(Point3::new(3., 3., 3.), 2.),
        &mut rows,
    );
    let expected = include_str!("../../../tools/oracle/tkbo_bop.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, expected);
}
