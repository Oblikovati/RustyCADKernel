//! Native-backed endpoint projection cases for ShapeFix_EdgeProjAux.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{GeomCircle, GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Axis2d;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Direction2, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_fix::ShapeFixEdgeProjAux;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

const ORACLE: &str = include_str!("../../../tools/oracle/shape_fix_edge_proj_aux.expected");

#[test]
fn plane_line_and_reversed_line_match_pinned_oracle()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    set_pcurve(
        &edge,
        &surface,
        &line(Point2::default(), Direction2::new(1.0, 0.0)?),
    )?;
    assert_result("plane-line", &face, &edge)?;

    let edge = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    set_pcurve(
        &edge,
        &surface,
        &line(Point2::new(1.0, 0.0), Direction2::new(-1.0, 0.0)?),
    )?;
    assert_result("plane-reversed-line", &face, &edge)
}

#[test]
fn periodic_loop_clips_infinite_line_to_cylinder_bounds()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = cylinder_face()?;
    let circle: Handle<dyn rustycad_tkg3d::geom::GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(Axis2::default(), 1.0)?)).cast();
    let edge = BRepLibMakeEdge::from_curve_params(&circle, 0.0, std::f64::consts::TAU)?
        .edge()?
        .clone();
    set_pcurve(
        &edge,
        &surface,
        &line(Point2::default(), Direction2::new(1.0, 0.0)?),
    )?;
    assert_result("cylinder-periodic-loop", &face, &edge)
}

#[test]
fn absent_cylinder_pcurve_preserves_default_outputs()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, _) = cylinder_face()?;
    let edge = straight_edge(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0))?;
    assert_result("cylinder-missing-pcurve", &face, &edge)
}

#[test]
fn init_resets_outputs_before_recomputing() -> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let first = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    set_pcurve(
        &first,
        &surface,
        &line(Point2::default(), Direction2::new(1.0, 0.0)?),
    )?;
    let second = straight_edge(Point3::new(4.0, 0.0, 0.0), Point3::new(5.0, 0.0, 0.0))?;
    set_pcurve(
        &second,
        &surface,
        &line(Point2::new(4.0, 0.0), Direction2::new(1.0, 0.0)?),
    )?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &first);
    projection.compute(1.0e-7)?;
    projection.init(&face, &second);
    projection.compute(1.0e-7)?;
    assert_row_matches_projection("plane-line-at-four", &projection);
    Ok(())
}

fn assert_result(
    name: &str,
    face: &Face,
    edge: &Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let expected = expected_row(name);
    let projection = ShapeFixEdgeProjAux::with_face_edge(face, edge);
    projection.compute(1.0e-7)?;
    assert_eq!(projection.is_first_done(), expected[0] == 1.0);
    assert_eq!(projection.is_last_done(), expected[1] == 1.0);
    assert_close(projection.first_param(), expected[2]);
    assert_close(projection.last_param(), expected[3]);
    Ok(())
}

fn expected_row(name: &str) -> [f64; 4] {
    let row = ORACLE
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap();
    let values = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse::<f64>().unwrap());
    values.collect::<Vec<_>>().try_into().unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-12,
        "{actual} != {expected}"
    );
}

fn plane_face() -> Result<(Face, Handle<dyn GeomSurface>), rustycad_tkernel::standard::Failure> {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    make_face(surface)
}

fn cylinder_face() -> Result<(Face, Handle<dyn GeomSurface>), rustycad_tkernel::standard::Failure> {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0)?).cast();
    make_face(surface)
}

fn make_face(
    surface: Handle<dyn GeomSurface>,
) -> Result<(Face, Handle<dyn GeomSurface>), rustycad_tkernel::standard::Failure> {
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut face, &surface, 1.0e-7)?;
    Ok((face, surface))
}

fn straight_edge(first: Point3, last: Point3) -> Result<Edge, rustycad_tkernel::standard::Failure> {
    Ok(BRepLibMakeEdge::from_points(first, last)?.edge()?.clone())
}

fn line(point: Point2, direction: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(point, direction)).cast()
}

fn set_pcurve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve(edge, curve, surface, &Location::default(), 1.0e-7, None)?;
    builder.range_on_surface(
        edge,
        surface,
        &Location::default(),
        curve.first_parameter(),
        curve.last_parameter(),
    )
}

#[test]
fn iso_query_is_false_for_lines() {
    let projection = ShapeFixEdgeProjAux::new();
    assert!(!projection.is_iso(&line(Point2::default(), Direction2::new(0.0, 1.0).unwrap())));
}

