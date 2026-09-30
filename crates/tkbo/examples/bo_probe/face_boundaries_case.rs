use super::{face_boundaries, fclass_faces, real, topol_surfaces};
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
fn face(kind: i32, radius: f64) -> Result<Face, Failure> {
    if kind >= 23 {
        return special_face(kind, radius);
    }
    if kind < 14 {
        return fclass_faces::face(kind, radius);
    }
    let surface = topol_surfaces::surface(&[f64::from(kind - 14), radius, 0., 1., 0., 1.])
        .cast::<GeomAdaptorSurface>();
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut face, &surface.surface(), 1e-7)?;
    Ok(face)
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let face = face(v[0] as i32, v[1])?;
    let (u1, u2, v1, v2) = Context::default().uv_bounds(&face)?;
    let mut bounds = if v[3] == 0. {
        [u1, u2, v1, v2]
    } else {
        [v[4], v[5], v[6], v[7]]
    };
    face_boundaries::correct_surface(&face, v[2], &mut bounds)?;
    for value in bounds {
        real(value);
    }
    face_boundaries::correct_plane(&mut bounds);
    for value in bounds {
        real(value);
    }
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}

fn special_face(kind: i32, radius: f64) -> Result<Face, Failure> {
    use rustycad_tkmath::gp::{Axis3, Cylinder3, Torus3};
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace as MakeFace;
    if kind == 23 {
        return MakeFace::from_torus(Torus3::new(Axis3::default(), radius * 2., radius)?)?.face();
    }
    if kind == 24 {
        return MakeFace::from_cylinder(Cylinder3::new(Axis3::default(), radius)?)?.face();
    }
    let surface = wrapped_surface(kind, radius)?;
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut face, &surface, 1e-7)?;
    Ok(face)
}
fn wrapped_surface(
    kind: i32,
    radius: f64,
) -> Result<rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>, Failure> {
    let base = topol_surfaces::surface(&[1., radius, 0., 1., 0., 1.])
        .cast::<GeomAdaptorSurface>()
        .surface();
    match kind {
        25 => offset(&trim(&base)?),
        26 => trim(&offset(&base)?),
        27 => offset(&offset(&base)?),
        28 => trim(&trim(&base)?),
        _ => trim(&base),
    }
}
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface};
fn trim(surface: &Handle<dyn GeomSurface>) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(Handle::new(GeomRectangularTrimmedSurface::new(
        surface,
        (0.2, 5.5),
        (-1., 1.),
        (true, true),
    )?)
    .cast())
}
fn offset(surface: &Handle<dyn GeomSurface>) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(Handle::new(GeomOffsetSurface::new(surface, 0.1, true)?).cast())
}
