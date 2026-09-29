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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.cxx (the fields, constructors, init and weight access that ModelingData/TKG3d/Geom/Geom_BezierCurve.cxx repeats) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The state of `Geom2d_BezierCurve` and `Geom_BezierCurve`, which OCCT writes twice
//! with the same operations in the same order; only the point dimension, the
//! evaluation representation and the closure test ([`BezierCurveKind`]) differ.
mod edits;
mod eval;

use super::arrays::{at, rebased};
use super::spline_curve_kind::SplineCurveKind;
use super::spline_data::{bezier_knot_sequence, bezier_multiplicities, bezier_rational};
use super::{dump_flag, errors};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::resolution;
use std::io;

/// The members where `Geom_BezierCurve` departs from `Geom2d_BezierCurve`; each curve
/// crate implements it on its [`SplineCurveKind`] marker, e.g.
/// `impl BezierCurveKind for Geom2dSplineKind { const BEZIER_CLASS: &str = ...; }`.
pub trait BezierCurveKind: SplineCurveKind {
    /// The native class name used in messages; `"Geom2d_BezierCurve"`.
    const BEZIER_CLASS: &'static str;
    /// `P1.Distance(P2)` of the closure test; `K::point_distance(a, b)`.
    fn point_distance(first: Self::Point, last: Self::Point) -> f64;
    /// The closure tolerance: gp::Resolution for Geom2d, Precision::Confusion for Geom
    /// (in init, Segment and SetPole); `K::closure_tolerance()`.
    fn closure_tolerance() -> f64;
}

/// The fields of a Bezier curve (`myPoles`, `myWeights`, `myEvalRep`, `myRational`,
/// `myClosed`, `myMaxDerivInv`, `myMaxDerivInvOk`);
/// `BezierCurveCore::<K>::checked(&poles, None)?`.
#[derive(Clone)]
pub struct BezierCurveCore<K: BezierCurveKind> {
    /// The one-based poles.
    pub poles: Array1<'static, K::Point>,
    /// The one-based weights; unit weights for a non-rational curve.
    pub weights: Array1<'static, f64>,
    /// The evaluation representation, possibly null.
    pub eval_rep: K::EvalRep,
    /// Whether the curve is rational.
    pub rational: bool,
    /// Whether the end poles coincide.
    pub closed: bool,
    /// The cached inverse derivative bound of `Resolution`.
    pub max_deriv_inv: f64,
    /// Whether `max_deriv_inv` is up to date.
    pub max_deriv_inv_ok: bool,
}

