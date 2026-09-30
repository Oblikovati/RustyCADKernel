//! Reached 3D tolerance uses pcurve deviations when available, surface projection otherwise.
use super::distance;
use super::*;
impl FaceFace {
    pub(super) fn compute_tolerance_reached(&mut self, parallel: bool) -> Result<(), Failure> {
        let [first, last] = self.kernel.face_tolerances;
        let minimum = if first < last { last } else { first };
        for curve in &mut self.curves {
            if curve.curve().is_null() {
                continue;
            }
            let tolerance =
                curve_deviation(curve, &self.surfaces, &self.faces, &self.context, parallel)?;
            curve.set_tolerance(tolerance);
            if curve.tangential_tolerance() < minimum {
                curve.set_tangential_tolerance(minimum);
            }
        }
        Ok(())
    }
}
fn curve_deviation(
    curve: &Curve,
    surfaces: &[Handle<GeomAdaptorSurface>; 2],
    faces: &[Face; 2],
    context: &Context,
    parallel: bool,
) -> Result<f64, Failure> {
    let geometry = curve.curve();
    let (first, last) = (geometry.first_parameter(), geometry.last_parameter());
    let mut tolerance = curve.tolerance();
    for (index, pcurve) in [curve.first_curve2d(), curve.second_curve2d()]
        .into_iter()
        .enumerate()
    {
        let (valid, reached) = if pcurve.is_null() {
            (
                true,
                distance::face_distance(geometry, first, last, &faces[index], context)?,
            )
        } else {
            pcurve_deviation(geometry, pcurve, &surfaces[index], (first, last), parallel)?
        };
        if valid && reached > tolerance {
            tolerance = reached;
        }
    }
    Ok(tolerance)
}

fn pcurve_deviation(
    geometry: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    pcurve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    surface: &GeomAdaptorSurface,
    range: (f64, f64),
    parallel: bool,
) -> Result<(bool, f64), Failure> {
    let mut reached = 0.;
    let mut parameter = 0.;
    let valid = Tools::compute_tolerance(
        geometry,
        pcurve,
        &surface.surface(),
        range.0,
        range.1,
        &mut reached,
        &mut parameter,
        precision::p_confusion(),
        parallel,
    )?;
    Ok((valid, reached))
}
