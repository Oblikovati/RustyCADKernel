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
// Rust translation of the static helpers of IntCurve_IntConicConic.cxx (SET_BOUNDED_DOMAIN,
// SetBinfBsupFromIntAna2d and the domain clipping shared by its Perform overloads) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parametric range of an unbounded conic that IntCurve_IntConicConic hands to
//! IntImpConicParConic: the analytic intersections of offset copies, widened, then
//! clipped by the conic's domain.
use super::{IConicTool, IntConicConic, PConic};
use crate::int_res2d::Domain;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib::{self, ElementaryCurve};
use rustycad_tkmath::gp::{Point2, Vector2};

/// `PARAM_MAX_ON_PARABOLA`: analytic parameters beyond it are ignored.
pub(super) const PARAM_MAX_ON_PARABOLA: f64 = 100000000.0;
/// `PARAM_MAX_ON_HYPERBOLA`.
pub(super) const PARAM_MAX_ON_HYPERBOLA: f64 = 10000.0;
/// `TOL_EXACT_INTER`: the tolerance of the first, exact, line-parabola search.
pub(super) const TOL_EXACT_INTER: f64 = 1.0e-7;

/// The accumulated range `[binf, bsup]` on the parametric conic and its end points.
#[derive(Clone, Copy, Debug)]
pub(super) struct AnaBounds {
    pub(super) binf: f64,
    pub(super) pntinf: Point2,
    pub(super) bsup: f64,
    pub(super) pntsup: Point2,
}

impl AnaBounds {
    /// The empty range: `binf = Precision::Infinite()`, `bsup = -Infinite()`.
    pub(super) fn new() -> Self {
        Self {
            binf: precision::infinite(),
            pntinf: Point2::default(),
            bsup: -precision::infinite(),
            pntsup: Point2::default(),
        }
    }

    /// `SetBinfBsupFromIntAna2d(theIntAna2d, binf, pntinf, bsup, pntsup, Curve, maxtol,
    /// LIMITE)`: widens the range by the analytic points of parameter below `limite`
    /// (on the first curve of the IntAna2d result, `curve`), each by
    /// `100 maxtol / |D1|` (at least 1e-3) on both sides.
    pub(super) fn widen<C>(
        &mut self,
        ana: &AnaIntersection,
        curve: &C,
        maxtol: f64,
        limite: f64,
    ) -> Result<(), Failure>
    where
        C: ElementaryCurve<Point = Point2, Vector = Vector2>,
    {
        if !ana.is_done() || ana.is_empty()? {
            return Ok(());
        }
        for p in 1..=ana.nb_points()? {
            let mut param: f64 = ana.point(p)?.param_on_first();
            if param.abs() >= limite {
                continue;
            }
            let (_, v) = elclib::d1(param, curve);
            let mut dparam: f64 = 100.0 * maxtol / v.magnitude();
            if dparam < 1e-3 {
                dparam = 1e-3;
            }
            param -= dparam;
            if param < self.binf {
                self.binf = param;
                self.pntinf = elclib::value(param, curve);
            }
            param += dparam + dparam;
            if param > self.bsup {
                self.bsup = param;
                self.pntsup = elclib::value(param, curve);
            }
        }
        Ok(())
    }

    /// `SET_BOUNDED_DOMAIN(domain, binf, tolinf, Pntinf, bsup, tolsup, Pntsup)` on a
    /// domain with at most one bound: the range is clipped by that bound (with its
    /// tolerance, the others 0) and must stay non-empty. The native test of the last
    /// bound compares `bsup` with the domain's FIRST parameter, which raises
    /// Standard_DomainError on a domain with only a last point; both are kept.
    pub(super) fn set_bounded_domain(
        &mut self,
        domain: &Domain,
    ) -> Result<Option<Domain>, Failure> {
        let (mut tolinf, mut tolsup) = (0.0, 0.0);
        if domain.has_first_point() && self.binf < domain.first_parameter()? {
            self.pntinf = domain.first_point()?;
            self.binf = domain.first_parameter()?;
            tolinf = domain.first_tolerance()?;
        }
        if domain.has_last_point() && self.bsup > domain.first_parameter()? {
            self.pntsup = domain.last_point()?;
            self.bsup = domain.last_parameter()?;
            tolsup = domain.last_tolerance()?;
        }
        if self.bsup > self.binf {
            return Ok(Some(self.domain(tolinf, tolsup)));
        }
        Ok(None)
    }

