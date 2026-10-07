//! BRepFill_OffsetWire, BRepFill_TrimEdgeTool and BRepFill_OffsetAncestors: the offset of
//! the wires of a planar face.
use rustycad_tkbool::brep_fill::{
    BRepFillOffsetAncestors, BRepFillOffsetWire, BRepFillTrimEdgeTool,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dGeometry, Geom2dPoint};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Axis2, Point2, Point3, Vector2};
use rustycad_tktopalgo::bisector::Bisec;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakeWire};

/// The face of the rectangle [0, 4] x [0, 3] on z = 0.
fn rectangle() -> Result<Face, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(0., 0.), (4., 0.), (4., 3.), (0., 3.)] {
        polygon.add_point(Point3::new(x, y, 0.))?;
    }
    polygon.close()?;
    let wire: Wire = polygon.wire()?;
    BRepLibMakeFace::from_wire(&wire, true)?.face()
}

/// The disc of radius 2 around the origin on z = 0.
fn disc() -> Result<Face, Failure> {
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(Axis2::default(), 2.0)?).cast();
    let edge = BRepLibMakeEdge::from_curve(&circle)?.edge()?;
    let wire = BRepLibMakeWire::from_edge(&edge)?.wire()?;
    BRepLibMakeFace::from_wire(&wire, true)?.face()
}

/// The sub-shapes of a kind.
fn all(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut out = Vec::new();
    let mut ex = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while ex.more() {
        out.push(ex.current().clone());
        ex.next()?;
    }
    Ok(out)
}

#[test]
fn a_rectangle_offset_by_a_negative_distance_keeps_its_corner_arcs() -> Result<(), Failure> {
    // Checked against OCCT: the face of BRepBuilderAPI_MakePolygon gives four wires of
    // one corner arc each at -0.3.
    let mut ow = BRepFillOffsetWire::new(&rectangle()?, JoinType::Arc, false)?;
    ow.perform(-0.3, 0.)?;
    assert!(ow.is_done());
    assert_eq!(ow.join_type(), JoinType::Arc);
    assert_eq!(all(ow.shape(), ShapeEnum::Wire)?.len(), 4);
    assert_eq!(all(ow.shape(), ShapeEnum::Edge)?.len(), 4);
    Ok(())
}

#[test]
fn a_rectangle_offset_inward_keeps_four_sides_with_their_ancestors() -> Result<(), Failure> {
    let mut ow = BRepFillOffsetWire::new(&rectangle()?, JoinType::Arc, false)?;
    ow.perform(0.5, 0.)?;
    assert!(ow.is_done());
    let edges = all(ow.shape(), ShapeEnum::Edge)?;
    assert_eq!(edges.len(), 4);
    let ancestors = BRepFillOffsetAncestors::with_offset(&mut ow)?;
    assert!(ancestors.is_done());
    for e in &edges {
        let e = topods::edge(e)?;
        assert!(ancestors.has_ancestor(&e));
        assert_eq!(ancestors.ancestor(&e)?.shape_type()?, ShapeEnum::Edge);
    }
    Ok(())
}

#[test]
fn a_null_offset_copies_the_spine_moved_by_the_altitude() -> Result<(), Failure> {
    let mut ow = BRepFillOffsetWire::new(&rectangle()?, JoinType::Arc, false)?;
    ow.perform(0., 1.)?;
    assert!(ow.is_done());
    for v in all(ow.shape(), ShapeEnum::Vertex)? {
        assert_eq!(BRepTool::pnt(&topods::vertex(&v)?)?.z(), 1.);
    }
    Ok(())
}

#[test]
fn a_disc_is_offset_as_one_circle() -> Result<(), Failure> {
    let mut ow = BRepFillOffsetWire::new(&disc()?, JoinType::Arc, false)?;
    ow.perform(0.5, 0.)?;
    assert!(ow.is_done());
    let edges = all(ow.shape(), ShapeEnum::Edge)?;
    assert_eq!(edges.len(), 1);
    let (curve, ..) = BRepTool::curve(&topods::edge(&edges[0])?)?;
    let circle: Handle<GeomCircle> = curve.cast();
    let radius = circle.get().map(|c| c.radius()).unwrap_or_default();
    assert!((radius - 1.5).abs() < 1e-12 || (radius - 2.5).abs() < 1e-12);
    Ok(())
}

#[test]
fn the_ancestors_need_a_perform() -> Result<(), Failure> {
    let ancestors = BRepFillOffsetAncestors::new();
    assert!(!ancestors.is_done());
    let edge = topods::edge(&all(rectangle()?.as_ref(), ShapeEnum::Edge)?[0])?;
    assert!(!ancestors.has_ancestor(&edge));
    let error = ancestors
        .ancestor(&edge)
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("StdFail_NotDone"));
    Ok(())
}

#[test]
fn a_trim_tool_between_two_points_measures_from_the_first() -> Result<(), Failure> {
    let p1: Handle<dyn Geom2dGeometry> = Handle::new(Geom2dCartesianPoint::new(0., 0.)).cast();
    let p2: Handle<dyn Geom2dGeometry> = Handle::new(Geom2dCartesianPoint::new(2., 0.)).cast();
    // A null bisector is refused by its adaptor, as natively.
    assert!(BRepFillTrimEdgeTool::new(&Bisec::new(), &p1, &p2, 1.0).is_err());
    let mut bisec = Bisec::new();
    let (q1, q2): (Handle<dyn Geom2dPoint>, Handle<dyn Geom2dPoint>) = (p1.cast(), p2.cast());
    bisec.perform_points(
        &q1,
        &q2,
        Point2::new(1., 0.),
        Vector2::new(0., -1.),
        Vector2::new(0., 1.),
        1.0,
        0.0,
        true,
    )?;
    let tool = BRepFillTrimEdgeTool::new(&bisec, &p1, &p2, 1.0)?;
    // Inside means closer than |Offset| - Precision::Confusion() to the first point.
    assert!(tool.is_inside(Point2::new(0.5, 0.))?);
    assert!(!tool.is_inside(Point2::new(1.0, 0.))?);
    Ok(())
}
