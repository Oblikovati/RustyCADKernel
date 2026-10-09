//! The BRepFill package functions: ruled faces and shells, Axe, SearchOrigin and the
//! reduced curvilinear abscissas.
use rustycad_tkbool::brep_fill;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakePolygon};

/// A polygon wire through the points, closed when asked.
fn polygon(points: &[(f64, f64, f64)], closed: bool) -> Result<Wire, Failure> {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        maker.add_point(Point3::new(x, y, z))?;
    }
    if closed {
        maker.close()?;
    }
    maker.wire()
}

/// The square of side 2 around the Z axis at height `z`.
fn square(z: f64) -> Result<Wire, Failure> {
    polygon(
        &[(-1., -1., z), (1., -1., z), (1., 1., z), (-1., 1., z)],
        true,
    )
}

/// The number of sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut n = 0;
    while explorer.more() {
        n += 1;
        explorer.next().unwrap();
    }
    n
}

/// The wire explorer's edge count.
fn explored(wire: &Wire) -> usize {
    let mut explorer = WireExplorer::with_wire(wire).unwrap();
    let mut n = 0;
    while explorer.more() {
        n += 1;
        explorer.next().unwrap();
    }
    n
}

#[test]
fn ruled_face_and_shell_bound_their_faces_with_four_edges() -> Result<(), Failure> {
    let e1 = BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 0., 0.))?
        .edge()?;
    let e2 = BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 1., 1.), Point3::new(2., 1., 1.))?
        .edge()?;
    let face = brep_fill::ruled_face(&e1, &e2)?;
    assert_eq!(count(&face, ShapeEnum::Edge), 4);
    let shell = brep_fill::ruled_shell(&square(0.)?, &square(1.)?)?;
    assert_eq!(count(&shell, ShapeEnum::Face), 4);
    // The explorer meets each of the 8 lateral and section edges once per face: 16.
    assert_eq!(count(&shell, ShapeEnum::Edge), 16);
    // A tube without caps has free section edges: BRep_Tool::IsClosed is false.
    assert!(!shell.closed()?);
    Ok(())
}

#[test]
fn compute_acr_reduces_the_cumulated_lengths() -> Result<(), Failure> {
    let wire = polygon(&[(0., 0., 0.), (1., 0., 0.), (1., 3., 0.)], false)?;
    let mut acr: Array1<'static, f64> = Array1::with_bounds(0, 2).unwrap();
    brep_fill::compute_acr(&wire, &mut acr)?;
    assert_eq!(acr.as_slice(), [4., 0.25, 1.]);
    Ok(())
}

#[test]
fn insert_acr_cuts_inside_the_edges_only() -> Result<(), Failure> {
    let wire = square(0.)?;
    let mut cuts: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    cuts.as_mut_slice().copy_from_slice(&[0.125, 0.25, 0.6]);
    let cut = brep_fill::insert_acr(&wire, &cuts, 1e-6)?;
    // 0.25 is a vertex already; 0.125 and 0.6 cut the first and the third edges.
    assert_eq!(explored(&cut), 6);
    Ok(())
}

#[test]
fn search_origin_restarts_a_closed_wire_and_refuses_an_open_one() -> Result<(), Failure> {
    let mut wire = square(0.)?;
    brep_fill::search_origin(
        &mut wire,
        Point3::new(0.2, -1., 0.),
        Vector3::new(1., 0., 0.),
        1e-7,
    )?;
    assert_eq!(explored(&wire), 5);
    assert!(wire.closed()?);
    let mut open = polygon(&[(0., 0., 0.), (1., 0., 0.)], false)?;
    let error =
        brep_fill::search_origin(&mut open, Point3::default(), Vector3::new(1., 0., 0.), 1e-7)
            .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
    Ok(())
}

#[test]
fn axe_frames_a_profile_touching_a_spine_vertex() -> Result<(), Failure> {
    let spine = square(0.)?;
    let profile = polygon(&[(-1., -1., 0.), (-1., -1., 1.)], false)?;
    let (frame, on_spine) = brep_fill::axe(&spine, &profile, 1e-7)?;
    assert!(on_spine);
    assert_eq!(frame.location(), Point3::new(-1., -1., 0.));
    assert!(frame.direction().z().abs() == 1.);
    Ok(())
}
