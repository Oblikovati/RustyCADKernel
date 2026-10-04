use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::{app_par_curves::Constraint, proj_lib::ProjectedCurve};
pub(super) struct ProjectionSettings {
    pub tolerance: f64,
    segments: i32,
    distance: f64,
    boundary: Constraint,
    extend: bool,
}
impl ProjectionSettings {
    pub fn new(input: f64, analytic: bool) -> Self {
        let confusion = precision::confusion();
        let maximum = 1e3 * confusion;
        let mut settings = Self {
            tolerance: confusion,
            segments: -1,
            distance: -1.,
            boundary: Constraint::TangencyPoint,
            extend: false,
        };
        if input >= 10. * confusion && (input <= maximum || analytic) {
            settings.tolerance = maximum.min(0.1 * input);
            settings.segments = 100;
            settings.distance = 1e3 * input;
            settings.extend = true;
            if !analytic || input > 1. {
                settings.boundary = Constraint::PassPoint;
            }
        } else if input > maximum {
            settings.tolerance = input.min(1e3 * maximum);
            settings.distance = 1e2 * settings.tolerance;
            settings.segments = 100;
            settings.extend = true;
        }
        settings
    }
    pub fn apply(
        &self,
        projected: &ProjectedCurve,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        if self.extend {
            extend_projection_surface(projected, surface)?;
        }
        projected.load_tolerance(self.tolerance);
        projected.set_degree(-1, -1);
        projected.set_max_segments(self.segments);
        projected.set_bnd_pnt(self.boundary);
        projected.set_max_dist(self.distance);
        Ok(())
    }
}
fn extend_projection_surface(
    projected: &ProjectedCurve,
    surface: &Handle<dyn Adaptor3dSurface>,
) -> Result<(), Failure> {
    let mut extended = Handle::default();
    let first = surface.first_u_parameter()?;
    let last = surface.last_u_parameter()?;
    let delta = last - first;
    if !surface.is_u_periodic()? || (delta - surface.u_period()?).abs() > 0.01 * delta {
        extended = surface.u_trim(first - 0.01 * delta, last + 0.01 * delta, 0.)?;
    }
    let first = surface.first_v_parameter()?;
    let last = surface.last_v_parameter()?;
    let delta = last - first;
    // OCCT trims V from the original adaptor, replacing (not composing with) the U trim.
    if !surface.is_v_periodic()? || (delta - surface.v_period()?).abs() > 0.01 * delta {
        extended = surface.v_trim(first - 0.01 * delta, last + 0.01 * delta, 0.)?;
    }
    if !extended.is_null() {
        projected.load_surface(&extended);
    }
    Ok(())
}
