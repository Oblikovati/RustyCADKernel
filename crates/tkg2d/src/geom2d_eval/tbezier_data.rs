// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_TBezierCurve.cxx and ModelingData/TKG3d/GeomEval/GeomEval_TBezierCurve.cxx (the members, constructors and EvalD0..EvalDN that GeomEval repeats with gp_XYZ) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fields, constructor checks and evaluation of `Geom2dEval_TBezierCurve` and
//! `GeomEval_TBezierCurve`, generic over the pole type; the classes convert the
//! returned coordinates into their points and vectors.
use super::bezier_curve_data::{check_weights, owned};
use super::eval_basis::EvalPole;
use super::eval_basis::sums::BasisSums;
use super::eval_basis::tbezier::{
    TBezierView, curve_basis, non_rational_dn, rational_derivatives, rational_dn, rational_point,
};
use crate::geom2d::{dump_flag, errors};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use std::io;

/// The members of a T-Bezier curve (`myPoles`, `myWeights`, `myAlpha`, `myRational`);
/// `TBezierData::checked("GeomEval_TBezierCurve", &poles, None, 1.0)?`.
#[derive(Clone)]
pub struct TBezierData<P: EvalPole + 'static> {
    /// The poles with their original bounds.
    pub poles: Array1<'static, P>,
    /// The weights, empty for a non-rational curve.
    pub weights: Array1<'static, f64>,
    /// The frequency parameter.
    pub alpha: f64,
    /// Whether the curve was built with weights.
    pub rational: bool,
}

// The checks both constructors share, in native order.
fn check_poles_and_alpha(class: &str, count: usize, alpha: f64) -> Result<(), Failure> {
    if count < 3 || count.is_multiple_of(2) {
        return Err(errors::construction(&format!(
            "{class}: {count} poles; expected an odd count >= 3"
        )));
    }
    if alpha <= 0.0 {
        return Err(errors::construction(&format!(
            "{class}: alpha = {alpha}; expected > 0"
        )));
    }
    Ok(())
}

impl<P: EvalPole + Clone + Default + 'static> TBezierData<P> {
    /// The two native constructors: the pole count and alpha checks, then the weight
    /// checks when `weights` is given (equal weights still give a rational curve);
    /// Standard_ConstructionError naming `class`;
    /// `TBezierData::checked(CLASS, &poles, Some(&w), 1.0)?`.
    pub fn checked(
        class: &str,
        poles: &Array1<'_, P>,
        weights: Option<&Array1<'_, f64>>,
        alpha: f64,
    ) -> Result<Self, Failure> {
        check_poles_and_alpha(class, poles.len(), alpha)?;
        let stored: Array1<'static, f64> = match weights {
            Some(weights) => {
                check_weights(class, weights, poles.len())?;
                owned(weights)
            }
            None => Array1::default(),
        };
        Ok(Self {
            poles: owned(poles),
            weights: stored,
            alpha,
            rational: weights.is_some(),
        })
    }
    fn view(&self) -> TBezierView<'_, P> {
        TBezierView {
            poles: self.poles.as_slice(),
            weights: self.rational.then(|| self.weights.as_slice()),
            alpha: self.alpha,
        }
    }
    /// The point and the first derivatives through the order-`MAX` sums;
    /// `let (c, [d1, _, _]) = data.evaluate::<1>(0.5);`.
    pub fn evaluate<const MAX: usize>(&self, u: f64) -> (P::Coord, [P::Coord; 3]) {
        let sums: BasisSums<P::Coord> = curve_basis::<MAX, P>(&self.view(), u);
        if !self.rational {
            return (sums.n[0], [sums.n[1], sums.n[2], sums.n[3]]);
        }
        let c: P::Coord = rational_point(&sums);
        let d: [P::Coord; 3] = rational_derivatives(&sums, c, MAX);
        (c, d)
    }
    /// `EvalDN` of a non-rational curve, which differentiates the basis directly for
    /// every order; `None` for a rational curve; `data.non_rational_dn(0.5, 2)`.
    pub fn non_rational_dn(&self, u: f64, n: i32) -> Option<P::Coord> {
        let view: TBezierView<'_, P> = self.view();
        match view.weights {
            None => Some(non_rational_dn(&view, u, n)),
            Some(_) => None,
        }
    }
    /// `EvalDN` of a rational curve above order 3; `data.rational_high_dn(0.5, 4)`.
    pub fn rational_high_dn(&self, u: f64, n: i32) -> P::Coord {
        let view: TBezierView<'_, P> = self.view();
        let weights: &[f64] = view.weights.unwrap_or_default();
        rational_dn(&view, weights, u, n)
    }
    /// The `DumpJson` fields after the bounded-curve base; `data.dump_fields(&mut stream)?`.
    pub fn dump_fields(&self, stream: &mut DumpStream) -> io::Result<()> {
        dump::dump_real(stream, "Alpha", self.alpha)?;
        dump_flag(stream, "Rational", self.rational)?;
        dump::dump_integer(stream, "Poles.Length", self.poles.len() as i32)
    }
}

