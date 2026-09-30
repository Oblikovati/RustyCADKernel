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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_CurveConstraint.hxx/.cxx
// (the constructors and the stored fields) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_CurveConstraint`: a curve constraint of a plate. The evaluations are in
//! `curve_constraint_eval`.
use crate::law::LawFunction;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::geom_lprop::SLProps;

#[derive(Clone)]
pub(super) struct CurveConstraintState {
    pub(super) frontiere: Handle<Adaptor3dCurveOnSurface>,
    pub(super) nb_points: i32,
    pub(super) order: i32,
    pub(super) curve_3d: Handle<dyn Adaptor3dCurve>,
    pub(super) curve_2d: Handle<dyn Geom2dCurve>,
    pub(super) hcurve_2d: Handle<dyn Adaptor2dCurve2d>,
    pub(super) g0_crit: Handle<dyn LawFunction>,
    pub(super) g1_crit: Handle<dyn LawFunction>,
    pub(super) g2_crit: Handle<dyn LawFunction>,
    pub(super) const_g0: bool,
    pub(super) const_g1: bool,
    pub(super) const_g2: bool,
    pub(super) tol_dist: f64,
    pub(super) tol_ang: f64,
    pub(super) tol_curv: f64,
    pub(super) tol_u: f64,
    pub(super) tol_v: f64,
}

impl CurveConstraintState {
    /// The fields of the default constructor (`myTang`, never read, is left out).
    fn empty() -> Self {
        Self {
            frontiere: Handle::default(),
            nb_points: 0,
            order: 0,
            curve_3d: Handle::default(),
            curve_2d: Handle::default(),
            hcurve_2d: Handle::default(),
            g0_crit: Handle::default(),
            g1_crit: Handle::default(),
            g2_crit: Handle::default(),
            const_g0: false,
            const_g1: false,
            const_g2: false,
            tol_dist: 0.0,
            tol_ang: 0.0,
            tol_curv: 0.0,
            tol_u: 0.0,
            tol_v: 0.0,
        }
    }
}

/// A curve constraint of a plate: a 3d curve (order G0 or none) or a curve on a
/// surface (`Adaptor3d_CurveOnSurface` on a `GeomAdaptor_Surface`, order up to G2),
/// the number of points to sample on it and the tolerances or criterion laws
/// (`GeomPlate_CurveConstraint`). The builder changes the number of points and the 2d
/// curve of a shared constraint, so the fields sit behind a lock
/// (docs/geomalgo-design.md section 19).
///
/// ```
/// use rustycad_tkgeomalgo::geom_plate::GeomPlateCurveConstraint;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// let line: Handle<dyn GeomCurve> =
///     Handle::new(GeomLine::from_point(Point3::default(), Direction3::new(1.0, 0.0, 0.0).unwrap())).cast();
/// let curve: Handle<dyn Adaptor3dCurve> =
///     Handle::new(GeomAdaptorCurve::with_range(&line, 0.0, 2.0).unwrap()).cast();
/// let c = GeomPlateCurveConstraint::new(&curve, 0, 10, 1e-4, 0.01, 0.1).unwrap();
/// assert_eq!(c.length().unwrap(), 2.0);
/// ```
pub struct GeomPlateCurveConstraint {
    parent: StandardTransient,
    pub(super) state: SharedState<CurveConstraintState>,
    pub(super) lprop: SharedState<SLProps>,
}
impl_transient!(
    GeomPlateCurveConstraint,
    "GeomPlate_CurveConstraint",
    StandardTransient,
    parent
);

// The Standard_Failure the native members raise for a constraint without surface.
pub(super) fn not_on_surface() -> Failure {
    Failure::new("GeomPlate_CurveConstraint.cxx : Curve must be on a Surface")
}

