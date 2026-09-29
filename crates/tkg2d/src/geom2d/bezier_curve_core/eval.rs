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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.cxx and ModelingData/TKG3d/Geom/Geom_BezierCurve.cxx (EvalD0..EvalDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of a Bezier curve, through the evaluation representation when it answers.
use super::{BezierCurveCore, BezierCurveKind};
use crate::geom2d::spline_curve_kind::{KindD1, KindD2, KindD3, KindVector};
use crate::geom2d::spline_data::bezier_knots;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, CurveSpan};

impl<K: BezierCurveKind> BezierCurveCore<K> {
    /// `EvalD0`; `core.eval_d0(0.5)?`.
    pub fn eval_d0(&self, u: f64) -> Result<K::Point, Failure> {
        if let Some(point) = K::rep_d0(&self.eval_rep, u)? {
            return Ok(point);
        }
        bsplclib::bezier_d0(u, &self.poles, self.weights_view())
    }
    /// `EvalD1`: the point and first derivative; `core.eval_d1(0.5)?`.
    pub fn eval_d1(&self, u: f64) -> Result<KindD1<K>, Failure> {
        if let Some(result) = K::rep_d1(&self.eval_rep, u)? {
            return Ok(result);
        }
        bsplclib::bezier_d1(u, &self.poles, self.weights_view())
    }
    /// `EvalD2`: the point and two derivatives; `core.eval_d2(0.5)?`.
    pub fn eval_d2(&self, u: f64) -> Result<KindD2<K>, Failure> {
        if let Some(result) = K::rep_d2(&self.eval_rep, u)? {
            return Ok(result);
        }
        bsplclib::bezier_d2(u, &self.poles, self.weights_view())
    }
    /// `EvalD3`: the point and three derivatives; `core.eval_d3(0.5)?`.
    pub fn eval_d3(&self, u: f64) -> Result<KindD3<K>, Failure> {
        if let Some(result) = K::rep_d3(&self.eval_rep, u)? {
            return Ok(result);
        }
        bsplclib::bezier_d3(u, &self.poles, self.weights_view())
    }
    /// `EvalDN`: the derivative of order `n >= 1`; `core.eval_dn(0.5, 4)?`.
    pub fn eval_dn(&self, u: f64, n: i32) -> Result<KindVector<K>, Failure> {
        if n < 1 {
            return Err(K::undefined_derivative(&format!(
                "{}::EvalDN: N = {n}; expected N >= 1",
                K::BEZIER_CLASS
            )));
        }
        if let Some(vector) = K::rep_dn(&self.eval_rep, u, n)? {
            return Ok(vector);
        }
        let (knots, mults) = (bezier_knots(), self.multiplicities()?);
        let span = CurveSpan {
            u,
            index: 0,
            degree: self.degree(),
            periodic: false,
            knots: &knots,
            mults: Some(&mults),
        };
        bsplclib::dn(&span, n, &self.poles, self.weights_view())
    }
}
