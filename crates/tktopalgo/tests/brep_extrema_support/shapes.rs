//! The shapes of the BRepExtrema shape proximity behaviour tests: arcs, circles and
//! Bezier edges, plane, cylinder and sphere patches, a polygon face and a split sphere,
//! built with BRepBuilderAPI as the upstream suite builds them.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCircle, GeomCurve, GeomCylindricalSurface, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
};
use std::f64::consts::PI;

pub fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

pub fn make_circular_arc(
    center: Point3,
    normal: Direction3,
    radius: f64,
    first: f64,
    last: f64,
) -> Shape {
    let circle = Circle3::new(Axis2::new(center, normal).unwrap(), radius).unwrap();
    edge_shape(
        BRepBuilderAPIMakeEdge::from_circ_params(circle, first, last)
            .unwrap()
            .edge()
            .unwrap(),
    )
}

pub fn circle_curve(center: Point3, normal: Direction3, radius: f64) -> Handle<dyn GeomCurve> {
    Handle::new(GeomCircle::new(Axis2::new(center, normal).unwrap(), radius).unwrap()).cast()
}

pub fn make_trimmed_circular_arc(
    center: Point3,
    normal: Direction3,
    radius: f64,
    first: f64,
    last: f64,
) -> Shape {
    let trimmed: Handle<dyn GeomCurve> = Handle::new(
        GeomTrimmedCurve::new(
            &circle_curve(center, normal, radius),
            first,
            last,
            true,
            true,
        )
        .unwrap(),
    )
    .cast();
    edge_shape(
        BRepBuilderAPIMakeEdge::from_curve(&trimmed)
            .unwrap()
            .edge()
            .unwrap(),
    )
}

pub fn make_full_circular_edge(center: Point3, normal: Direction3, radius: f64) -> Shape {
    edge_shape(
        BRepBuilderAPIMakeEdge::from_curve(&circle_curve(center, normal, radius))
            .unwrap()
            .edge()
            .unwrap(),
    )
}

pub fn edge_shape(edge: Edge) -> Shape {
    edge.into_shape()
}

pub fn face_shape(face: Face) -> Shape {
    face.into_shape()
}

pub fn make_plane_patch(
    location: Point3,
    normal: Direction3,
    bounds: (f64, f64, f64, f64),
) -> Shape {
    let plane = Plane3::new(location, normal).unwrap();
    face_shape(
        BRepBuilderAPIMakeFace::from_plane_bounds(plane, bounds)
            .unwrap()
            .face()
            .unwrap(),
    )
}

pub fn trimmed_face(surface: Handle<dyn GeomSurface>, u: (f64, f64), v: (f64, f64)) -> Shape {
    let trimmed: Handle<dyn GeomSurface> =
        Handle::new(GeomRectangularTrimmedSurface::new(&surface, u, v, (true, true)).unwrap())
            .cast();
    face_shape(
        BRepBuilderAPIMakeFace::from_surface(&trimmed, precision::confusion())
            .unwrap()
            .face()
            .unwrap(),
    )
}

pub fn make_cylinder_patch(
    location: Point3,
    direction: Direction3,
    radius: f64,
    u: (f64, f64),
    v: (f64, f64),
) -> Shape {
    let frame = Axis3::from_normal(location, direction).unwrap();
    trimmed_face(
        Handle::new(GeomCylindricalSurface::new(frame, radius).unwrap()).cast(),
        u,
        v,
    )
}

pub fn sphere_surface(center: Point3) -> Handle<dyn GeomSurface> {
    let frame = Axis3::from_normal(center, dir(0.0, 0.0, 1.0)).unwrap();
    Handle::new(GeomSphericalSurface::new(frame, 1.0).unwrap()).cast()
}

pub fn make_sphere_patch(center: Point3, first_u: f64, last_u: f64) -> Shape {
    trimmed_face(
        sphere_surface(center),
        (first_u, last_u),
        (-0.5 * PI, 0.5 * PI),
    )
}

pub fn make_sphere_face(center: Point3) -> Shape {
    face_shape(
        BRepBuilderAPIMakeFace::from_surface(&sphere_surface(center), precision::confusion())
            .unwrap()
            .face()
            .unwrap(),
    )
}

pub fn make_polygon_face() -> Shape {
    let mut a_polygon = BRepBuilderAPIMakePolygon::new();
    for p in [
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 2.0],
        [1.0, -1.0, 2.0],
    ] {
        a_polygon.add_point(Point3::new(p[0], p[1], p[2])).unwrap();
    }
    a_polygon.close().unwrap();
    face_shape(
        BRepBuilderAPIMakeFace::from_wire(&a_polygon.wire().unwrap(), true)
            .unwrap()
            .face()
            .unwrap(),
    )
}

pub fn make_split_sphere(center: Point3) -> Shape {
    let a_builder = BRepBuilder::default();
    let mut a_compound = Compound::default();
    a_builder.make_compound(&mut a_compound);
    a_builder
        .add(&mut a_compound, &make_sphere_patch(center, 0.0, 0.5 * PI))
        .unwrap();
    a_builder
        .add(&mut a_compound, &make_sphere_patch(center, 0.5 * PI, PI))
        .unwrap();
    a_compound.into_shape()
}

pub fn make_bezier_edge(poles: &[[f64; 3]]) -> Shape {
    let mut array: Array1<'static, Point3> = Array1::with_bounds(1, poles.len() as i32).unwrap();
    for (i, p) in poles.iter().enumerate() {
        array[i as i32 + 1] = Point3::new(p[0], p[1], p[2]);
    }
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&array).unwrap()).cast();
    edge_shape(
        BRepBuilderAPIMakeEdge::from_curve(&curve)
            .unwrap()
            .edge()
            .unwrap(),
    )
}
