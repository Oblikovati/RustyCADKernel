//! Native replay of the whole BOPAlgo_Builder (General Fuse) on solid pairs; see
//! tools/oracle/tkbo_builder_general_fuse.cxx.
use rustycad_tkbo::bop_algo::BOPAlgoBuilder;
use rustycad_tkbrep::topods::Shape;
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use shape_dump::{dump_result, index_of, o};
use std::fmt::Write;

fn run(label: &str, first: Shape, second: Shape, rows: &mut String) {
    let mut builder = BOPAlgoBuilder::new();
    builder.add_argument(&first);
    builder.add_argument(&second);
    builder.perform();
    writeln!(
        rows,
        "case {label} errors {} warnings {}",
        i32::from(builder.options().has_errors()),
        i32::from(builder.options().has_warnings())
    )
    .unwrap();
    let all = dump_result(builder.shape(), rows);
    let structure = builder.pave_filler().unwrap().data_structure().unwrap();
    for index in 0..structure.nb_source_shapes() {
        let shape = structure.shape(index).unwrap();
        write!(rows, "i {index} {}", shape.shape_type().unwrap() as i32).unwrap();
        if let Some(images) = builder.images().seek(shape) {
            rows.push_str(" im");
            for image in images.iter() {
                write!(rows, " {}{}", index_of(&all, image), o(image.orientation())).unwrap();
            }
        }
        writeln!(
            rows,
            " h {} {} {}",
            builder.modified(shape).unwrap().len(),
            builder.generated(shape).unwrap().len(),
            i32::from(builder.is_deleted(shape).unwrap())
        )
        .unwrap();
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
fn general_fuse_matches_native_oracle() {
    let mut rows = String::new();
    let unit = solid_box(Point3::default(), 10.);
    run(
        "box-box",
        unit.clone(),
        solid_box(Point3::new(5., 5., 5.), 10.),
        &mut rows,
    );
    let face_box = BRepPrimAPIMakeBox::at_point(Point3::new(10., 2., 2.), 5., 5., 5.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    run("box-box-face", unit.clone(), face_box, &mut rows);
    let axes = Axis2::new(
        Point3::new(5., 5., -5.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let cylinder = BRepPrimAPIMakeCylinder::with_axes(axes, 2., 20.)
        .shape()
        .unwrap()
        .clone();
    run("box-cylinder", unit.clone(), cylinder, &mut rows);
    let sphere = BRepPrimAPIMakeSphere::at_center(Point3::new(10., 10., 10.), 4.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    run("box-sphere", unit.clone(), sphere, &mut rows);
    run(
        "box-inner",
        unit,
        solid_box(Point3::new(3., 3., 3.), 2.),
        &mut rows,
    );
    let expected = include_str!("../../../tools/oracle/tkbo_builder_general_fuse.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, expected);
}
