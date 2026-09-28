// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Surface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAdaptor_Surface`: a Geom surface seen through the Adaptor3d interface.
use super::surface_eval::{self as eval, evaluate};
use super::surface_eval_order::{EvalD0, EvalD1, EvalD2, EvalD3, SurfaceEvalOrder};
use super::surface_query as query;
use super::surface_shape::{self as shape, Direction};
use super::surface_state::{self as state, LoadRange, SurfaceState};
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::{
    BasisSurfaceEvaluator, GeomBSplineSurface, GeomBezierSurface, GeomSurface, GeomSurfaceD1,
    GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// A Geom surface on `[U1, U2] x [V1, V2]` for the algorithms that take an adaptor
/// (`GeomAdaptor_Surface`).
///
/// Loading records the surface kind and the elementary geometry; a rectangular
/// trimmed surface is replaced by its basis, swept surfaces get an adaptor on their
/// basis curve and offset surfaces adaptors on their basis and on the equivalent
/// canonical surface. Bezier and B-spline surfaces are evaluated through a cached
/// span polynomial (`BSplSLib_Cache`), rebuilt under this object's lock. A
/// parameter within `TolU`/`TolV` of a bound is evaluated at the bound;
/// `GeomAdaptorSurface::with_bounds(&surface, (0.0, 1.0, 0.0, 1.0))?`.
#[derive(Default)]
pub struct GeomAdaptorSurface {
    parent: StandardTransient,
    state: SharedState<SurfaceState>,
}
impl_transient!(
    GeomAdaptorSurface,
    "GeomAdaptor_Surface",
    dyn Adaptor3dSurface,
    parent,
    interfaces: [dyn Adaptor3dSurface]
);

impl GeomAdaptorSurface {
    /// An adaptor on `surface` over its own bounds (`GeomAdaptor_Surface(S)`);
    /// Standard_NullObject for a null surface; `GeomAdaptorSurface::with_surface(&s)?`.
    pub fn with_surface(surface: &Handle<dyn GeomSurface>) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load(surface)?;
        Ok(adaptor)
    }
    /// An adaptor on `surface` over `(U1, U2, V1, V2)` with zero tolerances
    /// (`GeomAdaptor_Surface(S, U1, U2, V1, V2)`); the `Load` checks apply;
    /// `GeomAdaptorSurface::with_bounds(&s, (0.0, 1.0, 0.0, 1.0))?`.
    pub fn with_bounds(
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
    ) -> Result<Self, Failure> {
        Self::with_range(surface, (bounds, (0.0, 0.0)))
    }
    /// An adaptor on `surface` over `((U1, U2, V1, V2), (TolU, TolV))`
    /// (`GeomAdaptor_Surface(S, U1, U2, V1, V2, TolU, TolV)`); the `Load` checks apply;
    /// `GeomAdaptorSurface::with_range(&s, ((0.0, 1.0, 0.0, 1.0), (1e-7, 1e-7)))?`.
    pub fn with_range(
        surface: &Handle<dyn GeomSurface>,
        range: LoadRange,
    ) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load_range(surface, range)?;
        Ok(adaptor)
    }
    /// Load `surface` over its own bounds with zero tolerances (`Load(S)`);
    /// Standard_NullObject for a null surface; `adaptor.load(&surface)?`.
    pub fn load(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        state::load_whole(&mut self.state.write(), surface)
    }
    /// Load `surface` over `(U1, U2, V1, V2)` with zero tolerances
    /// (`Load(S, U1, U2, V1, V2)`); `adaptor.load_bounds(&surface, (0.0, 1.0, 0.0, 1.0))?`.
    pub fn load_bounds(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
    ) -> Result<(), Failure> {
        self.load_range(surface, (bounds, (0.0, 0.0)))
    }
    /// Load `surface` over `((U1, U2, V1, V2), (TolU, TolV))`
    /// (`Load(S, U1, U2, V1, V2, TolU, TolV)`); Standard_NullObject for a null surface,
    /// Standard_ConstructionError when `U1 > U2` or `V1 > V2`. Reloading the loaded
    /// surface changes only the bounds and tolerances;
    /// `adaptor.load_range(&surface, ((0.0, 1.0, 0.0, 1.0), (0.0, 0.0)))?`.
    pub fn load_range(
        &self,
        surface: &Handle<dyn GeomSurface>,
        range: LoadRange,
    ) -> Result<(), Failure> {
        state::load_range(&mut self.state.write(), surface, range)
    }
    /// The loaded surface, the basis of a loaded trimmed surface (`Surface`);
    /// `adaptor.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.state.read().surface.clone()
    }
    /// `(FirstU, LastU, FirstV, LastV)` of this adaptor (`Bounds`, non-virtual, calling
    /// the virtual bounds); `let (u1, u2, v1, v2) = adaptor.bounds()?;`.
    pub fn bounds(&self) -> Result<SurfaceBounds, Failure> {
        Ok((
            self.first_u_parameter()?,
            self.last_u_parameter()?,
            self.first_v_parameter()?,
            self.last_v_parameter()?,
        ))
    }
    /// The U tolerance of the bound snapping (`ToleranceU`); `adaptor.tolerance_u()`.
    pub fn tolerance_u(&self) -> f64 {
        self.state.read().tol_u
    }
    /// The V tolerance of the bound snapping (`ToleranceV`); `adaptor.tolerance_v()`.
    pub fn tolerance_v(&self) -> f64 {
        self.state.read().tol_v
    }
    /// A copy sharing the surface, with shallow copies of the nested adaptors and no
    /// caches (`ShallowCopy`, as a value); `adaptor.shallow_adaptor()?`.
    pub fn shallow_adaptor(&self) -> Result<Self, Failure> {
        let source = self.state.read();
        Ok(Self::from_state(state::shallow_state(&source)?))
    }
    /// An adaptor holding `state` (the derived adaptors build their copies with it).
    pub(crate) fn from_state(state: SurfaceState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// The protected state, for the derived adaptors.
    pub(crate) fn cell(&self) -> &SharedState<SurfaceState> {
        &self.state
    }
    /// One `EvalDk` of the native switch (for the equivalent-surface shortcut of
    /// the offset evaluators); `adaptor.evaluate::<EvalD1>(u, v)?`.
    pub(super) fn evaluate<O: SurfaceEvalOrder>(
        &self,
        u: f64,
        v: f64,
    ) -> Result<O::Output, Failure> {
        evaluate::<O>(&mut self.state.write(), u, v)
    }
    /// `EvalDN` on this adaptor (the equivalent-surface shortcut of offsetDN).
    pub(super) fn dn_at(&self, u: f64, v: f64, orders: (i32, i32)) -> Result<Vector3, Failure> {
        eval::dn(&self.state.read(), u, v, orders)
    }
}

