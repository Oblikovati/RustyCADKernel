//! Behavior tests of BRepLib_MakeFace, BRepLib_FindSurface, BRepLib::SameParameter and
//! BRepLib::ExtendFace (docs/tktopalgo-progress.md). The numeric oracle compares the
//! same functions with OCCT bit for bit (`blm_face`, `blp_findsurface`,
//! `blp_sameparam` and `blp_extendface` requests).
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Direction3, Line3, Plane3, Point2, Point3, Sphere3, StandardDirection3,
};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::{
    self, BRepLibFaceError, BRepLibFindSurface, BRepLibMakeFace, BRepLibMakePolygon,
};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

/// The first wire of a face.
fn first_wire(face: &Shape) -> Result<Shape, Failure> {
    Ok(Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?
        .current()
        .clone())
}

/// The X axis line.
fn x_line() -> Handle<dyn GeomCurve> {
    let x = Direction3::from_standard(StandardDirection3::X);
    Handle::new(GeomLine::from_lin(Line3::new(Point3::default(), x))).cast()
}

fn polygon(points: &[Point3]) -> Result<Wire, Failure> {
    let mut maker = BRepLibMakePolygon::new();
    for &point in points {
        maker.add_point(point)?;
    }
    maker.close()?;
    maker.wire()
}

const SQUARE: [Point3; 4] = [
    Point3::new(0., 0., 0.),
    Point3::new(1., 0., 0.),
    Point3::new(1., 1., 0.),
    Point3::new(0., 1., 0.),
];

#[test]
fn make_face_bounds_a_plane_with_one_closed_wire() -> Result<(), Failure> {
    let maker = BRepLibMakeFace::from_plane_bounds(Plane3::default(), (0., 2., 0., 1.))?;
    assert!(maker.is_done());
    let face = maker.face()?.into_shape();
    assert_eq!(count(&face, ShapeEnum::Wire)?, 1);
    assert_eq!(count(&face, ShapeEnum::Edge)?, 4);
    let wire = first_wire(&face)?;
    assert!(wire.closed()?);
    // An infinite band in V gives two wires of one edge each.
    let band = BRepLibMakeFace::from_plane_bounds(Plane3::default(), (0., 1., -2e100, 2e100))?;
    assert_eq!(count(&band.face()?.into_shape(), ShapeEnum::Wire)?, 2);
    Ok(())
}

#[test]
fn make_face_flags_the_poles_of_a_sphere_and_refuses_bad_bounds() -> Result<(), Failure> {
    let sphere = Sphere3::new(Axis3::default(), 1.)?;
    let half_pi = std::f64::consts::FRAC_PI_2;
    let maker = BRepLibMakeFace::from_sphere_bounds(sphere, (0., 6.0, -half_pi, half_pi))?;
    let degenerated = Explorer::new(
        &maker.face()?.into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?
    .filter(|e| BRepTool::degenerated(&Edge::unchecked(e.clone())).unwrap_or(false))
    .count();
    assert_eq!(degenerated, 2);
    // Swapped V bounds are reordered, then refused beyond the natural V range.
    let bad = BRepLibMakeFace::from_sphere_bounds(sphere, (0., 1., 2., -2.))?;
    assert_eq!(bad.error(), BRepLibFaceError::ParametersOutOfRange);
    assert!(!bad.is_done());
    Ok(())
}

#[test]
fn make_face_from_a_wire_finds_its_plane_and_orients_it() -> Result<(), Failure> {
    for points in [SQUARE.to_vec(), SQUARE.iter().rev().copied().collect()] {
        let maker = BRepLibMakeFace::from_wire(&polygon(&points)?, true)?;
        assert_eq!(maker.error(), BRepLibFaceError::FaceDone);
        let face = maker.face()?;
        // The plane of the square, with the material inside the wire (CheckInside).
        let surface = BRepAdaptorSurface::with_face(&face, true)?;
        assert_eq!(surface.plane()?.position().direction().z().abs(), 1.);
        let classifier = BRepTopAdaptorFClass2d::new(&face, 0.)?;
        assert_eq!(classifier.perform_infinite_point()?, State::Out);
    }
    let skew = polygon(&[p(0., 0., 0.), p(1., 0., 0.), p(1., 1., 0.), p(0., 1., 0.5)])?;
    assert_eq!(
        BRepLibMakeFace::from_wire(&skew, false)?.error(),
        BRepLibFaceError::NotPlanar
    );
    Ok(())
}

#[test]
fn is_degenerated_accepts_small_circles_only() -> Result<(), Failure> {
    let small: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(Axis2::default(), 1e-8)?)).cast();
    assert_eq!(BRepLibMakeFace::is_degenerated(&small, 1e-7)?, (true, 1e-7));
    let line = x_line();
    assert!(!BRepLibMakeFace::is_degenerated(&line, 1e-7)?.0);
    Ok(())
}

