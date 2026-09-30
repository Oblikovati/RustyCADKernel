//! Behavior tests of the BRepLib makers (docs/tktopalgo-progress.md): each status
//! value, the shapes built and the native quirks kept. The numeric oracle compares the
//! same makers with OCCT bit for bit (`blm_*` requests).
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{CompSolid, Edge, Face, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Direction2, Direction3, Point2, Point3};
use rustycad_tktopalgo::brep_lib::{
    self, BRepLibEdgeError, BRepLibMakeEdge, BRepLibMakeEdge2d, BRepLibMakePolygon,
    BRepLibMakeShell, BRepLibMakeSolid, BRepLibMakeVertex, BRepLibMakeWire,
    BRepLibShapeModification, BRepLibShellError, BRepLibWireError,
};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The process-wide BRepLib::Precision and BRepLib::Plane are shared by the tests.
static GLOBALS: Mutex<()> = Mutex::new(());

fn globals() -> MutexGuard<'static, ()> {
    GLOBALS.lock().unwrap_or_else(PoisonError::into_inner)
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn vertex(point: Point3, tolerance: f64) -> Result<Vertex, Failure> {
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, point, tolerance)?;
    Ok(v)
}

fn count(shape: &rustycad_tkbrep::topods::Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    let mut n = 0;
    let mut x = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while x.more() {
        n += 1;
        x.next()?;
    }
    Ok(n)
}

#[test]
fn precision_and_plane_are_process_wide_and_restored() {
    let _guard = globals();
    assert_eq!(brep_lib::precision(), precision::confusion());
    brep_lib::set_precision(1e-3);
    let vertex = BRepLibMakeVertex::new(p(1., 2., 3.))
        .unwrap()
        .vertex()
        .unwrap();
    assert_eq!(BRepTool::vertex_tolerance(&vertex).unwrap(), 1e-3);
    brep_lib::set_precision(precision::confusion());
    let first = brep_lib::plane();
    assert_eq!(first.object_address(), brep_lib::plane().object_address());
    brep_lib::set_plane(&Handle::default());
    assert!(brep_lib::plane().is_some());
}

#[test]
fn make_edge_segments_record_each_error() -> Result<(), Failure> {
    let _guard = globals();
    let edge = BRepLibMakeEdge::from_points(p(0., 0., 0.), p(1., 0., 0.))?;
    assert!(edge.is_done());
    assert_eq!(edge.error(), BRepLibEdgeError::EdgeDone);
    let (_, first, last) = BRepTool::curve(&edge.edge()?)?;
    assert_eq!((first, last), (0.0, 1.0));
    let same = BRepLibMakeEdge::from_points(p(0., 0., 0.), p(0., 0., 0.))?;
    assert!(!same.is_done());
    assert_eq!(same.error(), BRepLibEdgeError::LineThroughIdenticPoints);
    assert_eq!(
        same.vertex1().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let far = vertex(p(5., 5., 5.), 1e-7)?;
    let line = BRepLibMakeEdge::from_lin_vertices(
        rustycad_tkmath::gp::Line3::new(p(0., 0., 0.), Direction3::new(1., 0., 0.)?),
        &far,
        &Vertex::default(),
    )?;
    assert_eq!(line.error(), BRepLibEdgeError::PointProjectionFailed);
    Ok(())
}

#[test]
fn make_edge_closed_curves_share_one_vertex() -> Result<(), Failure> {
    let _guard = globals();
    let circle = Circle3::new(Axis2::new(p(0., 0., 0.), Direction3::new(0., 0., 1.)?)?, 2.)?;
    let edge = BRepLibMakeEdge::from_circ(circle)?;
    assert!(edge.is_done());
    assert!(edge.vertex1()?.is_same(edge.vertex2()?));
    assert!(edge.edge()?.closed()?);
    let tiny = Circle3::new(
        Axis2::new(p(0., 0., 0.), Direction3::new(0., 0., 1.)?)?,
        1e-9,
    )?;
    let v = vertex(p(1e-9, 0., 0.), 1e-7)?;
    let degenerated = BRepLibMakeEdge::from_circ_vertices(tiny, &v, &v)?;
    assert!(BRepTool::degenerated(&degenerated.edge()?)?);
    let other = vertex(p(2., 0., 0.), 1e-7)?;
    let two = BRepLibMakeEdge::from_circ_vertices(circle, &v, &other)?;
    assert!(!two.is_done());
    Ok(())
}

#[test]
fn make_edge_range_and_infinite_parameter_errors() -> Result<(), Failure> {
    let _guard = globals();
    let mut poles = Array1::<Point3>::with_bounds(1, 3).unwrap();
    poles[1] = p(0., 0., 0.);
    poles[2] = p(1., 1., 0.);
    poles[3] = p(2., 0., 0.);
    let bezier: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles)?).cast();
    let out = BRepLibMakeEdge::from_curve_params(&bezier, 0.5, 1.5)?;
    assert_eq!(out.error(), BRepLibEdgeError::ParameterOutOfRange);
    let line: Handle<dyn GeomCurve> = Handle::new(rustycad_tkg3d::geom::GeomLine::from_point(
        p(0., 0., 0.),
        Direction3::new(1., 0., 0.)?,
    ))
    .cast();
    let v = vertex(p(0., 0., 0.), 1e-7)?;
    let infinite =
        BRepLibMakeEdge::from_curve_vertices_params(&line, &v, &Vertex::default(), -2e100, 1.)?;
    assert_eq!(
        infinite.error(),
        BRepLibEdgeError::PointWithInfiniteParameter
    );
    let away = vertex(p(0., 3., 0.), 1e-7)?;
    let wrong =
        BRepLibMakeEdge::from_curve_vertices_params(&line, &away, &Vertex::default(), 0., 1.)?;
    assert_eq!(wrong.error(), BRepLibEdgeError::DifferentsPointAndParameter);
    // A failing Init keeps the edge of the earlier one.
    let mut maker = BRepLibMakeEdge::from_curve_params(&line, 0., 1.)?;
    maker.init_curve_params(&bezier, 3., 4.)?;
    assert!(maker.is_done());
    assert_eq!(maker.error(), BRepLibEdgeError::ParameterOutOfRange);
    Ok(())
}

