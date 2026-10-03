//! Failure, default-argument, face-filtering and replacement-context behavior.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2d, Direction3, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkshhealing::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

#[test]
fn missing_3d_curve_failure_matches_native() -> Result<(), rustycad_tkernel::standard::Failure> {
    let edge = edge_without_3d_curve()?;
    let mut fixer = ShapeFixEdge::new();
    assert_no3d_vertex_result(&mut fixer, &edge)?;
    assert_no3d_parameter_result(&mut fixer, &edge)?;
    Ok(())
}

#[test]
fn face_overload_filters_other_surface_pcurves() -> Result<(), rustycad_tkernel::standard::Failure>
{
    let (edge, _) = edge_with_pcurve(0.2)?;
    let face = plane_face(0.0)?;
    let mut fixer = ShapeFixEdge::new();
    let changed = fixer.fix_vertex_tolerance_on_face(&edge, &face)?;
    assert_vertex_row("face_unrelated", changed, &fixer, &edge)?;
    let changed = fixer.fix_vertex_tolerance(&edge)?;
    assert_vertex_row("all_pcurves", changed, &fixer, &edge)?;
    let changed = fixer.fix_vertex_tolerance_on_face(&edge, &face)?;
    assert_vertex_row("status_reset", changed, &fixer, &edge)
}

#[test]
fn default_tolerance_is_zero_for_both_same_parameter_overloads()
-> Result<(), rustycad_tkernel::standard::Failure> {
    assert_default_same_parameter(&mut ShapeFixEdge::new(), &edge_with_pcurve(0.2)?.0)?;
    let (edge, surface) = edge_with_pcurve(0.2)?;
    assert_default_face_same_parameter(&mut ShapeFixEdge::new(), &edge, &surface)?;
    Ok(())
}

#[test]
fn context_copies_repaired_vertices_without_mutating_source()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (edge, _) = edge_with_pcurve(0.2)?;
    let context = Handle::new(ShapeBuildReShape::new());
    let mut fixer = ShapeFixEdge::new();
    fixer.set_context(&context);
    assert_context_copy(&mut fixer, &context, &edge)
}

#[test]
fn context_rejects_a_replacement_of_the_wrong_shape_type()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (edge, _) = edge_with_pcurve(0.2)?;
    let vertex = ShapeAnalysisEdge::new().first_vertex(&edge);
    let context = Handle::new(ShapeBuildReShape::new());
    context.replace(&edge, &Shape::from(vertex))?;
    let mut fixer = ShapeFixEdge::new();
    fixer.set_context(&context);
    let changed = fixer.fix_vertex_tolerance(&edge)?;
    assert_native_row(
        "context_wrong_type",
        &[
            flag(changed),
            flag(fixer.status_flag(ShapeExtendStatus::Ok)),
            flag(fixer.status_flag(ShapeExtendStatus::Done)),
        ],
    );
    Ok(())
}

fn assert_no3d_vertex_result(
    fixer: &mut ShapeFixEdge,
    edge: &Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let changed = fixer.fix_vertex_tolerance(edge)?;
    let values = vertex_row(changed, fixer, edge)?;
    assert_native_row("no3d_vertex", &values);
    Ok(())
}

fn assert_no3d_parameter_result(
    fixer: &mut ShapeFixEdge,
    edge: &Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let changed = fixer.fix_same_parameter_default(edge)?;
    let values = [
        flag(changed),
        flag(fixer.status_flag(ShapeExtendStatus::Ok)),
        flag(fixer.status_flag(ShapeExtendStatus::Fail1)),
        flag(fixer.status_flag(ShapeExtendStatus::Fail2)),
        flag(BRepTool::same_parameter(edge)?),
    ];
    assert_native_row("no3d_parameter", &values);
    Ok(())
}

fn assert_vertex_row(
    label: &str,
    changed: bool,
    fixer: &ShapeFixEdge,
    edge: &Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    assert_native_row(label, &vertex_row(changed, fixer, edge)?);
    Ok(())
}

fn vertex_row(
    changed: bool,
    fixer: &ShapeFixEdge,
    edge: &Edge,
) -> Result<[f64; 6], rustycad_tkernel::standard::Failure> {
    Ok([
        flag(changed),
        flag(fixer.status_flag(ShapeExtendStatus::Ok)),
        flag(fixer.status_flag(ShapeExtendStatus::Done1)),
        flag(fixer.status_flag(ShapeExtendStatus::Done2)),
        vertex_tolerance(edge, true)?,
        vertex_tolerance(edge, false)?,
    ])
}

