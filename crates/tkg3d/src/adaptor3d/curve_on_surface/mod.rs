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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_CurveOnSurface.hxx/.cxx (construction, Load, accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_CurveOnSurface`: the 3D curve `S(C(t))` of a 2D curve `C` on an
//! adapted surface `S`.
//!
//! Loading recognises the lines and circles that the curve traces on elementary
//! surfaces (`EvalKPart`) and, on B-spline, swept and offset surfaces, trims the
//! surface to the knot patches at the curve ends (`EvalFirstLastSurf`), whose
//! evaluation there avoids a derivative jump. The native class also declares a
//! private `FindBounds` member that it never defines; the file-local function of
//! the same name is the one used.
mod curve;
mod eval;
mod intervals;
mod kpart;
mod locate;
mod locate_patch;
use super::curve::Adaptor3dCurve;
use super::surface::Adaptor3dSurface;
use crate::geom::errors::null_object;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{Circle3, Line3};

/// The fields of one `Adaptor3d_CurveOnSurface`.
#[derive(Clone)]
pub(super) struct CosState {
    pub(super) surface: Handle<dyn Adaptor3dSurface>,
    pub(super) curve: Handle<dyn Adaptor2dCurve2d>,
    pub(super) kind: CurveType,
    pub(super) circle: Circle3,
    pub(super) line: Line3,
    pub(super) first_surface: Handle<dyn Adaptor3dSurface>,
    pub(super) last_surface: Handle<dyn Adaptor3dSurface>,
    /// `myIntervals` (a shared immutable sequence natively) and `myIntCont`.
    pub(super) intervals: Option<Vec<f64>>,
    pub(super) interval_continuity: Shape,
}
impl Default for CosState {
    fn default() -> Self {
        Self {
            surface: Handle::default(),
            curve: Handle::default(),
            kind: CurveType::OtherCurve,
            circle: Circle3::default(),
            line: Line3::default(),
            first_surface: Handle::default(),
            last_surface: Handle::default(),
            intervals: None,
            interval_continuity: Shape::CN,
        }
    }
}
impl CosState {
    /// The adapted surface; natively a null one is dereferenced; `state.surface()?`.
    pub(super) fn surface(&self) -> Result<&dyn Adaptor3dSurface, Failure> {
        self.surface.get().ok_or_else(|| {
            null_object("Adaptor3d_CurveOnSurface: no surface is loaded; expected Load(S)")
        })
    }
    /// The 2D curve; natively a null one is dereferenced; `state.curve()?`.
    pub(super) fn curve(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        self.curve.get().ok_or_else(|| {
            null_object("Adaptor3d_CurveOnSurface: no 2D curve is loaded; expected Load(C)")
        })
    }
}

/// A 2D curve on an adapted surface seen as a 3D curve (`Adaptor3d_CurveOnSurface`);
/// `Adaptor3dCurveOnSurface::with_curve(&curve2d, &surface)?`.
#[derive(Default)]
pub struct Adaptor3dCurveOnSurface {
    parent: StandardTransient,
    state: SharedState<CosState>,
}
impl_transient!(
    Adaptor3dCurveOnSurface,
    "Adaptor3d_CurveOnSurface",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl Adaptor3dCurveOnSurface {
    /// An adaptor on `surface` without a curve (`Adaptor3d_CurveOnSurface(S)`);
    /// `Adaptor3dCurveOnSurface::with_surface(&surface)?`.
    pub fn with_surface(surface: &Handle<dyn Adaptor3dSurface>) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load_surface(surface)?;
        Ok(adaptor)
    }
    /// The curve `curve` on `surface` (`Adaptor3d_CurveOnSurface(C, S)`: Load(S) then
    /// Load(C)); `Adaptor3dCurveOnSurface::with_curve(&curve2d, &surface)?`.
    pub fn with_curve(
        curve: &Handle<dyn Adaptor2dCurve2d>,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.load_surface(surface)?;
        adaptor.load_curve(curve)?;
        Ok(adaptor)
    }
    /// The implicit copy constructor (`Adaptor3d_CurveOnSurface(const
    /// Adaptor3d_CurveOnSurface&)`): an adaptor with the same fields, sharing the
    /// curve and surface handles, whose later `Load` does not change `other`;
    /// `Adaptor3dCurveOnSurface::copy_of(&adaptor)`.
    pub fn copy_of(other: &Self) -> Self {
        Self::from_state(other.snapshot())
    }
    fn from_state(state: CosState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// A copy of the fields: members call the curve and surfaces without this lock.
    pub(super) fn snapshot(&self) -> CosState {
        self.state.snapshot()
    }
    /// Set the surface, re-deriving the special kind when a curve is loaded
    /// (`Load(S)`); `adaptor.load_surface(&surface)?`.
    pub fn load_surface(&self, surface: &Handle<dyn Adaptor3dSurface>) -> Result<(), Failure> {
        self.state.write().surface = surface.clone();
        if self.state.read().curve.is_some() {
            self.eval_k_part()?;
        }
        Ok(())
    }
    /// Set the 2D curve; with a surface, re-derive the special kind and, on a
    /// B-spline, swept or offset (of such a basis) surface, the end patches
    /// (`Load(C)`); `adaptor.load_curve(&curve2d)?`.
    pub fn load_curve(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<(), Failure> {
        self.state.write().curve = curve.clone();
        let surface: Handle<dyn Adaptor3dSurface> = self.state.read().surface.clone();
        let Some(adapted) = surface.get() else {
            return Ok(());
        };
        self.eval_k_part()?;
        let mut kind: SurfaceType = adapted.get_type()?;
        if kind == SurfaceType::OffsetSurface {
            kind = adapted.basis_surface()?.get_type()?;
        }
        if matches!(
            kind,
            SurfaceType::BSplineSurface
                | SurfaceType::SurfaceOfExtrusion
                | SurfaceType::SurfaceOfRevolution
        ) {
            self.eval_first_last_surf()?;
        }
        Ok(())
    }
    /// `Load(C, S)`: Load(C) then Load(S); `adaptor.load(&curve2d, &surface)?`.
    pub fn load(
        &self,
        curve: &Handle<dyn Adaptor2dCurve2d>,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        self.load_curve(curve)?;
        self.load_surface(surface)
    }
    /// The 2D curve (`GetCurve`); `adaptor.get_curve()`.
    pub fn get_curve(&self) -> Handle<dyn Adaptor2dCurve2d> {
        self.state.read().curve.clone()
    }
    /// The surface (`GetSurface`); `adaptor.get_surface()`.
    pub fn get_surface(&self) -> Handle<dyn Adaptor3dSurface> {
        self.state.read().surface.clone()
    }
    /// Change the 2D curve handle in place, without re-deriving anything
    /// (`ChangeCurve`, which natively returns a reference to the member);
    /// `adaptor.change_curve(|curve| *curve = other.clone())`.
    pub fn change_curve<R>(
        &self,
        change: impl FnOnce(&mut Handle<dyn Adaptor2dCurve2d>) -> R,
    ) -> R {
        change(&mut self.state.write().curve)
    }
    /// Change the surface handle in place, without re-deriving anything
    /// (`ChangeSurface`); `adaptor.change_surface(|surface| *surface = other.clone())`.
    pub fn change_surface<R>(
        &self,
        change: impl FnOnce(&mut Handle<dyn Adaptor3dSurface>) -> R,
    ) -> R {
        change(&mut self.state.write().surface)
    }
}
