//! Real topology recipes shared with TexturedShape wireframe proofs, matching pinned native face builders.
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg3d::geom::*;
use rustycad_tkgeombase::geom_convert;
use rustycad_tkmath::{
    gp::{Axis2, Axis3, Circle3, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
};
pub(super) fn isoline_fixture_surface(kind: i32) -> Handle<dyn GeomSurface> {
    let frame = Axis3::default();
    match kind {
        1 | 5 => Handle::new(GeomCylindricalSurface::new(frame, 3.).unwrap()).cast(),
        2 => Handle::new(GeomSphericalSurface::new(frame, 3.).unwrap()).cast(),
        3 => Handle::new(GeomToroidalSurface::new(frame, 5., 2.).unwrap()).cast(),
        4 => Handle::new(GeomConicalSurface::new(frame, 0.4, 3.).unwrap()).cast(),
        6 => isoline_bezier_fixture(),
        7 => geom_convert::surface_to_bspline_surface(&isoline_bezier_fixture())
            .unwrap()
            .cast(),
        _ => Handle::new(GeomPlane::new(frame)).cast(),
    }
}
fn isoline_bezier_fixture() -> Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            let z = f64::from(if i == 2 { 2 } else { 0 }) + f64::from(if j == 2 { 1 } else { 0 });
            poles
                .set_value(
                    i,
                    j,
                    Point3::new(f64::from((i - 1) * 2), f64::from((j - 1) * 2), z),
                )
                .unwrap();
        }
    }
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
pub(super) fn isoline_fixture_face(kind: i32) -> Face {
    if kind == 9 {
        return isoline_annulus_fixture();
    }
    let bounds = isoline_fixture_bounds(kind);
    let mut surface = isoline_fixture_surface(kind);
    surface = trimmed_isoline_fixture_surface(surface, kind, bounds);
    let mut face = BRepBuilderAPIMakeFace::from_surface_bounds(&surface, bounds, 1e-7)
        .unwrap()
        .face()
        .unwrap();
    if kind == 5 {
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(7., -3., 2.));
        face.set_location(&Location::new(transform), false).unwrap();
    }
    face
}
fn isoline_fixture_bounds(kind: i32) -> (f64, f64, f64, f64) {
    match kind {
        1 | 4 | 5 => (0., 1.6, -2., 2.),
        2 | 3 => (0., 1.6, -1., 1.),
        6 | 7 => (0., 1., 0., 1.),
        _ => (-2., 3., -1., 4.),
    }
}
fn isoline_annulus_fixture() -> Face {
    let outer = isoline_circle_wire(4.);
    let mut inner = isoline_circle_wire(1.);
    inner.reverse();
    let mut builder = BRepBuilderAPIMakeFace::from_wire(&outer, true).unwrap();
    builder.add(&inner).unwrap();
    builder.face().unwrap()
}
fn isoline_circle_wire(radius: f64) -> Wire {
    let circle = Circle3::new(Axis2::default(), radius).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
        .unwrap()
        .edge()
        .unwrap();
    BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap()
}

fn trimmed_isoline_fixture_surface(
    surface: Handle<dyn GeomSurface>,
    kind: i32,
    bounds: (f64, f64, f64, f64),
) -> Handle<dyn GeomSurface> {
    if kind != 8 {
        return surface;
    }
    let trimmed = GeomRectangularTrimmedSurface::new(
        &surface,
        (bounds.0, bounds.1),
        (bounds.2, bounds.3),
        (true, true),
    )
    .unwrap();
    Handle::new(trimmed).cast()
}
