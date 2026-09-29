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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx, Geom2d_BSplineCurve_1.cxx (the fields, constructors and updateKnots that ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx repeats) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The state of `Geom2d_BSplineCurve` and `Geom_BSplineCurve`, which OCCT writes
//! twice with the same operations in the same order; only the point dimension and
//! the members listed by [`BSplineCurveKind`] differ.
//!
//! Each curve class keeps one [`BSplineCurveCore`] behind its lock; the edits,
//! evaluators and queries live in the submodules and run once for both dimensions.
mod access;
mod eval;
mod knots;
mod periodic;
mod poles;
mod public_members;
mod segment;

use super::arrays::{one_based, rebased};
use super::dump_flag;
use super::spline_curve_kind::{KindVector, SplineCurveKind};
use super::spline_data::{KnotData, bspline_rational, knot_data};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use std::io;

/// The members where `Geom_BSplineCurve` departs from `Geom2d_BSplineCurve`; each
/// curve crate implements it on its [`SplineCurveKind`] marker, e.g.
/// `impl BSplineCurveKind for Geom2dSplineKind { const BSPLINE_CLASS: &str = ...; }`.
pub trait BSplineCurveKind: SplineCurveKind {
    /// The native class name used in messages; `"Geom2d_BSplineCurve"`.
    const BSPLINE_CLASS: &'static str;
    /// `ClearEvalRepresentation` at the points where an edit calls it: Geom clears
    /// the representation, Geom2d edits keep it; `K::clear_eval_rep_on_edit(&mut rep)`.
    fn clear_eval_rep_on_edit(eval_rep: &mut Self::EvalRep);
    /// `PeriodicNormalization`, whose algorithm differs between the classes;
    /// `K::periodic_normalization(&core, 7.0)?`.
    fn periodic_normalization(
        core: &BSplineCurveCore<Self>,
        parameter: f64,
    ) -> Result<f64, Failure>;
    /// The index check of `RemoveKnot`; `K::check_removable_index(&core, 2)?`.
    fn check_removable_index(core: &BSplineCurveCore<Self>, index: i32) -> Result<(), Failure>;
    /// Both tangents non-null and within `ang_tol` of each other (`IsG1`);
    /// `K::tangents_agree(v1, v2, 1e-9)?`.
    fn tangents_agree(
        v1: KindVector<Self>,
        v2: KindVector<Self>,
        ang_tol: f64,
    ) -> Result<bool, Failure>;
}

/// The fields of a B-spline curve (`myPoles`, `myWeights`, `myKnots`, `myFlatKnots`,
/// `myMults`, `myEvalRep`, `myDeg`, `myPeriodic`, `myRational`, `myKnotSet`, `mySmooth`,
/// `myMaxDerivInv`, `myMaxDerivInvOk`);
/// `BSplineCurveCore::<K>::from_arrays(&poles, None, &knots, &mults, 3, false)?`.
#[derive(Clone)]
pub struct BSplineCurveCore<K: BSplineCurveKind> {
    /// The one-based poles.
    pub poles: Array1<'static, K::Point>,
    /// The one-based weights; unit weights for a non-rational curve.
    pub weights: Array1<'static, f64>,
    /// The one-based distinct knots.
    pub knots: Array1<'static, f64>,
    /// The flat knot sequence built by `updateKnots`.
    pub flat_knots: Array1<'static, f64>,
    /// The one-based knot multiplicities.
    pub mults: Array1<'static, i32>,
    /// The evaluation representation, possibly null.
    pub eval_rep: K::EvalRep,
    /// The degree.
    pub deg: i32,
    /// Whether the curve is periodic.
    pub periodic: bool,
    /// Whether the curve is rational.
    pub rational: bool,
    /// The knot distribution.
    pub knot_set: BSplKnotDistribution,
    /// The continuity from the knot multiplicities.
    pub smooth: Shape,
    /// The cached inverse derivative bound of `Resolution`.
    pub max_deriv_inv: f64,
    /// Whether `max_deriv_inv` is up to date.
    pub max_deriv_inv_ok: bool,
}

/// The weights given to a rational constructor and whether to test them
/// (`CheckRational`, always true for Geom2d);
/// `GivenWeights { weights: &w, check_rational: true }`.
pub struct GivenWeights<'w, 'a> {
    /// The constructor weights.
    pub weights: &'w Array1<'a, f64>,
    /// Whether equal weights make the curve non-rational.
    pub check_rational: bool,
}