#[test]
fn find_surface_fits_a_plane_or_reuses_the_edge_surface() -> Result<(), Failure> {
    let finder = BRepLibFindSurface::new(&polygon(&SQUARE)?.into_shape(), -1., false, false)?;
    assert!(finder.found() && !finder.existed());
    assert_eq!(finder.tolerance_reached(), 0.);
    let face = BRepLibMakeFace::from_plane_bounds(Plane3::default(), (0., 1., 0., 1.))?.face()?;
    let wire = first_wire(&face.into_shape())?;
    let existing = BRepLibFindSurface::new(&wire, -1., false, true)?;
    assert!(existing.found() && existing.existed());
    let empty = BRepLibFindSurface::new(&Shape::default(), -1., false, false)?;
    assert!(!empty.found());
    Ok(())
}

/// A segment on the XY plane whose Bezier p-curve runs at another speed than the line.
fn reparametrized_edge() -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let line = x_line();
    let mut poles = Array1::<Point2>::with_bounds(1, 3).expect("three poles");
    for (i, x) in [0., 0.8, 1.].into_iter().enumerate() {
        poles[i as i32 + 1] = Point2::new(x, 0.);
    }
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dBezierCurve::new(&poles)?).cast();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, &line, 1e-7)?;
    builder.update_edge_pcurve(&edge, &pcurve, &plane, &Location::default(), 1e-7, None)?;
    builder.range(&edge, 0., 1., false)?;
    for (point, orientation) in [
        (p(0., 0., 0.), Orientation::Forward),
        (p(1., 0., 0.), Orientation::Reversed),
    ] {
        let mut vertex = Vertex::default();
        builder.make_vertex_at(&mut vertex, point, 1e-7)?;
        builder.add(&mut edge, &vertex.oriented(orientation))?;
    }
    builder.same_parameter(&edge, false)?;
    Ok(edge)
}

#[test]
fn same_parameter_reparametrizes_the_pcurve() -> Result<(), Failure> {
    let edge = reparametrized_edge()?;
    let mut new_tol = -1.;
    let copy = brep_lib::same_parameter_edge_with_tolerance(&edge, 1e-5, &mut new_tol, false)?;
    assert!(BRepTool::same_parameter(&copy)? && !BRepTool::same_parameter(&edge)?);
    assert!(new_tol > 0. && new_tol < 1e-5, "new tolerance {new_tol}");
    brep_lib::same_parameter_shape(&edge.clone().into_shape(), 1e-5, false)?;
    assert!(BRepTool::same_parameter(&edge)?);
    // A SameParameter edge is left alone.
    assert!(
        brep_lib::same_parameter_edge_with_tolerance(&edge, 1e-5, &mut new_tol, true)?.is_null()
    );
    Ok(())
}

#[test]
fn extend_face_widens_a_plane_face() -> Result<(), Failure> {
    let face = BRepLibMakeFace::from_plane_bounds(Plane3::default(), (0., 1., 0., 1.))?.face()?;
    let extended = brep_lib::extend_face(&face, 0.5, [true, false, false, true])?;
    let surface = BRepAdaptorSurface::with_face(&extended, true)?;
    assert_eq!(surface.first_u_parameter()?, -0.5);
    assert_eq!(surface.last_u_parameter()?, 1.);
    assert_eq!(surface.last_v_parameter()?, 1.5);
    Ok(())
}
