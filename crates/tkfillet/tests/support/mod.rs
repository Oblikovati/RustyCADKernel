//! Exact native classifications for face and edge orientations and zero midpoint tangent.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Wire},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};

use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCurve, GeomCylindricalSurface, GeomPlane, GeomSurface,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
pub fn fixture_edge(kind: i32) -> Edge {
    if kind == 0 {
        return BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(1., 0., 0.))
            .unwrap()
            .edge()
            .unwrap();
    }
    let mut poles = Array1::with_bounds(1, 3).unwrap();
    poles[1] = Point3::default();
    poles[2] = Point3::new(1., 0., 0.);
    poles[3] = Point3::default();
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
}
pub fn fixture_face(edge: &Edge, normal: (f64, f64, f64), reversed: bool, missing: bool) -> Face {
    let frame = Axis3::new(
        Point3::default(),
        Direction3::new(normal.0, normal.1, normal.2).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    let surface: Handle<dyn GeomSurface> = if missing {
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.).unwrap()).cast()
    } else {
        Handle::new(GeomPlane::new(frame)).cast()
    };
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, edge).unwrap();
    builder.add(&mut face, &wire).unwrap();
    if reversed {
        face.reverse();
    }
    face
}
