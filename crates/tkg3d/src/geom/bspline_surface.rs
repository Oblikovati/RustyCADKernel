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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface.hxx/.cxx, Geom_BSplineSurface_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BSplineSurface`: rational or non-rational, periodic or not, B-spline
//! surfaces. U runs along the rows of the pole grid and V along its columns.
mod access;
mod eval;
mod iso;
mod knots;
mod periodic;
mod poles;
mod segment;
mod state;
mod traits;
use super::bounded_surface::GeomBoundedSurface;
use super::errors;
use super::geometry::GeomGeometry;
use super::surface::GeomSurface;
use crate::geom_eval::{RepSurfaceDesc, rep_utils};
pub use eval::LocalKnots;
pub use poles::ModifiedPoles;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::{one_based, put, rebased};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, resolution};
use state::{Direction, SurfaceState, cell, check_surface_data, surface_rational, unit_weights};

/// A B-spline surface;
/// `Handle::new(GeomBSplineSurface::new(&poles, (&uk, &um), (&vk, &vm), (3, 3), (false, false))?)`.
pub struct GeomBSplineSurface {
    parent: StandardTransient,
    state: SharedState<SurfaceState>,
}
impl_transient!(
    GeomBSplineSurface,
    "Geom_BSplineSurface",
    dyn GeomBoundedSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomBoundedSurface]
);

/// The knots and multiplicities of one direction given to a constructor.
pub type KnotVector<'r, 'a> = (&'r Array1<'a, f64>, &'r Array1<'a, i32>);

// The weight checks of the rational constructor: sizes, then every weight above
// gp::Resolution().
fn check_weights(weights: &Array2<'_, f64>, poles: &Array2<'_, Point3>) -> Result<(), Failure> {
    for (name, given, expected) in [
        ("U", weights.nb_rows(), poles.nb_rows()),
        ("V", weights.nb_columns(), poles.nb_columns()),
    ] {
        if given != expected {
            return Err(errors::construction(&format!(
                "Geom_BSplineSurface: {given} {name} weights for {expected} {name} poles; \
                 {name} Weights and Poles array size mismatch"
            )));
        }
    }
    for i in weights.lower_row()..=weights.upper_row() {
        for j in weights.lower_col()..=weights.upper_col() {
            let weight: f64 = cell(weights, i, j)?;
            if weight <= resolution() {
                return Err(errors::construction(&format!(
                    "Geom_BSplineSurface: Weights({i}, {j}) = {weight}; Weights values too small"
                )));
            }
        }
    }
    Ok(())
}

// A one-based copy of a grid.
fn rebased_grid<T: Copy + Default>(source: &Array2<'_, T>) -> Result<Array2<'static, T>, Failure> {
    let mut result: Array2<'static, T> =
        state::grid(source.nb_rows() as i32, source.nb_columns() as i32)?;
    result.as_mut_slice().copy_from_slice(source.as_slice());
    Ok(result)
}

impl GeomBSplineSurface {
    fn from_state(state: SurfaceState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    // The shared tail of both constructors: one-based copies and `updateUKnots`/`updateVKnots`.
    fn stored(
        poles: &Array2<'_, Point3>,
        weights: Option<(&Array2<'_, f64>, (bool, bool))>,
        u: (KnotVector<'_, '_>, i32, bool),
        v: (KnotVector<'_, '_>, i32, bool),
    ) -> Result<Self, Failure> {
        let (u_rational, v_rational) = weights.map_or((false, false), |(_, flags)| flags);
        let stored_weights: Array2<'static, f64> =
            match weights.filter(|_| u_rational || v_rational) {
                Some((given, _)) => rebased_grid(given)?,
                None => unit_weights(poles.nb_rows(), poles.nb_columns())?,
            };
        let mut state = SurfaceState {
            poles: rebased_grid(poles)?,
            weights: stored_weights,
            u: Direction::stored(rebased(u.0.0)?, rebased(u.0.1)?, u.1, u.2),
            v: Direction::stored(rebased(v.0.0)?, rebased(v.0.1)?, v.1, v.2),
            eval_rep: Handle::default(),
            u_rational,
            v_rational,
            u_max_deriv_inv: 0.0,
            v_max_deriv_inv: 0.0,
            max_deriv_inv_ok: false,
        };
        state.update_knots(true)?;
        state.update_knots(false)?;
        Ok(Self::from_state(state))
    }
    /// Non-rational surface (`Geom_BSplineSurface(Poles, UKnots, VKnots, UMults, VMults,
    /// UDegree, VDegree, UPeriodic, VPeriodic)`); Standard_ConstructionError for
    /// inconsistent data;
    /// `GeomBSplineSurface::new(&poles, (&uk, &um), (&vk, &vm), (3, 3), (false, false))?`.
    pub fn new(
        poles: &Array2<'_, Point3>,
        u: KnotVector<'_, '_>,
        v: KnotVector<'_, '_>,
        degrees: (i32, i32),
        periodic: (bool, bool),
    ) -> Result<Self, Failure> {
        check_surface_data(
            poles,
            (u.0, u.1, degrees.0, periodic.0),
            (v.0, v.1, degrees.1, periodic.1),
        )?;
        Self::stored(
            poles,
            None,
            (u, degrees.0, periodic.0),
            (v, degrees.1, periodic.1),
        )
    }
    /// Rational surface (`Geom_BSplineSurface(Poles, Weights, UKnots, VKnots, UMults,
    /// VMults, UDegree, VDegree, UPeriodic, VPeriodic)`): the weights are checked before
    /// the knot data, and a direction whose weights do not vary is not rational;
    /// `GeomBSplineSurface::with_weights(&poles, &w, (&uk, &um), (&vk, &vm), (2, 2), (false, false))?`.
    pub fn with_weights(
        poles: &Array2<'_, Point3>,
        weights: &Array2<'_, f64>,
        u: KnotVector<'_, '_>,
        v: KnotVector<'_, '_>,
        degrees: (i32, i32),
        periodic: (bool, bool),
    ) -> Result<Self, Failure> {
        check_weights(weights, poles)?;
        let flags: (bool, bool) = surface_rational(weights)?;
        check_surface_data(
            poles,
            (u.0, u.1, degrees.0, periodic.0),
            (v.0, v.1, degrees.1, periodic.1),
        )?;
        Self::stored(
            poles,
            Some((weights, flags)),
            (u, degrees.0, periodic.0),
            (v, degrees.1, periodic.1),
        )
    }
    /// The native copy constructor: shared unit weights for a non-rational surface, a
    /// deep-copied evaluation representation, the cached resolution values kept but
    /// marked stale (`Geom_BSplineSurface(const Geom_BSplineSurface&)`);
    /// `GeomBSplineSurface::copy_of(&surface)?`.
    pub fn copy_of(other: &GeomBSplineSurface) -> Result<Self, Failure> {
        let mut copy: SurfaceState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_surface_desc(&copy.eval_rep)?;
        if !copy.rational() {
            copy.reset_unit_weights()?;
        }
        copy.max_deriv_inv_ok = false;
        Ok(Self::from_state(copy))
    }
    /// Whether an evaluation representation is set (`HasEvalRepresentation`);
    /// `surface.has_eval_representation()`.
    pub fn has_eval_representation(&self) -> bool {
        self.state.read().eval_rep.is_some()
    }
    /// The evaluation representation, possibly null (`EvalRepresentation`);
    /// `surface.eval_representation()`.
    pub fn eval_representation(&self) -> Handle<dyn RepSurfaceDesc> {
        self.state.read().eval_rep.clone()
    }
    /// Validate and set the evaluation representation (`SetEvalRepresentation`);
    /// `surface.set_eval_representation(desc)?`.
    pub fn set_eval_representation(&self, desc: Handle<dyn RepSurfaceDesc>) -> Result<(), Failure> {
        rep_utils::validate_surface_desc(&desc, std::ptr::from_ref(self).cast())?;
        self.state.write().eval_rep = desc;
        Ok(())
    }
    /// Remove the evaluation representation (`ClearEvalRepresentation`);
    /// `surface.clear_eval_representation()`.
    pub fn clear_eval_representation(&self) {
        self.state.write().clear_eval_rep();
    }
    /// Exchange the U and V parameters (`ExchangeUV`); `surface.exchange_uv()?`.
    pub fn exchange_uv(&self) -> Result<(), Failure> {
        self.state.write().exchange_uv()
    }
    /// Make the surface U-periodic (`SetUPeriodic`); `surface.set_u_periodic()?`.
    pub fn set_u_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_periodic(true)
    }
    /// Make the surface V-periodic (`SetVPeriodic`); `surface.set_v_periodic()?`.
    pub fn set_v_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_periodic(false)
    }
    /// Shift periodic parameters into the bounds by whole periods (`PeriodicNormalization`);
    /// `let (u, v) = surface.periodic_normalization(7.0, 0.5)?;`.
    pub fn periodic_normalization(&self, u: f64, v: f64) -> Result<(f64, f64), Failure> {
        self.state.read().periodic_normalization(u, v)
    }
    /// Make U knot `index` the origin of a U-periodic surface (`SetUOrigin`); `surface.set_u_origin(2)?`.
    pub fn set_u_origin(&self, index: i32) -> Result<(), Failure> {
        self.state.write().set_origin(true, index)
    }
    /// Make V knot `index` the origin of a V-periodic surface (`SetVOrigin`); `surface.set_v_origin(2)?`.
    pub fn set_v_origin(&self, index: i32) -> Result<(), Failure> {
        self.state.write().set_origin(false, index)
    }
    /// Make the surface non-periodic in U (`SetUNotPeriodic`); `surface.set_u_not_periodic()?`.
    pub fn set_u_not_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_not_periodic(true)
    }
    /// Make the surface non-periodic in V (`SetVNotPeriodic`); `surface.set_v_not_periodic()?`.
    pub fn set_v_not_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_not_periodic(false)
    }
    /// Raise the degrees, U first (`IncreaseDegree`); `surface.increase_degree(4, 3)?`.
    pub fn increase_degree(&self, u_degree: i32, v_degree: i32) -> Result<(), Failure> {
        self.state.write().increase_degree(u_degree, v_degree)
    }
    /// Insert U knots (`InsertUKnots(Knots, Mults, ParametricTolerance, Add)`);
    /// `surface.insert_u_knots(&k, &m, 0.0, true)?`.
    pub fn insert_u_knots(
        &self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .insert_knots(true, knots, mults, tolerance, add)
    }
    /// Insert V knots (`InsertVKnots`); `surface.insert_v_knots(&k, &m, 0.0, true)?`.
    pub fn insert_v_knots(
        &self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .insert_knots(false, knots, mults, tolerance, add)
    }
    /// Reduce U knot `index` to `mult` within `tolerance`; false when impossible
    /// (`RemoveUKnot`); `surface.remove_u_knot(2, 0, 1e-7)?`.
    pub fn remove_u_knot(&self, index: i32, mult: i32, tolerance: f64) -> Result<bool, Failure> {
        self.state.write().remove_knot(true, index, mult, tolerance)
    }
    /// Reduce V knot `index` to `mult` (`RemoveVKnot`); `surface.remove_v_knot(2, 0, 1e-7)?`.
    pub fn remove_v_knot(&self, index: i32, mult: i32, tolerance: f64) -> Result<bool, Failure> {
        self.state
            .write()
            .remove_knot(false, index, mult, tolerance)
    }
    /// Raise U knot `index` to multiplicity `mult` (`IncreaseUMultiplicity(UIndex, M)`);
    /// `surface.increase_u_multiplicity(2, 3)?`.
    pub fn increase_u_multiplicity(&self, index: i32, mult: i32) -> Result<(), Failure> {
        self.state.write().increase_multiplicity(true, index, mult)
    }
    /// Raise U knots `i1..=i2` to `mult` (`IncreaseUMultiplicity(FromI1, ToI2, M)`);
    /// `surface.increase_u_multiplicity_range(2, 3, 2)?`.
    pub fn increase_u_multiplicity_range(
        &self,
        i1: i32,
        i2: i32,
        mult: i32,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .raise_multiplicities(true, (i1, i2), mult, false)
    }
    /// Add `step` to U knots `i1..=i2` (`IncrementUMultiplicity`); `surface.increment_u_multiplicity(2, 3, 1)?`.
    pub fn increment_u_multiplicity(&self, i1: i32, i2: i32, step: i32) -> Result<(), Failure> {
        self.state
            .write()
            .raise_multiplicities(true, (i1, i2), step, true)
    }
    /// Raise V knot `index` to `mult` (`IncreaseVMultiplicity(VIndex, M)`);
    /// `surface.increase_v_multiplicity(2, 3)?`.
    pub fn increase_v_multiplicity(&self, index: i32, mult: i32) -> Result<(), Failure> {
        self.state.write().increase_multiplicity(false, index, mult)
    }
    /// Raise V knots `i1..=i2` to `mult` (`IncreaseVMultiplicity(FromI1, ToI2, M)`);
    /// `surface.increase_v_multiplicity_range(2, 3, 2)?`.
    pub fn increase_v_multiplicity_range(
        &self,
        i1: i32,
        i2: i32,
        mult: i32,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .raise_multiplicities(false, (i1, i2), mult, false)
    }
    /// Add `step` to V knots `i1..=i2` (`IncrementVMultiplicity`); `surface.increment_v_multiplicity(2, 3, 1)?`.
    pub fn increment_v_multiplicity(&self, i1: i32, i2: i32, step: i32) -> Result<(), Failure> {
        self.state
            .write()
            .raise_multiplicities(false, (i1, i2), step, true)
    }
    /// Insert U knot `u` with multiplicity `mult` (`InsertUKnot`); `surface.insert_u_knot(0.5, 1, 0.0, true)?`.
    pub fn insert_u_knot(
        &self,
        u: f64,
        mult: i32,
        tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let (knots, mults) = single_knot(u, mult)?;
        self.insert_u_knots(&knots, &mults, tolerance, add)
    }
    /// Insert V knot `v` with multiplicity `mult` (`InsertVKnot`); `surface.insert_v_knot(0.5, 1, 0.0, true)?`.
    pub fn insert_v_knot(
        &self,
        v: f64,
        mult: i32,
        tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let (knots, mults) = single_knot(v, mult)?;
        self.insert_v_knots(&knots, &mults, tolerance, add)
    }
    /// Move U knot `index` strictly between its neighbours (`SetUKnot(UIndex, K)`);
    /// `surface.set_u_knot(2, 0.4)?`.
    pub fn set_u_knot(&self, index: i32, knot: f64) -> Result<(), Failure> {
        self.state.write().set_knot(true, index, knot)
    }
    /// Replace U knots `Lower..Upper` of the array (`SetUKnots`); `surface.set_u_knots(&k)?`.
    pub fn set_u_knots(&self, knots: &Array1<'_, f64>) -> Result<(), Failure> {
        self.state.write().set_knots(true, knots)
    }
    /// Raise the multiplicity of U knot `index`, then move it (`SetUKnot(UIndex, K, M)`);
    /// `surface.set_u_knot_with_multiplicity(2, 0.4, 2)?`.
    pub fn set_u_knot_with_multiplicity(
        &self,
        index: i32,
        knot: f64,
        mult: i32,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.increase_multiplicity(true, index, mult)?;
        state.set_knot(true, index, knot)
    }
    /// Move V knot `index` (`SetVKnot(VIndex, K)`); `surface.set_v_knot(2, 0.4)?`.
    pub fn set_v_knot(&self, index: i32, knot: f64) -> Result<(), Failure> {
        self.state.write().set_knot(false, index, knot)
    }
    /// Replace V knots `Lower..Upper` of the array (`SetVKnots`); `surface.set_v_knots(&k)?`.
    pub fn set_v_knots(&self, knots: &Array1<'_, f64>) -> Result<(), Failure> {
        self.state.write().set_knots(false, knots)
    }
    /// Raise the multiplicity of V knot `index`, then move it (`SetVKnot(VIndex, K, M)`);
    /// `surface.set_v_knot_with_multiplicity(2, 0.4, 2)?`.
    pub fn set_v_knot_with_multiplicity(
        &self,
        index: i32,
        knot: f64,
        mult: i32,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.increase_multiplicity(false, index, mult)?;
        state.set_knot(false, index, knot)
    }
    /// `BSplCLib::MaxDegree()` (`MaxDegree`); `GeomBSplineSurface::max_degree() == 25`.
    pub fn max_degree() -> i32 {
        bsplclib::max_degree()
    }
    /// U and V tolerances for a 3D tolerance, caching the inverse derivative bounds
    /// (`Resolution`); `let (tu, tv) = surface.resolution(1e-7)?;`.
    pub fn resolution(&self, tolerance_3d: f64) -> Result<(f64, f64), Failure> {
        self.state.write().resolution(tolerance_3d)
    }
}

// The one-knot arrays of InsertUKnot/InsertVKnot.
fn single_knot(
    knot: f64,
    mult: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let (mut knots, mut mults) = (one_based::<f64>(1)?, one_based::<i32>(1)?);
    put(&mut knots, 1, knot)?;
    put(&mut mults, 1, mult)?;
    Ok((knots, mults))
}