fn assert_default_same_parameter(
    fixer: &mut ShapeFixEdge,
    edge: &Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let changed = fixer.fix_same_parameter_default(edge)?;
    assert_same_parameter_row("same_parameter_default", changed, fixer, edge)
}

fn assert_default_face_same_parameter(
    fixer: &mut ShapeFixEdge,
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let face = face_for_surface(surface)?;
    let changed = fixer.fix_same_parameter_on_face_default(edge, &face)?;
    assert_same_parameter_row("same_parameter_face_default", changed, fixer, edge)
}

fn assert_same_parameter_row(
    label: &str,
    changed: bool,
    fixer: &ShapeFixEdge,
    edge: &Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let values = [
        flag(changed),
        flag(fixer.status_flag(ShapeExtendStatus::Ok)),
        flag(fixer.status_flag(ShapeExtendStatus::Done1)),
        flag(BRepTool::same_parameter(edge)?),
        BRepTool::edge_tolerance(edge)?,
    ];
    assert_native_row(label, &values);
    Ok(())
}

fn assert_context_copy(
    fixer: &mut ShapeFixEdge,
    context: &Handle<ShapeBuildReShape>,
    edge: &Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let changed = fixer.fix_vertex_tolerance(edge)?;
    assert_vertex_row("context_result", changed, fixer, edge)?;
    let copy = Edge::unchecked(context.apply(edge, ShapeEnum::Shape)?);
    let tolerances = [
        vertex_tolerance(edge, true)?,
        vertex_tolerance(&copy, true)?,
    ];
    assert_native_row("context_tolerance", &tolerances);
    Ok(())
}

fn edge_without_3d_curve() -> Result<Edge, rustycad_tkernel::standard::Failure> {
    let edge = BRepLibMakeEdge::from_points(Point3::default(), Point3::new(1.0, 0.0, 0.0))?
        .edge()?
        .clone();
    ShapeBuildEdge.remove_curve_3d(&edge)?;
    assert!(!ShapeAnalysisEdge::new().has_curve_3d(&edge)?);
    Ok(edge)
}

fn edge_with_pcurve(
    offset: f64,
) -> Result<(Edge, Handle<dyn GeomSurface>), rustycad_tkernel::standard::Failure> {
    let edge = BRepLibMakeEdge::from_points(Point3::default(), Point3::new(1.0, 0.0, 0.0))?
        .edge()?
        .clone();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(
        Point3::new(0.0, 0.0, offset),
        Direction3::new(0.0, 0.0, 1.0)?,
    )?)
    .cast();
    attach_line_pcurve(&edge, &surface)?;
    Ok((edge, surface))
}

fn attach_line_pcurve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve(edge, &curve, surface, &Location::default(), 1.0e-7, None)?;
    builder.range_on_surface(edge, surface, &Location::default(), 0.0, 1.0)
}

fn plane_face(z: f64) -> Result<Face, rustycad_tkernel::standard::Failure> {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(
        Point3::new(0.0, 0.0, z),
        Direction3::new(0.0, 0.0, 1.0)?,
    )?)
    .cast();
    face_for_surface(&surface)
}

fn face_for_surface(
    surface: &Handle<dyn GeomSurface>,
) -> Result<Face, rustycad_tkernel::standard::Failure> {
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut face, surface, 1.0e-7)?;
    Ok(face)
}

fn vertex_tolerance(edge: &Edge, first: bool) -> Result<f64, rustycad_tkernel::standard::Failure> {
    let analyzer = ShapeAnalysisEdge::new();
    let vertex = if first {
        analyzer.first_vertex(edge)
    } else {
        analyzer.last_vertex(edge)
    };
    BRepTool::vertex_tolerance(&vertex)
}

fn flag(value: bool) -> f64 {
    f64::from(value)
}

fn assert_native_row(label: &str, actual: &[f64]) {
    let fixture = include_str!("fixtures/fix_edge_api_context.expected");
    let row = fixture
        .lines()
        .find(|line| line.split_whitespace().next() == Some(label))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(actual, expected, "native OCCT row {label}");
}
