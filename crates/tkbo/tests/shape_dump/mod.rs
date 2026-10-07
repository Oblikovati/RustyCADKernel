//! Shared dump of a Boolean result for the TKBO replays, the Rust side of
//! tools/oracle/tkbo_shape_dump.hxx.
#![allow(dead_code)]
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::fmt::Write;

pub fn h(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

pub fn o(orientation: Orientation) -> char {
    ['F', 'R', 'I', 'E'][orientation as usize]
}

pub fn index_of(map: &IndexedMapOfShape, shape: &Shape) -> usize {
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

pub fn dump_result(result: &Shape, rows: &mut String) -> IndexedMapOfShape {
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
