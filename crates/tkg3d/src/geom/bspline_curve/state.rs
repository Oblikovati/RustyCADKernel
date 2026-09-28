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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx (fields, constructors, updateKnots, PeriodicNormalization) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fields of Geom_BSplineCurve, `updateKnots` and `PeriodicNormalization`.
use crate::geom_eval::RepCurveDesc;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, rebased};
use rustycad_tkg2d::geom2d::spline_data::{KnotData, knot_data};
pub(super) use rustycad_tkg2d::geom2d::spline_data::{
    bspline_rational as rational, check_curve_data,
};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use rustycad_tkmath::gp::Point3;

#[derive(Clone)]
pub(super) struct BSplineState {
    pub(super) poles: Array1<'static, Point3>,
    pub(super) weights: Array1<'static, f64>,
    pub(super) knots: Array1<'static, f64>,
    pub(super) flat_knots: Array1<'static, f64>,
    pub(super) mults: Array1<'static, i32>,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
    pub(super) deg: i32,
    pub(super) periodic: bool,
    pub(super) rational: bool,
    pub(super) knot_set: BSplKnotDistribution,
    pub(super) smooth: Shape,
    pub(super) max_deriv_inv: f64,
    pub(super) max_deriv_inv_ok: bool,
}

/// The weights given to a rational constructor and whether to test them (`CheckRational`).
pub(super) struct GivenWeights<'w, 'a> {
    pub(super) weights: &'w Array1<'a, f64>,
    pub(super) check_rational: bool,
}

impl BSplineState {
    /// One-based copies of checked input arrays, as the two native constructors store
    /// them; `BSplineState::from_arrays(&p, None, &k, &m, 3, false)?`.
    ///
    /// With weights the curve starts rational and only `CheckRational` tests the
    /// weights, so equal weights with `check_rational == false` stay rational.
    pub(super) fn from_arrays(
        poles: &Array1<'_, Point3>,
        weights: Option<GivenWeights<'_, '_>>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        deg: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        let mut rational_flag: bool = weights.is_some();
        if let Some(given) = weights.as_ref().filter(|given| given.check_rational) {
            rational_flag = rational(given.weights)?;
        }
        let stored_weights: Array1<'static, f64> = match weights.filter(|_| rational_flag) {
            Some(given) => rebased(given.weights)?,
            None => bsplclib::unit_weights(poles.len() as i32)?,
        };
        let mut state = Self {
            poles: rebased(poles)?,
            weights: stored_weights,
            knots: rebased(knots)?,
            flat_knots: Array1::default(),
            mults: rebased(mults)?,
            eval_rep: Handle::default(),
            deg,
            periodic,
            rational: rational_flag,
            knot_set: BSplKnotDistribution::NonUniform,
            smooth: Shape::C0,
            max_deriv_inv: 0.0,
            max_deriv_inv_ok: false,
        };
        state.update_knots()?;
        Ok(state)
    }
    /// `Weights()`: the weights of a rational curve, `None` otherwise.
    pub(super) fn weights_view(&self) -> Option<&Array1<'static, f64>> {
        self.rational.then_some(&self.weights)
    }
    /// Replace the weights by unit weights for the current pole count.
    pub(super) fn reset_unit_weights(&mut self) -> Result<(), Failure> {
        self.weights = bsplclib::unit_weights(self.poles.len() as i32)?;
        Ok(())
    }
    /// A one-based output array for new weights when rational (`nweights.Resize(1, n)`).
    pub(super) fn new_weights(&self, count: i32) -> Result<Array1<'static, f64>, Failure> {
        if self.rational {
            return one_based(count);
        }
        Ok(Array1::default())
    }
    /// `ClearEvalRepresentation`, which most 3D edits call (unlike Geom2d).
    pub(super) fn clear_eval_rep(&mut self) {
        self.eval_rep = Handle::default();
    }
    /// `updateKnots`: knot form, flat knots and continuity from the knots and multiplicities.
    pub(super) fn update_knots(&mut self) -> Result<(), Failure> {
        self.max_deriv_inv_ok = false;
        let data: KnotData = knot_data(self.deg, self.periodic, &self.knots, &self.mults)?;
        self.knot_set = data.knot_set;
        self.flat_knots = data.flat_knots;
        self.smooth = data.smooth;
        Ok(())
    }
    /// `FirstUKnotIndex`; `state.first_u_knot_index()?`.
    pub(super) fn first_u_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(1);
        }
        bsplclib::first_u_knot_index(self.deg, &self.mults)
    }
    /// `LastUKnotIndex`; `state.last_u_knot_index()?`.
    pub(super) fn last_u_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(self.knots.len() as i32);
        }
        bsplclib::last_u_knot_index(self.deg, &self.mults)
    }
    /// `FirstParameter`: `FlatKnots(Degree + 1)`; `state.first_parameter()`.
    ///
    /// The constructor checks guarantee at least `Degree + 3` flat knots, so the
    /// index is always inside the one-based sequence.
    pub(super) fn first_parameter(&self) -> f64 {
        self.flat_knots.as_slice()[self.deg as usize]
    }
    /// `LastParameter`: `FlatKnots(Upper - Degree)`; `state.last_parameter()`.
    pub(super) fn last_parameter(&self) -> f64 {
        let flat: &[f64] = self.flat_knots.as_slice();
        flat[flat.len() - 1 - self.deg as usize]
    }
    /// `PeriodicNormalization`: shift a periodic parameter into the curve range by a
    /// whole number of periods, then by one period, stepping to the neighbouring
    /// double when the shift is absorbed by rounding.
    ///
    /// Geom_BSplineCurve replaces the loops of Geom2d_BSplineCurve by this closed
    /// form, so an infinite parameter gives NaN instead of looping.
    pub(super) fn periodic_normalization(&self, parameter: f64) -> Result<f64, Failure> {
        if !self.periodic {
            return Ok(parameter);
        }
        let first: f64 = at(&self.flat_knots, self.deg + 1)?;
        let last: f64 = at(&self.flat_knots, self.flat_knots.upper() - self.deg)?;
        if parameter >= first && parameter <= last {
            return Ok(parameter);
        }
        let period: f64 = last - first;
        let nb_periods: f64 = ((parameter - first) / period).floor();
        let mut value: f64 = parameter;
        if nb_periods != 0.0 {
            value -= nb_periods * period;
        }
        if value > last {
            return Ok(shifted(value, value - period, first));
        }
        if value < first {
            return Ok(shifted(value, value + period, last));
        }
        Ok(value)
    }
}

// `aNewParam == theParameter ? std::nextafter(aNewParam, toward) : aNewParam`.
fn shifted(value: f64, candidate: f64, toward: f64) -> f64 {
    if candidate == value {
        return next_after(candidate, toward);
    }
    candidate
}

/// C `nextafter(x, y)`: the next double after `x` in the direction of `y`;
/// `next_after(1.0, 2.0) > 1.0`.
pub(super) fn next_after(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return x + y;
    }
    if x == y {
        return y;
    }
    if y < x { x.next_down() } else { x.next_up() }
}
