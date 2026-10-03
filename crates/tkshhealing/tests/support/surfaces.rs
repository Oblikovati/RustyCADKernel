//! Shared support faces keep native geometry identical across analyzer fixtures.
use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::Axis3;

/// An XY-plane face at native confusion tolerance; `planar_face()`.
pub fn planar_face() -> Face {
    face_on_surface(Handle::new(GeomPlane::new(Axis3::default())).cast())
}

/// A Z-axis cylinder of the supplied radius; `cylindrical_face(0.25)`.
pub fn cylindrical_face(radius: f64) -> Face {
    let cylinder = GeomCylindricalSurface::new(Axis3::default(), radius).unwrap();
    face_on_surface(Handle::new(cylinder).cast())
}

fn face_on_surface(surface: Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
