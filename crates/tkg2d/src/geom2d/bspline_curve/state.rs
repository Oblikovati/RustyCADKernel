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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fields of Geom2d_BSplineCurve, its data checks and `updateKnots`.
use crate::geom2d::arrays::{at, one_based, rebased};
use crate::geom2d::spline_data::smoothness;
pub(super) use crate::geom2d::spline_data::{bspline_rational as rational, check_curve_data};
use crate::geom2d_eval::RepCurveDesc;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use rustycad_tkmath::gp::Point2;

#[derive(Clone)]
pub(super) struct BSplineState {
    pub(super) poles: Array1<'static, Point2>,
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

impl BSplineState {
    /// The fields before `updateKnots`; `BSplineState::stored(poles, weights, knots, mults, 3, false, false)`.
    #[allow(clippy::too_many_arguments)] // The native member list.
    pub(super) fn stored(
        poles: Array1<'static, Point2>,
        weights: Array1<'static, f64>,
        knots: Array1<'static, f64>,
        mults: Array1<'static, i32>,
        deg: i32,
        periodic: bool,
        rational: bool,
    ) -> Self {
        Self {
            poles,
            weights,
            knots,
            flat_knots: Array1::default(),
            mults,
            eval_rep: Handle::default(),
            deg,
            periodic,
            rational,
            knot_set: BSplKnotDistribution::NonUniform,
            smooth: Shape::C0,
            max_deriv_inv: 0.0,
            max_deriv_inv_ok: false,
        }
    }
    /// One-based copies of checked input arrays; `BSplineState::from_arrays(&p, None, &k, &m, 3, false)?`.
    pub(super) fn from_arrays(
        poles: &Array1<'_, Point2>,
        weights: Option<&Array1<'_, f64>>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        deg: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        let rational: bool = match weights {
            Some(weights) => rational(weights)?,
            None => false,
        };
        let stored_weights: Array1<'static, f64> = match weights.filter(|_| rational) {
            Some(weights) => rebased(weights)?,
            None => bsplclib::unit_weights(poles.len() as i32)?,
        };
        let (poles, knots, mults) = (rebased(poles)?, rebased(knots)?, rebased(mults)?);
        let mut state = Self::stored(poles, stored_weights, knots, mults, deg, periodic, rational);
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
    /// `updateKnots`: knot form, flat knots and continuity from the knots and multiplicities.
    pub(super) fn update_knots(&mut self) -> Result<(), Failure> {
        self.max_deriv_inv_ok = false;
        let (knot_set, max_knot_mult) =
            bsplclib::knot_analysis(self.deg, self.periodic, &self.knots, &self.mults)?;
        self.knot_set = knot_set;
        if knot_set == BSplKnotDistribution::Uniform && !self.periodic {
            self.flat_knots = self.knots.clone();
        } else {
            let length: i32 = bsplclib::knot_sequence_length(&self.mults, self.deg, self.periodic)?;
            let mut flat: Array1<'static, f64> = one_based(length)?;
            bsplclib::knot_sequence_with_degree(
                &self.knots,
                &self.mults,
                self.deg,
                self.periodic,
                &mut flat,
            )?;
            self.flat_knots = flat;
        }
        self.smooth = smoothness(self.deg, max_knot_mult);
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
    /// `PeriodicNormalization`: shift a periodic parameter into the curve range.
    pub(super) fn periodic_normalization(&self, parameter: f64) -> Result<f64, Failure> {
        if !self.periodic {
            return Ok(parameter);
        }
        let upper: f64 = at(&self.flat_knots, self.flat_knots.upper() - self.deg)?;
        let lower: f64 = at(&self.flat_knots, self.deg + 1)?;
        let period: f64 = upper - lower;
        let mut value: f64 = parameter;
        while value > upper {
            value -= period;
        }
        while value < lower {
            value += period;
        }
        Ok(value)
    }
}
