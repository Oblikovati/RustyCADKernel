//! BRepFill_Filling: the face of a plate surface through boundary edges, free constraint
//! faces and points.
use rustycad_tkbool::brep_fill::{BRepFillEdgeFaceAndOrder, BRepFillFaceAndOrder, BRepFillFilling};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakeVertex};

/// The corners of a quadrilateral twisted out of its plane.
const TWISTED: [(f64, f64, f64); 4] = [(0., 0., 0.), (2., 0., 0.5), (2., 2., 0.), (0., 2., 0.5)];

/// The four segments of the twisted quadrilateral, sharing their vertices.
fn twisted_edges() -> Result<Vec<Edge>, Failure> {
    let vertices: Vec<Vertex> = TWISTED
        .iter()
        .map(|&(x, y, z)| BRepLibMakeVertex::new(Point3::new(x, y, z))?.vertex())
        .collect::<Result<_, _>>()?;
    (0..4)
        .map(|k| BRepLibMakeEdge::from_vertices(&vertices[k], &vertices[(k + 1) % 4])?.edge())
        .collect()
}

/// The square face [0, 2] x [0, 2] on z = 0.
fn square_face() -> Result<Face, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(0., 0.), (2., 0.), (2., 2.), (0., 2.)] {
        polygon.add_point(Point3::new(x, y, 0.))?;
    }
    polygon.close()?;
    let wire: Wire = polygon.wire()?;
    BRepLibMakeFace::from_wire(&wire, true)?.face()
}

/// The edges of a shape.
fn edges_of(shape: &Shape) -> Result<Vec<Edge>, Failure> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|e| topods::edge(&e))
        .collect()
}

#[test]
fn a_twisted_boundary_is_filled_in_g0() -> Result<(), Failure> {
    let mut filling = BRepFillFilling::default();
    let edges = twisted_edges()?;
    for (k, e) in edges.iter().enumerate() {
        assert_eq!(filling.add_edge(e, Continuity::C0, true), k as i32 + 1);
    }
    filling.build()?;
    assert!(filling.is_done());
    let face = filling.face();
    assert_eq!(edges_of(face.as_ref())?.len(), 4);
    assert!(filling.g0_error()? < 1e-3);
    // Each boundary edge has its new edge, on the plate.
    for e in &edges {
        let generated = filling.generated(e);
        assert_eq!(generated.len(), 1);
        let new_edge = topods::edge(
            generated
                .first()
                .ok_or_else(|| Failure::new("no new edge"))?,
        )?;
        assert!(!BRepTool::curve_on_face(&new_edge, &face)?.pcurve.is_null());
    }
    Ok(())
}

#[test]
fn the_constraint_indices_follow_the_native_counts() -> Result<(), Failure> {
    let mut filling = BRepFillFilling::new(3, 15, 2, false, 1e-5, 1e-4, 0.01, 0.1, 8, 9);
    let edges = twisted_edges()?;
    assert_eq!(filling.add_edge(&edges[0], Continuity::C0, true), 1);
    // A free constraint counts after the boundary, an edge constraint after both.
    let face = square_face()?;
    assert_eq!(filling.add_face(&face, Continuity::G1), 2);
    assert_eq!(filling.add_edge(&edges[1], Continuity::C0, false), 3);
    assert_eq!(filling.add_point(Point3::new(1., 1., 0.5))?, 4);
    assert_eq!(
        filling.add_point_on_face(0.5, 0.5, &face, Continuity::G1)?,
        5
    );
    Ok(())
}

#[test]
fn an_empty_boundary_is_not_done() -> Result<(), Failure> {
    let mut filling = BRepFillFilling::default();
    filling.build()?;
    assert!(!filling.is_done());
    assert!(filling.face().is_null());
    Ok(())
}

#[test]
fn the_errors_need_a_build() {
    let filling = BRepFillFilling::default();
    let error = filling
        .g0_error()
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("Standard_NullObject"));
}

#[test]
fn a_boundary_on_its_face_is_filled_in_g1() -> Result<(), Failure> {
    let face = square_face()?;
    let mut filling = BRepFillFilling::default();
    for e in edges_of(face.as_ref())? {
        filling.add_edge_on_face(&e, &face, Continuity::G1, true);
    }
    filling.build()?;
    assert!(filling.is_done());
    assert!(filling.g0_error()? < 1e-6);
    Ok(())
}

#[test]
fn the_order_g2_is_refused_on_a_face_as_natively() -> Result<(), Failure> {
    // GeomAbs_G2 is 3 as an integer order: beyond the G0..G2 orders 0..=2.
    let face = square_face()?;
    let mut filling = BRepFillFilling::default();
    for e in edges_of(face.as_ref())? {
        filling.add_edge_on_face(&e, &face, Continuity::G2, true);
    }
    let error = filling
        .build()
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("Standard_Failure"));
    Ok(())
}

#[test]
fn a_free_face_closes_the_hole_of_an_open_boundary() -> Result<(), Failure> {
    let face = square_face()?;
    let mut filling = BRepFillFilling::default();
    let edges = edges_of(face.as_ref())?;
    for e in &edges[..3] {
        filling.add_edge(e, Continuity::C0, true);
    }
    assert_eq!(filling.add_face(&face, Continuity::C0), 4);
    filling.build()?;
    assert!(filling.is_done());
    assert_eq!(edges_of(filling.face().as_ref())?.len(), 4);
    Ok(())
}

#[test]
fn the_records_keep_their_edge_face_and_order() {
    let record = BRepFillEdgeFaceAndOrder::new();
    assert!(format!("{record:?}").contains("C0"));
    let free = BRepFillFaceAndOrder::with_face(&Face::default(), Continuity::G1);
    assert!(format!("{free:?}").contains("G1"));
}

#[test]
fn a_curve_constraint_on_a_face_keeps_its_order_and_points() -> Result<(), Failure> {
    use rustycad_tkbool::brep_fill::BRepFillCurveConstraint;
    use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
    use rustycad_tkg3d::adaptor3d::{Adaptor3dCurveOnSurface, Adaptor3dSurface};
    let face = square_face()?;
    let edge = edges_of(face.as_ref())?.remove(0);
    let surf: Handle<dyn Adaptor3dSurface> =
        Handle::new(BRepAdaptorSurface::with_face(&face, true)?).cast();
    let curve2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(BRepAdaptorCurve2d::with_edge_on_face(&edge, &face)?).cast();
    let on_surface = Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve2d, &surf)?);
    let constraint = BRepFillCurveConstraint::on_surface(&on_surface, 1, 12, [1e-4, 0.01, 0.1])?;
    let c = constraint
        .get()
        .ok_or_else(|| Failure::new("null constraint"))?;
    assert_eq!((c.order(), c.nb_points()), (1, 12));
    // An order beyond G2 is refused before the constraint is built.
    assert!(BRepFillCurveConstraint::on_surface(&on_surface, 3, 12, [1e-4, 0.01, 0.1]).is_err());
    Ok(())
}
