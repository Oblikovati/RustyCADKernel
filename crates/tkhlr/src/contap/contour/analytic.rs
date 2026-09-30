//! Contap_Contour analytic branch, preserving native result and search state.
use super::{
    boundary_points::put_points_on_lines,
    line_constructor::construct_lines,
    math_helpers::{circle_transition, line_transition},
    process_segments::process_segments,
};
use crate::contap::{
    ContapArcFunction, ContapContAna, ContapIType, ContapLine, ContapSurfFunction, ContapTFunction,
    ContapTheSearch,
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::h_surface_tool as surface_tool;
use rustycad_tkgeomalgo::int_start::TheTopolTool;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
pub(super) struct AnalyticContour {
    pub done: bool,
    pub lines: Vec<ContapLine>,
    pub search: ContapTheSearch,
    pub function: ContapSurfFunction,
    pub arc_function: ContapArcFunction,
}
impl AnalyticContour {
    pub fn perform(&mut self, domain: &(dyn TheTopolTool + 'static)) -> Result<(), Failure> {
        self.done = false;
        self.lines.clear();
        let kind = surface_tool::get_type(&self.function.surface())?;
        if kind != SurfaceType::Plane && !self.initial_lines(kind)? {
            return Ok(());
        }
        if kind == SurfaceType::Plane && !self.plane_has_restrictions()? {
            self.done = true;
            return Ok(());
        }
        self.search
            .perform(&mut self.arc_function, domain, 1e-5, 1e-5, false)?;
        if !self.search.is_done() {
            return Ok(());
        }
        self.finish_restrictions(domain)?;
        self.done = true;
        Ok(())
    }
    fn initial_lines(&mut self, kind: SurfaceType) -> Result<bool, Failure> {
        let mut analytic = ContapContAna::new();
        match kind {
            SurfaceType::Sphere => self.sphere(&mut analytic)?,
            SurfaceType::Cylinder => self.cylinder(&mut analytic)?,
            SurfaceType::Cone => self.cone(&mut analytic)?,
            _ => (),
        }
        if !analytic.is_done() {
            return Ok(false);
        }
        if analytic.nb_contours()? == 0 {
            self.done = true;
            return Ok(false);
        }
        self.append_analytic(&analytic)?;
        Ok(true)
    }
    fn plane_has_restrictions(&self) -> Result<bool, Failure> {
        let plane = surface_tool::plane(&self.function.surface())?;
        let direction = self.function.direction();
        let normal = plane.axis().direction();
        let excluded = match self.function.function_type() {
            ContapTFunction::ContourStd => direction.dot(normal).abs() > precision::angular(),
            ContapTFunction::ContourPrs => {
                plane.distance(self.function.eye()) > precision::confusion()
            }
            ContapTFunction::DraftStd => {
                (direction.dot(normal) + self.function.angle().sin()).abs() > precision::angular()
            }
            _ => false,
        };
        Ok(!excluded)
    }
    fn append_analytic(&mut self, analytic: &ContapContAna) -> Result<(), Failure> {
        let mut line = ContapLine::new();
        if analytic.type_contour()? == CurveType::Circle {
            let circle = analytic.circle()?;
            line.set_circle(circle);
            line.set_transition_on_s(circle_transition(&mut self.function, circle)?);
            self.lines.push(line);
            return Ok(());
        }
        if analytic.type_contour()? != CurveType::Line {
            return Ok(());
        }
        for rank in 1..=analytic.nb_contours()? {
            let geometry = analytic.line(rank)?;
            line.set_line(geometry);
            line.set_transition_on_s(line_transition(&mut self.function, geometry)?);
            self.lines.push(line.clone());
            line.clear();
        }
        Ok(())
    }
    fn finish_restrictions(
        &mut self,
        domain: &(dyn TheTopolTool + 'static),
    ) -> Result<(), Failure> {
        let surface = self.function.surface();
        if self.search.nb_points()? != 0 {
            put_points_on_lines(&self.search, &surface, &self.lines)?;
        }
        if self.search.nb_segments()? != 0 {
            process_segments(
                &self.search,
                &mut self.lines,
                1e-5,
                &mut self.function,
                domain,
            )?;
        }
        let count = self.lines.len();
        let mut remove = Vec::new();
        for index in 0..count {
            if self.lines[index].type_contour() == ContapIType::Restriction {
                continue;
            }
            let line = self.lines[index].clone();
            construct_lines(&mut self.lines, domain, &line, &surface)?;
            remove.push(index);
        }
        for index in remove.into_iter().rev() {
            self.lines.remove(index);
        }
        Ok(())
    }
    fn sphere(&self, analytic: &mut ContapContAna) -> Result<(), Failure> {
        let sphere = surface_tool::sphere(&self.function.surface())?;
        match self.function.function_type() {
            ContapTFunction::ContourStd => {
                analytic.perform_sphere(&sphere, self.function.direction())
            }
            ContapTFunction::ContourPrs => {
                analytic.perform_sphere_eye(&sphere, self.function.eye())
            }
            ContapTFunction::DraftStd => analytic.perform_sphere_draft(
                &sphere,
                self.function.direction(),
                self.function.angle(),
            ),
            _ => Ok(()),
        }
    }
    fn cylinder(&self, analytic: &mut ContapContAna) -> Result<(), Failure> {
        let cylinder = surface_tool::cylinder(&self.function.surface())?;
        match self.function.function_type() {
            ContapTFunction::ContourStd => {
                analytic.perform_cylinder(&cylinder, self.function.direction())
            }
            ContapTFunction::ContourPrs => {
                analytic.perform_cylinder_eye(&cylinder, self.function.eye())
            }
            ContapTFunction::DraftStd => analytic.perform_cylinder_draft(
                &cylinder,
                self.function.direction(),
                self.function.angle(),
            ),
            _ => Ok(()),
        }
    }
    fn cone(&self, analytic: &mut ContapContAna) -> Result<(), Failure> {
        let cone = surface_tool::cone(&self.function.surface())?;
        match self.function.function_type() {
            ContapTFunction::ContourStd => analytic.perform_cone(&cone, self.function.direction()),
            ContapTFunction::ContourPrs => analytic.perform_cone_eye(&cone, self.function.eye()),
            ContapTFunction::DraftStd => {
                analytic.perform_cone_draft(&cone, self.function.direction(), self.function.angle())
            }
            _ => Ok(()),
        }
    }
}