#[test]
fn null_face_curve_query_matches_native_plane_projection()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, _) = plane_face()?;
    let edge = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    let pcurve = BRepTool::curve_on_face(&edge, &face)?.pcurve;
    assert!(!pcurve.is_null());
    Ok(())
}

#[test]
fn exposed_init_2d_matches_protected_occt_operation()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    set_pcurve(
        &edge,
        &surface,
        &line(Point2::default(), Direction2::new(1.0, 0.0)?),
    )?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &edge);
    projection.init_2d(1.0e-7)?;
    assert_row_matches_projection("direct-init-2d", &projection);
    Ok(())
}

#[test]
fn init_2d_reversed_edge_uses_unoriented_first_and_last_vertices()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    set_pcurve(
        &edge,
        &surface,
        &line(Point2::default(), Direction2::new(1.0, 0.0)?),
    )?;
    let reversed_shape = edge.reversed();
    let reversed = rustycad_tkbrep::topods::edge(&reversed_shape)?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &reversed);
    projection.init_2d(1.0e-7)?;
    assert_row_matches_projection("init-2d-reversed-edge", &projection);
    Ok(())
}

#[test]
fn init_2d_null_pcurve_preserves_parameters_but_clears_done_flags()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    set_pcurve(
        &edge,
        &surface,
        &line(Point2::default(), Direction2::new(1.0, 0.0)?),
    )?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &edge);
    projection.compute(1.0e-7)?;
    remove_edge_curves(&edge, &surface)?;
    projection.init_2d(1.0e-7)?;
    assert!(!projection.is_first_done() && !projection.is_last_done());
    assert_close(projection.first_param(), 0.0);
    assert_close(projection.last_param(), 1.0);
    Ok(())
}

#[test]
fn exposed_init_3d_matches_protected_occt_operation()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = straight_edge(Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    let basis = line(Point2::default(), Direction2::new(1.0, 0.0)?);
    let bounded: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&basis, 0.0, 1.0, true, true)?).cast();
    set_pcurve(&edge, &surface, &bounded)?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &edge);
    projection.init_3d(1.0e-7)?;
    assert_row_matches_projection("direct-init-3d", &projection);
    Ok(())
}

#[test]
fn init_3d_ignores_reversed_edge_orientation_for_endpoint_selection()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = edge_with_bounded_line(&surface, Point3::default(), Point3::new(1.0, 0.0, 0.0))?;
    let reversed_shape = edge.reversed();
    let reversed = rustycad_tkbrep::topods::edge(&reversed_shape)?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &reversed);
    projection.init_3d(1.0e-7)?;
    assert_row_matches_projection("init-3d-reversed-edge", &projection);
    Ok(())
}

#[test]
fn init_3d_marks_done_when_extrema_throws() -> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = edge_with_bounded_line(
        &surface,
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
    )?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &edge);
    projection.init_3d(-1.0)?;
    assert_row_matches_projection("init-3d-extrema-failure", &projection);
    Ok(())
}

#[test]
fn init_3d_matches_native_periodic_curve_without_endpoint_minima()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = degenerate_center_edge(&surface)?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &edge);
    projection.init_3d(1.0e-7)?;
    assert_row_matches_projection("init-3d-no-minimum", &projection);
    Ok(())
}

#[test]
fn init_3d_keeps_native_done_flags_when_extrema_is_not_done()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (face, surface) = plane_face()?;
    let edge = edge_with_bounded_line(
        &surface,
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
    )?;
    let projection = ShapeFixEdgeProjAux::with_face_edge(&face, &edge);
    projection.init_3d(1.0e-7)?;
    assert_row_matches_projection("init-3d-no-extrema-fallback", &projection);
    Ok(())
}

#[test]
fn update_param_2d_recognizes_periodic_basis_through_trim()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let projection = ShapeFixEdgeProjAux::new();
    let circle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(Axis2d::default(), 1.0, true)?).cast();
    let trimmed: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
        &circle,
        0.0,
        std::f64::consts::TAU,
        true,
        true,
    )?)
    .cast();
    projection.update_param_2d(&trimmed)?;
    let expected = expected_row("direct-update-periodic-trimmed-circle");
    assert_close(projection.first_param(), expected[0]);
    assert_close(projection.last_param(), expected[1]);
    assert_close(trimmed.value(0.0)?.x(), expected[2]);
    assert_close(trimmed.value(std::f64::consts::TAU)?.x(), expected[3]);
    Ok(())
}

