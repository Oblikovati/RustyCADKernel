//! Multi-wire faces of the `sh_fix_shape` probe, the Rust twin of
//! tools/oracle/tkshhealing_probe_fix_shape_wires.cxx (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, flag};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3};
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_fix::ShapeFixIntersectionTool;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakePolygon};

// Kind 0 a closed polygon `m (point)*m`, kind 1 a full circle `center r`.
fn multi_wire(r: &mut Reader<'_>) -> Result<Wire, Failure> {
    if r.integer() == 0 {
        let mut polygon = BRepBuilderAPIMakePolygon::new();
        for _ in 0..r.integer() {
            polygon.add_point(r.pnt())?;
        }
        polygon.close()?;
        return polygon.wire();
    }
    let center = r.pnt();
    let circle = GeomCircle::new(Axis2::new(center, Direction3::new(0., 0., 1.)?)?, r.real())?;
    let curve: Handle<dyn GeomCurve> = Handle::new(circle).cast();
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let edge = BRepBuilderAPIMakeEdge::from_curve(&curve)?.edge()?;
    builder.add(&mut wire, &edge.into_shape())?;
    Ok(wire)
}

/// Family 6: a face on the XY plane with `k` wires.
pub(super) fn multi_wire_face(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    builder.make_face_on_surface(&mut face, &plane, 1e-7)?;
    for _ in 0..r.integer() {
        builder.add(&mut face, &multi_wire(r)?.into_shape())?;
    }
    Ok(face.into_shape())
}

/// Op 33: FixIntersectingWires on each face with a fresh context.
pub(super) fn intersecting_wires(s: &Shape, prec: f64, maxtol: f64) -> Result<(), Failure> {
    for input in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        attempt(|| fix_face_wires(&input, prec, maxtol));
    }
    Ok(())
}

fn fix_face_wires(input: &Shape, prec: f64, maxtol: f64) -> Result<(), Failure> {
    let ctx = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::with_max_tolerance(&ctx, prec, maxtol);
    let mut face = topods::face(input)?;
    flag(tool.fix_intersecting_wires(&mut face)?);
    shape(&face.into_shape())?;
    let mut children = ShapeIterator::new(input, false, true);
    while children.more() {
        shape(&ctx.apply(children.value()?, ShapeEnum::Shape)?)?;
        children.next();
    }
    shape(&ctx.apply(input, ShapeEnum::Shape)?)
}
