//! BRepAlgo_NormalProjection and BRepProj_Projection: edges and wires projected on
//! faces.
use rustycad_tkbool::brep_algo::BRepAlgoNormalProjection;
use rustycad_tkbool::brep_proj::BRepProjProjection;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace};

/// The polyline wire through the points (closed when asked).
fn polyline(points: &[(f64, f64, f64)], closed: bool) -> Result<Wire, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        polygon.add_point(Point3::new(x, y, z))?;
    }
    if closed {
        polygon.close()?;
    }
    polygon.wire()
}

/// The square face [0, 4]^2 on z = 0.
fn square() -> Result<Shape, Failure> {
    let w = polyline(
        &[(0., 0., 0.), (4., 0., 0.), (4., 4., 0.), (0., 4., 0.)],
        true,
    )?;
    Ok(BRepLibMakeFace::from_wire(&w, true)?.face()?.into_shape())
}

/// The edges of a shape.
fn edges(s: &Shape) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)?.collect())
}

#[test]
fn a_crossing_segment_is_projected_inside_the_face() -> Result<(), Failure> {
    let face = square()?;
    let segment =
        BRepLibMakeEdge::from_points(Point3::new(-1., 2., 1.), Point3::new(5., 2., 1.))?.edge()?;
    let mut np = BRepAlgoNormalProjection::with_shape(&face);
    np.add(segment.as_ref())?;
    np.build()?;
    assert!(np.is_done());
    let projected = edges(np.projection())?;
    assert_eq!(projected.len(), 1);
    let e = topods::edge(&projected[0])?;
    assert!(np.ancestor(&e)?.is_same(segment.as_ref()));
    assert!(np.couple(&e)?.is_same(&face));
    let mut wires: List<Shape> = List::default();
    assert!(np.build_wire(&mut wires)?);
    Ok(())
}

#[test]
fn a_square_wire_projects_along_minus_z() -> Result<(), Failure> {
    let w = polyline(
        &[(1., 1., 3.), (3., 1., 3.), (3., 3., 3.), (1., 3., 3.)],
        true,
    )?;
    let mut p = BRepProjProjection::along(w.as_ref(), &square()?, Direction3::new(0., 0., -1.)?)?;
    assert!(p.is_done());
    p.init();
    assert!(p.more());
    assert_eq!(edges(p.current()?.as_ref())?.len(), 4);
    Ok(())
}

#[test]
fn a_face_is_not_a_projected_shape() -> Result<(), Failure> {
    let face = square()?;
    let error = BRepProjProjection::along(&face, &face, Direction3::Z)
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("Standard_ConstructionError"));
    Ok(())
}