impl<K: BSplineCurveKind> BSplineCurveCore<K> {
    /// One-based copies of checked input arrays, as the native constructors store
    /// them; `BSplineCurveCore::<K>::from_arrays(&p, None, &k, &m, 3, false)?`.
    ///
    /// With weights the curve starts rational and only `CheckRational` tests the
    /// weights, so equal weights with `check_rational == false` stay rational.
    pub fn from_arrays(
        poles: &Array1<'_, K::Point>,
        weights: Option<GivenWeights<'_, '_>>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        deg: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        let mut rational_flag: bool = weights.is_some();
        if let Some(given) = weights.as_ref().filter(|given| given.check_rational) {
            rational_flag = bspline_rational(given.weights)?;
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
            eval_rep: K::EvalRep::default(),
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
    /// `Weights()`: the weights of a rational curve, `None` otherwise; `core.weights_view()`.
    pub fn weights_view(&self) -> Option<&Array1<'static, f64>> {
        self.rational.then_some(&self.weights)
    }
    /// Replace the weights by unit weights for the current pole count;
    /// `core.reset_unit_weights()?`.
    pub fn reset_unit_weights(&mut self) -> Result<(), Failure> {
        self.weights = bsplclib::unit_weights(self.poles.len() as i32)?;
        Ok(())
    }
    /// A one-based output array for new weights when rational (`nweights.Resize(1, n)`);
    /// `core.new_weights(4)?`.
    pub fn new_weights(&self, count: i32) -> Result<Array1<'static, f64>, Failure> {
        if self.rational {
            return one_based(count);
        }
        Ok(Array1::default())
    }
    /// `ClearEvalRepresentation` where the edits of the class call it;
    /// `core.clear_eval_rep_on_edit()`.
    pub fn clear_eval_rep_on_edit(&mut self) {
        K::clear_eval_rep_on_edit(&mut self.eval_rep);
    }
    /// `updateKnots`: knot form, flat knots and continuity from the knots and
    /// multiplicities; `core.update_knots()?`.
    pub fn update_knots(&mut self) -> Result<(), Failure> {
        self.max_deriv_inv_ok = false;
        let data: KnotData = knot_data(self.deg, self.periodic, &self.knots, &self.mults)?;
        self.knot_set = data.knot_set;
        self.flat_knots = data.flat_knots;
        self.smooth = data.smooth;
        Ok(())
    }
    /// `FirstUKnotIndex`; `core.first_u_knot_index()?`.
    pub fn first_u_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(1);
        }
        bsplclib::first_u_knot_index(self.deg, &self.mults)
    }
    /// `LastUKnotIndex`; `core.last_u_knot_index()?`.
    pub fn last_u_knot_index(&self) -> Result<i32, Failure> {
        if self.periodic {
            return Ok(self.knots.len() as i32);
        }
        bsplclib::last_u_knot_index(self.deg, &self.mults)
    }
    /// `FirstParameter`: `FlatKnots(Degree + 1)`; `core.first_parameter()`.
    ///
    /// The constructor checks guarantee at least `Degree + 3` flat knots, so the
    /// index is always inside the one-based sequence.
    pub fn first_parameter(&self) -> f64 {
        self.flat_knots.as_slice()[self.deg as usize]
    }
    /// `LastParameter`: `FlatKnots(Upper - Degree)`; `core.last_parameter()`.
    pub fn last_parameter(&self) -> f64 {
        let flat: &[f64] = self.flat_knots.as_slice();
        flat[flat.len() - 1 - self.deg as usize]
    }
    /// `PeriodicNormalization`: shift a periodic parameter into the curve range;
    /// `core.periodic_normalization(7.0)?`.
    pub fn periodic_normalization(&self, parameter: f64) -> Result<f64, Failure> {
        K::periodic_normalization(self, parameter)
    }
    /// The `DumpJson` fields after the bounded-curve base, from `Rational` to
    /// `MaxDerivInvOk`; array sizes are dumped when non-zero, the weights only for a
    /// rational curve; `core.dump_fields(&mut stream)?`.
    pub fn dump_fields(&self, stream: &mut DumpStream) -> io::Result<()> {
        dump_flag(stream, "Rational", self.rational)?;
        dump_flag(stream, "Periodic", self.periodic)?;
        dump::dump_integer(stream, "KnotSet", self.knot_set as i32)?;
        dump::dump_integer(stream, "Smooth", self.smooth as i32)?;
        dump::dump_integer(stream, "Deg", self.deg)?;
        let sizes = [
            ("Poles.Size", self.poles.len(), !self.poles.is_empty()),
            ("Weights.Size", self.weights.len(), self.rational),
            (
                "FlatKnots.Size",
                self.flat_knots.len(),
                !self.flat_knots.is_empty(),
            ),
            ("Knots.Size", self.knots.len(), !self.knots.is_empty()),
            ("Mults.Size", self.mults.len(), !self.mults.is_empty()),
        ];
        for (name, size, shown) in sizes {
            if shown {
                dump::dump_integer(stream, name, size as i32)?;
            }
        }
        dump::dump_real(stream, "MaxDerivInv", self.max_deriv_inv)?;
        dump_flag(stream, "MaxDerivInvOk", self.max_deriv_inv_ok)
    }
}
