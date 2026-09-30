// ParameterOutOfBoundary from IntTools_FaceFace.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Move a conic endpoint away from face boundaries, retaining the native iteration limit.
use super::Context;
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::GeomCurve, top_abs::State};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::gp::{Point2, Point3};
pub(super) struct BoundaryParameter {
    pub(super) parameter: f64,
    pub(super) other: f64,
    pub(super) increase: bool,
    pub(super) tolerance: f64,
}
impl BoundaryParameter {
    pub(super) fn compute(
        &self,
        curve: &Handle<dyn GeomCurve>,
        faces: &[Face; 2],
        context: &Context,
        parameter: &mut f64,
    ) -> Result<bool, Failure> {
        *parameter = self.parameter;
        let mut projectors = [projector(&faces[0])?, projector(&faces[1])?];
        let Some(current) = self.walk(curve, faces, &mut projectors, context)? else {
            return Ok(false);
        };
        *parameter = self.clamped(current);
        Ok(true)
    }
    fn walk(
        &self,
        curve: &Handle<dyn GeomCurve>,
        faces: &[Face; 2],
        projectors: &mut [ProjectPointOnSurf; 2],
        context: &Context,
    ) -> Result<Option<f64>, Failure> {
        let (mut current, mut state, mut iteration) = (self.parameter, State::On, 0);
        let step = self.step();
        while state == State::On {
            current = if self.increase {
                current + step
            } else {
                current - step
            };
            let point = curve.eval_d0(current)?;
            classify_pair(projectors, point, faces, context, &mut state)?;
            if iteration > 11 {
                break;
            }
            iteration += 1;
        }
        Ok(if iteration > 11 { None } else { Some(current) })
    }
    fn step(&self) -> f64 {
        let delta = self.tolerance * 0.1;
        if delta < precision::confusion() {
            precision::confusion()
        } else {
            delta
        }
    }
    fn clamped(&self, parameter: f64) -> f64 {
        if (self.increase && parameter >= self.other) || (!self.increase && parameter <= self.other)
        {
            self.other
        } else {
            parameter
        }
    }
}
fn projector(face: &Face) -> Result<ProjectPointOnSurf, Failure> {
    let surface = BRepTool::surface(face)?;
    let mut projector = ProjectPointOnSurf::new();
    projector.init_surface(&surface, surface.bounds(), ExtAlgo::Grad)?;
    Ok(projector)
}
fn classify(
    projector: &mut ProjectPointOnSurf,
    point: Point3,
    face: &Face,
    context: &Context,
    state: &mut State,
) -> Result<(), Failure> {
    projector.perform(point)?;
    if projector.is_done() {
        let (u, v) = projector.lower_distance_parameters()?;
        *state = context.state_point_face(face, Point2::new(u, v))?;
    }
    Ok(())
}

fn classify_pair(
    projectors: &mut [ProjectPointOnSurf; 2],
    point: Point3,
    faces: &[Face; 2],
    context: &Context,
    state: &mut State,
) -> Result<(), Failure> {
    classify(&mut projectors[0], point, &faces[0], context, state)?;
    if *state != State::On {
        classify(&mut projectors[1], point, &faces[1], context, state)?;
    }
    Ok(())
}