/// The public read accessors of the T-Bezier curve classes over their
/// [`TBezierData`] field `data`; `rustycad_tkg2d::tbezier_accessors!(Point3);`.
#[macro_export]
macro_rules! tbezier_accessors {
    ($point:ty) => {
        /// A copy of the poles with their original bounds (`Poles`); `curve.poles().len()`.
        pub fn poles(&self) -> ::rustycad_tkernel::ncollection::Array1<'static, $point> {
            self.data.poles.clone()
        }
        /// A copy of the weights, empty for a non-rational curve (`Weights`); `curve.weights().len()`.
        pub fn weights(&self) -> ::rustycad_tkernel::ncollection::Array1<'static, f64> {
            self.data.weights.clone()
        }
        /// The frequency parameter (`Alpha`); `curve.alpha()`.
        pub fn alpha(&self) -> f64 {
            self.data.alpha
        }
        /// The number of poles (`NbPoles`); `curve.nb_poles()`.
        pub fn nb_poles(&self) -> i32 {
            self.data.poles.len() as i32
        }
        /// The trigonometric order n, with NbPoles = 2n + 1 (`Order`); `curve.order()`.
        pub fn order(&self) -> i32 {
            (self.nb_poles() - 1) / 2
        }
        /// Whether the curve was built with weights (`IsRational`); `curve.is_rational()`.
        pub fn is_rational(&self) -> bool {
            self.data.rational
        }
    };
}

/// `EvalD0`..`EvalDN` of the T-Bezier curve classes over their [`TBezierData`] field
/// `data`; rational curves delegate orders 1 to 3 of `EvalDN` to the explicit
/// evaluators, as natively; `check_derivative_order` must be in scope;
/// `rustycad_tkg2d::tbezier_evaluators!(Point3, Vector3, GeomCurveD1, GeomCurveD2, GeomCurveD3, "GeomEval_TBezierCurve");`.
#[macro_export]
macro_rules! tbezier_evaluators {
    ($point:ident, $vector:ident, $d1:ident, $d2:ident, $d3:ident, $class:literal) => {
        fn eval_d0(&self, u: f64) -> Result<$point, Failure> {
            Ok($point::from(self.data.evaluate::<0>(u).0))
        }
        fn eval_d1(&self, u: f64) -> Result<$d1, Failure> {
            let (c, d) = self.data.evaluate::<1>(u);
            let [d1, _, _] = d.map($vector::from);
            Ok($d1 {
                point: $point::from(c),
                d1,
            })
        }
        fn eval_d2(&self, u: f64) -> Result<$d2, Failure> {
            let (c, d) = self.data.evaluate::<2>(u);
            let [d1, d2, _] = d.map($vector::from);
            Ok($d2 {
                point: $point::from(c),
                d1,
                d2,
            })
        }
        fn eval_d3(&self, u: f64) -> Result<$d3, Failure> {
            let (c, d) = self.data.evaluate::<3>(u);
            let [d1, d2, d3] = d.map($vector::from);
            Ok($d3 {
                point: $point::from(c),
                d1,
                d2,
                d3,
            })
        }
        fn eval_dn(&self, u: f64, n: i32) -> Result<$vector, Failure> {
            check_derivative_order(concat!($class, "::EvalDN"), n)?;
            if let Some(value) = self.data.non_rational_dn(u, n) {
                return Ok($vector::from(value));
            }
            Ok(match n {
                1 => self.eval_d1(u)?.d1,
                2 => self.eval_d2(u)?.d2,
                3 => self.eval_d3(u)?.d3,
                _ => $vector::from(self.data.rational_high_dn(u, n)),
            })
        }
    };
}
