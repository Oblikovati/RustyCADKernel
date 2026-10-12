//! Native planarity uses exact dynamic type equality and unwraps one rectangular trim only.
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::standard::{Failure, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface};
/// Bounded native shape-tool prerequisite; `StdPrsShapeTool::is_planar_face(&face)?`.
pub struct StdPrsShapeTool;
impl StdPrsShapeTool {
    /// Check exact Geom_Plane identity after at most one trim; `Self::is_planar_face(&face)?`.
    pub fn is_planar_face(face: &Face) -> Result<bool, Failure> {
        let (mut surface, _) = BRepTool::surface_with_location(face)?;
        if surface.is_null() {
            return Ok(false);
        }
        if exact_shaded_surface_type::<GeomRectangularTrimmedSurface>(&surface) {
            surface = surface
                .cast::<GeomRectangularTrimmedSurface>()
                .basis_surface();
        }
        Ok(exact_shaded_surface_type::<GeomPlane>(&surface))
    }
}

fn exact_shaded_surface_type<T: RuntimeTyped>(
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
) -> bool {
    surface
        .dynamic_type()
        .is_some_and(|descriptor| std::ptr::eq(descriptor, T::static_type()))
}
