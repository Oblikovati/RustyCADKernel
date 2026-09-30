use super::Tools;
use crate::int_tools::Curve;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{GeomCurve, GeomLine, GeomParabola, GeomTrimmedCurve},
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkmath::{bnd::BoundingBox3, geom_abs::CurveType};
impl Tools {
    /// Adapts tolerance only for a trimmed parabola; `Tools::curve_tolerance(&curve, 1e-7)?`.
    pub fn curve_tolerance(curve: &Handle<dyn GeomCurve>, tolerance: f64) -> Result<f64, Failure> {
        let trimmed = curve.cast::<GeomTrimmedCurve>();
        if trimmed.is_null() {
            return Ok(tolerance);
        }
        if GeomAdaptorCurve::with_curve(curve)?.get_type()? != CurveType::Parabola {
            return Ok(tolerance);
        }
        let parabola = trimmed.basis_curve().cast::<GeomParabola>();
        if parabola.is_null() || parabola.focal() == 0. {
            return Ok(tolerance);
        }
        let axis: Handle<dyn GeomCurve> =
            Handle::new(GeomLine::new(parabola.parab()?.x_axis())).into_cast();
        let first = parabola_tolerance(&parabola, &axis, curve.first_parameter(), tolerance)?;
        let last = parabola_tolerance(&parabola, &axis, curve.last_parameter(), tolerance)?;
        Ok(if first > last { first } else { last })
    }
    /// Accumulates a curve box and checks size without its gap; `Tools::check_curve(&curve, &mut bounds)?`.
    pub fn check_curve(curve: &Curve, bounds: &mut BoundingBox3) -> Result<bool, Failure> {
        if curve.curve().is_null() {
            return Ok(false);
        }
        let adaptor = GeomAdaptorCurve::with_curve(curve.curve())?;
        Add3dCurve::add(
            &adaptor,
            std_max(curve.tolerance(), curve.tangential_tolerance()),
            bounds,
        )?;
        Ok(!bounds.is_thin(3. * precision::confusion()))
    }
}
fn parabola_tolerance(
    parabola: &Handle<GeomParabola>,
    axis: &Handle<dyn GeomCurve>,
    parameter: f64,
    tolerance: f64,
) -> Result<f64, Failure> {
    let mut projector = ProjectPointOnCurve::new();
    projector.init_curve(axis, axis.first_parameter(), axis.last_parameter())?;
    projector.perform(parabola.eval_d0(parameter)?)?;
    let x = if projector.nb_points() > 0 {
        projector.lower_distance_parameter()?
    } else {
        0.
    };
    let adjusted = if x >= 0. {
        tolerance * (0.5 * x / parabola.focal()).sqrt()
    } else {
        tolerance
    };
    Ok(if adjusted == 0. { tolerance } else { adjusted })
}
