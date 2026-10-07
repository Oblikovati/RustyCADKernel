//! Behavior tests of ShapeCustom_SweptToElementary (the native comparison is the sc_modif
//! kind 4 requests of tools/tkshhealing_corpus_shapecustom.py).
mod healing_support;

use healing_support::{edges_of, square_face};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewSurface};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomCylindricalSurface, GeomLine, GeomRectangularTrimmedSurface, GeomSphericalSurface,
    GeomSurface, GeomSurfaceOfRevolution,
};
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3};
use rustycad_tkshhealing::shape_custom::ShapeCustomSweptToElementary;

fn face_on(surface: Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn revolved_line() -> Handle<dyn GeomSurface> {
    let z = Direction3::new(0., 0., 1.).unwrap();
    let line = Handle::new(GeomLine::new(Axis1::new(Point3::new(1.5, 0., 0.), z))).cast();
    let axis = Axis1::new(Point3::new(0., 0., 0.), z);
    Handle::new(GeomSurfaceOfRevolution::new(&line, axis).unwrap()).cast()
}

#[test]
fn revolved_parallel_line_becomes_a_cylinder() {
    let face = face_on(revolved_line());
    let mut result = NewSurface::default();
    assert!(
        ShapeCustomSweptToElementary::new()
            .new_surface(&face, &mut result)
            .unwrap()
    );
    let cylinder = result.surface.cast::<GeomCylindricalSurface>();
    assert_eq!(cylinder.get().expect("a cylinder").radius(), 1.5);
}

#[test]
fn trimmed_swept_surface_is_kept_as_natively() {
    let trimmed =
        GeomRectangularTrimmedSurface::new(&revolved_line(), (0., 1.), (0., 1.), (true, true))
            .unwrap();
    let face = face_on(Handle::new(trimmed).cast());
    let mut result = NewSurface::default();
    assert!(
        !ShapeCustomSweptToElementary::new()
            .new_surface(&face, &mut result)
            .unwrap()
    );
}

#[test]
fn copied_edge_onto_a_sphere_from_a_plane_raises_the_native_null_dereference() {
    let plane_face = square_face(0.);
    let edges = edges_of(&plane_face.clone().into_shape());
    let sphere = GeomSphericalSurface::new(Axis3::default(), 2.).unwrap();
    let sphere_face = face_on(Handle::new(sphere).cast());
    let mut curve: Handle<dyn Geom2dCurve> = Handle::default();
    let mut tolerance = -1.;
    let outcome = ShapeCustomSweptToElementary::new().new_curve2d(
        &edges[0],
        &plane_face,
        &edges[1],
        &sphere_face,
        &mut curve,
        &mut tolerance,
    );
    // The planar projection gives a p-curve, then native tests the null swept surface.
    let failure = outcome.unwrap_err();
    assert!(failure.is_kind("Standard_NullObject"));
}
