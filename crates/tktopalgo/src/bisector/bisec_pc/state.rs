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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecPC.cxx (the fields, Values, Extension and Distance) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The state of `Bisector_BisecPC` and its evaluation: the point of the bisector
//! associated with each point of the curve, and the extensions by half lines.
use crate::bisector::intervals::Intervals;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{self, Direction2, Point2, Vector2};

/// The fields of `Bisector_BisecPC`. A curve built by `new Bisector_BisecPC(...)`
/// starts from zeroed memory (the native allocator clears it), which the
/// default here reproduces; the default constructor then marks it empty.
#[derive(Clone, Default)]
pub(crate) struct PcState {
    pub curve: Handle<dyn Geom2dCurve>,
    pub point: Point2,
    pub sign: f64,
    pub start_intervals: Intervals,
    pub end_intervals: Intervals,
    pub bis_interval: i32,
    pub current_interval: i32,
    pub shift_parameter: f64,
    pub dist_max: f64,
    pub is_empty: bool,
    pub is_convex: bool,
    pub extension_start: bool,
    pub extension_end: bool,
    pub point_start_bis: Point2,
    pub point_end_bis: Point2,
}

/// The outputs of `Values`: the point and three derivatives, each left as the
/// caller initialised it when the member returns early.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PcValues {
    pub p: Point2,
    pub v1: Vector2,
    pub v2: Vector2,
    pub v3: Vector2,
}

// The curve point and derivatives Values reads: D1 for N = 0, D2 for 1, D3 for 2
// and none for another N (natively the switch has no case, so the point and the
// tangents keep their default zero value).
fn curve_terms(
    curve: &dyn Geom2dCurve,
    u: f64,
    n: i32,
) -> Result<(Point2, Vector2, Vector2, Vector2), Failure> {
    let zero = Vector2::new(0., 0.);
    match n {
        0 => {
            let (pc, tu) = curve.d1(u)?;
            Ok((pc, tu, zero, zero))
        }
        1 => {
            let (pc, tu, tuu) = curve.d2(u)?;
            Ok((pc, tu, tuu, zero))
        }
        2 => curve.d3(u),
        _ => Ok((Point2::default(), zero, zero, zero)),
    }
}

impl PcState {
    /// `LinkBisCurve(U)`: `U - shiftParameter`.
    pub(crate) fn link_bis_curve(&self, u: f64) -> f64 {
        u - self.shift_parameter
    }
    /// `LinkCurveBis(U)`: `U + shiftParameter`.
    pub(crate) fn link_curve_bis(&self, u: f64) -> f64 {
        u + self.shift_parameter
    }
    /// `Values(U, N, P, V1, V2, V3)` on outputs initialised by the caller: the point
    /// `F(u) = P(u) - 1/2 |PP(u)|^2 / (N(u).PP(u)) N(u)` and its derivatives.
    pub(crate) fn values(&self, u: f64, n: i32, mut out: PcValues) -> Result<PcValues, Failure> {
        if u < self.start_intervals.value(self.bis_interval)?
            || u > self.end_intervals.value(self.bis_interval)?
        {
            return self.extension(u, out);
        }
        let u_on_curve: f64 = self.link_bis_curve(u);
        let (pc, tu, tuu, t3u) = curve_terms(&*self.curve, u_on_curve, n)?;
        let a_ppc = Vector2::new(pc.x() - self.point.x(), pc.y() - self.point.y());
        let nor = Vector2::new(-tu.y(), tu.x());
        let square_ppc: f64 = a_ppc.square_magnitude();
        let nor_ppc: f64 = nor.dot(a_ppc);
        if !(nor_ppc.abs() > gp::resolution() && (nor_ppc * self.sign) < 0.) {
            return Ok(out);
        }
        let a1: f64 = 0.5 * square_ppc / nor_ppc;
        out.p = Point2::new(pc.x() - nor.x() * a1, pc.y() - nor.y() * a1);
        if n == 0 {
            return Ok(out);
        }
        let terms = FirstOrder::new(tu, tuu, a_ppc, square_ppc, nor_ppc);
        out.v1 = tu - a1 * terms.nu - terms.a2 * nor;
        if n == 1 {
            return Ok(out);
        }
        out.v2 = terms.second_derivative(t3u, a_ppc, square_ppc, nor_ppc, a1, nor);
        Ok(out)
    }
    /// `Extension(U, P, V1, V2, V3)`: the half lines that extend the bisector before
    /// its start and after its end, from the point through the bounds.
    pub(crate) fn extension(&self, u: f64, mut out: PcValues) -> Result<PcValues, Failure> {
        out.v1 = Vector2::new(0., 0.);
        out.v2 = Vector2::new(0., 0.);
        out.v3 = Vector2::new(0., 0.);
        let start: f64 = self.start_intervals.value(self.bis_interval)?;
        if u < start {
            return self.extend(
                out,
                u - start,
                self.point_start_bis,
                self.point,
                self.point_start_bis,
            );
        }
        let end: f64 = self.end_intervals.value(self.bis_interval)?;
        if u > end {
            return self.extend(
                out,
                u - end,
                self.point_end_bis,
                self.point_end_bis,
                self.point,
            );
        }
        Ok(out)
    }
    // One extension: from `bound` along the direction `from -> to`, or the bound
    // itself when it is the point.
    fn extend(
        &self,
        mut out: PcValues,
        du: f64,
        bound: Point2,
        from: Point2,
        to: Point2,
    ) -> Result<PcValues, Failure> {
        if bound.is_equal(self.point, precision::p_confusion()) {
            out.p = bound;
            return Ok(out);
        }
        let dir_ext = Direction2::new(to.x() - from.x(), to.y() - from.y())?;
        out.p = Point2::new(bound.x() + du * dir_ext.x(), bound.y() + du * dir_ext.y());
        out.v1 = Vector2::new(dir_ext.x(), dir_ext.y());
        Ok(out)
    }
    /// `Distance(U)`: the square distance from the bisector point of `u` to the
    /// curve and the point, `1/4 |PP|^4 / (Nor.PP)^2 |Nor|^2`; infinite where the
    /// bisector point does not exist (and on the point of a concave curve).
    pub(crate) fn distance(&self, u: f64) -> Result<f64, Failure> {
        let u_on_curve: f64 = self.link_bis_curve(u);
        let (pc, tan) = self.curve.d1(u_on_curve)?;
        let a_ppc = Vector2::new(pc.x() - self.point.x(), pc.y() - self.point.y());
        let nor = Vector2::new(-tan.y(), tan.x());
        let nor_nor: f64 = nor.square_magnitude();
        let square_mag_ppc: f64 = a_ppc.square_magnitude();
        let prosca: f64 = nor.dot(a_ppc);
        if self.point.is_equal(pc, precision::confusion()) {
            // A point on a concave curve is not the required common point; this
            // avoids a discontinuity of the bisector.
            return Ok(if self.is_convex {
                0.
            } else {
                precision::infinite()
            });
        }
        if prosca.abs() < precision::confusion() || (prosca * self.sign) > 0. {
            return Ok(precision::infinite());
        }
        let a: f64 = 0.5 * square_mag_ppc / prosca;
        Ok(a * a * nor_nor)
    }
}

