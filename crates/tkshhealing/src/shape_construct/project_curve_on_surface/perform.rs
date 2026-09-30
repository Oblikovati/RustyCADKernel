use super::iso_sampling::IsoSampling;
use super::sampling::{CurveSamples, uneven_bspline};
use super::*;
use rustycad_tkg3d::geom::GeomTrimmedCurve;

impl ShapeConstructProjectCurveOnSurface {
    /// Projects a curve interval, retaining native status and endpoint cache.
    /// `projector.perform(&curve, (first,last), &mut pcurve, (-1.,-1.))?;`
    pub fn perform(
        &self,
        curve: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        output: &mut Handle<dyn Geom2dCurve>,
        endpoint_tolerances: (f64, f64),
    ) -> Result<bool, Failure> {
        self.set_status(ShapeExtendStatus::Ok);
        if self.analysis().is_null() || curve.is_null() {
            *output = Handle::default();
            self.add_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        let trimmed = bounded_projection_curve(curve, range)?;
        *output = self.project_analytic(&trimmed)?;
        if !output.is_null() {
            self.add_status(ShapeExtendStatus::Done1);
            return Ok(true);
        }
        if uneven_bspline(curve, range)? && self.perform_by_proj_lib(curve, range, output) {
            return Ok(self.status(ShapeExtendStatus::Done));
        }
        self.perform_sampled(curve, range, output, endpoint_tolerances)
    }

    fn perform_sampled(
        &self,
        curve: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        output: &mut Handle<dyn Geom2dCurve>,
        tolerances: (f64, f64),
    ) -> Result<bool, Failure> {
        let mut samples = CurveSamples::generate(curve, range)?;
        *output = self.approximate_samples(curve, &mut samples, tolerances)?;
        if output.is_null() {
            *output = self.interpolate_pcurve(&samples.uv, &samples.parameters);
        }
        self.add_status(if output.is_null() {
            ShapeExtendStatus::Fail1
        } else {
            ShapeExtendStatus::Done2
        });
        Ok(self.status(ShapeExtendStatus::Done))
    }

    fn approximate_samples(
        &self,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        tolerances: (f64, f64),
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let line = self.get_line(samples)?;
        if !line.curve.is_null() {
            self.cache_samples(samples, self.reverse_samples(samples));
            return Ok(line.curve);
        }
        let iso = IsoSampling::new(
            &self.analysis(),
            self.recognize_iso(samples),
            self.fields.read().precision,
        )?;
        let walk = self.walk_samples(samples, &line, &iso)?;
        if !iso.recognition.same_parameters {
            self.project_degenerate_samples(samples)?;
        }
        self.correct_singular_ends(curve, samples, tolerances)?;
        self.resolve_seams(curve, samples, &walk)?;
        self.adjust_degenerated_walk(samples)?;
        self.cache_samples(samples, walk.reverse);
        Ok(Handle::default())
    }

    fn project_degenerate_samples(&self, samples: &mut CurveSamples) -> Result<(), Failure> {
        let analysis = self.analysis();
        let tolerance = self.fields.read().precision;
        analysis.project_degenerated_points(
            samples.points.as_slice(),
            samples.uv.as_mut_slice(),
            tolerance,
            true,
        )?;
        analysis.project_degenerated_points(
            samples.points.as_slice(),
            samples.uv.as_mut_slice(),
            tolerance,
            false,
        )?;
        Ok(())
    }
}
fn bounded_projection_curve(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if curve.is_kind_name("Geom_BoundedCurve") {
        return Ok(curve.clone());
    }
    Ok(Handle::new(GeomTrimmedCurve::new(curve, range.0, range.1, true, true)?).cast())
}
