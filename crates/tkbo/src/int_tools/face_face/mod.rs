// IntTools_FaceFace.hxx/.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Face intersection state and the native curve preparation policy.
use super::{Context, Curve, PntOn2Faces, Tools};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
mod prepare;
/// `IntTools_FaceFace`; `FaceFace::default().is_done() == false`.
pub struct FaceFace {
    done: bool,
    tangent: bool,
    surfaces: [Handle<GeomAdaptorSurface>; 2],
    faces: [Face; 2],
    curves: Vec<Curve>,
    points: Vec<PntOn2Faces>,
    context: Handle<Context>,
    approximation: [bool; 3],
    approximation_tolerance: f64,
    fuzzy: f64,
    seeds: Vec<IntSurfPntOn2S>,
    kernel: perform::IntersectionKernel,
}
impl Default for FaceFace {
    fn default() -> Self {
        Self {
            done: false,
            tangent: false,
            surfaces: std::array::from_fn(|_| Handle::new(GeomAdaptorSurface::default())),
            faces: std::array::from_fn(|_| Face::default()),
            curves: Vec::new(),
            points: Vec::new(),
            context: Handle::default(),
            approximation: [true; 3],
            approximation_tolerance: 1e-7,
            fuzzy: precision::confusion(),
            seeds: Vec::new(),
            kernel: perform::IntersectionKernel::default(),
        }
    }
}
impl FaceFace {
    /// Sets approximation policy without resetting results; `ff.set_parameters(true, true, false, 1e-7)`.
    pub fn set_parameters(&mut self, spatial: bool, first: bool, second: bool, tolerance: f64) {
        self.approximation = [spatial, first, second];
        self.approximation_tolerance = tolerance;
    }
    /// Reports intersection completion; `ff.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Borrows intersection curves after completion; `ff.lines()?`.
    pub fn lines(&self) -> Result<&[Curve], Failure> {
        if !self.done {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "IntTools_FaceFace::Lines() => myIntersector NOT DONE",
                "",
            ));
        }
        Ok(&self.curves)
    }
    /// Borrows isolated points without a done-state check; `ff.points()`.
    pub fn points(&self) -> &[PntOn2Faces] {
        &self.points
    }
    /// Borrows the first internally ordered face; `ff.face1()`.
    pub fn face1(&self) -> &Face {
        &self.faces[0]
    }
    /// Borrows the second internally ordered face; `ff.face2()`.
    pub fn face2(&self) -> &Face {
        &self.faces[1]
    }
    /// Reports the intersector's tangency result; `ff.tangent_faces()`.
    pub fn tangent_faces(&self) -> bool {
        self.tangent
    }
    /// Copies walking start points; `ff.set_list(&points)`.
    pub fn set_list(&mut self, points: &[IntSurfPntOn2S]) {
        self.seeds = points.to_vec();
    }
    /// Shares geometric caches, including a null handle; `ff.set_context(&context)`.
    pub fn set_context(&mut self, context: &Handle<Context>) {
        self.context = context.clone();
    }
    /// Borrows the shared context handle; `ff.context()`.
    pub fn context(&self) -> &Handle<Context> {
        &self.context
    }
    /// Floors fuzz at Confusion while retaining native NaN ordering; `ff.set_fuzzy_value(0.0)`.
    pub fn set_fuzzy_value(&mut self, fuzzy: f64) {
        self.fuzzy = if fuzzy < precision::confusion() {
            precision::confusion()
        } else {
            fuzzy
        };
    }
    /// Reads the additional intersection tolerance; `ff.fuzzy_value()`.
    pub fn fuzzy_value(&self) -> f64 {
        self.fuzzy
    }
}
#[cfg(test)]
mod tests;

mod analytic;
mod boundaries;
mod boundary_parameter;
mod closed_conics;
mod conic_support;
mod curves;
mod distance;
mod open_conics;
mod pcurve_checks;
mod perform;
mod plane_clip;
mod planes;
mod points;
mod restriction;
mod surface_policy;
mod tolerance;
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "BOPAlgo_FaceFace wrapper calls this in its follow-on PerformFF wiring"
    )
)]
mod transform_results;
mod walking;
mod walking_fit;
mod walking_result;
mod walking_spline;
