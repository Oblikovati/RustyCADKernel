//! Differential loop graph reconstruction against the pinned OCCT fixture.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{self, Face, Shape, Vertex, Wire},
};
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomCylindricalSurface, GeomPlane, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisEdge, shape_extend::ShapeExtendStatus, shape_fix::ShapeFixFace,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn vertex(x: f64, y: f64) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, y, 0.), 1e-7)
        .unwrap();
    vertex
}
fn graph(mode: usize) -> Wire {
    let vertices = [
        vertex(0., 0.),
        vertex(1., 0.),
        vertex(0., 1.),
        vertex(-1., 0.),
        vertex(0., -1.),
    ];
    let pairs: &[(usize, usize)] = match mode {
        0 => &[(0, 1), (1, 2), (2, 0)],
        1 => &[(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0)],
        2 => &[(0, 1), (0, 2), (0, 3)],
        3 => &[(0, 1), (0, 1), (0, 1)],
        4 => &[(0, 1), (0, 2), (0, 3), (0, 4)],
        _ => unreachable!(),
    };
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for &(first, last) in pairs {
        let edge = BRepBuilderAPIMakeEdge::from_vertices(&vertices[first], &vertices[last])
            .unwrap()
            .edge()
            .unwrap();
        builder.add(&mut wire, &edge).unwrap();
    }
    wire
}
fn fixer(wire: &Wire, curved: bool) -> ShapeFixFace {
    let mut surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    if curved {
        surface = Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast();
    }
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    builder.add(&mut face, wire).unwrap();
    let tool = ShapeFixFace::new(face.clone().into_shape());
    tool.fix_wire_tool().load(wire).unwrap();
    tool.fix_wire_tool().set_face(&face).unwrap();
    tool
}
fn run(name: &str, mode: usize, preload: bool) {
    let tool = fixer(&graph(mode), name == "curved_missing_pcurves");
    let mut output: Sequence<Shape> = Sequence::default();
    if preload {
        output.append(graph(0).into_shape());
    }
    let changed = tool.fix_loop_wire(&mut output).unwrap();
    let mut actual = vec![
        i32::from(changed),
        i32::from(tool.status(ShapeExtendStatus::Done7)),
        output.length(),
    ];
    for wire in &output {
        actual.push(
            Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)
                .unwrap()
                .count() as i32,
        );
        actual.push(i32::from(wire.closed().unwrap()));
        actual.extend(endpoints(wire));
    }
    let expected: Vec<i32> =
        include_str!("../../../tools/oracle/tkshhealing_face_loop_wire.expected")
            .lines()
            .find(|line| line.starts_with(&format!("{name} ")))
            .unwrap()
            .split_whitespace()
            .skip(1)
            .map(|v| v.parse().unwrap())
            .collect();
    assert_eq!(actual, expected, "{name}");
}
#[test]
fn native_branching_loop_reconstruction() {
    for (name, mode) in [
        ("curved_missing_pcurves", 1),
        ("four_spokes", 4),
        ("triangle", 0),
        ("two_loops", 1),
        ("spokes", 2),
        ("parallel", 3),
    ] {
        run(name, mode, false);
    }
}
#[test]
fn native_output_sequence_append_and_early_return() {
    run("preloaded", 1, true);
    run("unchanged_output", 0, true);
}

fn endpoints(wire: &Shape) -> Vec<i32> {
    let mut coordinates = Vec::new();
    let analysis = ShapeAnalysisEdge::new();
    for shape in Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let edge = topods::edge(&shape).unwrap();
        for vertex in [analysis.first_vertex(&edge), analysis.last_vertex(&edge)] {
            let point = BRepTool::pnt(&vertex).unwrap();
            coordinates.extend([point.x() as i32, point.y() as i32]);
        }
    }
    coordinates
}
