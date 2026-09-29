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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve_1.cxx and ModelingData/TKG3d/Geom/Geom_BSplineCurve_1.cxx (evaluation, continuity, end points) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation, local evaluation, continuity and end points.
use super::{BSplineCurveCore, BSplineCurveKind, KindD1, KindD2, KindD3, KindVector};
use crate::geom2d::arrays::at;
use crate::geom2d::errors;
use crate::geom2d::spline_data::spline_is_cn;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, CurveSpan};

impl<K: BSplineCurveKind> BSplineCurveCore<K> {
    // The span location of EvalD0..D3. Quirk kept from the native code: the normalized
    // parameter is computed, then overwritten, because LocateParameter receives the
    // raw `U` and wraps it itself.
    fn eval_span(&self, u: f64) -> Result<CurveSpan<'_, 'static>, Failure> {
        let mut index: i32 = 0;
        let mut new_u: f64 = self.periodic_normalization(u)?;
        bsplclib::locate_parameter_for_eval(
            self.deg,
            &self.knots,
            Some(&self.mults),
            u,
            self.periodic,
            &mut index,
            &mut new_u,
        )?;
        if new_u < at(&self.knots, index)? {
            index -= 1;
        }
        Ok(self.span(new_u, index, false))
    }
    fn span(&self, u: f64, index: i32, flat: bool) -> CurveSpan<'_, 'static> {
        CurveSpan {
            u,
            index,
            degree: self.deg,
            periodic: self.periodic,
            knots: if flat { &self.flat_knots } else { &self.knots },
            mults: if flat { None } else { Some(&self.mults) },
        }
    }
    /// `EvalD0`, through the evaluation representation when it answers;
    /// `core.eval_d0(0.5)?`.
    pub fn eval_d0(&self, u: f64) -> Result<K::Point, Failure> {
        if let Some(point) = K::rep_d0(&self.eval_rep, u)? {
            return Ok(point);
        }
        bsplclib::d0(&self.eval_span(u)?, &self.poles, self.weights_view())
    }
    /// `EvalD1`: the point and first derivative; `core.eval_d1(0.5)?`.
    pub fn eval_d1(&self, u: f64) -> Result<KindD1<K>, Failure> {
        if let Some(result) = K::rep_d1(&self.eval_rep, u)? {
            return Ok(result);
        }
        bsplclib::d1(&self.eval_span(u)?, &self.poles, self.weights_view())
    }
    /// `EvalD2`: the point and two derivatives; `core.eval_d2(0.5)?`.
    pub fn eval_d2(&self, u: f64) -> Result<KindD2<K>, Failure> {
        if let Some(result) = K::rep_d2(&self.eval_rep, u)? {
            return Ok(result);
        }
        bsplclib::d2(&self.eval_span(u)?, &self.poles, self.weights_view())
    }
    /// `EvalD3`: the point and three derivatives; `core.eval_d3(0.5)?`.
    pub fn eval_d3(&self, u: f64) -> Result<KindD3<K>, Failure> {
        if let Some(result) = K::rep_d3(&self.eval_rep, u)? {
            return Ok(result);
        }
        let span: CurveSpan<'_, 'static> = self.eval_span(u)?;
        bsplclib::d3(&span, &self.poles, self.weights_view())
    }
    /// `EvalDN`: the derivative of order `n >= 1`; `core.eval_dn(0.5, 4)?`.
    pub fn eval_dn(&self, u: f64, n: i32) -> Result<KindVector<K>, Failure> {
        if n < 1 {
            return Err(K::undefined_derivative(&format!(
                "{}::EvalDN: N = {n}; expected N >= 1",
                K::CLASS
            )));
        }
        if let Some(vector) = K::rep_dn(&self.eval_rep, u, n)? {
            return Ok(vector);
        }
        bsplclib::dn(&self.span(u, 0, true), n, &self.poles, self.weights_view())
    }
    // The span of LocalD0..DN: located in the flat knots between FromK1 and ToK2.
    fn local_span(
        &self,
        u: f64,
        from_k1: i32,
        to_k2: i32,
        member: &str,
    ) -> Result<CurveSpan<'_, 'static>, Failure> {
        if from_k1 == to_k2 {
            return Err(errors::domain(&format!(
                "{}::{member}: FromK1 = ToK2 = {from_k1}; expected distinct knots",
                K::CLASS
            )));
        }
        let (mut local_u, mut index): (f64, i32) = (u, 0);
        bsplclib::locate_parameter_flat_knots(
            self.deg,
            &self.flat_knots,
            u,
            self.periodic,
            from_k1,
            to_k2,
            &mut index,
            &mut local_u,
        )?;
        let flat_index: i32 = bsplclib::flat_index(self.deg, index, &self.mults, self.periodic)?;
        Ok(self.span(local_u, flat_index, true))
    }
    /// `LocalD0`: evaluation on the knot span `[FromK1, ToK2]`; `core.local_d0(0.5, 1, 2)?`.
    pub fn local_d0(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<K::Point, Failure> {
        let span = self.local_span(u, from_k1, to_k2, "LocalValue")?;
        bsplclib::d0(&span, &self.poles, self.weights_view())
    }
    /// `LocalD1`; `core.local_d1(0.5, 1, 2)?`.
    pub fn local_d1(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<KindD1<K>, Failure> {
        let span = self.local_span(u, from_k1, to_k2, "LocalD1")?;
        bsplclib::d1(&span, &self.poles, self.weights_view())
    }
    /// `LocalD2`; `core.local_d2(0.5, 1, 2)?`.
    pub fn local_d2(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<KindD2<K>, Failure> {
        let span = self.local_span(u, from_k1, to_k2, "LocalD2")?;
        bsplclib::d2(&span, &self.poles, self.weights_view())
    }
    /// `LocalD3`; `core.local_d3(0.5, 1, 2)?`.
    pub fn local_d3(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<KindD3<K>, Failure> {
        let span = self.local_span(u, from_k1, to_k2, "LocalD3")?;
        bsplclib::d3(&span, &self.poles, self.weights_view())
    }
    /// `LocalDN`; `core.local_dn(0.5, 1, 2, 4)?`.
    pub fn local_dn(
        &self,
        u: f64,
        from_k1: i32,
        to_k2: i32,
        n: i32,
    ) -> Result<KindVector<K>, Failure> {
        // The native message names LocalD3 here too.
        let span = self.local_span(u, from_k1, to_k2, "LocalD3")?;
        bsplclib::dn(&span, n, &self.poles, self.weights_view())
    }
    /// `IsCN`; Standard_RangeError for `n < 0`; `core.is_cn(2)?`.
    pub fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "{}::IsCN: N = {n}; expected N >= 0",
                K::CLASS
            )));
        }
        spline_is_cn(self.smooth, n, self.deg, &self.mults)
    }
    /// `StartPoint`: the first pole when the first knot is fully multiple;
    /// `core.start_point()?`.
    pub fn start_point(&self) -> Result<K::Point, Failure> {
        if at(&self.mults, 1)? == self.deg + 1 {
            return at(&self.poles, 1);
        }
        self.eval_d0(self.first_parameter())
    }
    /// `EndPoint`: the last pole when the last knot is fully multiple;
    /// `core.end_point()?`.
    pub fn end_point(&self) -> Result<K::Point, Failure> {
        if at(&self.mults, self.knots.upper())? == self.deg + 1 {
            return at(&self.poles, self.poles.upper());
        }
        self.eval_d0(self.last_parameter())
    }
    /// `IsG1`: tangent directions agree within `ang_tol` at every interior knot of
    /// multiplicity >= Degree in `[tf, tl]`, and across the seam of a periodic curve;
    /// `core.is_g1(0.0, 1.0, 1e-12)?`.
    pub fn is_g1(&self, tf: f64, tl: f64, ang_tol: f64) -> Result<bool, Failure> {
        if self.is_cn(1)? {
            return Ok(true);
        }
        let (start, finish) = (
            self.first_u_knot_index()? + 1,
            self.last_u_knot_index()? - 1,
        );
        for knot_index in start..=finish {
            let t: f64 = at(&self.knots, knot_index)?;
            if t < tf {
                continue;
            }
            if t > tl {
                break;
            }
            if at(&self.mults, knot_index)? < self.deg {
                continue;
            }
            let (_, v1) = self.local_d1(t, knot_index - 1, knot_index)?;
            let (_, v2) = self.local_d1(t, knot_index, knot_index + 1)?;
            if !K::tangents_agree(v1, v2, ang_tol)? {
                return Ok(false);
            }
        }
        if !self.periodic {
            return Ok(true);
        }
        self.seam_is_g1(tf, tl, ang_tol)
    }
    fn seam_is_g1(&self, tf: f64, tl: f64, ang_tol: f64) -> Result<bool, Failure> {
        let (first, last) = (self.first_parameter(), self.last_parameter());
        if ((first - tf) * (tl - first) < 0.0) && ((last - tf) * (tl - last) < 0.0) {
            return Ok(true);
        }
        let (_, v1) = self.eval_d1(at(&self.knots, self.first_u_knot_index()?)?)?;
        let (_, v2) = self.eval_d1(at(&self.knots, self.last_u_knot_index()?)?)?;
        K::tangents_agree(v1, v2, ang_tol)
    }
}
