//! Shared native normal-scale updates, retaining partial state on failures.
use super::{ContapHContTool, ContapSurfProps};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::Point3;
pub(super) fn sample_normal_scale(
    surface: &Adaptor3dSurfaceHandle,
    mean: &mut f64,
    point: &mut Point3,
) -> Result<(), Failure> {
    let count = ContapHContTool::nb_sample_points(surface)?;
    if count <= 0 {
        return Ok(());
    }
    *mean = 0.;
    for index in 1..=count {
        let (u, v) = ContapHContTool::sample_point(surface, index)?;
        let (sample, normal) = ContapSurfProps::normale(surface, u, v)?;
        *point = sample;
        *mean += normal.magnitude();
    }
    *mean /= f64::from(count);
    Ok(())
}
