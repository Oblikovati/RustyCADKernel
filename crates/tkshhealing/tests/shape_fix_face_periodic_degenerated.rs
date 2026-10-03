//! Native cone apex repair without subsequent seam creation.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomConicalSurface, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2};
use rustycad_tkshhealing::{
    shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixFace},
};
use std::f64::consts::PI;
fn fixture(angle: f64, span: f64, reverse: bool, pcurve: bool, twice: bool) -> Face {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomConicalSurface::new(Axis3::default(), angle, 1.).unwrap()).cast();
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let edge = ring_edge(&face, &surface, span, pcurve);
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    if reverse {
        wire.reverse();
    }
    builder.add(&mut face, &wire).unwrap();
    if twice {
        builder.add(&mut face, &wire).unwrap();
    }
    face
}
fn ring_edge(face: &Face, surface: &Handle<dyn GeomSurface>, span: f64, pcurve: bool) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    if pcurve {
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
            Point2::new(0., 0.),
            Direction2::new(1., 0.).unwrap(),
        )))
        .cast();
        builder
            .update_edge_pcurve_on_face(&edge, &line, face, 1e-7)
            .unwrap();
        builder.range_on_face(&edge, face, 0., span).unwrap();
    }
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, surface.value(0., 0.).unwrap(), 1e-7)
        .unwrap();
    builder.add(&mut edge, &vertex).unwrap();
    builder.add(&mut edge, &vertex.reversed()).unwrap();
    edge
}
fn count(shape: &Shape, kind: ShapeEnum) -> f64 {
    if shape.is_null() {
        return 0.;
    }
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count() as f64
}
fn observations(
    tool: &ShapeFixFace,
    face: &Face,
    context: &Handle<ShapeBuildReShape>,
    changed: bool,
) -> Vec<f64> {
    let result = tool.result();
    let mut values = vec![
        f64::from(changed),
        f64::from(tool.face().unwrap().is_same(face)),
        count(&result, ShapeEnum::Wire),
        count(&result, ShapeEnum::Edge),
        f64::from(tool.status(ShapeExtendStatus::Done)),
        f64::from(context.is_recorded(face)),
    ];
    if result.is_null() {
        return values;
    }
    values.push(result.orientation() as i32 as f64);
    for wire in ShapeIterator::new(&result, false, true) {
        values.push(wire.orientation() as i32 as f64);
    }
    for shape in Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let edge = topods::edge(&shape).unwrap();
        if BRepTool::degenerated(&edge).unwrap() {
            values.extend(apex_values(&edge, &topods::face(&result).unwrap()));
        }
    }
    values
}
fn apex_values(edge: &Edge, face: &Face) -> [f64; 7] {
    let curve = BRepTool::curve_on_face(edge, face).unwrap();
    let start = curve.pcurve.value(curve.first).unwrap();
    let end = curve.pcurve.value(curve.last).unwrap();
    [
        BRepTool::edge_tolerance(edge).unwrap(),
        curve.first,
        curve.last,
        start.x(),
        start.y(),
        end.x(),
        end.y(),
    ]
}
fn run(expected_name: &str) {
    let name = expected_name
        .strip_prefix("aggregate_")
        .unwrap_or(expected_name);
    let angle = if name == "above" { -0.3 } else { 0.3 };
    let span = if name == "partial" { PI } else { 2. * PI };
    let mut face = fixture(
        angle,
        span,
        name == "reverse",
        name != "missing",
        name == "multiple",
    );
    if name == "reversed_face" {
        face.reverse();
    }
    let tool = ShapeFixFace::new(face.clone().into_shape());
    tool.set_precision(0.002);
    let context = Handle::new(ShapeBuildReShape::new());
    if name == "context" {
        tool.set_context(&context);
    }
    let changed = if expected_name.starts_with("aggregate_") {
        tool.set_fix_wire_mode(0);
        tool.perform()
            .unwrap_or_else(|error| panic!("{expected_name}: {error:?}"))
    } else {
        tool.fix_periodic_degenerated().unwrap()
    };
    let actual = observations(&tool, &face, &context, changed);
    let expected =
        include_str!("../../../tools/oracle/tkshhealing_face_periodic_degenerated.expected")
            .lines()
            .find(|line| line.split_whitespace().next() == Some(expected_name))
            .unwrap()
            .split_whitespace()
            .skip(1)
            .map(|token| token.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
    assert_eq!(actual.len(), expected.len(), "{name}: {actual:?}");
    for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
        assert!((a - b).abs() < 1e-12, "{name} field {index}: {a} != {b}");
    }
}
#[test]
fn native_apex_orientation_parameters_and_context() {
    for name in [
        "below",
        "above",
        "reverse",
        "reversed_face",
        "partial",
        "missing",
        "multiple",
        "context",
    ] {
        run(name);
    }
}

#[test]
fn native_periodic_aggregate_without_wire_repair() {
    for name in [
        "below",
        "above",
        "reverse",
        "reversed_face",
        "partial",
        "missing",
        "multiple",
        "context",
    ] {
        run(&format!("aggregate_{name}"));
    }
}
