//! Native approximation output, pcurve validity checks and restart decisions.
use super::*;
use super::{
    conic_support::{empty_descriptor, set_pcurve},
    curves::ApproximationRetry,
    pcurve_checks,
    walking::WalkingPart,
    walking_spline::WalkingSpline,
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntWLApprox};
impl FaceFace {
    pub(super) fn append_approximations(
        &mut self,
        approximation: &GeomIntWLApprox,
        part: &WalkingPart,
        kinds: [SurfaceType; 2],
        flags: [bool; 2],
        tolerance: f64,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        for index in 1..=approximation.nb_multi_curves() {
            let spline = WalkingSpline::new(approximation.value(index))?;
            let again = if kinds[0] == SurfaceType::Plane {
                self.approximate_plane1(&spline, tolerance, retry)?
            } else if kinds[1] == SurfaceType::Plane {
                self.approximate_plane2(&spline, part, tolerance, retry)?
            } else {
                self.approximate_space(&spline, flags, tolerance, retry)?
            };
            if again {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn approximate_plane1(
        &mut self,
        spline: &WalkingSpline<'_>,
        tolerance: f64,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        let (spatial, poles) = spline.on_plane(self.surfaces[0].plane()?, 1)?;
        let mut curve = empty_descriptor(&spatial);
        for index in [0, 1] {
            if !self.approximation[index + 1] {
                continue;
            }
            let pcurve = if index == 0 {
                spline.curve_from_planar_poles(&poles)?
            } else {
                spline.pcurve(2)?
            };
            if invalid_pcurve(&pcurve, retry)? {
                return Ok(true);
            }
            set_pcurve(&mut curve, index, &pcurve);
        }
        curve.set_tolerance(tolerance);
        self.curves.push(curve);
        Ok(false)
    }
    fn approximate_plane2(
        &mut self,
        spline: &WalkingSpline<'_>,
        part: &WalkingPart,
        tolerance: f64,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        let index = if self.approximation[1] { 2 } else { 1 };
        let (spatial, poles) = spline.on_plane(self.surfaces[1].plane()?, index)?;
        let mut curve = empty_descriptor(&spatial);
        curve.set_tolerance(tolerance);
        if self.plane2_second_pcurve(spline, &poles, &mut curve, retry)? {
            return Ok(true);
        }
        if self.plane2_first_pcurve(spline, &mut curve, retry)? {
            return Ok(true);
        }
        let fallback = if !retry.pcurves_ok {
            self.plane2_fallback(part, tolerance, retry)?
        } else {
            None
        };
        self.curves.push(fallback.unwrap_or(curve));
        Ok(false)
    }
    fn plane2_second_pcurve(
        &self,
        spline: &WalkingSpline<'_>,
        poles: &rustycad_tkernel::ncollection::Array1<'_, rustycad_tkmath::gp::Point2>,
        curve: &mut Curve,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        if !self.approximation[2] {
            return Ok(false);
        }
        let pcurve = spline.curve_from_planar_poles(poles)?;
        if invalid_pcurve(&pcurve, retry)? {
            return Ok(true);
        }
        retry.pcurves_ok = pcurve_checks::check_pcurve(&pcurve, &self.faces[1], &self.context)?;
        curve.set_second_curve2d(&pcurve);
        Ok(false)
    }
    fn plane2_first_pcurve(
        &self,
        spline: &WalkingSpline<'_>,
        curve: &mut Curve,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        if !self.approximation[1] {
            return Ok(false);
        }
        let pcurve = spline.pcurve(1)?;
        if invalid_pcurve(&pcurve, retry)? {
            return Ok(true);
        }
        retry.pcurves_ok = retry.pcurves_ok
            && pcurve_checks::check_pcurve(&pcurve, &self.faces[0], &self.context)?;
        curve.set_first_curve2d(&pcurve);
        Ok(false)
    }
    fn plane2_fallback(
        &self,
        part: &WalkingPart,
        tolerance: f64,
        retry: &mut ApproximationRetry,
    ) -> Result<Option<Curve>, Failure> {
        retry.pcurves_ok = true;
        let spatial = GeomIntIntSS::make_bspline(&part.line, part.first, part.last)?;
        let mut curve = empty_descriptor(&spatial);
        for index in [0, 1] {
            if !self.approximation[index + 1] {
                continue;
            }
            let pcurve =
                GeomIntIntSS::make_bspline2d(&part.line, part.first, part.last, index == 0)?.cast();
            retry.pcurves_ok = retry.pcurves_ok
                && pcurve_checks::check_pcurve(&pcurve, &self.faces[index], &self.context)?;
            set_pcurve(&mut curve, index, &pcurve);
        }
        curve.set_tolerance(tolerance);
        Ok(if retry.pcurves_ok { Some(curve) } else { None })
    }
    fn approximate_space(
        &mut self,
        spline: &WalkingSpline<'_>,
        flags: [bool; 2],
        tolerance: f64,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        let mut curve = empty_descriptor(&spline.spatial()?);
        curve.set_tolerance(tolerance);
        let mut valid = [true; 2];
        for index in [0, 1] {
            if !self.approximation[index + 1] {
                continue;
            }
            let pcurve = self.space_pcurve(spline, &curve, index, flags[index])?;
            if flags[index] && !retry.reapproximate {
                valid[index] =
                    pcurve_checks::check_pcurve(&pcurve, &self.faces[index], &self.context)?;
            }
            set_pcurve(&mut curve, index, &pcurve);
        }
        if !valid[0] || !valid[1] {
            return Ok(self.restart_space_approximation(retry));
        }
        self.curves.push(curve);
        Ok(false)
    }
    fn restart_space_approximation(&mut self, retry: &mut ApproximationRetry) -> bool {
        self.approximation_tolerance = 1e-5;
        retry.reapproximate = true;
        true
    }
    fn space_pcurve(
        &self,
        spline: &WalkingSpline<'_>,
        curve: &Curve,
        index: usize,
        approximated: bool,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        if approximated {
            let index = if index == 0 {
                2
            } else if self.approximation[1] {
                3
            } else {
                2
            };
            return spline.pcurve(index);
        }
        let range = (
            curve.curve().first_parameter(),
            curve.curve().last_parameter(),
        );
        let mut tolerance = self.approximation_tolerance;
        let projected = self.project_pcurve(index, curve.curve(), range, &mut tolerance, false)?;
        Ok(projected.cast::<Geom2dBSplineCurve>().cast())
    }
}
fn invalid_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    retry: &mut ApproximationRetry,
) -> Result<bool, Failure> {
    if !retry.reject_surface && !retry.reapproximate && !pcurve_checks::curve_valid(curve)? {
        retry.reapproximate = true;
        return Ok(true);
    }
    Ok(false)
}
