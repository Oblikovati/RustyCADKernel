//! BOPAlgo_ArgumentAnalyzer against tools/oracle/tkbo_argument_analyzer.cxx: the check results
//! of every test on valid and faulty arguments.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::{BOPAlgoArgumentAnalyzer, CheckResult, Operation};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICheck;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Axis1, Axis2d, Axis3, Direction2, Plane3, Point2, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeVertex,
};
use shape_dump::{h, o};
use std::fmt::Write;

fn compound(first: &Shape, second: &Shape) -> Shape {
    let builder = BRepBuilder::default();
    let mut result = Compound::default();
    builder.make_compound(&mut result);
    builder.add(&mut result, first).unwrap();
    builder.add(&mut result, second).unwrap();
    result.into_shape()
}

fn cube(origin: Point3, size: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn vertex(point: Point3) -> Shape {
    BRepBuilderAPIMakeVertex::new(point)
        .unwrap()
        .vertex()
        .unwrap()
        .into_shape()
}

fn segment(first: Point3, second: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(first, second)
        .unwrap()
        .edge()
        .unwrap()
}

fn faulty(tag: &str, argument: &Shape, shapes: &List<Shape>, out: &mut String) {
    let mut all = IndexedMapOfShape::default();
    if !argument.is_null() {
        topexp::map_shapes(argument, &mut all, true, true);
    }
    write!(out, "{tag} {}:", shapes.extent()).unwrap();
    for shape in shapes.iter() {
        let kind = shape.shape_type().unwrap() as i32;
        write!(
            out,
            " {kind}/{}{}",
            all.find_index(shape),
            o(shape.orientation())
        )
        .unwrap();
    }
    writeln!(out).unwrap();
}

fn dump_result(result: &CheckResult, first: &Shape, second: &Shape, out: &mut String) {
    let status = result.check_status() as i32;
    let nulls = (
        i32::from(result.shape1().is_null()),
        i32::from(result.shape2().is_null()),
    );
    let distances = [
        result.max_distance1(),
        result.max_parameter1(),
        result.max_distance2(),
        result.max_parameter2(),
    ];
    let distances = distances.map(h).join(" ");
    writeln!(
        out,
        "status {status} shapes {} {} dist {distances}",
        nulls.0, nulls.1
    )
    .unwrap();
    faulty("f1", first, result.faulty_shapes1(), out);
    faulty("f2", second, result.faulty_shapes2(), out);
}

fn run(
    label: &str,
    first: &Shape,
    second: &Shape,
    configure: fn(&mut BOPAlgoArgumentAnalyzer),
    out: &mut String,
) {
    let mut analyzer = BOPAlgoArgumentAnalyzer::new();
    if !first.is_null() {
        analyzer.set_shape1(first);
    }
    if !second.is_null() {
        analyzer.set_shape2(second);
    }
    analyzer.options_mut().set_run_parallel(false);
    configure(&mut analyzer);
    analyzer.perform();
    let results = analyzer.check_result();
    writeln!(
        out,
        "case {label} faulty {} results {}",
        i32::from(analyzer.has_faulty()),
        results.extent()
    )
    .unwrap();
    for result in results.iter() {
        dump_result(result, first, second, out);
    }
}

fn check(label: &str, checker: Result<BRepAlgoAPICheck, Failure>, out: &mut String) {
    let checker = checker.unwrap();
    let results = checker.result();
    write!(
        out,
        "check {label} valid {} results {}",
        i32::from(checker.is_valid()),
        results.extent()
    )
    .unwrap();
    for result in results.iter() {
        let nulls = (
            i32::from(result.shape1().is_null()),
            i32::from(result.shape2().is_null()),
        );
        let counts = (
            result.faulty_shapes1().extent(),
            result.faulty_shapes2().extent(),
        );
        let status = result.check_status() as i32;
        write!(
            out,
            " {status}/{}{}/{}{}",
            nulls.0, nulls.1, counts.0, counts.1
        )
        .unwrap();
    }
    writeln!(out).unwrap();
}

fn bop_test_modes(analyzer: &mut BOPAlgoArgumentAnalyzer) {
    *analyzer.argument_type_mode() = true;
    *analyzer.self_inter_mode() = true;
    *analyzer.small_edge_mode() = true;
    *analyzer.rebuild_face_mode() = true;
    *analyzer.continuity_mode() = true;
    *analyzer.curve_on_surface_mode() = true;
}

fn all_modes(analyzer: &mut BOPAlgoArgumentAnalyzer) {
    bop_test_modes(analyzer);
    *analyzer.tangent_mode() = true;
    *analyzer.merge_vertex_mode() = true;
    *analyzer.merge_edge_mode() = true;
}

fn fuse_all(analyzer: &mut BOPAlgoArgumentAnalyzer) {
    all_modes(analyzer);
    *analyzer.operation_type() = Operation::Fuse;
}

fn cut_types(analyzer: &mut BOPAlgoArgumentAnalyzer) {
    *analyzer.argument_type_mode() = true;
    *analyzer.operation_type() = Operation::Cut;
}

fn section_small(analyzer: &mut BOPAlgoArgumentAnalyzer) {
    *analyzer.small_edge_mode() = true;
    *analyzer.operation_type() = Operation::Section;
}

fn stop_first(analyzer: &mut BOPAlgoArgumentAnalyzer) {
    fuse_all(analyzer);
    *analyzer.stop_on_first_faulty() = true;
}

fn xoy_plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::from_axis2(gp::xoy()))).into_cast()
}

