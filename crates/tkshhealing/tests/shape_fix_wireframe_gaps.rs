//! Native wireframe gap orchestration and 3D spline endpoint repair.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{self, Compound, Face, Shape},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomBSplineCurve, GeomPlane, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisEdge,
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
fn fixture(mode: i32) -> Shape {
    let builder = BRepBuilder::default();
    let mut wire = BRepBuilderAPIMakePolygon::from_three_points(
        Point3::new(0., 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(0., 1., 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap();
    if mode == 10 || mode == 11 {
        let source = topods::edge(
            Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape)
                .unwrap()
                .current(),
        )
        .unwrap();
        let mut edge = topods::edge(&source.empty_copied().unwrap()).unwrap();
        let analysis = ShapeAnalysisEdge::new();
        let vertex = if mode == 10 {
            analysis.first_vertex(&source)
        } else {
            analysis.last_vertex(&source)
        };
        builder
            .add(
                &mut edge,
                &vertex.oriented(rustycad_tkg3d::top_abs::Orientation::Forward),
            )
            .unwrap();
        builder
            .add(
                &mut edge,
                &vertex.oriented(rustycad_tkg3d::top_abs::Orientation::Reversed),
            )
            .unwrap();
        builder.make_wire(&mut wire);
        builder.add(&mut wire, &edge).unwrap();
    }
    if mode == 1 || mode == 6 || mode == 7 {
        let edge = topods::edge(
            Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape)
                .unwrap()
                .current(),
        )
        .unwrap();
        builder.range(&edge, 0., 0.9, true).unwrap();
    }
    if mode == 6 {
        wire.reverse();
    }
    if mode == 7 {
        let mut exp = Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
        exp.next().unwrap();
        let edge = topods::edge(exp.current()).unwrap();
        builder.range(&edge, 0.1, 2f64.sqrt(), true).unwrap();
    }
    if mode == 8 || mode == 9 {
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        builder
            .add(&mut compound, &fixture(if mode == 8 { 1 } else { 0 }))
            .unwrap();
        builder
            .add(&mut compound, &fixture(if mode == 8 { 0 } else { 1 }))
            .unwrap();
        return compound.into_shape();
    }
    if mode == 2 {
        let mut face = Face::default();
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        builder
            .make_face_on_surface(&mut face, &surface, 1e-7)
            .unwrap();
        builder.add(&mut face, &wire).unwrap();
        return face.into_shape();
    }
    if mode == 3 {
        builder.make_wire(&mut wire);
    }
    if mode == 4 {
        return Shape::default();
    }
    if mode == 5 {
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        builder.add(&mut compound, &wire).unwrap();
        builder.add(&mut compound, &wire).unwrap();
        return compound.into_shape();
    }
    wire.into_shape()
}
fn observe(tool: &ShapeFixWireframe, input: &Shape, changed: bool) -> Vec<f64> {
    let result = tool.shape();
    let edges: Vec<Shape> = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect();
    let mut values = vec![f64::from(changed)];
    values.extend(
        [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Done2,
            ShapeExtendStatus::Fail1,
            ShapeExtendStatus::Fail2,
        ]
        .map(|status| f64::from(tool.status_wire_gaps(status))),
    );
    values.extend([
        f64::from(result.is_same(input)),
        f64::from(tool.context().is_null()),
        edges.len() as f64,
    ]);
    for edge in edges {
        let (curve, first, last) = BRepTool::curve(&topods::edge(&edge).unwrap()).unwrap();
        let p = curve.value(first).unwrap();
        let q = curve.value(last).unwrap();
        values.extend([first, last, p.x(), p.y(), q.x(), q.y()]);
        let mid = curve.value(0.5 * (first + last)).unwrap();
        let spline = curve.cast::<GeomBSplineCurve>();
        values.extend([
            mid.x(),
            mid.y(),
            spline.get().map_or(0., |spline| f64::from(spline.degree())),
        ]);
    }
    values
}
fn run(name: &str, mode: i32) {
    let shape = fixture(mode);
    let tool = ShapeFixWireframe::from_shape(&shape);
    let changed = tool.fix_wire_gaps().unwrap();
    let actual = observe(&tool, &shape, changed);
    let expected: Vec<f64> =
        include_str!("../../../tools/oracle/tkshhealing_wireframe_gaps.expected")
            .lines()
            .find(|line| line.starts_with(&format!("{name} ")))
            .unwrap()
            .split_whitespace()
            .skip(1)
            .map(|v| v.parse().unwrap())
            .collect();
    assert_eq!(actual.len(), expected.len(), "{name}");
    for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
        assert!((a - b).abs() < 1e-12, "{name} field {index}: {a} != {b}");
    }
}
#[test]
fn native_gap_free_wireframe_status_and_context() {
    for (name, mode) in [
        ("closed", 0),
        ("face", 2),
        ("empty_wire", 3),
        ("null", 4),
        ("repeated_child", 5),
    ] {
        run(name, mode);
    }
}
#[test]
fn native_shortened_curve_converts_and_repairs_its_endpoint() {
    run("trimmed_gap", 1);
}

#[test]
fn native_reversed_and_both_curve_endpoint_conversion() {
    run("reversed_gap", 6);
    run("both_ends", 7);
}
#[test]
fn native_compound_return_uses_final_recursive_child_status() {
    run("gap_then_closed", 8);
    run("closed_then_gap", 9);
}

#[test]
fn native_single_edge_conversion_moves_both_poles() {
    run("closed_edge_first_vertex", 10);
    run("closed_edge_last_vertex", 11);
}

#[test]
fn repeated_wire_gap_pass_resets_previous_done_status() {
    let wire = topods::wire(&fixture(1)).unwrap();
    let tool = rustycad_tkshhealing::shape_fix::ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.fix_reorder().unwrap();
    let first = tool.fix_gaps_3d().unwrap();
    let done = tool.status_gaps_3d(ShapeExtendStatus::Done);
    let second = tool.fix_gaps_3d().unwrap();
    let actual = [
        first,
        done,
        second,
        tool.status_gaps_3d(ShapeExtendStatus::Ok),
    ]
    .map(i32::from);
    let expected: Vec<i32> =
        include_str!("../../../tools/oracle/tkshhealing_wireframe_gaps.expected")
            .lines()
            .find(|line| line.starts_with("wire_repeat "))
            .unwrap()
            .split_whitespace()
            .skip(1)
            .map(|v| v.parse().unwrap())
            .collect();
    assert_eq!(actual.as_slice(), expected.as_slice());
}
