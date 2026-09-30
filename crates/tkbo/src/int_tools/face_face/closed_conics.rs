//! Closed conics crossing zero split in native order and retain native endpoint adjustment.
use super::*;
use super::{
    boundary_parameter::BoundaryParameter,
    conic_support::{self, empty_descriptor, set_pcurve, trimmed},
};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::{int_patch::IntPatchLine, int_start::TheTopolTool};
use std::{f64::consts::PI, rc::Rc};
impl FaceFace {
    pub(super) fn closed_conics(
        &mut self,
        line: &Handle<dyn IntPatchLine>,
        domains: &[Rc<dyn TheTopolTool>; 2],
        tolerance: &mut f64,
    ) -> Result<(), Failure> {
        let geometry = conic_support::geometry(line)?;
        let ranges = self.closed_ranges(&geometry, *tolerance)?;
        for range in &ranges {
            if range.0.abs() > f64::EPSILON || (range.1 - 2. * PI).abs() > f64::EPSILON {
                self.closed_part(&geometry, *range, tolerance, true, true)?;
                continue;
            }
            if ranges.len() == 1
                && range.0.abs() <= f64::EPSILON
                && (range.1 - 2. * PI).abs() <= f64::EPSILON
            {
                self.closed_part(&geometry, *range, tolerance, true, false)?;
                break;
            }
            if self.closed_part_inside(&geometry, domains)? {
                self.closed_part(&geometry, *range, tolerance, false, true)?;
            }
        }
        Ok(())
    }
    fn closed_part_inside(
        &self,
        curve: &Handle<dyn GeomCurve>,
        domains: &[Rc<dyn TheTopolTool>; 2],
    ) -> Result<bool, Failure> {
        let step = 2. * PI / 17.;
        for index in 0..=17 {
            if self.inside_domains(curve.eval_d0(f64::from(index) * step)?, domains)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn closed_part(
        &mut self,
        geometry: &Handle<dyn GeomCurve>,
        mut range: (f64, f64),
        tolerance: &mut f64,
        trim: bool,
        bounded: bool,
    ) -> Result<(), Failure> {
        let spatial = if trim {
            trimmed(geometry, range)?
        } else {
            geometry.clone()
        };
        if trim {
            range = (spatial.first_parameter(), spatial.last_parameter());
        }
        let mut curve = empty_descriptor(&spatial);
        for index in [0, 1] {
            if self.approximation[index + 1] {
                set_pcurve(
                    &mut curve,
                    index,
                    &self.project_pcurve(index, geometry, range, tolerance, bounded)?,
                );
            }
        }
        self.curves.push(curve);
        Ok(())
    }
    fn closed_ranges(
        &self,
        geometry: &Handle<dyn GeomCurve>,
        tolerance: f64,
    ) -> Result<Vec<(f64, f64)>, Failure> {
        let mut ranges = Vec::new();
        for index in 1..=self.kernel.constructor.nb_parts()? {
            let range = self.kernel.constructor.part(index)?;
            if range.0 < 0. && range.1 > 0. {
                self.seam_ranges(geometry, range, tolerance, &mut ranges)?;
            } else {
                ranges.push(range);
            }
        }
        Ok(ranges)
    }
    fn seam_ranges(
        &self,
        curve: &Handle<dyn GeomCurve>,
        (mut first, mut last): (f64, f64),
        tolerance: f64,
        ranges: &mut Vec<(f64, f64)>,
    ) -> Result<(), Failure> {
        let period = PI + PI;
        while first < 0. || first > period {
            first += period;
        }
        while last < 0. || last > period {
            last += period;
        }
        if period - first > tolerance {
            ranges.push((first, period));
        } else if curve.eval_d0(first)?.distance(&curve.eval_d0(period)?) > self.tolerance() {
            first = self.boundary_parameter(curve, first, last, false)?;
            ranges.push((first, period));
        }
        if last > tolerance {
            ranges.push((0., last));
        } else if curve.eval_d0(0.)?.distance(&curve.eval_d0(last)?) > self.tolerance() {
            last = self.boundary_parameter(curve, last, first, true)?;
            ranges.push((0., last));
        }
        Ok(())
    }
    fn boundary_parameter(
        &self,
        curve: &Handle<dyn GeomCurve>,
        parameter: f64,
        other: f64,
        increase: bool,
    ) -> Result<f64, Failure> {
        let search = BoundaryParameter {
            parameter,
            other,
            increase,
            tolerance: self.tolerance(),
        };
        let mut result = parameter;
        search.compute(curve, &self.faces, &self.context, &mut result)?;
        Ok(result)
    }
}