impl GeomPlateCurveConstraint {
    fn from_parts(state: CurveConstraintState, lprop: SLProps) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
            lprop: SharedState::new(lprop),
        }
    }

    /// An empty constraint of order 0 and no point (`GeomPlate_CurveConstraint()`); its
    /// curve members raise Standard_NullObject (null dereferences natively).
    /// `GeomPlateCurveConstraint::empty()?`.
    pub fn empty() -> Result<Self, Failure> {
        Ok(Self::from_parts(
            CurveConstraintState::empty(),
            SLProps::without_surface(2, 1.0e-4)?,
        ))
    }

    /// The constraint of order `order` on `boundary` with `npt` points and the
    /// tolerances (`GeomPlate_CurveConstraint(Boundary, Order, NPt, TolDist, TolAng,
    /// TolCurv)`; the native defaults are 10, 1e-4, 0.01 and 0.1). A boundary that is an
    /// `Adaptor3d_CurveOnSurface` must lie on a `GeomAdaptor_Surface`. Standard_Failure
    /// for an order outside -1..=2 or another surface adaptor.
    /// `GeomPlateCurveConstraint::new(&curve, 0, 10, 1e-4, 0.01, 0.1)?`.
    pub fn new(
        boundary: &Handle<dyn Adaptor3dCurve>,
        order: i32,
        npt: i32,
        tol_dist: f64,
        tol_ang: f64,
        tol_curv: f64,
    ) -> Result<Self, Failure> {
        let mut lprop: SLProps = SLProps::without_surface(2, tol_dist)?;
        if !(-1..=2).contains(&order) {
            return Err(Failure::new(&format!(
                "GeomPlate : The continuity is not G0 G1 or G2; got order {order}"
            )));
        }
        let frontiere: Handle<Adaptor3dCurveOnSurface> = boundary.cast();
        let mut state = CurveConstraintState {
            frontiere: frontiere.clone(),
            nb_points: npt,
            order,
            const_g0: true,
            const_g1: true,
            const_g2: true,
            tol_dist,
            tol_ang,
            tol_curv,
            ..CurveConstraintState::empty()
        };
        match frontiere.get() {
            None => state.curve_3d = boundary.clone(),
            Some(on_surface) => lprop.set_surface(geom_surface_of(on_surface)?),
        }
        Ok(Self::from_parts(state, lprop))
    }

    /// Sets the order (`SetOrder`); `c.set_order(1)`.
    pub fn set_order(&self, order: i32) {
        self.state.write().order = order;
    }

    /// The order: -1 (none), 0, 1 or 2 (`Order`); `c.order()`.
    pub fn order(&self) -> i32 {
        self.state.read().order
    }

    /// The number of points (`NbPoints`); `c.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.state.read().nb_points
    }

    /// Sets the number of points (`SetNbPoints`); `c.set_nb_points(12)`.
    pub fn set_nb_points(&self, new_nb: i32) {
        self.state.write().nb_points = new_nb;
    }

    /// The distance criterion law, in place of the tolerance (`SetG0Criterion`);
    /// `c.set_g0_criterion(&law)`.
    pub fn set_g0_criterion(&self, g0_crit: &Handle<dyn LawFunction>) {
        let mut state = self.state.write();
        state.g0_crit = g0_crit.clone();
        state.const_g0 = false;
    }

    /// The angular criterion law (`SetG1Criterion`); Standard_Failure for a
    /// constraint without surface. `c.set_g1_criterion(&law)?`.
    pub fn set_g1_criterion(&self, g1_crit: &Handle<dyn LawFunction>) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.curve_3d.is_some() {
            return Err(not_on_surface());
        }
        state.g1_crit = g1_crit.clone();
        state.const_g1 = false;
        Ok(())
    }

    /// The curvature criterion law (`SetG2Criterion`); Standard_Failure for a
    /// constraint without surface. `c.set_g2_criterion(&law)?`.
    pub fn set_g2_criterion(&self, g2_crit: &Handle<dyn LawFunction>) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.curve_3d.is_some() {
            return Err(not_on_surface());
        }
        state.g2_crit = g2_crit.clone();
        state.const_g2 = false;
        Ok(())
    }

    /// The 2d curve on the plate's initial surface (`SetCurve2dOnSurf`);
    /// `c.set_curve2d_on_surf(&curve)`.
    pub fn set_curve2d_on_surf(&self, curve: &Handle<dyn Geom2dCurve>) {
        self.state.write().curve_2d = curve.clone();
    }

    /// The projection of the curve on the initial surface and its parametric
    /// tolerances (`SetProjectedCurve`); `c.set_projected_curve(&c2d, 1e-6, 1e-6)`.
    pub fn set_projected_curve(
        &self,
        curve: &Handle<dyn Adaptor2dCurve2d>,
        tol_u: f64,
        tol_v: f64,
    ) {
        let mut state = self.state.write();
        state.hcurve_2d = curve.clone();
        state.tol_u = tol_u;
        state.tol_v = tol_v;
    }

    /// The projected curve, null until set (`ProjectedCurve`); `c.projected_curve()`.
    pub fn projected_curve(&self) -> Handle<dyn Adaptor2dCurve2d> {
        self.state.read().hcurve_2d.clone()
    }

    /// The 3d curve: the boundary itself (`Curve3d`); `c.curve3d()`.
    pub fn curve3d(&self) -> Handle<dyn Adaptor3dCurve> {
        let state = self.state.read();
        if state.curve_3d.is_null() {
            return state.frontiere.cast();
        }
        state.curve_3d.clone()
    }
}

/// The `Geom_Surface` of a curve on surface: its surface adaptor must be a
/// `GeomAdaptor_Surface` (Standard_Failure otherwise).
fn geom_surface_of(
    on_surface: &Adaptor3dCurveOnSurface,
) -> Result<Handle<dyn rustycad_tkg3d::geom::GeomSurface>, Failure> {
    let adaptor: Handle<GeomAdaptorSurface> = on_surface.get_surface().cast();
    let geom = adaptor.get().ok_or_else(|| {
        Failure::new("GeomPlate_CurveConstraint : Surface must be GeomAdaptor_Surface")
    })?;
    Ok(geom.surface())
}
