//! Native non-elementary contour orchestration; tangency analysis is an injected private dependency.
use super::{
    analytic::AnalyticContour,
    boundary_points::keep_inside_points,
    general_join::{RestrictionJoin, mark_multiple},
    general_settings::GeneralSettings,
    process_segments::process_segments,
    walking_endpoints::{EndpointBuilder, EndpointState},
    walking_lines::WalkingLines,
};
use crate::contap::{
    ContapSurfFunction, ContapTFunction, ContapTheIWalking, ContapTheSearch, ContapTheSearchInside,
};
use rustycad_tkernel::{ncollection::Array1, precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as surface_tool, Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::{
    int_start::TheTopolTool,
    int_surf::{IntSurfInteriorPoint, IntSurfPathPoint},
};
use rustycad_tkmath::geom_abs::SurfaceType;
pub(super) type TangencySearch = fn(
    &ContapTheSearch,
    &(dyn TheTopolTool + 'static),
    &mut ContapSurfFunction,
    &mut Vec<IntSurfPathPoint>,
    &mut Array1<'static, i32>,
) -> Result<(), Failure>;
pub(super) struct GeneralContour<'a> {
    pub state: &'a mut AnalyticContour,
    pub inside: &'a mut ContapTheSearchInside,
    pub domain: &'a (dyn TheTopolTool + 'static),
    pub diagnostics: &'a mut dyn std::io::Write,
    pub tangency: TangencySearch,
}
struct StartingPoints {
    boundary: Vec<IntSurfPathPoint>,
    interior: Vec<IntSurfInteriorPoint>,
    destination: Array1<'static, i32>,
}
impl GeneralContour<'_> {
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.state.done = false;
        self.state.lines.clear();
        let settings = GeneralSettings::new(&self.state.function.surface())?;
        self.search_restrictions(&settings)?;
        if !self.state.search.is_done() {
            return Ok(());
        }
        let starts = self.starting_points()?;
        let mut endpoints = EndpointState::default();
        if !self.walk_and_join(&starts, &settings, &mut endpoints)? {
            return Ok(());
        }
        self.finish_restrictions(&settings, &mut endpoints)?;
        self.state.done = true;
        Ok(())
    }
    fn walk_and_join(
        &mut self,
        starts: &StartingPoints,
        settings: &GeneralSettings,
        endpoints: &mut EndpointState,
    ) -> Result<bool, Failure> {
        if !starts.boundary.is_empty() || !starts.interior.is_empty() {
            if !self.walk(starts, settings, endpoints)? {
                return Ok(false);
            }
            mark_multiple(&self.state.lines, settings.arc_tolerance)?;
        }
        Ok(true)
    }
    fn search_restrictions(&mut self, settings: &GeneralSettings) -> Result<(), Failure> {
        self.state.function.set_tolerance(precision::confusion());
        self.state.search.perform(
            &mut self.state.arc_function,
            self.domain,
            settings.arc_tolerance,
            settings.arc_tolerance,
            true,
        )
    }
    fn starting_points(&mut self) -> Result<StartingPoints, Failure> {
        let (boundary, destination) = self.boundary_seeds()?;
        let surface = self.state.function.surface();
        self.inside.perform(
            &mut self.state.function,
            &surface,
            self.domain,
            precision::confusion(),
        )?;
        let mut interior = Vec::new();
        if self.inside.nb_points()? != 0 {
            self.keep_interior(&mut interior)?;
        }
        Ok(StartingPoints {
            boundary,
            interior,
            destination,
        })
    }
    fn boundary_seeds(&mut self) -> Result<(Vec<IntSurfPathPoint>, Array1<'static, i32>), Failure> {
        let count = self.state.search.nb_points()?;
        let mut destination =
            Array1::with_bounds(1, count + 1).expect("positive contour destination extent");
        destination.init(0);
        let mut boundary = Vec::new();
        if count != 0 {
            (self.tangency)(
                &self.state.search,
                self.domain,
                &mut self.state.function,
                &mut boundary,
                &mut destination,
            )?;
        }
        Ok((boundary, destination))
    }
    fn keep_interior(&self, output: &mut Vec<IntSurfInteriorPoint>) -> Result<(), Failure> {
        if keep_all_torus_points(&self.state.search, &self.state.function)? {
            for index in 1..=self.inside.nb_points()? {
                output.push(*self.inside.value(index)?);
            }
            return Ok(());
        }
        keep_inside_points(
            self.inside,
            &self.state.search,
            &self.state.function,
            output,
        )
    }
    fn walk(
        &mut self,
        starts: &StartingPoints,
        settings: &GeneralSettings,
        endpoints: &mut EndpointState,
    ) -> Result<bool, Failure> {
        let surface = self.state.function.surface();
        let mut walk =
            ContapTheIWalking::new(settings.precision, settings.deflection, 0.005, true)?;
        walk.perform(
            &starts.boundary,
            &starts.interior,
            &mut self.state.function,
            &surface,
            false,
        )?;
        if !walk.is_done() {
            return Ok(false);
        }
        self.append_walked_lines(&walk, starts, settings, endpoints, &surface)?;
        Ok(true)
    }
    fn append_walked_lines(
        &mut self,
        walk: &ContapTheIWalking,
        starts: &StartingPoints,
        settings: &GeneralSettings,
        endpoints: &mut EndpointState,
        surface: &Adaptor3dSurfaceHandle,
    ) -> Result<(), Failure> {
        let builder = EndpointBuilder {
            search: &self.state.search,
            paths: &starts.boundary,
            destination: &starts.destination,
            surface,
        };
        WalkingLines {
            function: &mut self.state.function,
            output: &mut self.state.lines,
            domain: self.domain,
            diagnostics: self.diagnostics,
            settings,
            builder,
        }
        .append(walk, endpoints)
    }
    fn finish_restrictions(
        &mut self,
        settings: &GeneralSettings,
        endpoints: &mut EndpointState,
    ) -> Result<(), Failure> {
        if self.state.search.nb_segments()? == 0 {
            return Ok(());
        }
        process_segments(
            &self.state.search,
            &mut self.state.lines,
            settings.arc_tolerance,
            &mut self.state.function,
            self.domain,
        )?;
        if self.state.search.nb_segments()? == 0 {
            return Ok(());
        }
        let surface = self.state.function.surface();
        let mut join = RestrictionJoin {
            surface: &surface,
            precision: settings.precision,
            scratch: &mut endpoints.first,
            line_transition: &mut endpoints.line_transition,
            arc_transition: &mut endpoints.arc_transition,
        };
        join.perform(&self.state.lines)
    }
}
fn keep_all_torus_points(
    search: &ContapTheSearch,
    function: &ContapSurfFunction,
) -> Result<bool, Failure> {
    if search.nb_segments()? > 0 || function.function_type() != ContapTFunction::ContourStd {
        return Ok(false);
    }
    let surface = function.surface();
    if surface_tool::get_type(&surface)? != SurfaceType::Torus {
        return Ok(false);
    }
    let torus = surface_tool::torus(&surface)?;
    Ok(torus.axis().direction().dot(function.direction()) < precision::confusion())
}