impl<K: BezierCurveKind> BezierCurveCore<K> {
    /// The native constructors: pole count and weight checks, then `init` with the
    /// weights only when they differ, and the copy-constructor member reset;
    /// `BezierCurveCore::<K>::checked(&poles, Some(&weights))?`.
    pub fn checked(
        poles: &Array1<'_, K::Point>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<Self, Failure> {
        Self::check_pole_count(poles.len())?;
        let used: Option<&Array1<'_, f64>> = match weights {
            Some(weights) => {
                Self::check_weights(weights, poles.len())?;
                bezier_rational(weights)?.then_some(weights)
            }
            None => None,
        };
        let mut state: Self = Self::initialized(poles, used)?;
        state.reset_copied()?;
        Ok(state)
    }
    fn check_pole_count(count: usize) -> Result<(), Failure> {
        let max: usize = bsplclib::max_degree() as usize + 1;
        if count < 2 || count > max {
            return Err(errors::construction(&format!(
                "{}: {count} poles; expected 2..={max}",
                K::BEZIER_CLASS
            )));
        }
        Ok(())
    }
    // The native constructor reads Weights(1)..Weights(nbpoles) with one-based indices.
    fn check_weights(weights: &Array1<'_, f64>, nbpoles: usize) -> Result<(), Failure> {
        if weights.len() != nbpoles {
            return Err(errors::construction(&format!(
                "{}: {} weights; expected {nbpoles}",
                K::BEZIER_CLASS,
                weights.len()
            )));
        }
        for i in 1..=nbpoles as i32 {
            let weight: f64 = at(weights, i)?;
            if weight <= resolution() {
                return Err(errors::construction(&format!(
                    "{}: Weights({i}) = {weight}; expected > gp::Resolution()",
                    K::BEZIER_CLASS
                )));
            }
        }
        Ok(())
    }
    /// The member initialization of the native copy constructor: unit weights for a
    /// non-rational curve and a reset resolution cache (`myMaxDerivInv` keeps 0);
    /// `copy.reset_copied()?`.
    pub fn reset_copied(&mut self) -> Result<(), Failure> {
        if !self.rational {
            self.weights = bsplclib::unit_weights(self.poles.len() as i32)?;
        }
        self.max_deriv_inv = 0.0;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    // `init(thePoles, theWeights)` on a fresh state.
    fn initialized(
        poles: &Array1<'_, K::Point>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<Self, Failure> {
        let mut state = Self {
            poles: Array1::default(),
            weights: Array1::default(),
            eval_rep: K::EvalRep::default(),
            rational: false,
            closed: false,
            max_deriv_inv: 0.0,
            max_deriv_inv_ok: false,
        };
        state.init(poles, weights)?;
        Ok(state)
    }
    // `init`: closure flag, one-based copies, rationality, cache and evaluation
    // representation reset.
    fn init(
        &mut self,
        poles: &Array1<'_, K::Point>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<(), Failure> {
        let (first, last) = (at(poles, poles.lower())?, at(poles, poles.upper())?);
        self.closed = K::point_distance(first, last) <= K::closure_tolerance();
        self.poles = rebased(poles)?;
        self.rational = false;
        self.weights = bsplclib::unit_weights(poles.len() as i32)?;
        if let Some(weights) = weights {
            let copied: Array1<'static, f64> = rebased(weights)?;
            self.rational = bezier_rational(&copied)?;
            if self.rational {
                self.weights = copied;
            }
        }
        self.max_deriv_inv = 0.0;
        self.max_deriv_inv_ok = false;
        self.eval_rep = K::EvalRep::default();
        Ok(())
    }
    /// `Degree`: `NbPoles - 1`; `core.degree()`.
    pub fn degree(&self) -> i32 {
        self.poles.len() as i32 - 1
    }
    /// `Weights()`: the weights of a rational curve, `None` otherwise; `core.weights_view()`.
    pub fn weights_view(&self) -> Option<&Array1<'static, f64>> {
        self.rational.then_some(&self.weights)
    }
    /// The multiplicities `{Degree+1, Degree+1}` (`Multiplicities`); `core.multiplicities()?`.
    pub fn multiplicities(&self) -> Result<Array1<'static, i32>, Failure> {
        bezier_multiplicities(self.degree(), K::BEZIER_CLASS)
    }
    /// The flat knots (`KnotSequence`); `core.knot_sequence()?`.
    pub fn knot_sequence(&self) -> Result<Array1<'static, f64>, Failure> {
        bezier_knot_sequence(self.degree(), K::BEZIER_CLASS)
    }
    /// `Weight(Index)`: 1 for a non-rational curve; `core.weight(1)?`.
    pub fn weight(&self, index: i32) -> Result<f64, Failure> {
        let member: String = format!("{}::Weight", K::BEZIER_CLASS);
        super::arrays::check_one_based(index, self.poles.len(), &member)?;
        if !self.rational {
            return Ok(1.);
        }
        at(&self.weights, index)
    }
    /// Copy the weights into `target` of equal length, writing `target(1..=n)` for a
    /// non-rational curve (deprecated `Weights(W)`); `core.weights_into(&mut w)?`.
    pub fn weights_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let nbpoles: usize = self.poles.len();
        if target.len() != nbpoles {
            return Err(errors::dimension(&format!(
                "{}::Weights: length {}; expected {nbpoles}",
                K::BEZIER_CLASS,
                target.len()
            )));
        }
        if self.rational {
            target
                .as_mut_slice()
                .copy_from_slice(self.weights.as_slice());
            return Ok(());
        }
        for i in 1..=nbpoles as i32 {
            super::arrays::put(target, i, 1.)?;
        }
        Ok(())
    }
    /// The `DumpJson` fields after the bounded-curve base; `core.dump_fields(&mut stream)?`.
    pub fn dump_fields(&self, stream: &mut DumpStream) -> io::Result<()> {
        dump_flag(stream, "Rational", self.rational)?;
        dump_flag(stream, "Closed", self.closed)?;
        dump::dump_integer(stream, "Poles.Length", self.poles.len() as i32)?;
        if self.rational {
            dump::dump_integer(stream, "Weights.Length", self.weights.len() as i32)?;
        }
        dump::dump_real(stream, "MaxDerivInv", self.max_deriv_inv)?;
        dump_flag(stream, "MaxDerivInvOk", self.max_deriv_inv_ok)
    }
}
