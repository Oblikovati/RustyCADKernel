//! Native Contap_Contour::LineConstructor at OCCT 3d097a0.
use super::math_helpers::{analytic_parameters, recadre};
use crate::contap::{ContapIType, ContapLine};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle, top_abs::State};
use rustycad_tkgeomalgo::{int_start::TheTopolTool, int_surf::IntSurfLineOn2S};
use rustycad_tkmath::{
    elclib,
    gp::{self, Point2, Point3},
};
struct SectionConstructor<'a> {
    domain: &'a dyn TheTopolTool,
    line: &'a ContapLine,
    surface: &'a Adaptor3dSurfaceHandle,
}
pub(super) fn construct_lines(
    output: &mut Vec<ContapLine>,
    domain: &dyn TheTopolTool,
    line: &ContapLine,
    surface: &Adaptor3dSurfaceHandle,
) -> Result<(), Failure> {
    let constructor = SectionConstructor {
        domain,
        line,
        surface,
    };
    match line.type_contour() {
        ContapIType::Walking => constructor.walking_sections(output),
        ContapIType::Lin | ContapIType::Circle => constructor.analytic_sections(output),
        _ => {
            output.push(line.clone());
            Ok(())
        }
    }
}
impl SectionConstructor<'_> {
    fn is_outside(&self, (mut u, mut v): (f64, f64)) -> Result<bool, Failure> {
        recadre(self.surface, &mut u, &mut v)?;
        Ok(self
            .domain
            .classify(Point2::new(u, v), precision::p_confusion(), true)?
            == State::Out)
    }
    fn walking_sections(&self, output: &mut Vec<ContapLine>) -> Result<(), Failure> {
        for rank in 1..self.line.nb_vertex() {
            let (first, last) = self.vertex_limits(rank, rank + 1)?;
            let (first, last) = (first as i32, last as i32);
            if first == last {
                continue;
            }
            let midpoint = self.line.point((first + last) / 2)?;
            if self.is_outside(midpoint.parameters_on_s2())? {
                continue;
            }
            self.append_walking(output, rank, first, last)?;
        }
        Ok(())
    }
    fn walking_points(&self, first: i32, last: i32) -> Result<Handle<IntSurfLineOn2S>, Failure> {
        let points = Handle::new(IntSurfLineOn2S::new(None));
        points.add(&self.line.point(first)?);
        for index in first + 1..=last {
            let point = self.line.point(index)?;
            let previous = self.line.point(index - 1)?.value();
            if point.value().square_distance(&previous) > gp::resolution() {
                points.add(&point);
            }
        }
        Ok(points)
    }
    fn append_walking(
        &self,
        output: &mut Vec<ContapLine>,
        rank: i32,
        first: i32,
        last: i32,
    ) -> Result<(), Failure> {
        let points = self.walking_points(first, last)?;
        if points.nb_points() < 2 {
            return Ok(());
        }
        let limits = (1., f64::from(points.nb_points()));
        let mut section = ContapLine::new();
        section.set_line_on_2s(points);
        self.append_vertices(&section, rank, rank + 1, Some(limits))?;
        section.set_transition_on_s(self.line.transition_on_s());
        output.push(section);
        Ok(())
    }
    fn append_vertices(
        &self,
        section: &ContapLine,
        first: i32,
        last: i32,
        parameters: Option<(f64, f64)>,
    ) -> Result<(), Failure> {
        let mut begin = self.line.vertex(first)?;
        let mut end = self.line.vertex(last)?;
        if let Some((a, b)) = parameters {
            begin.set_parameter(a);
            end.set_parameter(b);
        }
        section.add_vertex(&begin);
        section.add_vertex(&end);
        Ok(())
    }
    fn vertex_limits(&self, first: i32, last: i32) -> Result<(f64, f64), Failure> {
        let begin = self.line.vertex(first)?.parameter_on_line();
        let end = self.line.vertex(last)?.parameter_on_line();
        Ok((begin, end))
    }
    fn analytic_sections(&self, output: &mut Vec<ContapLine>) -> Result<(), Failure> {
        let circle = self.line.type_contour() == ContapIType::Circle;
        for rank in 1..self.line.nb_vertex() {
            let limits = self.vertex_limits(rank, rank + 1)?;
            if acceptable_span(limits, if circle { 1e-9 } else { 0. }) {
                self.analytic_section(output, limits, Some((rank, rank + 1, false)))?;
            }
        }
        if circle {
            self.circle_completion(output)?;
        }
        Ok(())
    }
    fn circle_completion(&self, output: &mut Vec<ContapLine>) -> Result<(), Failure> {
        let count = self.line.nb_vertex();
        let period = std::f64::consts::PI + std::f64::consts::PI;
        if count == 0 {
            return self.analytic_section(output, (0., period), None);
        }
        let (first, last) = self.vertex_limits(count, 1)?;
        let limits = (first, last + period);
        if acceptable_span(limits, 1e-10) {
            self.analytic_section(output, limits, Some((count, 1, true)))?;
        }
        Ok(())
    }
    fn analytic_geometry(&self, middle: f64) -> Result<(ContapLine, Point3), Failure> {
        let mut section = ContapLine::new();
        if self.line.type_contour() == ContapIType::Lin {
            let geometry = self.line.line()?;
            section.set_line(geometry);
            return Ok((section, elclib::value(middle, &geometry)));
        }
        let geometry = self.line.circle()?;
        section.set_circle(geometry);
        Ok((section, elclib::value(middle, &geometry)))
    }
    fn analytic_section(
        &self,
        output: &mut Vec<ContapLine>,
        limits: (f64, f64),
        vertices: Option<(i32, i32, bool)>,
    ) -> Result<(), Failure> {
        let (mut section, point) = self.analytic_geometry((limits.0 + limits.1) * 0.5)?;
        if self.is_outside(analytic_parameters(self.surface, point)?)? {
            return Ok(());
        }
        if let Some((first, last, wrap)) = vertices {
            self.append_vertices(
                &section,
                first,
                last,
                if wrap { Some(limits) } else { None },
            )?;
        }
        section.set_transition_on_s(self.line.transition_on_s());
        output.push(section);
        Ok(())
    }
}
fn acceptable_span((first, last): (f64, f64), tolerance: f64) -> bool {
    if tolerance == 0. {
        return first != last;
    }
    (first - last).abs() > tolerance
}
