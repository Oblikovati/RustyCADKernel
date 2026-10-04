use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Edge, Face, Shape, Wire},
};
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
pub fn pair(slope: f64) -> Shape {
    let builder = BRepBuilder::default();
    let mut pair = Compound::default();
    builder.make_compound(&mut pair);
    let edge = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(1., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    for side in 0..2 {
        builder
            .add(&mut pair, &spline_face(&edge, f64::from(side) * slope))
            .unwrap();
    }
    pair.into_shape()
}
fn spline_face(edge: &Edge, slope: f64) -> Face {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for u in 1..=2 {
        for v in 1..=2 {
            *poles.change_value(u, v).unwrap() =
                Point3::new(f64::from(u - 1), f64::from(v - 1), slope * f64::from(u - 1));
        }
    }
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    builder
        .update_edge_pcurve_on_face(edge, &curve, &face, 1e-7)
        .unwrap();
    builder.range_on_face(edge, &face, 0., 1.).unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, edge).unwrap();
    builder.add(&mut face, &wire).unwrap();
    face
}
