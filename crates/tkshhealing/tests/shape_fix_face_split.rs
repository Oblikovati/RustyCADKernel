//! Native wire-map splitting, hole orientation and resulting working-face state.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{Face, Shape, ShapeIterator, Wire},
};
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::{
    shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixFace, ShapeFixFaceWireMap},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
fn rectangle(x: f64, y: f64, size: f64, close: bool) -> Wire {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(x, y), (x + size, y), (x + size, y + size), (x, y + size)] {
        maker.add_point(Point3::new(x, y, 0.)).unwrap();
    }
    if close {
        maker.close().unwrap();
    }
    maker.wire().unwrap()
}
fn fixture(mode: i32) -> (Face, ShapeFixFaceWireMap) {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let mut first = rectangle(0., 0., 4., mode != 2);
    let second = rectangle(10., 0., 4., true);
    if mode == 3 {
        builder.make_wire(&mut first);
    }
    builder.add(&mut face, &first).unwrap();
    if mode != 8 {
        builder.add(&mut face, &second).unwrap();
    }
    let holes = append_hole(&mut face, mode);
    let mut groups = ShapeFixFaceWireMap::default();
    groups.bind(first.into_shape(), holes);
    if mode != 1 && mode != 8 {
        groups.bind(second.into_shape(), List::default());
    }
    if mode == 4 {
        face.reverse();
    }
    if mode == 7 {
        let mut ignored = rectangle(20., 0., 1., true);
        ignored.set_orientation(Orientation::Internal);
        builder.add(&mut face, &ignored).unwrap();
    }
    (face, groups)
}
fn append_hole(face: &mut Face, mode: i32) -> List<Shape> {
    let mut holes = List::default();
    if mode != 5 && mode != 6 {
        return holes;
    }
    let mut hole = rectangle(1., 1., 1., true);
    if mode == 6 {
        hole.reverse();
    }
    BRepBuilder::default().add(face, &hole).unwrap();
    holes.append(hole.into_shape());
    holes
}
fn count(shape: &Shape, kind: ShapeEnum) -> i32 {
    if shape.is_null() {
        return 0;
    }
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count() as i32
}
fn output(
    tool: &ShapeFixFace,
    face: &Face,
    context: &ShapeBuildReShape,
    changed: bool,
) -> Vec<i32> {
    let result = tool.result();
    let working = tool.face().unwrap();
    let mut values = vec![
        i32::from(changed),
        i32::from(result.is_null()),
        count(&result, ShapeEnum::Face),
        count(&result, ShapeEnum::Wire),
        i32::from(working.is_same(face)),
        working.orientation() as i32,
        i32::from(tool.status(ShapeExtendStatus::Done)),
        i32::from(context.is_recorded(face)),
    ];
    if result.is_null() {
        return values;
    }
    for face in Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape).unwrap() {
        values.push(face.orientation() as i32);
        values.extend(ShapeIterator::new(&face, false, true).map(|wire| wire.orientation() as i32));
    }
    values
}
fn run(name: &str, mode: i32) {
    let (face, groups) = fixture(mode);
    let tool = ShapeFixFace::new(face.clone().into_shape());
    let context = Handle::new(ShapeBuildReShape::new());
    if mode == 9 {
        tool.set_context(&context);
    }
    let changed = tool.fix_split_face(&groups).unwrap();
    let expected: Vec<i32> = include_str!("../../../tools/oracle/tkshhealing_face_split.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse().unwrap())
        .collect();
    assert_eq!(output(&tool, &face, &context, changed), expected, "{name}");
}
#[test]
fn native_split_validation_and_result_mutations() {
    for (name, mode) in [
        ("two_outer", 0),
        ("unmapped", 1),
        ("open", 2),
        ("empty", 3),
        ("single", 8),
        ("context", 9),
    ] {
        run(name, mode);
    }
}
#[test]
fn native_hole_and_face_orientation() {
    for (name, mode) in [
        ("reversed_face", 4),
        ("forward_hole", 5),
        ("reversed_hole", 6),
        ("internal_ignored", 7),
    ] {
        run(name, mode);
    }
}