/// A planar face whose edge lies 0.01 above the plane while its p-curve is on it.
fn lifted_edge_face() -> Shape {
    let builder = BRepBuilder::default();
    let bounds = (0.0, 10.0, 0.0, 10.0);
    let face = BRepBuilderAPIMakeFace::from_surface_bounds(&xoy_plane(), bounds, 1.0e-7)
        .unwrap()
        .face()
        .unwrap();
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(0.0, 5.0, 0.01),
        gp::dx(),
    )))
    .into_cast();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &line, 1.0e-7)
        .unwrap();
    builder.range(&edge, 0.0, 10.0, false).unwrap();
    let axis = Axis2d::new(Point2::new(0.0, 5.0), Direction2::new(1.0, 0.0).unwrap());
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(axis)).into_cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, &face, 1.0e-7)
        .unwrap();
    builder
        .add(&mut edge, &vertex(Point3::new(0.0, 5.0, 0.01)))
        .unwrap();
    builder
        .add(&mut edge, &vertex(Point3::new(10.0, 5.0, 0.01)).reversed())
        .unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    let mut internal = face.clone();
    builder
        .add(&mut internal, &wire.oriented(Orientation::Internal))
        .unwrap();
    internal.into_shape()
}

/// A planar face bounded by an open wire of three sides of a square.
fn open_wire_face() -> Shape {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &xoy_plane(), 1.0e-7)
        .unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let corners = [(0.0, 0.0), (5.0, 0.0), (5.0, 5.0), (0.0, 5.0)];
    for pair in corners.windows(2) {
        let edge = segment(
            Point3::new(pair[0].0, pair[0].1, 0.0),
            Point3::new(pair[1].0, pair[1].1, 0.0),
        );
        builder.add(&mut wire, &edge).unwrap();
    }
    builder.add(&mut face, &wire).unwrap();
    face.into_shape()
}

fn array<T: Clone + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (index, value) in values.iter().enumerate() {
        array.set_value(index as i32 + 1, value.clone()).unwrap();
    }
    array
}

/// A degree 1 B-spline edge with a kink, hence C0.
fn c0_edge() -> Shape {
    let poles = array(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ]);
    let curve = GeomBSplineCurve::new(
        &poles,
        &array(&[0.0, 1.0, 2.0]),
        &array(&[2, 1, 2]),
        1,
        false,
    )
    .unwrap();
    let curve: Handle<dyn GeomCurve> = Handle::new(curve).into_cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

#[test]
fn argument_analyzer_matches_native_check_results() {
    let mut out = String::new();
    let none = Shape::default();
    let origin = Point3::new(0.0, 0.0, 0.0);
    let box_shape = cube(origin, 10.0);
    let sphere = BRepPrimAPIMakeSphere::at_center(origin, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let inner_vertex = vertex(Point3::new(3.0, 3.0, 3.0));
    run("sphere", &sphere, &none, bop_test_modes, &mut out);
    run(
        "box-vertex",
        &compound(&box_shape, &inner_vertex),
        &none,
        bop_test_modes,
        &mut out,
    );
    run("nothing", &none, &none, bop_test_modes, &mut out);
    run("single-fuse", &box_shape, &none, fuse_all, &mut out);
    let twins = compound(&box_shape, &cube(origin, 10.0));
    run("box-twin-boxes", &box_shape, &twins, fuse_all, &mut out);
    let diagonal = segment(origin, Point3::new(5.0, 5.0, 5.0)).into_shape();
    run("cut-solid-edge", &box_shape, &diagonal, cut_types, &mut out);
    let tiny = segment(origin, Point3::new(1.5e-7, 0.0, 0.0)).into_shape();
    run("tiny-edge", &tiny, &none, bop_test_modes, &mut out);
    let far = cube(Point3::new(20.0, 20.0, 20.0), 5.0);
    run(
        "tiny-edge-section-far",
        &tiny,
        &far,
        section_small,
        &mut out,
    );
    run(
        "tiny-edge-section-on",
        &tiny,
        &vertex(origin),
        section_small,
        &mut out,
    );
    run(
        "lifted-edge",
        &lifted_edge_face(),
        &box_shape,
        fuse_all,
        &mut out,
    );
    run("c0-edge", &c0_edge(), &none, bop_test_modes, &mut out);
    let unit =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 1.0)).unwrap();
    run(
        "open-wire-face",
        &open_wire_face(),
        &unit.face().unwrap().into_shape(),
        fuse_all,
        &mut out,
    );
    let stop = compound(&box_shape, &inner_vertex);
    run(
        "stop-first",
        &stop,
        &compound(&box_shape, &cube(origin, 10.0)),
        stop_first,
        &mut out,
    );
    check(
        "check-box",
        BRepAlgoAPICheck::from_shape(&box_shape, true, true),
        &mut out,
    );
    check(
        "check-null",
        BRepAlgoAPICheck::from_shape(&none, true, true),
        &mut out,
    );
    let box_vertex = compound(&box_shape, &inner_vertex);
    let checker =
        BRepAlgoAPICheck::from_shapes(&box_vertex, &box_vertex, Operation::Fuse, true, true);
    check("check-box-vertex", checker, &mut out);
    let checker = BRepAlgoAPICheck::from_shapes(
        &open_wire_face(),
        &box_shape,
        Operation::Common,
        false,
        false,
    );
    check("check-open-wire-face", checker, &mut out);
    let checker = BRepAlgoAPICheck::from_shapes(&box_shape, &diagonal, Operation::Cut, true, true);
    check("check-cut-solid-edge", checker, &mut out);
    let expected = include_str!("../../../tools/oracle/tkbo_argument_analyzer.expected");
    for (line, (actual, wanted)) in out.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(out.lines().count(), expected.lines().count());
}
