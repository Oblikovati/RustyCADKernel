//! BRepFill_Pipe: the pipe of a vertex, wire or face profile along a spine, and its
//! queries by spine and profile shapes.
use rustycad_tkbool::brep_fill::BRepFillPipe;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Solid, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::GeomFillTrihedron;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeVertex};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The polyline through the points (closed on request).
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

/// The square of half side 0.5 around the origin in the XY plane.
fn square() -> Result<Wire, Failure> {
    polyline(
        &[
            (-0.5, -0.5, 0.),
            (0.5, -0.5, 0.),
            (0.5, 0.5, 0.),
            (-0.5, 0.5, 0.),
        ],
        true,
    )
}

/// The sub-shapes of a kind.
fn explored(s: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, kind, ShapeEnum::Shape)?.collect())
}

#[test]
fn a_square_along_a_segment_gives_a_shell_and_its_queries() -> Result<(), Failure> {
    let spine = polyline(&[(0., 0., 0.), (0., 0., 3.)], false)?;
    let profile = square()?;
    let mut pipe = BRepFillPipe::with_profile(
        &spine,
        profile.as_ref(),
        GeomFillTrihedron::IsCorrectedFrenet,
        false,
        false,
    )?;
    assert_eq!(pipe.shape().shape_type()?, ShapeEnum::Shell);
    assert_eq!(explored(pipe.shape(), ShapeEnum::Face)?.len(), 4);
    let spine_edge = topods::edge(&explored(spine.as_ref(), ShapeEnum::Edge)?[0])?;
    for e in explored(profile.as_ref(), ShapeEnum::Edge)? {
        assert_eq!(pipe.generated(&e).len(), 1);
        assert!(!pipe.face(&spine_edge, &topods::edge(&e)?)?.is_null());
    }
    for v in explored(profile.as_ref(), ShapeEnum::Vertex)? {
        assert!(!pipe.edge(&spine_edge, &topods::vertex(&v)?)?.is_null());
    }
    for v in explored(spine.as_ref(), ShapeEnum::Vertex)? {
        assert_eq!(
            explored(&pipe.section(&topods::vertex(&v)?)?, ShapeEnum::Edge)?.len(),
            4
        );
    }
    let line = pipe.pipe_line(Point3::new(0.2, 0., 0.))?;
    assert_eq!(explored(line.as_ref(), ShapeEnum::Edge)?.len(), 1);
    Ok(())
}

#[test]
fn a_face_profile_gives_a_solid_and_a_vertex_a_wire() -> Result<(), Failure> {
    let spine = polyline(&[(0., 0., 0.), (0., 0., 2.), (1., 0., 3.)], false)?;
    let face = BRepLibMakeFace::from_wire(&square()?, true)?.face()?;
    let pipe = BRepFillPipe::with_profile(
        &spine,
        face.as_ref(),
        GeomFillTrihedron::IsCorrectedFrenet,
        false,
        false,
    )?;
    assert_eq!(pipe.shape().shape_type()?, ShapeEnum::Solid);
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(0.2, 0., 0.))?.vertex()?;
    let pipe = BRepFillPipe::with_profile(
        &spine,
        vertex.as_ref(),
        GeomFillTrihedron::IsFrenet,
        false,
        false,
    )?;
    assert_eq!(pipe.shape().shape_type()?, ShapeEnum::Wire);
    assert_eq!(explored(pipe.shape(), ShapeEnum::Edge)?.len(), 2);
    Ok(())
}

#[test]
fn a_solid_profile_and_foreign_shapes_are_refused() -> Result<(), Failure> {
    let spine = polyline(&[(0., 0., 0.), (0., 0., 3.)], false)?;
    let mut a_solid = Solid::default();
    BRepBuilder::default().make_solid(&mut a_solid);
    let a_box: Shape = a_solid.into_shape();
    let error = BRepFillPipe::with_profile(
        &spine,
        &a_box,
        GeomFillTrihedron::IsCorrectedFrenet,
        false,
        false,
    )
    .err();
    // The placement of the profile refuses the solid first, with the native
    // Standard_TypeMismatch (see the bf_pipe probe).
    assert_eq!(
        error.map(|e| e.exception_type().to_string()),
        Some("Standard_TypeMismatch".to_string())
    );
    let pipe = BRepFillPipe::with_profile(
        &spine,
        square()?.as_ref(),
        GeomFillTrihedron::IsCorrectedFrenet,
        false,
        false,
    )?;
    let other = polyline(&[(5., 5., 5.), (6., 5., 5.)], false)?;
    let other_edge = topods::edge(&explored(other.as_ref(), ShapeEnum::Edge)?[0])?;
    let error = pipe.face(&other_edge, &other_edge).err();
    assert_eq!(
        error.map(|e| e.exception_type().to_string()),
        Some("Standard_DomainError".to_string())
    );
    Ok(())
}
