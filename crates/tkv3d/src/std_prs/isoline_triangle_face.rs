//! Face triangulation aligns its surface location and trims exact infinite offset surfaces before crossing.
use super::{
    StdPrsIsolineDestination, StdPrsIsolineTriangulationInputs,
    isoline_parameters::face_isoline_parameters,
    isoline_triangle_collect::collect_triangle_isolines,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, RuntimeTyped, SharedState},
};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkmath::toploc::Location;
pub(super) fn face_triangulation_isolines(
    face: &Face,
    drawer: &Prs3dDrawer,
    output: &mut StdPrsIsolineDestination<'_>,
) -> Result<(), Failure> {
    let Some(prepared) = prepare_face_isoline_mesh(face, drawer)? else {
        return Ok(());
    };
    collect_triangle_isolines(&prepared.references(), output)
}
struct FaceIsolineMesh {
    mesh: SharedState<Handle<rustycad_tkmath::poly::Triangulation>>,
    surface: SharedState<Handle<dyn GeomSurface>>,
    location: SharedState<Location>,
    parameters: [SharedState<rustycad_tkernel::ncollection::Sequence<f64>>; 2],
}
impl FaceIsolineMesh {
    fn references(&self) -> StdPrsIsolineTriangulationInputs<'_> {
        StdPrsIsolineTriangulationInputs {
            mesh: &self.mesh,
            surface: &self.surface,
            location: &self.location,
            parameters: self.parameters.each_ref(),
        }
    }
}
fn prepare_face_isoline_mesh(
    face: &Face,
    drawer: &Prs3dDrawer,
) -> Result<Option<FaceIsolineMesh>, Failure> {
    let counts = [
        drawer.u_iso_aspect().number(),
        drawer.v_iso_aspect().number(),
    ];
    if counts[0] < 1 && counts[1] < 1 {
        return Ok(None);
    }
    let (parameters, bounds) =
        face_isoline_parameters(face, counts, drawer.maximal_parameter_value())?;
    let (surface, surface_location) = BRepTool::surface_with_location(face)?;
    if surface.is_null() {
        return Ok(None);
    }
    prepare_located_isoline_mesh(face, surface, surface_location, parameters, bounds)
}
fn prepare_located_isoline_mesh(
    face: &Face,
    surface: Handle<dyn GeomSurface>,
    surface_location: Location,
    parameters: [rustycad_tkernel::ncollection::Sequence<f64>; 2],
    bounds: [f64; 4],
) -> Result<Option<FaceIsolineMesh>, Failure> {
    let (mesh, location) = BRepTool::triangulation(face, 0)?;
    if mesh.is_null() {
        return Ok(None);
    }
    let surface = align_isoline_surface(surface, &surface_location, &location)?;
    let surface = trim_infinite_isoline_offset(surface, bounds)?;
    Ok(Some(FaceIsolineMesh {
        mesh: SharedState::new(mesh),
        surface: SharedState::new(surface),
        location: SharedState::new(location),
        parameters: parameters.map(SharedState::new),
    }))
}

fn align_isoline_surface(
    surface: Handle<dyn GeomSurface>,
    surface_location: &Location,
    mesh_location: &Location,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if mesh_location.is_equal(surface_location) {
        return Ok(surface);
    }
    let transform = surface_location.divided(mesh_location)?.transformation();
    let transformed = surface.transformed(transform)?;
    Ok(transformed.cast())
}
fn trim_infinite_isoline_offset(
    surface: Handle<dyn GeomSurface>,
    face_bounds: [f64; 4],
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if !exact_isoline_offset(&surface) {
        return Ok(surface);
    }
    let (u0, u1, v0, v1) = surface.bounds();
    if ![u0, u1, v0, v1]
        .iter()
        .any(|value| precision::is_infinite(*value))
    {
        return Ok(surface);
    }
    let u = (std_max(face_bounds[0], u0), std_min(face_bounds[1], u1));
    let v = (std_max(face_bounds[2], v0), std_min(face_bounds[3], v1));
    let trimmed = GeomRectangularTrimmedSurface::new(&surface, u, v, (true, true))?;
    Ok(Handle::new(trimmed).cast())
}

fn exact_isoline_offset(surface: &Handle<dyn GeomSurface>) -> bool {
    // Native compares exact dynamic type, deliberately excluding application-derived offset classes.
    surface
        .dynamic_type()
        .is_some_and(|ty| std::ptr::eq(ty, GeomOffsetSurface::static_type()))
}
