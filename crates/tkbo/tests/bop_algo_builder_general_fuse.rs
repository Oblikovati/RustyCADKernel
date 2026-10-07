//! Native replay of the whole BOPAlgo_Builder (General Fuse) on solid pairs; see
//! tools/oracle/tkbo_builder_general_fuse.cxx.
use rustycad_tkbo::bop_algo::BOPAlgoBuilder;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use std::fmt::Write;

fn h(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn o(orientation: Orientation) -> char {
    ['F', 'R', 'I', 'E'][orientation as usize]
}

fn index_of(map: &IndexedMapOfShape, shape: &Shape) -> usize {
    map.find_index(shape)
}

fn typed(result: &Shape, kind: ShapeEnum) -> IndexedMapOfShape {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(result, kind, &mut map).unwrap();
    map
}

fn write_vertices(vertices: &IndexedMapOfShape, rows: &mut String) {
    for vertex in vertices.iter() {
        let vertex = Vertex::unchecked(vertex.clone());
        let point = BRepTool::pnt(&vertex).unwrap();
        let tolerance = BRepTool::vertex_tolerance(&vertex).unwrap();
        writeln!(
            rows,
            "v {} {} {} {}",
            h(point.x()),
            h(point.y()),
            h(point.z()),
            h(tolerance)
        )
        .unwrap();
    }
}

fn write_edges(vertices: &IndexedMapOfShape, edges: &IndexedMapOfShape, rows: &mut String) {
    for edge in edges.iter() {
        let typed_edge = Edge::unchecked(edge.clone());
        let (first, last) = BRepTool::range(&typed_edge).unwrap();
        write!(
            rows,
            "e {} {} {} {}:",
            h(BRepTool::edge_tolerance(&typed_edge).unwrap()),
            h(first),
            h(last),
            i32::from(BRepTool::degenerated(&typed_edge).unwrap())
        )
        .unwrap();
        for vertex in ShapeIterator::new(edge, true, true) {
            write!(
                rows,
                " {}{}",
                index_of(vertices, &vertex),
                o(vertex.orientation())
            )
            .unwrap();
        }
        rows.push('\n');
    }
}

fn write_children(
    prefix: &str,
    parents: &IndexedMapOfShape,
    children: &IndexedMapOfShape,
    rows: &mut String,
) {
    for parent in parents.iter() {
        rows.push_str(prefix);
        if prefix == "f" {
            let face = Face::unchecked(parent.clone());
            write!(rows, " {}", h(BRepTool::face_tolerance(&face).unwrap())).unwrap();
        }
        for container in ShapeIterator::new(parent, true, true) {
            write!(rows, " |{}", o(container.orientation())).unwrap();
            for child in ShapeIterator::new(&container, true, true) {
                write!(
                    rows,
                    " {}{}",
                    index_of(children, &child),
                    o(child.orientation())
                )
                .unwrap();
            }
        }
        rows.push('\n');
    }
}

fn dump_result(result: &Shape, rows: &mut String) -> IndexedMapOfShape {
    let vertices = typed(result, ShapeEnum::Vertex);
    let edges = typed(result, ShapeEnum::Edge);
    let faces = typed(result, ShapeEnum::Face);
    let solids = typed(result, ShapeEnum::Solid);
    let mut all = IndexedMapOfShape::default();
    topexp::map_shapes(result, &mut all, true, true);
    writeln!(
        rows,
        "result {} {} {} {} {}",
        vertices.extent(),
        edges.extent(),
        faces.extent(),
        solids.extent(),
        all.extent()
    )
    .unwrap();
    write_vertices(&vertices, rows);
    write_edges(&vertices, &edges, rows);
    write_children("f", &faces, &edges, rows);
    write_children("s", &solids, &faces, rows);
    all
}

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
