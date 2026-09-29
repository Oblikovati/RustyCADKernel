// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_ProjectedCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_ProjectedCurve`: the 2D curve of a 3D curve projected on a surface, as
//! a 2D adaptor. The exact projectors are tried first on elementary surfaces, then
//! the analytic approximation; Bezier and B-spline surfaces use the polar
//! approximation and other surfaces the approximation of `CompProjectedCurve`.
mod adaptor;
mod general;
mod perform;
mod periodic;
mod singular;

use super::projector::Projector;
use crate::app_par_curves::Constraint;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};

/// The fields of one `ProjLib_ProjectedCurve`.
#[derive(Clone)]
pub(super) struct ProjectedState {
    pub(super) tolerance: f64,
    pub(super) surface: Handle<dyn Adaptor3dSurface>,
    pub(super) curve: Handle<dyn Adaptor3dCurve>,
    pub(super) result: Projector,
    pub(super) deg_min: i32,
    pub(super) deg_max: i32,
    pub(super) max_segments: i32,
    pub(super) max_dist: f64,
    pub(super) bnd_pnt: Constraint,
}

impl ProjectedState {
    fn new(tolerance: f64) -> Self {
        Self {
            tolerance,
            surface: Handle::default(),
            curve: Handle::default(),
            result: Projector::new(),
            deg_min: -1,
            deg_max: -1,
            max_segments: -1,
            max_dist: -1.,
            bnd_pnt: Constraint::TangencyPoint,
        }
    }
}

/// `ProjLib_ProjectedCurve`: an `Adaptor2d_Curve2d` answering for the projected
/// curve. Its point evaluators are not overridden natively, so they raise
/// Standard_NotImplemented; the result is read through `GetType` and the typed
/// accessors (`Line`, `BSpline`...), or `proj_lib::make_pcurve_of_type`.
///
/// ```ignore
/// let projected = ProjectedCurve::with_tolerance(&surface, &curve, 1e-6)?;
/// let pcurve = rustycad_tkgeombase::proj_lib::make_pcurve_of_type(&projected)?;
/// ```
pub struct ProjectedCurve {
    parent: StandardTransient,
    state: SharedState<ProjectedState>,
}
impl_transient!(
    ProjectedCurve,
    "ProjLib_ProjectedCurve",
    dyn Adaptor2dCurve2d,
    parent,
    interfaces: [dyn Adaptor2dCurve2d]
);

impl Default for ProjectedCurve {
    fn default() -> Self {
        Self::from_state(ProjectedState::new(precision::confusion()))
    }
}

impl ProjectedCurve {
    fn from_state(state: ProjectedState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    pub(super) fn snapshot(&self) -> ProjectedState {
        self.state.read().clone()
    }
    /// `ProjLib_ProjectedCurve()`: tolerance Confusion, default options;
    /// `ProjectedCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_ProjectedCurve(S)`; `ProjectedCurve::with_surface(&surface)`.
    pub fn with_surface(surface: &Handle<dyn Adaptor3dSurface>) -> Self {
        let projected: Self = Self::new();
        projected.load_surface(surface);
        projected
    }
    /// `ProjLib_ProjectedCurve(S, C)`: performs with the tolerance Confusion;
    /// `ProjectedCurve::with_curve(&surface, &curve)?`.
    pub fn with_curve(
        surface: &Handle<dyn Adaptor3dSurface>,
        curve: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Self, Failure> {
        let projected: Self = Self::with_surface(surface);
        projected.perform(curve)?;
        Ok(projected)
    }
    /// `ProjLib_ProjectedCurve(S, C, Tol)`: performs with `max(tol, Confusion)`;
    /// `ProjectedCurve::with_tolerance(&surface, &curve, 1e-6)?`.
    pub fn with_tolerance(
        surface: &Handle<dyn Adaptor3dSurface>,
        curve: &Handle<dyn Adaptor3dCurve>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let projected: Self = Self::from_state(ProjectedState::new(tol.max(precision::confusion())));
        projected.load_surface(surface);
        projected.perform(curve)?;
        Ok(projected)
    }

    /// `ShallowCopy()` as a concrete adaptor: shallow copies of the surface and the
    /// curve, and the other fields; `projected.shallow_projection()?`.
    pub fn shallow_projection(&self) -> Result<ProjectedCurve, Failure> {
        let mut copy: ProjectedState = self.snapshot();
        if let Some(surface) = copy.surface.get() {
            copy.surface = surface.shallow_copy()?;
        }
        if let Some(curve) = copy.curve.get() {
            copy.curve = curve.shallow_copy()?;
        }
        Ok(Self::from_state(copy))
    }

    /// `Load(double)`: the tolerance of the projection; `projected.load_tolerance(1e-6)`.
    pub fn load_tolerance(&self, tolerance: f64) {
        self.state.write().tolerance = tolerance;
    }
    /// `Load(const occ::handle<Adaptor3d_Surface>&)`; `projected.load_surface(&surface)`.
    pub fn load_surface(&self, surface: &Handle<dyn Adaptor3dSurface>) {
        self.state.write().surface = surface.clone();
    }
    /// `SetDegree(theDegMin, theDegMax)`; `projected.set_degree(2, 8)`.
    pub fn set_degree(&self, deg_min: i32, deg_max: i32) {
        let mut state = self.state.write();
        (state.deg_min, state.deg_max) = (deg_min, deg_max);
    }
    /// `SetMaxSegments(theMaxSegments)`; `projected.set_max_segments(100)`.
    pub fn set_max_segments(&self, max_segments: i32) {
        self.state.write().max_segments = max_segments;
    }
    /// `SetBndPnt(theBndPnt)`; `projected.set_bnd_pnt(Constraint::PassPoint)`.
    pub fn set_bnd_pnt(&self, bnd_pnt: Constraint) {
        self.state.write().bnd_pnt = bnd_pnt;
    }
    /// `SetMaxDist(theMaxDist)`; `projected.set_max_dist(0.1)`.
    pub fn set_max_dist(&self, max_dist: f64) {
        self.state.write().max_dist = max_dist;
    }
    /// `GetSurface()`; `projected.get_surface()`.
    pub fn get_surface(&self) -> Handle<dyn Adaptor3dSurface> {
        self.state.read().surface.clone()
    }
    /// `GetCurve()`: the projected curve (trimmed near a pole); `projected.get_curve()`.
    pub fn get_curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.state.read().curve.clone()
    }
    /// `GetTolerance()`: the tolerance reached by an approximation;
    /// `projected.get_tolerance()`.
    pub fn get_tolerance(&self) -> f64 {
        self.state.read().tolerance
    }
    /// The projector holding the result (the native `myResult`);
    /// `projected.result()`.
    pub fn result(&self) -> Projector {
        self.state.read().result.clone()
    }

    /// `Perform(C)`: the projection of `curve` with the current options. The state
    /// is updated even when the computation fails part way, as natively;
    /// `projected.perform(&curve)?`.
    pub fn perform(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<(), Failure> {
        let mut state: ProjectedState = self.snapshot();
        let outcome = state.perform(curve);
        *self.state.write() = state;
        outcome
    }
}
