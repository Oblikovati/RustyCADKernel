//! Curve discovery and split-state behavior over real builder-created geometry.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::{
    gp::{Axis1, Axis3, Point2, Point3, dx},
    toploc::Location,
};
use rustycad_tkshhealing::shape_build::ShapeBuildEdge;
pub fn spatial_line() -> Handle<dyn GeomCurve> {
    let axis = Axis1::new(Point3::default(), dx());
    let line = GeomLine::new(axis);
    Handle::new(line).cast()
}
pub fn reparameterized_pcurve() -> Handle<dyn Geom2dCurve> {
    let points = [Point2::new(2., 0.), Point2::new(5., 0.)];
    let poles = Array1::from_shared(&points, 1);
    let knots = Array1::from_shared(&[10., 16.], 1);
    let mults = Array1::from_shared(&[2, 2], 1);
    Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false).unwrap()).cast()
}
pub fn edge_with_parameterizations() -> (Edge, Face) {
    let mut edge = Edge::default();
    ShapeBuildEdge
        .make_edge_on_curve_range(&mut edge, &spatial_line(), &Location::default(), 2., 5.)
        .unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    let builder = BRepBuilder::default();
    builder
        .make_face_on_surface(&mut face, &plane, 1e-7)
        .unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &reparameterized_pcurve(), &face, 0.)
        .unwrap();
    builder.range_on_face(&edge, &face, 10., 16.).unwrap();
    (edge, face)
}
