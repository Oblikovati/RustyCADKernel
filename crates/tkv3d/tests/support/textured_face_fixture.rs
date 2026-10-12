//! Native geometric face cases preserve real BRep topology and shared mesh handles.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{self, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::{
    gp::{Axis2, Direction3, Plane3, Point2, Point3, Transform3},
    poly::{Triangle, Triangulation},
    toploc::Location,
};
pub fn textured_face_fixture(scenario: i32) -> Result<Face, Failure> {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder.make_face_with_triangulation(&mut face, &textured_face_mesh(scenario)?)?;
    if scenario & 4 == 0 {
        attach_textured_face_surface(&face)?;
    }
    if scenario & 2 != 0 {
        face.set_orientation(Orientation::Reversed);
    }
    if scenario & 8 != 0 {
        return mirrored_textured_face(&face);
    }
    Ok(face)
}
fn textured_face_mesh(scenario: i32) -> Result<Handle<Triangulation>, Failure> {
    let mesh = Handle::new(Triangulation::with_sizes(4, 3, true, scenario & 1 != 0)?);
    for (offset, [x, y]) in [[-2., -3.], [4., -3.], [4., 5.], [-2., 5.]]
        .into_iter()
        .enumerate()
    {
        let index = offset as i32 + 1;
        mesh.set_node(index, Point3::new(x, y, 0.))?;
        mesh.set_uv_node(index, Point2::new(x, y))?;
        if scenario & 1 != 0 {
            mesh.set_normal(index, Direction3::Z)?;
        }
    }
    for (offset, [a, b, c]) in [[1, 2, 3], [1, 3, 4], [1, 2, 2]].into_iter().enumerate() {
        mesh.set_triangle(offset as i32 + 1, Triangle::new(a, b, c))?;
    }
    Ok(mesh)
}
fn attach_textured_face_surface(face: &Face) -> Result<(), Failure> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(Plane3::new(
        Point3::default(),
        Direction3::Z,
    )?))
    .cast();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomRectangularTrimmedSurface::new(
        &plane,
        (-2., 4.),
        (-3., 5.),
        (true, true),
    )?)
    .cast();
    BRepBuilder::default().update_face(face, &surface, &Location::default(), 1e-7)
}
fn mirrored_textured_face(face: &Face) -> Result<Face, Failure> {
    let mut transform = Transform3::default();
    transform.set_mirror_plane(Axis2::new(Point3::new(1., 2., 3.), Direction3::Z)?);
    topods::face(&face.located(&Location::new(transform), false)?)
}
