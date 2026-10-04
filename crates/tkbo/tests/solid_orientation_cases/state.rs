use super::*;
use rustycad_tkbo::{bop_tools, int_tools::Context};
use rustycad_tkbrep::{
    topexp::{self, Explorer},
    topods::{Compound, Edge, Face, Shape, Vertex},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine},
    top_abs::{ShapeEnum, State},
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
fn orientation(index: usize) -> Orientation {
    [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ][index]
}
fn parsed(line: &str) -> Vec<usize> {
    line.split_whitespace()
        .map(|token| token.parse().unwrap())
        .collect()
}
#[test]
fn native_point_vertex_and_dispatch_states_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_compute_state.expected").lines() {
        let row = parsed(line);
        let mut solid = solid_fixture(row[0]);
        solid.set_orientation(orientation(row[1]));
        let point = [
            Point3::new(0.5, 0.5, 1.),
            Point3::default(),
            Point3::new(10., 10., 10.),
            Point3::new(1. + 1e-8, 1., 1.),
        ][row[2]];
        let mut vertex = Vertex::default();
        BRepBuilder::default()
            .make_vertex_at(&mut vertex, point, 1e-7)
            .unwrap();
        let context = Context::default();
        let actual = [
            bop_tools::compute_state_point(point, &solid, 1e-7, &context).unwrap(),
            bop_tools::compute_state_vertex(&vertex, &solid, 1e-7, &context).unwrap(),
            bop_tools::compute_state_by_one_point(&vertex, &solid, 1e-7, &context).unwrap(),
        ];
        assert_eq!(actual.map(|state| state as usize), row[3..], "{line}");
    }
}
#[test]
fn native_edge_face_and_container_states_match() {
    for line in
        include_str!("../../../../tools/oracle/tkbo_compute_state_topology.expected").lines()
    {
        let row = parsed(line);
        let context = Context::default();
        let mut solid = solid_fixture(0);
        solid.set_orientation(orientation(row[1]));
        let (shape, direct) = topology_case(row[0], &solid, &context);
        let dispatched =
            bop_tools::compute_state_by_one_point(&shape, &solid, 1e-7, &context).unwrap();
        assert_eq!([direct as usize, dispatched as usize], row[2..], "{line}");
    }
}
fn topology_case(mode: usize, solid: &Solid, context: &Context) -> (Shape, State) {
    if mode < 6 {
        let edge = edge_case(mode);
        let state = bop_tools::compute_state_edge(&edge, solid, 1e-7, context).unwrap();
        return (edge.into_shape(), state);
    }
    if mode < 8 {
        return face_case(mode, solid, context);
    }
    (compound_case(mode), State::Unknown)
}
fn edge_case(mode: usize) -> Edge {
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    if mode < 4 {
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
            Point3::new(0.5, 0.5, 1.),
            Direction3::X,
        )))
        .cast();
        builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
        builder
            .range(
                &edge,
                [-0.1, -2e100, -0.1, -2e100][mode],
                [0.1, 0.1, 2e100, 2e100][mode],
                false,
            )
            .unwrap();
    } else if mode == 4 {
        let mut vertex = Vertex::default();
        builder
            .make_vertex_at(&mut vertex, Point3::new(0.5, 0.5, 1.), 1e-7)
            .unwrap();
        builder.add(&mut edge, &vertex).unwrap();
    }
    edge
}
fn face_case(mode: usize, solid: &Solid, context: &Context) -> (Shape, State) {
    let mut explorer = Explorer::new(solid, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let face = Face::unchecked(Iterator::next(&mut explorer).unwrap());
    let mut bounds = IndexedMapOfShape::default();
    if mode == 7 {
        topexp::map_shapes_of_type(solid, ShapeEnum::Edge, &mut bounds).unwrap();
    }
    let state = bop_tools::compute_state_face(&face, solid, 1e-7, &bounds, context).unwrap();
    (face.into_shape(), state)
}
fn compound_case(mode: usize) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    if mode == 8 {
        let mut vertex = Vertex::default();
        builder
            .make_vertex_at(&mut vertex, Point3::new(0.5, 0.5, 1.), 1e-7)
            .unwrap();
        builder.add(&mut compound, &vertex).unwrap();
    }
    compound.into_shape()
}