impl Adaptor3dSurface for GeomAdaptorSurface {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Ok(Handle::new(self.shallow_adaptor()?).cast())
    }
    fn first_u_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().u_first)
    }
    fn last_u_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().u_last)
    }
    fn first_v_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().v_first)
    }
    fn last_v_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().v_last)
    }
    fn u_continuity(&self) -> Result<Shape, Failure> {
        shape::continuity(&self.state.read(), Direction::U)
    }
    fn v_continuity(&self) -> Result<Shape, Failure> {
        shape::continuity(&self.state.read(), Direction::V)
    }
    fn nb_u_intervals(&self, continuity: Shape) -> Result<i32, Failure> {
        shape::nb_intervals(&self.state.read(), Direction::U, continuity)
    }
    fn nb_v_intervals(&self, continuity: Shape) -> Result<i32, Failure> {
        shape::nb_intervals(&self.state.read(), Direction::V, continuity)
    }
    fn u_intervals(&self, bounds: &mut Array1<'_, f64>, continuity: Shape) -> Result<(), Failure> {
        shape::intervals(&self.state.read(), Direction::U, bounds, continuity)
    }
    fn v_intervals(&self, bounds: &mut Array1<'_, f64>, continuity: Shape) -> Result<(), Failure> {
        shape::intervals(&self.state.read(), Direction::V, bounds, continuity)
    }
    fn u_trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let (surface, range) = {
            let state = self.state.read();
            let range: LoadRange = (
                (first, last, state.v_first, state.v_last),
                (tolerance, state.tol_v),
            );
            (state.surface.clone(), range)
        };
        Ok(Handle::new(Self::with_range(&surface, range)?).cast())
    }
    fn v_trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let (surface, range) = {
            let state = self.state.read();
            let range: LoadRange = (
                (state.u_first, state.u_last, first, last),
                (state.tol_u, tolerance),
            );
            (state.surface.clone(), range)
        };
        Ok(Handle::new(Self::with_range(&surface, range)?).cast())
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        shape::is_closed(&self.state.read(), Direction::U)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        shape::is_closed(&self.state.read(), Direction::V)
    }
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        Ok(self.state.read().surface()?.is_u_periodic())
    }
    fn u_period(&self) -> Result<f64, Failure> {
        shape::period(&self.state.read(), Direction::U)
    }
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        Ok(self.state.read().surface()?.is_v_periodic())
    }
    fn v_period(&self) -> Result<f64, Failure> {
        shape::period(&self.state.read(), Direction::V)
    }
    fn u_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        query::u_resolution(&self.state.read(), r3d)
    }
    fn v_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        query::v_resolution(&self.state.read(), r3d)
    }
    fn get_type(&self) -> Result<SurfaceType, Failure> {
        Ok(self.state.read().surface_type)
    }
    fn plane(&self) -> Result<Plane3, Failure> {
        query::plane(&self.state.read())
    }
    fn cylinder(&self) -> Result<Cylinder3, Failure> {
        query::cylinder(&self.state.read())
    }
    fn cone(&self) -> Result<Cone3, Failure> {
        query::cone(&self.state.read())
    }
    fn sphere(&self) -> Result<Sphere3, Failure> {
        query::sphere(&self.state.read())
    }
    fn torus(&self) -> Result<Torus3, Failure> {
        query::torus(&self.state.read())
    }
    fn u_degree(&self) -> Result<i32, Failure> {
        query::degree(&self.state.read(), Direction::U)
    }
    fn nb_u_poles(&self) -> Result<i32, Failure> {
        query::nb_poles(&self.state.read(), Direction::U)
    }
    fn v_degree(&self) -> Result<i32, Failure> {
        query::degree(&self.state.read(), Direction::V)
    }
    fn nb_v_poles(&self) -> Result<i32, Failure> {
        query::nb_poles(&self.state.read(), Direction::V)
    }
    fn nb_u_knots(&self) -> Result<i32, Failure> {
        query::nb_knots(&self.state.read(), Direction::U)
    }
    fn nb_v_knots(&self) -> Result<i32, Failure> {
        query::nb_knots(&self.state.read(), Direction::V)
    }
    fn is_u_rational(&self) -> Result<bool, Failure> {
        Ok(query::is_rational(&self.state.read(), Direction::U))
    }
    fn is_v_rational(&self) -> Result<bool, Failure> {
        Ok(query::is_rational(&self.state.read(), Direction::V))
    }
    fn bezier(&self) -> Result<Handle<GeomBezierSurface>, Failure> {
        query::bezier(&self.state.read())
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        query::bspline(&self.state.read())
    }
    fn axe_of_revolution(&self) -> Result<Axis1, Failure> {
        query::axe_of_revolution(&self.state.read())
    }
    fn direction(&self) -> Result<Direction3, Failure> {
        query::direction(&self.state.read())
    }
    fn basis_curve(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        query::basis_curve(&self.state.read())
    }
    fn basis_surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let basis: Self = query::basis_surface_adaptor(&self.state.read())?;
        Ok(Handle::new(basis).cast())
    }
    fn offset_value(&self) -> Result<f64, Failure> {
        query::offset_value(&self.state.read())
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.evaluate::<EvalD0>(u, v)
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.evaluate::<EvalD1>(u, v)
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.evaluate::<EvalD2>(u, v)
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.evaluate::<EvalD3>(u, v)
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        self.dn_at(u, v, (nu, nv))
    }
}

// The basis adaptor of an offset adaptor is the basis of Geom_OffsetSurfaceUtils, as
// `theData.BasisAdaptor` is natively: its Bounds are the adaptor bounds.
impl BasisSurfaceEvaluator for GeomAdaptorSurface {
    fn basis_bounds(&self) -> Result<SurfaceBounds, Failure> {
        self.bounds()
    }
    fn basis_u_periodic(&self) -> Result<bool, Failure> {
        self.is_u_periodic()
    }
    fn basis_v_periodic(&self) -> Result<bool, Failure> {
        self.is_v_periodic()
    }
    fn basis_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.eval_d1(u, v)
    }
    fn basis_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.eval_d2(u, v)
    }
    fn basis_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.eval_d3(u, v)
    }
    fn basis_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        self.eval_dn(u, v, nu, nv)
    }
}
