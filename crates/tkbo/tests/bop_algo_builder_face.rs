//! Native replay of BOPAlgo_BuilderFace (with BOPAlgo_WireSplitter) on faces split by extra
//! edges given in both orientations; see tools/oracle/tkbo_builder_face.cxx.
use rustycad_tkbo::bop_algo::BuilderFace;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{self, Axis3, Direction2, Plane3, Point2, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeVertex,
};
use std::fmt::Write;

fn h(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn o(orientation: Orientation) -> char {
    ['F', 'R', 'I', 'E'][orientation as usize]
}

fn index_of(edges: &[Shape], edge: &Shape) -> i32 {
    edges
        .iter()
        .position(|candidate| candidate.is_same(edge))
        .map_or(-1, |position| position as i32)
}

fn run(label: &str, face: &Face, edges: &[Shape], rows: &mut String) {
    let mut builder = BuilderFace::new();
    let shapes: List<Shape> = edges.iter().cloned().collect();
    builder.set_face(face);
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
        let (u1, u2, v1, v2) = uv_bounds(&Face::unchecked(area.clone())).unwrap();
        writeln!(
            rows,
            " a {} {} {} {} {}",
            o(area.orientation()),
            h(u1),
            h(u2),
            h(v1),
            h(v2)
        )
        .unwrap();
        for wire in ShapeIterator::new(area, true, true) {
            write!(
                rows,
                "  w {} {}:",
                o(wire.orientation()),
                i32::from(wire.closed().unwrap())
            )
            .unwrap();
            for edge in ShapeIterator::new(&wire, true, true) {
                write!(rows, " {}{}", index_of(edges, &edge), o(edge.orientation())).unwrap();
            }
            rows.push('\n');
        }
    }
}

fn vertex_at(shape: &Shape, point: Point3) -> Vertex {
    for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap() {
        let vertex = Vertex::unchecked(vertex);
        if BRepTool::pnt(&vertex).unwrap().distance(&point) < 1e-9 {
            return vertex;
        }
    }
    new_vertex(point)
}

fn new_vertex(point: Point3) -> Vertex {
    BRepBuilderAPIMakeVertex::new(point)
        .unwrap()
        .vertex()
        .unwrap()
}

fn both(edges: &mut Vec<Shape>, edge: Edge) {
    edges.push(edge.oriented(Orientation::Forward));
    edges.push(edge.oriented(Orientation::Reversed));
}

fn face_edges(face: &Face) -> Vec<Shape> {
    Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect()
}

fn edge_between(first: &Vertex, second: &Vertex) -> Edge {
    BRepBuilderAPIMakeEdge::from_vertices(first, second)
        .unwrap()
        .edge()
        .unwrap()
}

fn inner_square(edges: &mut Vec<Shape>, vertices: &[Vertex; 4]) {
    for index in 0..4 {
        both(
            edges,
            edge_between(&vertices[index], &vertices[(index + 1) % 4]),
        );
    }
}

fn planar_cases(rows: &mut String) {
    let plane = Plane3::from(Axis3::from_axis2(gp::xoy()));
    let square = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (0., 10., 0., 10.))
        .unwrap()
        .face()
        .unwrap();
    let c00 = vertex_at(&square, Point3::new(0., 0., 0.));
    let c11 = vertex_at(&square, Point3::new(10., 10., 0.));
    let mut edges = face_edges(&square);
    both(&mut edges, edge_between(&c00, &c11));
    run("diagonal", &square, &edges, rows);
    let inner =
        [(2., 2.), (4., 2.), (4., 4.), (2., 4.)].map(|(x, y)| new_vertex(Point3::new(x, y, 0.)));
    let mut edges = face_edges(&square);
    inner_square(&mut edges, &inner);
    run("hole", &square, &edges, rows);
    let mut edges = face_edges(&square);
    inner_square(&mut edges, &inner);
    both(&mut edges, edge_between(&c00, &inner[0]));
    run("bridge", &square, &edges, rows);
    let mut edges = face_edges(&square);
    both(
        &mut edges,
        edge_between(&c00, &new_vertex(Point3::new(5., 5., 0.))),
    );
    run("dangling", &square, &edges, rows);
    let mut edges = face_edges(&square);
    both(
        &mut edges,
        BRepBuilderAPIMakeEdge::from_points(Point3::new(3., 3., 0.), Point3::new(6., 3., 0.))
            .unwrap()
            .edge()
            .unwrap(),
    );
    run("floating", &square, &edges, rows);
}

fn cylinder_case(rows: &mut String) {
    let cylinder = BRepPrimAPIMakeCylinder::new(2., 5.)
        .shape()
        .unwrap()
        .clone();
    let lateral = Face::unchecked(
        Iterator::next(&mut Explorer::new(&cylinder, ShapeEnum::Face, ShapeEnum::Shape).unwrap())
            .unwrap(),
    );
    let mut edges = face_edges(&lateral);
    let surface = BRepTool::surface(&lateral).unwrap();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(3., 1.),
        Direction2::new(0., 1.).unwrap(),
    ))
    .cast();
    both(
        &mut edges,
        BRepBuilderAPIMakeEdge::from_pcurve_params(&line, &surface, 0., 2.)
            .unwrap()
            .edge()
            .unwrap(),
    );
    run("cylinder", &lateral, &edges, rows);
}

#[test]
fn builder_face_matches_native_oracle() {
    let mut rows = String::new();
    planar_cases(&mut rows);
    cylinder_case(&mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_builder_face.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, expected);
}
