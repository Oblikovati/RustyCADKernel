//! Apparent contours with the pinned OCCT initialization and result-state rules.
mod analytic;
mod boundary_points;
mod constructors;
mod general;
mod general_join;
mod general_settings;
mod line_constructor;
mod math_helpers;
mod process_segments;
mod restriction_internal;
mod tangency;
mod walking_endpoints;
mod walking_internal;
mod walking_lines;
mod walking_root;
use self::{analytic::AnalyticContour, general::GeneralContour};
use super::{
    ContapArcFunction, ContapLine, ContapSurfFunction, ContapTheSearch, ContapTheSearchInside,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as surface_tool, Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_start::TheTopolTool;
use rustycad_tkmath::{
    geom_abs::SurfaceType,
    gp::{Direction3, Point3, Vector3},
};
/// Apparent contour search retaining OCCT's mutable surface function.
///
/// ```
/// use rustycad_tkhlr::contap::ContapContour;
/// assert!(!ContapContour::new().is_done());
/// ```
pub struct ContapContour {
    state: AnalyticContour,
    inside: ContapTheSearchInside,
    mode_set: bool,
}
impl Default for ContapContour {
    fn default() -> Self {
        Self::new()
    }
}
impl ContapContour {
    /// Create an uninitialized search; select a view before performing it.
    ///
    /// `let contour = ContapContour::new();`
    pub fn new() -> Self {
        Self {
            state: AnalyticContour {
                done: false,
                lines: Vec::new(),
                search: ContapTheSearch::new(),
                function: ContapSurfFunction::new(),
                arc_function: ContapArcFunction::new(),
            },
            inside: ContapTheSearchInside::new(),
            mode_set: false,
        }
    }
    /// Select a parallel view and invalidate the previous result.
    ///
    /// `contour.init_direction(Vector3::new(0., 0., 1.))?;`
    pub fn init_direction(&mut self, direction: Vector3) -> Result<(), Failure> {
        self.state.done = false;
        self.mode_set = true;
        let direction = Direction3::new(direction.x(), direction.y(), direction.z())?;
        self.state.function.set_direction(direction);
        self.state.arc_function.set_direction(direction);
        Ok(())
    }
    /// Select a draft view and invalidate the previous result.
    ///
    /// `contour.init_direction_angle(Vector3::new(0., 0., 1.), 0.2)?;`
    pub fn init_direction_angle(&mut self, direction: Vector3, angle: f64) -> Result<(), Failure> {
        self.state.done = false;
        self.mode_set = true;
        let direction = Direction3::new(direction.x(), direction.y(), direction.z())?;
        self.state.function.set_direction_angle(direction, angle);
        self.state
            .arc_function
            .set_direction_angle(direction, angle);
        Ok(())
    }
    /// Select a perspective eye and invalidate the previous result.
    ///
    /// `contour.init_eye(Point3::new(3., 4., 5.));`
    pub fn init_eye(&mut self, eye: Point3) {
        self.state.done = false;
        self.mode_set = true;
        self.state.function.set_eye(eye);
        self.state.arc_function.set_eye(eye);
    }
    /// Search a surface with the selected view and native diagnostic output.
    ///
    /// `contour.perform(&surface, &domain)?;`
    pub fn perform(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
    ) -> Result<(), Failure> {
        self.perform_with_diagnostics(surface, domain, &mut std::io::stdout().lock())
    }
    /// Search while routing native internal-point diagnostics to a supplied writer.
    ///
    /// `contour.perform_with_diagnostics(&surface, &domain, &mut messages)?;`
    pub fn perform_with_diagnostics(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        diagnostics: &mut dyn std::io::Write,
    ) -> Result<(), Failure> {
        self.prepare_surface(surface)?;
        match surface_tool::get_type(surface)? {
            SurfaceType::Plane
            | SurfaceType::Sphere
            | SurfaceType::Cylinder
            | SurfaceType::Cone => self.state.perform(domain),
            _ => GeneralContour {
                state: &mut self.state,
                inside: &mut self.inside,
                domain,
                diagnostics,
                tangency: tangency::compute_tangency,
            }
            .perform(),
        }
    }
    fn prepare_surface(&mut self, surface: &Adaptor3dSurfaceHandle) -> Result<(), Failure> {
        if !self.mode_set {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "Contour view is unset; expected direction, draft angle, or eye initialization",
            ));
        }
        self.state.function.set_surface(surface)?;
        self.state.arc_function.set_surface(surface)?;
        Ok(())
    }
    /// Return completion even when result access would raise NotDone.
    ///
    /// `assert!(!ContapContour::new().is_done());`
    pub fn is_done(&self) -> bool {
        self.state.done
    }
    /// Report whether a completed search has no contour lines.
    ///
    /// `let empty = contour.is_empty()?;`
    pub fn is_empty(&self) -> Result<bool, Failure> {
        Ok(self.nb_lines()? == 0)
    }
    /// Count result lines, raising NotDone before successful completion.
    ///
    /// `let count = contour.nb_lines()?;`
    pub fn nb_lines(&self) -> Result<i32, Failure> {
        self.require_done()?;
        Ok(self.state.lines.len() as i32)
    }
    /// Borrow a result using OCCT's one-based indexing.
    ///
    /// `let first = contour.line(1)?;`
    pub fn line(&self, index: i32) -> Result<&ContapLine, Failure> {
        self.require_done()?;
        self.state
            .lines
            .get(index.wrapping_sub(1) as usize)
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "Contour line {index}; expected 1..={}",
                    self.state.lines.len()
                ))
            })
    }
    /// Mutate the original internal function after successful completion.
    ///
    /// `contour.surface_function()?.set_tolerance(1e-7);`
    pub fn surface_function(&mut self) -> Result<&mut ContapSurfFunction, Failure> {
        self.require_done()?;
        Ok(&mut self.state.function)
    }
    fn require_done(&self) -> Result<(), Failure> {
        if !self.state.done {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "Contour completion is false; expected a completed search",
            ));
        }
        Ok(())
    }
}
