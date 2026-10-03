//! Native-backed endpoint projection cases for ShapeFix_EdgeProjAux.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCircle, GeomCylindricalSurface, GeomPlane, GeomSurface};
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
    let row = ORACLE.lines().find(|line| line.starts_with(name)).unwrap();
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
