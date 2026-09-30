// Rust translation of BRepLib::SortFaces and BRepLib::ReverseSortFaces (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::SortFaces` and `BRepLib::ReverseSortFaces`: the faces of a shape grouped by
//! the kind of their surface.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;

/// The face groups in `SortFaces` order: plane, cylinder, cone, sphere, torus, other
/// surface, then the faces without surface (triangulations).
const GROUPS: usize = 7;
const TRIANGULATION_GROUP: usize = 6;

/// The group of a surface kind.
fn group_of(kind: SurfaceType) -> usize {
    match kind {
        SurfaceType::Plane => 0,
        SurfaceType::Cylinder => 1,
        SurfaceType::Cone => 2,
        SurfaceType::Sphere => 3,
        SurfaceType::Torus => 4,
        _ => 5,
    }
}

/// The faces of `shape` in explorer order, grouped; `unwrap_trimmed` replaces a
/// rectangular trimmed surface by its basis first (SortFaces does, ReverseSortFaces not).
fn grouped_faces(shape: &Shape, unwrap_trimmed: bool) -> Result<[Vec<Shape>; GROUPS], Failure> {
    let mut groups: [Vec<Shape>; GROUPS] = Default::default();
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        let face = Face::unchecked(explorer.current().clone());
        explorer.next()?;
        let (mut surface, _) = BRepTool::surface_with_location(&face)?;
        if surface.is_null() {
            groups[TRIANGULATION_GROUP].push(face.into_shape());
            continue;
        }
        if unwrap_trimmed
            && let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get()
        {
            surface = trimmed.basis_surface();
        }
        let kind = GeomAdaptorSurface::with_surface(&surface)?.get_type()?;
        groups[group_of(kind)].push(face.into_shape());
    }
    Ok(groups)
}

/// `BRepLib::SortFaces(S, LF)`: the faces of the shape on planes, then cylinders, cones,
/// spheres, tori, other surfaces, and last the faces without surface;
/// `brep_lib::sort_faces(&shape)?`.
pub fn sort_faces(shape: &Shape) -> Result<List<Shape>, Failure> {
    let groups = grouped_faces(shape, true)?;
    Ok(groups.into_iter().flatten().collect())
}

/// `BRepLib::ReverseSortFaces(S, LF)`: the faces without surface first, then other
/// surfaces, tori, spheres, cones, cylinders and planes (a rectangular trimmed surface
/// is classified through the adaptor of the trimmed surface itself);
/// `brep_lib::reverse_sort_faces(&shape)?`.
pub fn reverse_sort_faces(shape: &Shape) -> Result<List<Shape>, Failure> {
    let groups = grouped_faces(shape, false)?;
    Ok(groups.into_iter().rev().flatten().collect())
}
