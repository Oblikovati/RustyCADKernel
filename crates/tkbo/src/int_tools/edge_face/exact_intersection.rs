//! Exact IntCurveSurface fallback used when Extrema finds no stationary point.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::int_curve_surface::HInter;

#[cfg(test)]
#[path = "exact_intersection_tests.rs"]
mod tests;

pub(super) fn exact_intersection_parameter(
    curve: &dyn Adaptor3dCurve,
    surface: &dyn Adaptor3dSurface,
    first: f64,
    last: f64,
) -> Result<Option<f64>, Failure> {
    let curve = curve.shallow_copy()?.cast();
    let surface = surface.shallow_copy()?.cast();
    let mut intersector = HInter::new();
    intersector.perform(&curve, &surface)?;
    if !intersector.is_done() {
        return Ok(None);
    }
    last_intersection_parameter(&intersector, first, last)
}

fn last_intersection_parameter(
    intersector: &HInter,
    first: f64,
    last: f64,
) -> Result<Option<f64>, Failure> {
    let mut selected = None;
    for index in 1..=intersector.nb_points()? {
        let parameter = intersector.point(index)?.w();
        if parameter >= first && parameter <= last {
            selected = Some(parameter);
        }
    }
    Ok(selected)
}
