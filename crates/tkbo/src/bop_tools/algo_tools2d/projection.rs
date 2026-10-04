//! MakePCurveOnFace from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod settings;
use super::adjust_pcurve_on_surface;
use crate::int_tools::{Context, Tools};
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    geom::GeomCurve,
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkgeombase::{
    geom_lib,
    proj_lib::{self, ProjectedCurve},
};
use rustycad_tkmath::geom_abs::SurfaceType;
use settings::ProjectionSettings;
/// Project a curve interval and update the reached edge tolerance; `make_pcurve_on_face(&face, &curve, (0.,1.), &mut pcurve, &mut tolerance, None)?`.
pub fn make_pcurve_on_face(
    face: &Face,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    pcurve: &mut Handle<dyn Geom2dCurve>,
    tolerance: &mut f64,
    context: Option<&Context>,
) -> Result<(), Failure> {
    if let Some(context) = context {
        let cached = context.surface_adaptor(face)?;
        return project_on_adaptor(&cached.read(), curve, range, pcurve, tolerance);
    }
    project_on_adaptor(
        &BRepAdaptorSurface::with_face(face, true)?,
        curve,
        range,
        pcurve,
        tolerance,
    )
}
/// Project the complete curve, resetting the initial tolerance to zero; `make_pcurve_full_curve(&face, &curve, &mut pcurve, &mut tolerance, None)?`.
pub fn make_pcurve_full_curve(
    face: &Face,
    curve: &Handle<dyn GeomCurve>,
    pcurve: &mut Handle<dyn Geom2dCurve>,
    tolerance: &mut f64,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let range = (curve.first_parameter(), curve.last_parameter());
    *tolerance = 0.;
    make_pcurve_on_face(face, curve, range, pcurve, tolerance, context)
}
fn project_on_adaptor(
    surface: &BRepAdaptorSurface,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    pcurve: &mut Handle<dyn Geom2dCurve>,
    tolerance: &mut f64,
) -> Result<(), Failure> {
    let curve_adaptor: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(curve, range.0, range.1)?).cast();
    let surface_adaptor = surface.shallow_copy()?;
    let (trial_tolerance, reached) = initial_projection(
        surface,
        &surface_adaptor,
        &curve_adaptor,
        *tolerance,
        pcurve,
    )?;
    let reached = retry_projection(
        &surface_adaptor,
        &curve_adaptor,
        trial_tolerance,
        *tolerance,
        reached,
        pcurve,
    )?;
    if pcurve.is_null() {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "BOPTools_AlgoTools2D::MakePCurveOnFace: pcurve=null; expected a projected curve",
        ));
    }
    *tolerance = reached;
    *pcurve = adjust_pcurve_on_surface(surface, range.0, range.1, pcurve)?;
    ensure_range(pcurve, range)?;
    update_tolerance(surface, curve, range, pcurve, tolerance)
}
fn initial_projection(
    surface: &BRepAdaptorSurface,
    surface_adaptor: &Handle<dyn Adaptor3dSurface>,
    curve_adaptor: &Handle<dyn Adaptor3dCurve>,
    tolerance: f64,
    pcurve: &mut Handle<dyn Geom2dCurve>,
) -> Result<(f64, f64), Failure> {
    let analytic = proj_lib::is_ana_surf(surface_adaptor)?;
    if surface.get_type()? == SurfaceType::SurfaceOfRevolution {
        let trial = if tolerance > precision::confusion() {
            tolerance
        } else {
            precision::confusion()
        };
        let projected = ProjectedCurve::with_tolerance(surface_adaptor, curve_adaptor, trial)?;
        *pcurve = proj_lib::make_pcurve_of_type(&projected)?;
        return Ok((trial, projected.get_tolerance()));
    }
    let settings = ProjectionSettings::new(tolerance, analytic);
    let projected = ProjectedCurve::with_surface(surface_adaptor);
    settings.apply(&projected, surface_adaptor)?;
    projected.perform(curve_adaptor)?;
    *pcurve = proj_lib::make_pcurve_of_type(&projected)?;
    Ok((settings.tolerance, projected.get_tolerance()))
}
fn retry_projection(
    surface: &Handle<dyn Adaptor3dSurface>,
    curve: &Handle<dyn Adaptor3dCurve>,
    trial: f64,
    input: f64,
    reached: f64,
    pcurve: &mut Handle<dyn Geom2dCurve>,
) -> Result<f64, Failure> {
    let maximum = 1e3 * precision::confusion();
    if !pcurve.is_null() || !(trial < maximum || trial < input) {
        return Ok(reached);
    }
    let projected = ProjectedCurve::with_tolerance(
        surface,
        curve,
        if input < maximum { maximum } else { input },
    )?;
    *pcurve = proj_lib::make_pcurve_of_type(&projected)?;
    Ok(projected.get_tolerance())
}
fn ensure_range(pcurve: &mut Handle<dyn Geom2dCurve>, range: (f64, f64)) -> Result<(), Failure> {
    let mut first = pcurve.first_parameter();
    let mut last = pcurve.last_parameter();
    if !(first - range.0 > precision::p_confusion() || range.1 - last > precision::p_confusion()) {
        return Ok(());
    }
    if first < range.0 {
        first = range.0;
    }
    if last > range.1 {
        last = range.1;
    }
    let original = pcurve.clone();
    geom_lib::same_range(
        precision::p_confusion(),
        &original,
        (first, last),
        range,
        pcurve,
    )
}
fn update_tolerance(
    surface: &BRepAdaptorSurface,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    pcurve: &Handle<dyn Geom2dCurve>,
    tolerance: &mut f64,
) -> Result<(), Failure> {
    let support = surface.transformed_surface().geom_surface_transformed()?;
    let mut distance = *tolerance;
    let mut parameter = 0.;
    if Tools::compute_tolerance(
        curve,
        pcurve,
        &support,
        range.0,
        range.1,
        &mut distance,
        &mut parameter,
        precision::p_confusion(),
        false,
    )? && distance > *tolerance
    {
        *tolerance = distance;
    }
    Ok(())
}
