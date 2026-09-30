//! Native contour restriction extrema, including parameter-based vertex merging.
use crate::contap::{
    ContapHContTool, ContapHCurve2dTool, ContapIType, ContapLine, ContapPoint, ContapSurfFunction,
    ContapTFunction,
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as surface_tool, Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkmath::gp::{self, Point2, Point3, Vector3};
struct RestrictionSample {
    point: Point3,
    uv: Point2,
    cross: Vector3,
}
struct RestrictionSearch<'a> {
    function: &'a ContapSurfFunction,
    arc: TheArc,
    surface: Adaptor3dSurfaceHandle,
    first: f64,
    last: f64,
    samples: i32,
    tolerance: f64,
}
pub(super) fn compute_restriction_internal(
    line: &ContapLine,
    first: f64,
    last: f64,
    function: &ContapSurfFunction,
) -> Result<(), Failure> {
    if line.type_contour() != ContapIType::Restriction {
        return Ok(());
    }
    let arc = line.arc()?;
    let search = RestrictionSearch {
        function,
        surface: function.surface(),
        first,
        last,
        samples: ContapHContTool::nb_samples_on_arc(&arc)?,
        tolerance: ContapHCurve2dTool::resolution(&arc, precision::confusion())?,
        arc,
    };
    search.perform(line)
}
impl RestrictionSearch<'_> {
    fn parameter(&self, index: i32) -> f64 {
        (f64::from(self.samples - index) * self.first + f64::from(index - 1) * self.last)
            / f64::from(self.samples - 1)
    }
    fn evaluate(&self, parameter: f64) -> Result<RestrictionSample, Failure> {
        let (uv, d2) = ContapHCurve2dTool::d1(&self.arc, parameter)?;
        let derivative = surface_tool::d1(&self.surface, uv.x(), uv.y())?;
        let mut tangent = Vector3::default();
        tangent.set_linear_form(d2.x(), derivative.d1u, d2.y(), derivative.d1v);
        let mut cross = Vector3::default();
        if tangent.magnitude() > gp::resolution() {
            cross = self.view_direction(derivative.point).crossed(tangent);
        }
        Ok(RestrictionSample {
            point: derivative.point,
            uv,
            cross,
        })
    }
    fn view_direction(&self, point: Point3) -> Vector3 {
        match self.function.function_type() {
            ContapTFunction::ContourPrs | ContapTFunction::DraftPrs => {
                Vector3::from(point.xyz() - self.function.eye().xyz())
            }
            _ => Vector3::from(self.function.direction()),
        }
    }
    fn initial_reference(&self) -> Result<Option<(i32, f64, Vector3)>, Failure> {
        for index in 1..=self.samples {
            let parameter = self.parameter(index);
            let sample = self.evaluate(parameter)?;
            // Native skips only <= resolution: NaN crosses remain candidate references.
            if sample.cross.magnitude() <= gp::resolution() {
                continue;
            }
            return Ok(Some((index, parameter, sample.cross)));
        }
        Ok(None)
    }
    fn perform(&self, line: &ContapLine) -> Result<(), Failure> {
        let Some((start, mut lower, mut reference)) = self.initial_reference()? else {
            return Ok(());
        };
        for index in start + 1..=self.samples {
            let upper = self.parameter(index);
            let current = self.evaluate(upper)?;
            if current.cross.magnitude() <= gp::resolution() {
                continue;
            }
            if current.cross.dot(reference) < 0. {
                let (parameter, sample) = self.bisect(lower, upper, reference)?;
                self.record(line, parameter, sample)?;
            }
            reference = current.cross;
            lower = upper;
        }
        Ok(())
    }
    fn bisect(
        &self,
        mut lower: f64,
        mut upper: f64,
        reference: Vector3,
    ) -> Result<(f64, RestrictionSample), Failure> {
        loop {
            let parameter = (lower + upper) / 2.;
            let sample = self.evaluate(parameter)?;
            if sample.cross.magnitude() <= gp::resolution()
                || (parameter - lower).abs() <= self.tolerance
                || (parameter - upper).abs() <= self.tolerance
            {
                return Ok((parameter, sample));
            }
            if sample.cross.dot(reference) < 0. {
                upper = parameter;
            } else {
                lower = parameter;
            }
        }
    }
    fn record(
        &self,
        line: &ContapLine,
        parameter: f64,
        sample: RestrictionSample,
    ) -> Result<(), Failure> {
        let mut found = false;
        for index in 1..=line.nb_vertex() {
            if (line.vertex(index)?.parameter_on_line() - parameter).abs() <= self.tolerance {
                line.change_vertex(index, ContapPoint::set_internal)?;
                found = true;
            }
        }
        if found {
            return Ok(());
        }
        let mut point = ContapPoint::with_value(sample.point, sample.uv.x(), sample.uv.y());
        point.set_parameter(parameter);
        point.set_internal();
        line.add_vertex(&point);
        Ok(())
    }
}
