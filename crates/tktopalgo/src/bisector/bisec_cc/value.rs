// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.cxx (ValueAndDist) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ValueAndDist`: the point of the bisector of two curves by an iterative method
//! started from the polygon, then by intersection when it fails.
use super::state::{CcState, OnCurves};
use crate::bisector::{BisecPC, BisectorFunctionH};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, real};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::gp::{self, Direction2, Point2, Vector2};
use rustycad_tkmath::math::{BissecNewton, Function, FunctionRoot};

// The polygon bracket of `u`: the parameter bounds on C2 and the linear start
// point `VInit`.
struct Bracket {
    v_min: f64,
    v_max: f64,
    v_init: f64,
}

impl CcState {
    /// `ValueAndDist(U, U1, U2, Distance)`: the point of parameter `u`, the
    /// parameters of its projections on the two curves and its square distance to
    /// them (`OnCurves`, in-out).
    ///
    /// The point `F(u, v) = P1(u) - 1/2 |P2(v) P1(u)|^2 / (N(u).P2(v)P1(u)) N(u)`
    /// where `v` solves `H(u, v) = 0` (Bisector_FunctionH); outside the polygon it is
    /// on the extensions.
    pub(crate) fn value_and_dist(&self, u: f64, out: &mut OnCurves) -> Result<Point2, Failure> {
        let poly = &self.my_polygon;
        if poly.length() <= 1 || u < poly.first().param_on_bis() || u > poly.last().param_on_bis() {
            return Ok(self.extension(u, out)?.0);
        }
        let bracket: Bracket = self.bracket(u)?;
        out.u1 = self.link_bis_curve(u);
        let (p1, t1) = self.curve1.d1(out.u1)?;
        let n1 = Vector2::new(t1.y(), -t1.x());
        let solved: Option<f64> = self.solve_h(&bracket, p1, t1)?;
        if let Some(u2) = solved {
            out.u2 = u2;
        }
        let mut valid: bool = solved.is_some();
        let mut pbis: Point2 = self.point_start;
        if valid {
            match self.point_from(p1, n1, out)? {
                Some(point) => pbis = point,
                None => valid = false,
            }
        }
        if !valid {
            pbis = self.point_by_intersection(p1, n1, &bracket, pbis, out)?;
        }
        Ok(pbis)
    }

    // The bracket of `u` in the polygon, with the bounds sorted.
    fn bracket(&self, u: f64) -> Result<Bracket, Failure> {
        let poly = &self.my_polygon;
        let index: i32 = poly.interval(u)?;
        let (low, high) = (poly.value(index)?, poly.value(index + 1)?);
        let (u_min, u_max) = (low.param_on_bis(), high.param_on_bis());
        let (v_min, v_max) = (low.param_on_c2(), high.param_on_c2());
        let v_init: f64 = if (u_max - u_min).abs() < gp::resolution() {
            v_min
        } else {
            let alpha: f64 = (u - u_min) / (u_max - u_min);
            v_min + alpha * (v_max - v_min)
        };
        Ok(Bracket {
            v_min: std_min(v_min, v_max),
            v_max: std_max(v_min, v_max),
            v_init,
        })
    }

    // The parameter on C2 solution of H(u, v) = 0: the start point when H is small
    // there, else Bisector_FunctionH by math_BissecNewton, then math_FunctionRoot.
    fn solve_h(&self, bracket: &Bracket, p1: Point2, t1: Vector2) -> Result<Option<f64>, Failure> {
        let eps_h: f64 = 1.0E-9;
        let eps_h100: f64 = 1.0E-7;
        if (bracket.v_max - bracket.v_min) < precision::p_confusion() {
            return Ok(Some(bracket.v_init));
        }
        let mut h = BisectorFunctionH::new(&self.curve2, p1, self.sign1 * self.sign2 * t1)?;
        let mut f_init: f64 = 0.;
        h.value(bracket.v_init, &mut f_init);
        h.raised()?;
        if f_init.abs() < eps_h {
            return Ok(Some(bracket.v_init));
        }
        let (low, high) = (bracket.v_min - eps_h100, bracket.v_max + eps_h100);
        let mut newton = BissecNewton::new(eps_h);
        newton.perform(&mut h, low, high, 10);
        h.raised()?;
        if newton.is_done() {
            return Ok(Some(newton.root()?));
        }
        let root = FunctionRoot::bounded(&mut h, bracket.v_init, eps_h, low, high, 100)?;
        h.raised()?;
        if root.is_done() {
            return Ok(Some(root.root()?));
        }
        Ok(None)
    }

    // `P(U) = F(U1, U2)`: the point on the normal of C1; `None` when the second
    // curve point is on the wrong side (the point is then not valid).
    fn point_from(
        &self,
        p1: Point2,
        n1: Vector2,
        out: &mut OnCurves,
    ) -> Result<Option<Point2>, Failure> {
        let p2: Point2 = self.curve2.value(out.u2)?;
        let p2p1 = Vector2::new(p1.x() - p2.x(), p1.y() - p2.y());
        let square_p2p1: f64 = p2p1.square_magnitude();
        let n1p2p1: f64 = n1.dot(p2p1);
        let an_eps: f64 = real::epsilon(1.);
        if p1.is_equal(p2, precision::confusion()) {
            out.dist = 0.0;
            return Ok(Some(p1));
        }
        if n1p2p1 * self.sign1 < an_eps {
            return Ok(None);
        }
        let pbis: Point2 = p1.translated(-(0.5 * square_p2p1 / n1p2p1) * n1);
        out.dist = p1.square_distance(pbis);
        Ok(Some(pbis))
    }

    // The point by intersection of the point-curve bisector of P1 and C2 (limited to
    // the bracket) with the normal of C1 at P1: the closest on the side of the
    // normal; `pbis` when there is none.
    #[allow(clippy::neg_cmp_op_on_partial_ord)] // The negated native test keeps its NaN answer.
    fn point_by_intersection(
        &self,
        p1: Point2,
        n1: Vector2,
        bracket: &Bracket,
        mut pbis: Point2,
        out: &mut OnCurves,
    ) -> Result<Point2, Failure> {
        let bis_pc =
            BisecPC::with_bounds(&self.curve2, p1, self.sign2, bracket.v_min, bracket.v_max)?;
        let bis_pc: Handle<BisecPC> = Handle::new(bis_pc);
        let nor_li: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::from_point(p1, Direction2::from_vector(n1)?)).cast();
        let a_bis_pc = Geom2dAdaptorCurve::with_curve(&bis_pc.cast::<dyn Geom2dCurve>())?;
        let a_nor_li = Geom2dAdaptorCurve::with_curve(&nor_li)?;
        let tol: f64 = precision::confusion();
        let intersect = GInter::between(&a_bis_pc, &a_nor_li, tol, tol)?;
        if !intersect.is_done() || intersect.is_empty()? {
            return Ok(pbis);
        }
        let mut d_min: f64 = precision::infinite();
        for i in 1..=intersect.nb_points()? {
            let point = intersect.point(i)?;
            if !(point.param_on_second() * self.sign1 < precision::p_confusion()) {
                continue;
            }
            let p: Point2 = point.value();
            if p.square_distance(p1) < d_min {
                d_min = p.square_distance(p1);
                pbis = p;
                out.u2 = bis_pc.link_bis_curve(point.param_on_first());
                out.dist = d_min;
            }
        }
        Ok(pbis)
    }
}
