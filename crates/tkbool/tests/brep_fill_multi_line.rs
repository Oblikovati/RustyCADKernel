//! BRepFill_MultiLine, BRepFill_ApproxSeewing and BRepFill_TrimSurfaceTool: the
//! intersection line of two faces of an evolved shape along a bisector.
use rustycad_tkbool::brep_fill::{
    BRepFillApproxSeewing, BRepFillMultiLine, BRepFillTrimSurfaceTool,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Wire};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::app_cont::Function;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Direction2, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The planar face of a closed polygon, with its edges in order.
fn polygon_face(points: &[(f64, f64, f64)]) -> Result<(Face, Vec<Edge>), Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        polygon.add_point(Point3::new(x, y, z))?;
    }
    polygon.close()?;
    let wire: Wire = polygon.wire()?;
    let face = BRepLibMakeFace::from_wire(&wire, true)?.face()?;
    let mut edges = Vec::new();
    let mut ex = Explorer::new(wire.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex.more() {
        edges.push(topods::edge(ex.current())?);
        ex.next()?;
    }
    Ok((face, edges))
}

/// The two faces rising at 45 degrees from the sides A-B and B-F of the corner B =
/// (4, 0, 0), with their bottom and top edges.
fn corner() -> Result<(Face, Face, Vec<Edge>, Vec<Edge>), Failure> {
    let (f1, e1) = polygon_face(&[(0., 0., 0.), (4., 0., 0.), (3., 1., 1.), (0., 1., 1.)])?;
    let (f2, e2) = polygon_face(&[(4., 0., 0.), (4., 4., 0.), (3., 4., 1.), (3., 1., 1.)])?;
    Ok((f1, f2, e1, e2))
}

/// The bisector of the corner from B inward, trimmed to [0, last].
fn bisector(last: f64) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(4., 0.),
        Direction2::new(-s, s)?,
    ))
    .cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(&line, 0., last, true, true)?).cast())
}

#[test]
fn a_line_bisector_between_two_planes_is_a_particular_case() -> Result<(), Failure> {
    let (f1, f2, e1, e2) = corner()?;
    let ml = BRepFillMultiLine::new(&f1, &f2, &e1[0], &e2[0], false, false, &bisector(1.)?)?;
    assert!(ml.is_particular_case());
    assert_eq!(ml.continuity(), Continuity::C0);
    assert_eq!((ml.first_parameter(), ml.last_parameter()), (0., 1.));
    // The point above the bisector at the distance s from both sides is at height s.
    let p = ml.value_3d(1.)?;
    let s = std::f64::consts::FRAC_1_SQRT_2;
    assert!((p.z() - s).abs() < 1e-9, "{p:?}");
    let (curve, pc1, pc2) = ml.curves()?;
    assert!(!curve.is_null() && !pc1.is_null() && !pc2.is_null());
    Ok(())
}

#[test]
fn the_approximation_follows_the_intersection_line() -> Result<(), Failure> {
    let (f1, f2, e1, e2) = corner()?;
    let ml = BRepFillMultiLine::new(&f1, &f2, &e1[0], &e2[0], false, false, &bisector(1.)?)?;
    let app = BRepFillApproxSeewing::with_multi_line(&ml)?;
    assert!(app.is_done());
    let curve = app
        .curve()?
        .get()
        .ok_or_else(|| Failure::new("null curve"))?;
    let end = curve.value(curve.last_parameter())?;
    assert!(end.distance(&ml.value_3d(1.)?) < 1e-6);
    assert!(!app.curve_on_f1()?.is_null() && !app.curve_on_f2()?.is_null());
    Ok(())
}

#[test]
fn the_approximation_needs_a_perform() {
    let app = BRepFillApproxSeewing::new();
    assert!(!app.is_done());
    let error = app.curve().err().map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("StdFail_NotDone"));
}

#[test]
fn the_trim_tool_cuts_the_bisector_at_the_top_edges() -> Result<(), Failure> {
    let (f1, f2, e1, e2) = corner()?;
    let tool = BRepFillTrimSurfaceTool::new(&bisector(2.)?, &f1, &f2, &e1[0], &e2[0], false, false);
    let mut points: Sequence<Point3> = Sequence::default();
    tool.intersect_with(&e1[2], &e2[2], &mut points)?;
    assert_eq!(points.len(), 1);
    let s = std::f64::consts::SQRT_2;
    let first = points.first().copied().unwrap_or_default();
    assert!((first.x() - s).abs() < 1e-9, "{first:?}");
    assert!(tool.is_on_face(Point2::new(2., 0.5))?);
    assert!(!tool.is_on_face(Point2::new(-1., -1.))?);
    let ((curve, ..), cont) = tool.project(0.2, 1.2)?;
    assert!(!curve.is_null());
    assert_eq!(cont, Continuity::C0);
    Ok(())
}