    /// The bounded domain of the range with the given end tolerances.
    pub(super) fn domain(&self, tolinf: f64, tolsup: f64) -> Domain {
        Domain::bounded(
            self.pntinf,
            self.binf,
            tolinf,
            self.pntsup,
            self.bsup,
            tolsup,
        )
    }

    /// The range clipped by a bounded domain, with the domain's tolerances at the
    /// clipped ends; `None` when it misses the domain. `early` is the check, after
    /// each clip, that the other end is on the domain; `ordered` the final check
    /// that the range is not empty (lbr, 22 Sep 97: binf = -5.6, bsup = -5.3 and a
    /// domain 5.6..5.7 give binf = +5.6 > bsup).
    pub(super) fn clip(
        &mut self,
        domain: &Domain,
        early: bool,
        ordered: bool,
    ) -> Result<Option<Domain>, Failure> {
        let (mut ft, mut lt) = (0.0, 0.0);
        if self.binf < domain.first_parameter()? {
            self.binf = domain.first_parameter()?;
            self.pntinf = domain.first_point()?;
            ft = domain.first_tolerance()?;
            if early && self.bsup < domain.first_parameter()? {
                return Ok(None);
            }
        }
        if self.bsup > domain.last_parameter()? {
            self.bsup = domain.last_parameter()?;
            self.pntsup = domain.last_point()?;
            lt = domain.last_tolerance()?;
            if early && self.binf > domain.last_parameter()? {
                return Ok(None);
            }
        }
        if ordered && self.binf >= self.bsup {
            return Ok(None);
        }
        Ok(Some(self.domain(ft, lt)))
    }
}

/// `BOUNDED_DOMAIN(domain)`: both bounds exist.
pub(super) fn bounded_domain(domain: &Domain) -> bool {
    domain.has_first_point() && domain.has_last_point()
}

/// How the bounded parametric domain clips the analytic range (see
/// [`AnaBounds::clip`]).
#[derive(Clone, Copy, Debug)]
pub(super) struct ClipMode {
    pub(super) early: bool,
    pub(super) ordered: bool,
}

/// The clipping of most overloads: early checks, no final order check.
pub(super) const CLIP_EARLY: ClipMode = ClipMode {
    early: true,
    ordered: false,
};

/// The implicit conic, its domain, the parametric conic and its domain.
pub(super) struct ConicPair<'a> {
    pub(super) itool: &'a IConicTool,
    pub(super) imp_domain: &'a Domain,
    pub(super) pcurve: &'a PConic,
    pub(super) par_domain: &'a Domain,
}

impl IntConicConic {
    /// The end shared by the Perform overloads of IntCurve_IntConicConic.cxx: an
    /// empty analytic range is an empty result; otherwise IntImpConicParConic runs on
    /// the range reduced by the parametric domain (clipped per `mode` when it is
    /// bounded) and its result is copied.
    pub(super) fn perform_on_bounds(
        &mut self,
        pair: &ConicPair<'_>,
        mut bounds: AnaBounds,
        (tol_conf, tol): (f64, f64),
        mode: ClipMode,
    ) -> Result<(), Failure> {
        if bounds.binf > bounds.bsup || bounds.binf.is_nan() || bounds.bsup.is_nan() {
            self.base.done = true;
            return Ok(());
        }
        let reduced: Option<Domain> = if bounded_domain(pair.par_domain) {
            bounds.clip(pair.par_domain, mode.early, mode.ordered)?
        } else {
            bounds.set_bounded_domain(pair.par_domain)?
        };
        let Some(par_domain) = reduced else {
            self.base.done = true;
            return Ok(());
        };
        self.inter.perform(
            pair.itool,
            pair.imp_domain,
            pair.pcurve,
            &par_domain,
            tol_conf,
            tol,
        )?;
        self.base.set_values(&self.inter);
        Ok(())
    }
}