#[test]
fn update_param_2d_does_not_reverse_nonclosed_bspline()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let projection = ShapeFixEdgeProjAux::new();
    let spline = nonclosed_bspline()?;
    projection.update_param_2d(&spline)?;
    assert_bspline_row(&projection, &spline);
    Ok(())
}

fn remove_edge_curves(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve(
        edge,
        &Handle::default(),
        surface,
        &Location::default(),
        1.0e-7,
        None,
    )?;
    builder.update_edge_curve(edge, &Handle::default(), 1.0e-7)
}

fn nonclosed_bspline() -> Result<Handle<dyn Geom2dCurve>, rustycad_tkernel::standard::Failure> {
    let mut poles = Array1::<Point2>::with_bounds(1, 2).unwrap();
    poles[1] = Point2::new(0.0, 0.0);
    poles[2] = Point2::new(1.0, 0.0);
    let mut knots = Array1::<f64>::with_bounds(1, 2).unwrap();
    knots[1] = 0.0;
    knots[2] = 1.0;
    let mut multiplicities = Array1::<i32>::with_bounds(1, 2).unwrap();
    multiplicities[1] = 2;
    multiplicities[2] = 2;
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles,
        &knots,
        &multiplicities,
        1,
        false,
    )?)
    .cast())
}

fn assert_bspline_row(projection: &ShapeFixEdgeProjAux, spline: &Handle<dyn Geom2dCurve>) {
    let expected = expected_row("direct-update-nonclosed-bspline");
    assert_close(projection.first_param(), expected[0]);
    assert_close(projection.last_param(), expected[1]);
    assert_close(
        spline.value(spline.first_parameter()).unwrap().x(),
        expected[2],
    );
    assert_close(
        spline.value(spline.last_parameter()).unwrap().x(),
        expected[3],
    );
}

fn edge_with_bounded_line(
    surface: &Handle<dyn GeomSurface>,
    first: Point3,
    last: Point3,
) -> Result<Edge, rustycad_tkernel::standard::Failure> {
    let edge = straight_edge(first, last)?;
    let basis = line(Point2::default(), Direction2::new(1.0, 0.0)?);
    let bounded: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&basis, 0.0, 1.0, true, true)?).cast();
    set_pcurve(&edge, surface, &bounded)?;
    Ok(edge)
}

fn degenerate_center_edge(
    surface: &Handle<dyn GeomSurface>,
) -> Result<Edge, rustycad_tkernel::standard::Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    add_coincident_vertices(&builder, &mut edge)?;
    builder.degenerated(&edge, true)?;
    set_pcurve(&edge, surface, &circle())?;
    builder.range_on_surface(
        &edge,
        surface,
        &Location::default(),
        0.0,
        std::f64::consts::TAU,
    )?;
    Ok(edge)
}

fn add_coincident_vertices(
    builder: &BRepBuilder,
    edge: &mut Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut vertex = Vertex::default();
    builder.make_vertex_at(&mut vertex, Point3::default(), 1.0e-7)?;
    builder.add(edge, &vertex.oriented(Orientation::Forward))?;
    builder.add(edge, &vertex.oriented(Orientation::Reversed))
}

fn circle() -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dCircle::from_axis(Axis2d::default(), 1.0, true).unwrap()).cast()
}

#[test]
fn exposed_update_param_2d_matches_protected_occt_operation()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let projection = ShapeFixEdgeProjAux::new();
    let curve = line(Point2::default(), Direction2::new(1.0, 0.0)?);
    projection.update_param_2d(&curve)?;
    let actual_line = curve.cast::<Geom2dLine>();
    let direction = actual_line.get().unwrap().direction();
    let expected = expected_row("direct-update-param-2d");
    assert_close(projection.first_param(), expected[0]);
    assert_close(projection.last_param(), expected[1]);
    assert_close(direction.x(), expected[2]);
    assert_close(direction.y(), expected[3]);
    Ok(())
}

fn assert_row_matches_projection(name: &str, projection: &ShapeFixEdgeProjAux) {
    let expected = expected_row(name);
    assert_eq!(projection.is_first_done(), expected[0] == 1.0);
    assert_eq!(projection.is_last_done(), expected[1] == 1.0);
    assert_close(projection.first_param(), expected[2]);
    assert_close(projection.last_param(), expected[3]);
}