#[test]
fn make_edge_on_surface_reverses_swapped_parameters() -> Result<(), Failure> {
    let _guard = globals();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let segment: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0., 0.),
        Direction2::new(1., 0.)?,
    ))
    .cast();
    let edge = BRepLibMakeEdge::from_pcurve_params(&segment, &plane, 2., 1.)?;
    assert!(edge.is_done());
    assert_eq!(edge.edge()?.orientation(), Orientation::Reversed);
    assert!(BRepTool::curve(&edge.edge()?)?.0.is_null());
    Ok(())
}

#[test]
fn make_edge2d_uses_the_current_plane() -> Result<(), Failure> {
    let _guard = globals();
    let edge = BRepLibMakeEdge2d::from_points(Point2::new(0., 0.), Point2::new(3., 4.))?;
    assert!(edge.is_done());
    assert_eq!(BRepTool::pnt(edge.vertex2()?)?.coordinates(), &[3., 4., 0.]);
    brep_lib::set_plane(&Handle::new(GeomPlane::from_point(
        p(0., 0., 5.),
        Direction3::new(0., 0., 1.)?,
    )?));
    let lifted = BRepLibMakeEdge2d::from_points(Point2::new(0., 0.), Point2::new(3., 4.));
    brep_lib::set_plane(&Handle::default());
    assert_eq!(BRepTool::pnt(lifted?.vertex2()?)?.z(), 5.);
    let identical = BRepLibMakeEdge2d::from_points(Point2::new(1., 1.), Point2::new(1., 1.))?;
    assert_eq!(
        identical.error(),
        BRepLibEdgeError::LineThroughIdenticPoints
    );
    Ok(())
}

#[test]
fn make_polygon_closes_on_the_first_vertex() -> Result<(), Failure> {
    let _guard = globals();
    let square = BRepLibMakePolygon::from_four_points(
        p(0., 0., 0.),
        p(1., 0., 0.),
        p(1., 1., 0.),
        p(0., 1., 0.),
        true,
    )?;
    assert!(square.is_done());
    let wire: Wire = square.wire()?;
    assert!(wire.closed()?);
    assert_eq!(count(&wire, ShapeEnum::Edge)?, 4);
    let mut open = BRepLibMakePolygon::new();
    open.add_point(p(0., 0., 0.))?;
    assert!(!open.added());
    open.add_point(p(0., 0., 0.))?;
    assert!(!open.is_done());
    assert!(open.last_vertex().is_null());
    Ok(())
}

fn segment(a: &Vertex, b: &Vertex) -> Result<Edge, Failure> {
    BRepLibMakeEdge::from_vertices(a, b)?.edge()
}

#[test]
fn make_wire_connects_and_reports_disconnection() -> Result<(), Failure> {
    let _guard = globals();
    let v: Vec<Vertex> = [(0., 0.), (1., 0.), (1., 1.), (0., 1.)]
        .iter()
        .map(|&(x, y)| vertex(p(x, y, 0.), 1e-7))
        .collect::<Result<_, _>>()?;
    let (e0, e1, e2, e3) = (
        segment(&v[0], &v[1])?,
        segment(&v[1], &v[2])?,
        segment(&v[2], &v[3])?,
        segment(&v[3], &v[0])?,
    );
    let wire = BRepLibMakeWire::from_four_edges(&e0, &e1, &e2, &e3)?;
    assert_eq!(wire.error(), BRepLibWireError::WireDone);
    assert!(wire.wire()?.closed()?);
    let far = segment(&vertex(p(5., 5., 5.), 1e-7)?, &vertex(p(6., 5., 5.), 1e-7)?)?;
    let broken = BRepLibMakeWire::from_two_edges(&e0, &far)?;
    assert_eq!(broken.error(), BRepLibWireError::DisconnectedWire);
    assert!(!broken.is_done());
    // A vertex within tolerance of the wire end is merged: the edge is copied.
    let near = vertex(p(1., 1e-5, 0.), 1e-3)?;
    let merged =
        BRepLibMakeWire::from_two_edges(&e0, &segment(&near, &vertex(p(2., 0., 0.), 1e-7)?)?)?;
    assert_eq!(merged.error(), BRepLibWireError::WireDone);
    assert!(merged.vertex().is_same(&v[1]));
    Ok(())
}