// The first order terms of Values: the derivative of the normal and A2.
struct FirstOrder {
    tuu: Vector2,
    nu: Vector2,
    nu_ppc: f64,
    tu_ppc: f64,
    nor_ppc_e2: f64,
    a2: f64,
}

impl FirstOrder {
    fn new(tu: Vector2, tuu: Vector2, a_ppc: Vector2, square_ppc: f64, nor_ppc: f64) -> Self {
        // The derivative of the normal by U.
        let nu = Vector2::new(-tuu.y(), tuu.x());
        let nu_ppc: f64 = nu.dot(a_ppc);
        let tu_ppc: f64 = tu.dot(a_ppc);
        let nor_ppc_e2: f64 = nor_ppc * nor_ppc;
        let a2: f64 = tu_ppc / nor_ppc - 0.5 * nu_ppc * square_ppc / nor_ppc_e2;
        Self {
            tuu,
            nu,
            nu_ppc,
            tu_ppc,
            nor_ppc_e2,
            a2,
        }
    }
    // `V2 = Tuu - 2 A2 Nu - A1 Nuu - A2u Nor`.
    fn second_derivative(
        &self,
        t3u: Vector2,
        a_ppc: Vector2,
        square_ppc: f64,
        nor_ppc: f64,
        a1: f64,
        nor: Vector2,
    ) -> Vector2 {
        let nuu = Vector2::new(-t3u.y(), t3u.x());
        let nor_ppc_e4: f64 = self.nor_ppc_e2 * self.nor_ppc_e2;
        let nuu_ppc: f64 = nuu.dot(a_ppc);
        let tuu_ppc: f64 = self.tuu.dot(a_ppc);
        let a21: f64 = tuu_ppc / nor_ppc - self.tu_ppc * self.nu_ppc / self.nor_ppc_e2;
        let a22: f64 = (0.5 * nuu_ppc * square_ppc + self.nu_ppc * self.tu_ppc) / self.nor_ppc_e2
            - self.nu_ppc * square_ppc * nor_ppc * self.nu_ppc / nor_ppc_e4;
        let a2u: f64 = a21 - a22;
        self.tuu - 2. * self.a2 * self.nu - a1 * nuu - a2u * nor
    }
}
