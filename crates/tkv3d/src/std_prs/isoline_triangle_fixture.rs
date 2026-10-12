//! Actual shared node/UV/triangle storage, varying insertion order and nonlinear weighting geometry.
use super::isoline_face_fixture::isoline_fixture_surface;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Handle, SharedState},
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::{
    gp::{Point2, Point3, Transform3, Vector3},
    poly::{Triangle, Triangulation},
    toploc::Location,
};
pub(super) fn isoline_triangle_surface(kind: i32) -> Handle<dyn GeomSurface> {
    if kind == 0 {
        return Handle::default();
    }
    isoline_fixture_surface(if kind == 1 { 0 } else { 4 })
}
pub(super) fn isoline_triangle_grid(
    surface: &Handle<dyn GeomSurface>,
    order: i32,
) -> Handle<Triangulation> {
    let mesh = Handle::new(Triangulation::with_sizes(9, 8, true, false).unwrap());
    initialize_isoline_grid_nodes(&mesh, surface);
    let mut triangles = isoline_grid_triangles();
    if order == 1 {
        triangles.reverse();
    }
    if order == 2 {
        triangles.rotate_left(3);
    }
    for (index, triangle) in triangles.into_iter().enumerate() {
        mesh.set_triangle(index as i32 + 1, triangle).unwrap();
    }
    mesh
}
fn initialize_isoline_grid_nodes(mesh: &Triangulation, surface: &Handle<dyn GeomSurface>) {
    for j in 0..3 {
        for i in 0..3 {
            let uv = Point2::new(f64::from(i) * 0.5, f64::from(j) * 0.5);
            let xyz = if surface.is_null() {
                Point3::new(uv.x(), uv.y(), 0.)
            } else {
                surface.value(uv.x(), uv.y()).unwrap()
            };
            let xyz = Point3::new(xyz.x(), xyz.y(), xyz.z() + 0.1 * f64::from(i + j));
            mesh.set_node(j * 3 + i + 1, xyz).unwrap();
            mesh.set_uv_node(j * 3 + i + 1, uv).unwrap();
        }
    }
}
fn isoline_grid_triangles() -> Vec<Triangle> {
    let mut triangles = Vec::new();
    for j in 0..2 {
        for i in 0..2 {
            let a = j * 3 + i + 1;
            triangles.push(Triangle::new(a, a + 1, a + 4));
            triangles.push(Triangle::new(a, a + 4, a + 3));
        }
    }
    triangles
}
pub(super) fn isoline_triangle_location(transformed: bool) -> SharedState<Location> {
    if !transformed {
        return SharedState::new(Location::default());
    }
    let mut trsf = Transform3::default();
    trsf.set_translation(Vector3::new(7., -3., 2.));
    SharedState::new(Location::new(trsf))
}
pub(super) fn isoline_triangle_parameters() -> [SharedState<Sequence<f64>>; 2] {
    let parameters: [SharedState<Sequence<f64>>; 2] =
        std::array::from_fn(|_| SharedState::new(Sequence::default()));
    for value in [0.25, 0.5, 0.75] {
        parameters[0].write().append(value);
    }
    for value in [0.2, 0.5, 0.8] {
        parameters[1].write().append(value);
    }
    parameters
}