#[test]
fn make_wire_list_merges_coincident_vertices() -> Result<(), Failure> {
    let _guard = globals();
    // Three segments in shuffled order whose ends miss each other by less than the
    // tolerances: the list Add merges them into one open chain.
    let a = segment(&vertex(p(0., 0., 0.), 1e-7)?, &vertex(p(1., 0., 0.), 1e-3)?)?;
    let b = segment(
        &vertex(p(1., 1e-4, 0.), 1e-3)?,
        &vertex(p(2., 0., 0.), 1e-3)?,
    )?;
    let c = segment(
        &vertex(p(2., 1e-4, 0.), 1e-3)?,
        &vertex(p(3., 0., 0.), 1e-7)?,
    )?;
    let mut edges: List<rustycad_tkbrep::topods::Shape> = List::default();
    for edge in [&c, &a, &b] {
        edges.append(edge.clone().into_shape());
    }
    let mut maker = BRepLibMakeWire::default();
    maker.add_list(&edges)?;
    assert!(maker.is_done());
    assert_eq!(maker.error(), BRepLibWireError::WireDone);
    let wire = maker.wire()?;
    assert_eq!(count(&wire, ShapeEnum::Edge)?, 3);
    assert_eq!(count(&wire, ShapeEnum::Vertex)?, 6);
    let mut distinct = rustycad_tkbrep::toptools::IndexedMapOfShape::default();
    rustycad_tkbrep::topexp::map_shapes_of_type(&wire, ShapeEnum::Vertex, &mut distinct)?;
    assert_eq!(distinct.extent(), 4);
    // A far edge is never reached: the wire keeps the others and reports it.
    let far = segment(&vertex(p(9., 9., 9.), 1e-7)?, &vertex(p(9., 8., 9.), 1e-7)?)?;
    let mut rest: List<rustycad_tkbrep::topods::Shape> = List::default();
    rest.append(a.into_shape());
    rest.append(far.into_shape());
    let mut broken = BRepLibMakeWire::default();
    broken.add_list(&rest)?;
    assert!(!broken.is_done());
    assert_eq!(broken.error(), BRepLibWireError::DisconnectedWire);
    // An empty list adds 0 of 0 edges: the maker is done.
    let mut empty = BRepLibMakeWire::default();
    empty.add_list(&List::default())?;
    assert!(empty.is_done());
    Ok(())
}

fn square_face(size: f64) -> Result<Face, Failure> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let shell = BRepLibMakeShell::with_bounds(&plane, (0., size, 0., size), false)?.shell()?;
    let x = Explorer::new(&shell, ShapeEnum::Face, ShapeEnum::Shape)?;
    Ok(Face::unchecked(x.current().clone()))
}

#[test]
fn make_solid_deletes_the_faces_shared_by_a_compsolid() -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let (f1, f2) = (square_face(1.)?, square_face(2.)?);
    let mut shells = [Shell::default(), Shell::default()];
    for (shell, faces) in shells.iter_mut().zip([[&f1, &f2], [&f2, &f2]]) {
        builder.make_shell(shell);
        builder.add(shell, faces[0])?;
        if !faces[0].is_same(faces[1]) {
            builder.add(shell, faces[1])?;
        }
    }
    let mut cs = CompSolid::default();
    builder.make_comp_solid(&mut cs);
    for shell in &shells {
        let mut solid = Solid::default();
        builder.make_solid(&mut solid);
        builder.add(&mut solid, shell)?;
        builder.add(&mut cs, &solid)?;
    }
    let maker = BRepLibMakeSolid::from_comp_solid(&cs)?;
    assert!(maker.is_done());
    assert_eq!(maker.face_status(&f2), BRepLibShapeModification::Deleted);
    assert_eq!(maker.face_status(&f1), BRepLibShapeModification::Preserved);
    assert_eq!(count(&*maker.solid()?, ShapeEnum::Face)?, 1);
    let two = BRepLibMakeSolid::from_two_shells(&shells[0], &shells[1])?;
    assert_eq!(count(&*two.solid()?, ShapeEnum::Shell)?, 2);
    Ok(())
}

#[test]
fn make_shell_builds_one_face_per_c2_patch() -> Result<(), Failure> {
    let cylinder: Handle<dyn GeomSurface> = Handle::new(
        rustycad_tkg3d::geom::GeomCylindricalSurface::new(Axis3::default(), 1.)?,
    )
    .cast();
    let shell =
        BRepLibMakeShell::with_bounds(&cylinder, (0., 2. * std::f64::consts::PI, 0., 1.), false)?;
    assert_eq!(shell.error(), BRepLibShellError::ShellDone);
    assert_eq!(count(&*shell.shell()?, ShapeEnum::Face)?, 1);
    let empty = BRepLibMakeShell::new();
    assert_eq!(empty.error(), BRepLibShellError::EmptyShell);
    assert!(!empty.is_done());
    Ok(())
}
