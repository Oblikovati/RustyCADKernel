//! Native replay of BOPAlgo_BuilderSolid (with BOPAlgo_ShellSplitter and
//! BOPAlgo_Tools::ClassifyFaces) on face sets of primitive solids; see
//! tools/oracle/tkbo_builder_solid.cxx.
use rustycad_tkbo::bop_algo::BuilderSolid;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use std::fmt::Write;

fn h(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn o(orientation: Orientation) -> char {
    ['F', 'R', 'I', 'E'][orientation as usize]
}

fn index_of(faces: &[Shape], face: &Shape) -> i32 {
    faces
        .iter()
        .position(|candidate| candidate.is_same(face))
        .map_or(-1, |position| position as i32)
}

fn write_box(builder: &BuilderSolid, area: &Shape, rows: &mut String) {
    let Some(bounds) = builder.boxes_map().seek(area) else {
        rows.push_str(" nobox");
        return;
    };
    if bounds.is_void() || bounds.is_whole() {
        write!(
            rows,
            " box {}{}",
            i32::from(bounds.is_void()),
            i32::from(bounds.is_whole())
        )
        .unwrap();
        return;
    }
    let limits = bounds.get().unwrap();
    write!(rows, " box").unwrap();
    for value in limits {
        write!(rows, " {}", h(value)).unwrap();
    }
}

fn run(label: &str, faces: &[Shape], rows: &mut String) {
    let mut builder = BuilderSolid::new();
    let shapes: List<Shape> = faces.iter().cloned().collect();
    builder.set_shapes(&shapes);
    builder.perform().unwrap();
    writeln!(
        rows,
        "case {label} errors {} warnings {} loops {} areas {}",
        i32::from(builder.options().has_errors()),
        i32::from(builder.options().has_warnings()),
        builder.loops().len(),
        builder.areas().len()
    )
    .unwrap();
    for area in builder.areas().iter() {
        write!(rows, " a {}", o(area.orientation())).unwrap();
        write_box(&builder, area, rows);
        rows.push('\n');
        for shell in ShapeIterator::new(area, true, true) {
            write!(
                rows,
                "  s {} {}:",
                o(shell.orientation()),
                i32::from(shell.closed().unwrap())
            )
            .unwrap();
            for face in ShapeIterator::new(&shell, true, true) {
                write!(rows, " {}{}", index_of(faces, &face), o(face.orientation())).unwrap();
            }
            rows.push('\n');
        }
    }
}

fn add_faces(faces: &mut Vec<Shape>, solid: &Shape, reversed: bool) {
    for face in Explorer::new(solid, ShapeEnum::Face, ShapeEnum::Shape).unwrap() {
        faces.push(if reversed { face.reversed() } else { face });
    }
}

fn make_box(origin: Point3, size: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn square(u_min: f64) -> Shape {
    let plane = Plane3::new(
        Point3::new(0., 0., 5.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (u_min, u_min + 2., 2., 4.))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn box_cases(rows: &mut String) {
    let big = make_box(Point3::default(), 10.);
    let small = make_box(Point3::new(3., 3., 3.), 2.);
    let far = make_box(Point3::new(20., 0., 0.), 2.);
    let mut faces = Vec::new();
    add_faces(&mut faces, &big, false);
    run("box", &faces, rows);
    add_faces(&mut faces, &far, false);
    run("two", &faces, rows);
    let mut faces = Vec::new();
    add_faces(&mut faces, &big, false);
    add_faces(&mut faces, &small, true);
    run("cavity", &faces, rows);
    let mut faces = Vec::new();
    add_faces(&mut faces, &small, true);
    run("hole-only", &faces, rows);
    let mut faces = Vec::new();
    add_faces(&mut faces, &big, false);
    for face in [square(2.), square(12.)] {
        faces.push(face.clone());
        faces.push(face.reversed());
    }
    run("internal", &faces, rows);
    let faces: Vec<Shape> = Explorer::new(&big, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .skip(1)
        .collect();
    run("open", &faces, rows);
}

#[test]
fn builder_solid_matches_native_oracle() {
    let mut rows = String::new();
    box_cases(&mut rows);
    let cylinder = BRepPrimAPIMakeCylinder::new(2., 5.)
        .shape()
        .unwrap()
        .clone();
    let mut faces = Vec::new();
    add_faces(&mut faces, &cylinder, false);
    run("cylinder", &faces, &mut rows);
    let big = make_box(Point3::default(), 10.);
    let mut faces = Vec::new();
    add_faces(&mut faces, &big, false);
    add_faces(&mut faces, &big, false);
    run("twice", &faces, &mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_builder_solid.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, expected);
}
