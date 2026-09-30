use super::*;
use rustycad_tkgeombase::extrema::{ExtPC, LocateExtPC, POnCurv};
pub(super) struct EndpointSearch<'a> {
    point: Point3,
    adaptor: &'a GeomAdaptorCurve,
    vertex_tolerance: f64,
    limit: f64,
}
impl<'a> EndpointSearch<'a> {
    pub(super) fn new(
        point: Point3,
        adaptor: &'a GeomAdaptorCurve,
        vertex_tolerance: f64,
        limit: f64,
    ) -> Self {
        Self {
            point,
            adaptor,
            vertex_tolerance,
            limit,
        }
    }
    pub(super) fn accept_bounds(
        &self,
        curve: &Handle<dyn GeomCurve>,
        parameter: &mut f64,
    ) -> Result<bool, Failure> {
        let bounds = (curve.first_parameter(), curve.last_parameter());
        let first_distance = self.bound_distance(curve, bounds.0)?;
        let first_valid = !precision::is_infinite(bounds.0) && first_distance < self.limit;
        if first_valid {
            self.refine(curve, bounds, true, first_distance, parameter)?;
        }
        if precision::is_infinite(bounds.1) {
            return Ok(first_valid);
        }
        let last_distance = self.bound_distance(curve, bounds.1)?;
        if first_valid && first_distance < last_distance {
            return Ok(true);
        }
        if last_distance < self.limit {
            self.refine(curve, bounds, false, last_distance, parameter)?;
            return Ok(true);
        }
        Ok(false)
    }
    fn bound_distance(&self, curve: &Handle<dyn GeomCurve>, bound: f64) -> Result<f64, Failure> {
        if precision::is_infinite(bound) {
            return Ok(precision::infinite());
        }
        Ok(self.point.distance(&curve.eval_d0(bound)?))
    }
    fn refine(
        &self,
        curve: &Handle<dyn GeomCurve>,
        bounds: (f64, f64),
        first: bool,
        distance: f64,
        parameter: &mut f64,
    ) -> Result<(), Failure> {
        let bound = if first { bounds.0 } else { bounds.1 };
        *parameter = bound;
        if distance.partial_cmp(&self.vertex_tolerance) != Some(std::cmp::Ordering::Greater) {
            return Ok(());
        }
        if let Some(candidate) = nearest_extremum(self.point, self.adaptor, bound)? {
            *parameter = candidate.parameter();
            let middle = (bounds.1 + bounds.0) * 0.5;
            let wrong_half = wrong_endpoint_half(*parameter, middle, first);
            if wrong_half
                || self.point.distance(candidate.value()) > self.limit
                || curve.eval_d0(bound)?.distance(candidate.value()) < precision::confusion()
            {
                *parameter = bound;
            }
        }
        Ok(())
    }
}
fn nearest_extremum(
    point: Point3,
    adaptor: &GeomAdaptorCurve,
    bound: f64,
) -> Result<Option<POnCurv>, Failure> {
    let local = LocateExtPC::with_curve(point, adaptor, bound, 1e-10)?;
    if local.is_done() {
        return Ok(Some(local.point()?));
    }
    let global = ExtPC::with_curve(point, adaptor, 1e-10)?;
    if !global.is_done() {
        return Ok(None);
    }
    let mut best = f64::MAX;
    let mut candidate = None;
    for i in 1..=global.nb_ext()? {
        if global.is_min(i)? && global.square_distance(i)? < best {
            best = global.square_distance(i)?;
            candidate = Some(global.point(i)?);
        }
    }
    Ok(candidate)
}

fn wrong_endpoint_half(parameter: f64, middle: f64, first: bool) -> bool {
    if first {
        return parameter > middle;
    }
    parameter < middle
}
