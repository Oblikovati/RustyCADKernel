//! BRepFill_Draft: the draft surface of a wire, face or shell swept along a direction.
use rustycad_tkbool::brep_fill::{BRepFillDraft, TransitionStyle};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeVertex};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The closed square of side 2 around the Z axis.
fn square() -> Result<Wire, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] {
        polygon.add_point(Point3::new(x, y, 0.))?;
    }
    polygon.close()?;
    polygon.wire()
}

/// The count of the sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

#[test]
fn a_wire_drafted_to_a_length_gives_one_face_per_edge() -> Result<(), Failure> {
    let wire = square()?;
    let mut draft = BRepFillDraft::new(wire.as_ref(), Direction3::Z, 0.1)?;
    draft.perform_length(3.0)?;
    assert!(draft.is_done());
    assert_eq!(count(&draft.shape(), ShapeEnum::Face)?, 4);
    assert_eq!(count(draft.shell().as_ref(), ShapeEnum::Face)?, 4);
    for e in Explorer::new(wire.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
        assert_eq!(draft.generated(&e)?.len(), 1);
    }
    // A vertex is refused by the edge cast, as natively.
    let first_vertex = Explorer::new(wire.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?
        .current()
        .clone();
    let error = draft
        .generated(&first_vertex)
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error, Some("Standard_TypeMismatch".to_string()));
    Ok(())
}

#[test]
fn a_face_drafted_to_a_plane_is_sewn_to_the_face() -> Result<(), Failure> {
    let face = BRepLibMakeFace::from_wire(&square()?, true)?.face()?;
    let mut draft = BRepFillDraft::new(face.as_ref(), Direction3::Z, 0.1)?;
    draft.set_options(TransitionStyle::Round, 0.01, 3.0);
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(
        Point3::new(0., 0., 2.),
        Direction3::Z,
    )?)
    .cast();
    draft.perform_surface(&plane, true)?;
    assert!(draft.is_done());
    // The input face is sewn to the draft surface.
    assert!(count(&draft.shape(), ShapeEnum::Face)? >= 5);
    Ok(())
}

#[test]
fn a_vertex_cannot_be_drafted() -> Result<(), Failure> {
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::default())?.vertex()?;
    let error = BRepFillDraft::new(vertex.as_ref(), Direction3::Z, 0.1).err();
    assert_eq!(
        error.map(|e| e.exception_type().to_string()),
        Some("Standard_ConstructionError".to_string())
    );
    Ok(())
}
