//! Native ProcessSegments preserves shared vertex edits before appending restrictions.
use super::{
    line_constructor::construct_lines, math_helpers::transition_on_line,
    restriction_internal::compute_restriction_internal,
};
use crate::contap::{
    ContapHCurve2dTool, ContapLine, ContapPoint, ContapSurfFunction,
    ContapThePathPointOfTheSearch as PathPoint, ContapTheSearch,
    ContapTheSegmentOfTheSearch as Segment,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool;
use rustycad_tkgeomalgo::int_start::TheTopolTool;
use rustycad_tkmath::gp::Vector3;
struct SegmentProcessor {
    point: ContapPoint,
    line: ContapLine,
}
#[derive(Default)]
struct SegmentEnds {
    first: Option<PathPoint>,
    last: Option<PathPoint>,
}
pub(super) fn process_segments(
    search: &ContapTheSearch,
    lines: &mut Vec<ContapLine>,
    tolerance: f64,
    function: &mut ContapSurfFunction,
    domain: &dyn TheTopolTool,
) -> Result<(), Failure> {
    let mut processor = SegmentProcessor {
        point: ContapPoint::new(),
        line: ContapLine::new(),
    };
    for index in 1..=search.nb_segments()? {
        processor.process(search.segment(index)?, lines, tolerance, function, domain)?;
    }
    Ok(())
}
impl SegmentEnds {
    fn new(segment: &Segment) -> Result<Self, Failure> {
        let first = if segment.has_first_point() {
            Some(segment.first_point()?.clone())
        } else {
            None
        };
        let last = if segment.has_last_point() {
            Some(segment.last_point()?.clone())
        } else {
            None
        };
        Ok(Self { first, last })
    }
    fn middle(&self) -> f64 {
        match (&self.first, &self.last) {
            (Some(first), Some(last)) => (first.parameter() + last.parameter()) / 2.,
            (Some(first), None) => first.parameter() + 1.,
            (None, Some(last)) => last.parameter() - 1.,
            _ => 0.,
        }
    }
}
impl SegmentProcessor {
    fn process(
        &mut self,
        segment: &Segment,
        lines: &mut Vec<ContapLine>,
        tolerance: f64,
        function: &mut ContapSurfFunction,
        domain: &dyn TheTopolTool,
    ) -> Result<(), Failure> {
        self.line.set_arc(&segment.curve());
        let mut ends = SegmentEnds::new(segment)?;
        self.set_transition(ends.middle(), function)?;
        self.match_existing(lines, &mut ends, tolerance)?;
        if let Some(first) = &ends.first {
            self.add_endpoint(first)?;
        }
        if let Some(last) = &ends.last {
            self.add_endpoint(last)?;
        }
        if segment.has_first_point() && segment.has_last_point() {
            compute_restriction_internal(
                &self.line,
                segment.first_point()?.parameter(),
                segment.last_point()?.parameter(),
                function,
            )?;
        }
        construct_lines(lines, domain, &self.line, &function.surface())?;
        self.line.clear();
        Ok(())
    }
    fn set_transition(
        &mut self,
        parameter: f64,
        function: &mut ContapSurfFunction,
    ) -> Result<(), Failure> {
        let (uv, d2) = ContapHCurve2dTool::d1(&self.line.arc()?, parameter)?;
        let derivative = h_surface_tool::d1(&function.surface(), uv.x(), uv.y())?;
        let mut tangent = Vector3::default();
        tangent.set_linear_form(d2.x(), derivative.d1u, d2.y(), derivative.d1v);
        let transition = transition_on_line(function, uv.x(), uv.y(), tangent)?;
        self.line.set_transition_on_s(transition);
        Ok(())
    }
    fn match_existing(
        &mut self,
        lines: &[ContapLine],
        ends: &mut SegmentEnds,
        tolerance: f64,
    ) -> Result<(), Failure> {
        if ends.first.is_none() && ends.last.is_none() {
            return Ok(());
        }
        for line in lines {
            let (mut first, mut last) = (false, false);
            for rank in 1..=line.nb_vertex() {
                self.point = line.vertex(rank)?;
                first |= self.match_point(line, rank, ends.first.as_ref(), tolerance)?;
                last |= self.match_point(line, rank, ends.last.as_ref(), tolerance)?;
            }
            if first {
                ends.first = None;
            }
            if last {
                ends.last = None;
            }
        }
        Ok(())
    }
    fn match_point(
        &mut self,
        line: &ContapLine,
        rank: i32,
        start: Option<&PathPoint>,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let Some(start) = start else {
            return Ok(false);
        };
        let matches = self.point.value().distance(&start.value()) <= tolerance;
        if !matches {
            return Ok(false);
        }
        line.change_vertex(rank, ContapPoint::set_multiple)?;
        self.point.set_multiple();
        self.point.set_parameter(start.parameter());
        self.line.add_vertex(&self.point);
        Ok(true)
    }
    fn add_endpoint(&mut self, start: &PathPoint) -> Result<(), Failure> {
        let uv = ContapHCurve2dTool::value(&self.line.arc()?, start.parameter())?;
        self.point.set_value(start.value(), uv.x(), uv.y());
        self.point.set_parameter(start.parameter());
        if !start.is_new() {
            self.point.set_vertex(&start.vertex()?);
        }
        self.line.add_vertex(&self.point);
        Ok(())
    }
}
