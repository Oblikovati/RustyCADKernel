//! Direct analytic/finite-domain face fixtures for native range calculations.
use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface, GeomSurface,
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkmesh::{brep_mesh_data::DiscreteFace, imesh_data::FaceHandle};
pub fn range_face(kind: i32, tolerance: f64) -> FaceHandle {
    let surface = range_surface(kind);
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, tolerance)
        .unwrap();
    Handle::new(DiscreteFace::new(&face, &Handle::default()).unwrap()).cast()
}
fn range_surface(kind: i32) -> Handle<dyn GeomSurface> {
    if kind == 0 {
        return Handle::new(GeomConicalSurface::new(Axis3::default(), 0.3, 2.0).unwrap()).cast();
    }
    if kind == 1 {
        return Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap()).cast();
    }
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for row in 1..=2 {
        for col in 1..=2 {
            poles[(row, col)] =
                Point3::new(f64::from(row - 1) * 2.0, f64::from(col - 1) * 3.0, 0.0);
        }
    }
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
pub fn range_bounds(kind: i32) -> [f64; 4] {
    [
        [0.0, 1.0, 0.0, 1.0],
        [-1.0, 8.0, -2.0, 3.0],
        [0.2, 0.7, 0.1, 0.8],
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, -9.0, -7.0],
        [0.0, 0.001, 0.0, 0.002],
    ][kind as usize]
}
