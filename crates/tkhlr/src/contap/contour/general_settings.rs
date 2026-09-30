//! Native general-contour step sizes and scale from the surface bounding box.
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as surface_tool, Adaptor3dSurfaceHandle};
use rustycad_tkgeombase::bnd_lib::AddSurface;
use rustycad_tkmath::bnd::BoundingBox3;
pub(super) struct GeneralSettings {
    pub u_resolution: f64,
    pub v_resolution: f64,
    pub precision: f64,
    pub deflection: f64,
    pub arc_tolerance: f64,
}
impl GeneralSettings {
    pub fn new(surface: &Adaptor3dSurfaceHandle) -> Result<Self, Failure> {
        let u_resolution = surface_tool::u_resolution(surface, precision::confusion())?;
        let v_resolution = surface_tool::v_resolution(surface, precision::confusion())?;
        let precision = if v_resolution < u_resolution {
            v_resolution
        } else {
            u_resolution
        };
        let scale = bounding_scale(surface)?;
        Ok(Self {
            u_resolution,
            v_resolution,
            precision,
            deflection: 0.01 * scale,
            arc_tolerance: precision::confusion() * scale,
        })
    }
}
fn bounding_scale(surface: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
    let u_first = surface.first_u_parameter()?;
    let v_first = surface.first_v_parameter()?;
    let u_last = surface.last_u_parameter()?;
    let v_last = surface.last_v_parameter()?;
    if precision::is_negative_infinite(u_first)
        || precision::is_positive_infinite(u_last)
        || precision::is_negative_infinite(v_first)
        || precision::is_positive_infinite(v_last)
    {
        return Ok(1.);
    }
    let mut bounds = BoundingBox3::default();
    AddSurface::add(&**surface, 1e-8, &mut bounds)?;
    let [x0, y0, z0, x1, y1, z1] = bounds.get()?;
    let (mut dx, dy, dz) = (x1 - x0, y1 - y0, z1 - z0);
    if dx < dy {
        dx = dy;
    }
    if dx < dz {
        dx = dz;
    }
    if dx > 10000. {
        dx = 10000.;
    }
    Ok(dx)
}
