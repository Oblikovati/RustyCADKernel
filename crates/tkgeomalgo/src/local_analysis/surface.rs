// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/LocalAnalysis/LocalAnalysis_SurfaceContinuity.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocalAnalysis_SurfaceContinuity`: the continuity between two surfaces at a point.
use super::{LocalAnalysisStatusErrorType, folded_angle, not_done};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

/// The tolerances of a surface continuity analysis, with the native defaults
/// (`EpsNul`...`Maxlen` of `LocalAnalysis_SurfaceContinuity`).
///
/// `SurfaceContinuityTolerances { eps_g1: 1e-4, ..Default::default() }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceContinuityTolerances {
    /// `EpsNul`: the null derivative length (0.001).
    pub eps_nul: f64,
    /// `EpsC0`: the C0 distance (0.001).
    pub eps_c0: f64,
    /// `EpsC1`: the C1 angle (0.001).
    pub eps_c1: f64,
    /// `EpsC2`: the C2 angle (0.001).
    pub eps_c2: f64,
    /// `EpsG1`: the G1 angle (0.001).
    pub eps_g1: f64,
    /// `Percent`: the relative curvature gap of G2 (0.01).
    pub percent: f64,
    /// `Maxlen`: the largest length, which bounds the null curvature (10000).
    pub maxlen: f64,
}

impl Default for SurfaceContinuityTolerances {
    fn default() -> Self {
        Self {
            eps_nul: 0.001,
            eps_c0: 0.001,
            eps_c1: 0.001,
            eps_c2: 0.001,
            eps_g1: 0.001,
            percent: 0.01,
            maxlen: 10000.0,
        }
    }
}

/// The continuity between two surfaces at a point of each
/// (`LocalAnalysis_SurfaceContinuity`).
///
/// `let c = LocalAnalysisSurfaceContinuity::new(&s1, (0.0, 0.5), &s2, (1.0, 0.5), Shape::G1, &Default::default())?;`
#[derive(Clone, Debug)]
pub struct LocalAnalysisSurfaceContinuity {
    my_cont_c0: f64,
    my_cont_c1u: f64,
    my_cont_c1v: f64,
    my_cont_c2u: f64,
    my_cont_c2v: f64,
    my_cont_g1: f64,
    my_lambda1u: f64,
    my_lambda2u: f64,
    my_lambda1v: f64,
    my_lambda2v: f64,
    my_eta1: f64,
    my_eta2: f64,
    my_eta: f64,
    my_zeta1: f64,
    my_zeta2: f64,
    my_zeta: f64,
    my_alpha: f64,
    my_type_cont: Option<Shape>,
    eps: SurfaceContinuityTolerances,
    my_gap: f64,
    my_is_done: bool,
    my_error_status: Option<LocalAnalysisStatusErrorType>,
}

/// The two derivatives compared by one direction of the C1 and C2 tests.
struct Pair {
    first: Vector3,
    second: Vector3,
}

/// `(|a| + |b|) / 2` of the curvature directions, the mean curvature direction of `SurfG2`.
fn mean_direction((dmin, dmax): (Direction3, Direction3)) -> Result<Direction3, Failure> {
    let (x1, y1, z1) = (dmin.x(), dmin.y(), dmin.z());
    let (x2, y2, z2) = (dmax.x(), dmax.y(), dmax.z());
    Direction3::new(
        (x1.abs() + x2.abs()) / 2.0,
        (y1.abs() + y2.abs()) / 2.0,
        (z1.abs() + z2.abs()) / 2.0,
    )
}

impl LocalAnalysisSurfaceContinuity {
    /// An analysis with the given tolerances and nothing computed
    /// (`LocalAnalysis_SurfaceContinuity(EpsNul, ..., Maxlen)`), for `ComputeAnalysis`.
    /// `LocalAnalysisSurfaceContinuity::with_tolerances(&Default::default())`.
    pub fn with_tolerances(eps: &SurfaceContinuityTolerances) -> Self {
        Self {
            my_cont_c0: 0.0,
            my_cont_c1u: 0.0,
            my_cont_c1v: 0.0,
            my_cont_c2u: 0.0,
            my_cont_c2v: 0.0,
            my_cont_g1: 0.0,
            my_lambda1u: 0.0,
            my_lambda2u: 0.0,
            my_lambda1v: 0.0,
            my_lambda2v: 0.0,
            my_eta1: 0.0,
            my_eta2: 0.0,
            my_eta: 0.0,
            my_zeta1: 0.0,
            my_zeta2: 0.0,
            my_zeta: 0.0,
            my_alpha: 0.0,
            my_type_cont: None,
            eps: *eps,
            my_gap: 0.0,
            my_is_done: true,
            my_error_status: None,
        }
    }

    /// Analyses `surf1` at `(u1, v1)` against `surf2` at `(u2, v2)`
    /// (`LocalAnalysis_SurfaceContinuity(Surf1, u1, v1, Surf2, u2, v2, Order, ...)`);
    /// fails as the local properties do.
    /// `LocalAnalysisSurfaceContinuity::new(&s1, (0.0, 0.5), &s2, (1.0, 0.5), Shape::C1, &Default::default())?`.
    pub fn new(
        surf1: &Handle<dyn GeomSurface>,
        (u1, v1): (f64, f64),
        surf2: &Handle<dyn GeomSurface>,
        (u2, v2): (f64, f64),
        order: Shape,
        eps: &SurfaceContinuityTolerances,
    ) -> Result<Self, Failure> {
        let mut analysis = Self::with_tolerances(eps);
        analysis.my_type_cont = Some(order);
        analysis.analyse(surf1, (u1, v1), surf2, (u2, v2), order)?;
        Ok(analysis)
    }

    /// Analyses the surfaces at the points of the two parametric curves at `u`
    /// (`LocalAnalysis_SurfaceContinuity(curv1, curv2, U, Surf1, Surf2, Order, ...)`);
    /// not done when `u` is outside either curve's range.
    /// `LocalAnalysisSurfaceContinuity::on_curves(&c1, &c2, 0.5, &s1, &s2, Shape::G1, &Default::default())?`.
    pub fn on_curves(
        curves: (&Handle<dyn Geom2dCurve>, &Handle<dyn Geom2dCurve>),
        u: f64,
        surf1: &Handle<dyn GeomSurface>,
        surf2: &Handle<dyn GeomSurface>,
        order: Shape,
        eps: &SurfaceContinuityTolerances,
    ) -> Result<Self, Failure> {
        let mut analysis = Self::with_tolerances(eps);
        analysis.my_type_cont = Some(order);
        let (curv1, curv2) = (live_curve(curves.0)?, live_curve(curves.1)?);
        let (pard1, pard2) = (curv1.first_parameter(), curv2.first_parameter());
        let (parf1, parf2) = (curv1.last_parameter(), curv2.last_parameter());
        if u > parf1 || u < pard1 || u > parf2 || u < pard2 {
            analysis.my_is_done = false;
            return Ok(analysis);
        }
        let (pt1, pt2): (Point2, Point2) = (curv1.value(u)?, curv2.value(u)?);
        analysis.analyse(surf1, (pt1.x(), pt1.y()), surf2, (pt2.x(), pt2.y()), order)?;
        Ok(analysis)
    }

    fn analyse(
        &mut self,
        surf1: &Handle<dyn GeomSurface>,
        (u1, v1): (f64, f64),
        surf2: &Handle<dyn GeomSurface>,
        (u2, v2): (f64, f64),
        order: Shape,
    ) -> Result<(), Failure> {
        let level: i32 = match order {
            Shape::C0 => 0,
            Shape::C1 | Shape::G1 => 1,
            Shape::C2 | Shape::G2 => 2,
            _ => return Ok(()),
        };
        let mut surfa1 = SLProps::with_parameters(surf1.clone(), u1, v1, level, self.eps.eps_nul)?;
        let mut surfa2 = SLProps::with_parameters(surf2.clone(), u2, v2, level, self.eps.eps_nul)?;
        self.compute_steps(&mut surfa1, &mut surfa2, order)
    }

    /// Computes the analysis of order `order` on existing local properties
    /// (`ComputeAnalysis`); the order becomes the continuity status.
    /// `analysis.compute_analysis(&mut props1, &mut props2, Shape::G2)?`.
    pub fn compute_analysis(
        &mut self,
        surf1: &mut SLProps,
        surf2: &mut SLProps,
        order: Shape,
    ) -> Result<(), Failure> {
        self.my_type_cont = Some(order);
        self.compute_steps(surf1, surf2, order)
    }

    fn compute_steps(
        &mut self,
        surf1: &mut SLProps,
        surf2: &mut SLProps,
        order: Shape,
    ) -> Result<(), Failure> {
        if matches!(
            order,
            Shape::C0 | Shape::C1 | Shape::C2 | Shape::G1 | Shape::G2
        ) {
            self.my_cont_c0 = surf1.value().distance(&surf2.value());
        }
        match order {
            Shape::C1 => self.surf_c1(surf1, surf2),
            Shape::C2 => {
                self.surf_c1(surf1, surf2)?;
                self.surf_c2(surf1, surf2)
            }
            Shape::G1 => self.surf_g1(surf1, surf2),
            Shape::G2 => {
                self.surf_g1(surf1, surf2)?;
                self.surf_g2(surf1, surf2)
            }
            _ => Ok(()),
        }
    }

    fn fail(&mut self, status: LocalAnalysisStatusErrorType) {
        self.my_is_done = false;
        self.my_error_status = Some(status);
    }

    fn surf_c1(&mut self, surf1: &mut SLProps, surf2: &mut SLProps) -> Result<(), Failure> {
        let u = Pair {
            first: surf1.d1u()?,
            second: surf2.d1u()?,
        };
        let v = Pair {
            first: surf1.d1v()?,
            second: surf2.d1v()?,
        };
        let (n1u, n2u) = (u.first.magnitude(), u.second.magnitude());
        let (n1v, n2v) = (v.first.magnitude(), v.second.magnitude());
        let nul: f64 = self.eps.eps_nul;
        if !(n1u > nul && n2u > nul && n1v > nul && n2v > nul) {
            self.fail(LocalAnalysisStatusErrorType::NullFirstDerivative);
            return Ok(());
        }
        self.my_lambda1u = if n1u >= n2u { n2u / n1u } else { n1u / n2u };
        self.my_lambda1v = if n1v >= n2v { n2v / n1v } else { n1v / n2v };
        self.my_cont_c1u = folded_angle(u.first.angle(u.second)?);
        self.my_cont_c1v = folded_angle(v.first.angle(v.second)?);
        Ok(())
    }

    /// One direction of `SurfC2`: `(Lambda1, Lambda2, angle)` from the first and
    /// second derivatives, or the status of a null one.
    fn second_order(
        &self,
        d1: &Pair,
        d2: &Pair,
    ) -> Result<Result<(f64, f64, f64), LocalAnalysisStatusErrorType>, Failure> {
        let (n11, n12) = (d1.first.magnitude(), d1.second.magnitude());
        let (n21, n22) = (d2.first.magnitude(), d2.second.magnitude());
        let nul: f64 = self.eps.eps_nul;
        if !(n11 > nul && n12 > nul) {
            return Ok(Err(LocalAnalysisStatusErrorType::NullFirstDerivative));
        }
        if !(n21 > nul && n22 > nul) {
            return Ok(Err(LocalAnalysisStatusErrorType::NullSecondDerivative));
        }
        let (lambda1, lambda2) = if n11 >= n12 {
            (n12 / n11, n22 / n21)
        } else {
            (n11 / n12, n21 / n22)
        };
        Ok(Ok((
            lambda1,
            lambda2,
            folded_angle(d2.first.angle(d2.second)?),
        )))
    }

    fn surf_c2(&mut self, surf1: &mut SLProps, surf2: &mut SLProps) -> Result<(), Failure> {
        let d1u = Pair {
            first: surf1.d1u()?,
            second: surf2.d1u()?,
        };
        let d2u = Pair {
            first: surf1.d2u()?,
            second: surf2.d2u()?,
        };
        match self.second_order(&d1u, &d2u)? {
            Ok((l1, l2, angle)) => {
                (self.my_lambda1u, self.my_lambda2u, self.my_cont_c2u) = (l1, l2, angle)
            }
            Err(status) => self.fail(status),
        }
        let d1v = Pair {
            first: surf1.d1v()?,
            second: surf2.d1v()?,
        };
        let d2v = Pair {
            first: surf1.d2v()?,
            second: surf2.d2v()?,
        };
        match self.second_order(&d1v, &d2v)? {
            Ok((l1, l2, angle)) => {
                (self.my_lambda1v, self.my_lambda2v, self.my_cont_c2v) = (l1, l2, angle)
            }
            Err(status) => self.fail(status),
        }
        Ok(())
    }

    fn surf_g1(&mut self, surf1: &mut SLProps, surf2: &mut SLProps) -> Result<(), Failure> {
        if !(surf1.is_normal_defined()? && surf2.is_normal_defined()?) {
            self.fail(LocalAnalysisStatusErrorType::NormalNotDefined);
            return Ok(());
        }
        let (d1, d2) = (surf1.normal()?, surf2.normal()?);
        self.my_cont_g1 = folded_angle(d1.angle(&d2));
        Ok(())
    }

    fn surf_g2(&mut self, surf1: &mut SLProps, surf2: &mut SLProps) -> Result<(), Failure> {
        if !(surf1.is_curvature_defined()? && surf2.is_curvature_defined()?) {
            self.fail(LocalAnalysisStatusErrorType::CurvatureNotDefined);
            return Ok(());
        }
        let mcd1: Direction3 = mean_direction(surf1.curvature_directions()?)?;
        let mcd2: Direction3 = mean_direction(surf2.curvature_directions()?)?;
        self.my_alpha = mcd1.angle(&mcd2);
        let (rmin1, rmax1) = (surf1.min_curvature()?, surf1.max_curvature()?);
        let (rmin2, rmax2) = (surf2.min_curvature()?, surf2.max_curvature()?);
        self.my_eta1 = (rmin1 + rmax1) / 2.0;
        self.my_eta2 = (rmin2 + rmax2) / 2.0;
        self.my_eta = (self.my_eta1 + self.my_eta2) / 2.0;
        self.my_zeta1 = (rmax1 - rmin1) / 2.0;
        self.my_zeta2 = (rmax2 - rmin2) / 2.0;
        self.my_zeta = (self.my_zeta1 + self.my_zeta2) / 2.0;
        let deta: f64 = (self.my_eta1 - self.my_eta2) / 2.0;
        let dzeta: f64 = (self.my_zeta1 - self.my_zeta2) / 2.0;
        let (cos, sin) = (self.my_alpha.cos(), self.my_alpha.sin());
        self.my_gap = deta.abs()
            + (dzeta * dzeta * cos * cos + self.my_zeta * self.my_zeta * sin * sin).sqrt();
        Ok(())
    }
}

fn live_curve(curve: &Handle<dyn Geom2dCurve>) -> Result<&dyn Geom2dCurve, Failure> {
    curve.get().ok_or_else(|| {
        rustycad_tkg2d::geom2d::errors::null_object("LocalAnalysis_SurfaceContinuity: null curve")
    })
}

impl LocalAnalysisSurfaceContinuity {
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.my_is_done {
            return Err(not_done("LocalAnalysis_SurfaceContinuity", member));
        }
        Ok(())
    }

    fn done_value(&self, member: &str, value: f64) -> Result<f64, Failure> {
        self.check_done(member)?;
        Ok(value)
    }

    /// Whether the analysis succeeded (`IsDone`); `c.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// The analysed order (`ContinuityStatus`); StdFail_NotDone for a failed analysis,
    /// Standard_NoSuchObject before any analysis (uninitialized natively).
    /// `c.continuity_status()?`.
    pub fn continuity_status(&self) -> Result<Shape, Failure> {
        self.check_done("ContinuityStatus")?;
        self.my_type_cont.ok_or_else(|| {
            rustycad_tkg2d::geom2d::errors::no_such_object(
                "LocalAnalysis_SurfaceContinuity::ContinuityStatus: no analysis computed; expected ComputeAnalysis",
            )
        })
    }

    /// The reason of a failed analysis (`StatusError`); `None` when none was recorded.
    /// `c.status_error()`.
    pub fn status_error(&self) -> Option<LocalAnalysisStatusErrorType> {
        self.my_error_status
    }

    /// The distance between the points (`C0Value`); `c.c0_value()?`.
    pub fn c0_value(&self) -> Result<f64, Failure> {
        self.done_value("C0Value", self.my_cont_c0)
    }

    /// The folded angle between the U first derivatives (`C1UAngle`); `c.c1u_angle()?`.
    pub fn c1u_angle(&self) -> Result<f64, Failure> {
        self.done_value("C1UAngle", self.my_cont_c1u)
    }

    /// The ratio of the U first derivative lengths (`C1URatio`); `c.c1u_ratio()?`.
    pub fn c1u_ratio(&self) -> Result<f64, Failure> {
        self.done_value("C1URatio", self.my_lambda1u)
    }

    /// The folded angle between the V first derivatives (`C1VAngle`); `c.c1v_angle()?`.
    pub fn c1v_angle(&self) -> Result<f64, Failure> {
        self.done_value("C1VAngle", self.my_cont_c1v)
    }

    /// The ratio of the V first derivative lengths (`C1VRatio`); `c.c1v_ratio()?`.
    pub fn c1v_ratio(&self) -> Result<f64, Failure> {
        self.done_value("C1VRatio", self.my_lambda1v)
    }

    /// The folded angle between the U second derivatives (`C2UAngle`); `c.c2u_angle()?`.
    pub fn c2u_angle(&self) -> Result<f64, Failure> {
        self.done_value("C2UAngle", self.my_cont_c2u)
    }

    /// The ratio of the U second derivative lengths (`C2URatio`); `c.c2u_ratio()?`.
    pub fn c2u_ratio(&self) -> Result<f64, Failure> {
        self.done_value("C2URatio", self.my_lambda2u)
    }

    /// The folded angle between the V second derivatives (`C2VAngle`); `c.c2v_angle()?`.
    pub fn c2v_angle(&self) -> Result<f64, Failure> {
        self.done_value("C2VAngle", self.my_cont_c2v)
    }

    /// The ratio of the V second derivative lengths (`C2VRatio`); `c.c2v_ratio()?`.
    pub fn c2v_ratio(&self) -> Result<f64, Failure> {
        self.done_value("C2VRatio", self.my_lambda2v)
    }

    /// The folded angle between the normals (`G1Angle`); `c.g1_angle()?`.
    pub fn g1_angle(&self) -> Result<f64, Failure> {
        self.done_value("G1Angle", self.my_cont_g1)
    }

    /// The gap between the curvature ellipses (`G2CurvatureGap`); `c.g2_curvature_gap()?`.
    pub fn g2_curvature_gap(&self) -> Result<f64, Failure> {
        self.done_value("G2CurvatureGap", self.my_gap)
    }

    /// Whether the points are within `EpsC0` (`IsC0`); `c.is_c0()?`.
    pub fn is_c0(&self) -> Result<bool, Failure> {
        self.check_done("IsC0")?;
        Ok(self.my_cont_c0 <= self.eps.eps_c0)
    }

    /// C0 and both first-derivative angles within `EpsC1` (`IsC1`); `c.is_c1()?`.
    pub fn is_c1(&self) -> Result<bool, Failure> {
        self.check_done("IsC1")?;
        Ok(self.is_c0()?
            && self.my_cont_c1u <= self.eps.eps_c1
            && self.my_cont_c1v <= self.eps.eps_c1)
    }

    /// C1, both second-derivative angles below `EpsC2` and matching derivative
    /// ratios in U and V (`IsC2`); `c.is_c2()?`.
    pub fn is_c2(&self) -> Result<bool, Failure> {
        self.check_done("IsC2")?;
        if !self.is_c1()? {
            return Ok(false);
        }
        let (c1, c2) = (self.eps.eps_c1, self.eps.eps_c2);
        let eps1u: f64 = 0.5 * c1 * c1 * self.my_lambda1u;
        let eps1v: f64 = 0.5 * c1 * c1 * self.my_lambda1v;
        let eps2u: f64 = 0.5 * c2 * c2 * self.my_lambda2u;
        let eps2v: f64 = 0.5 * c2 * c2 * self.my_lambda2v;
        if !(self.my_cont_c2u < c2 && self.my_cont_c2v < c2) {
            return Ok(false);
        }
        if (self.my_lambda1u * self.my_lambda1u - self.my_lambda2u).abs() > eps1u * eps1u + eps2u {
            return Ok(false);
        }
        Ok((self.my_lambda1v * self.my_lambda1v - self.my_lambda2v).abs() <= eps1v * eps1v + eps2v)
    }

    /// C0 and normals within `EpsG1` (`IsG1`); `c.is_g1()?`.
    pub fn is_g1(&self) -> Result<bool, Failure> {
        self.check_done("IsG1")?;
        Ok(self.is_c0()? && self.my_cont_g1 <= self.eps.eps_g1)
    }

    /// G1 and a curvature gap within `Percent` of the curvatures, for the curvature
    /// configurations the native test accepts (`IsG2`); `c.is_g2()?`.
    pub fn is_g2(&self) -> Result<bool, Failure> {
        self.check_done("IsG2")?;
        let epsnl: f64 = 8.0 * self.eps.eps_c0 / (self.eps.maxlen * self.eps.maxlen);
        if !self.is_g1()? {
            return Ok(false);
        }
        if self.my_eta.abs() < epsnl && self.my_zeta.abs() < epsnl {
            return Ok(true);
        }
        if !self.comparable_curvatures(epsnl) {
            return Ok(false);
        }
        let (eta, zeta, gap, perce) = (self.my_eta, self.my_zeta, self.my_gap, self.eps.percent);
        if eta >= 2.0 * zeta && gap <= perce * (eta - zeta) {
            return Ok(true);
        }
        if zeta >= eta && gap <= perce * zeta {
            return Ok(true);
        }
        Ok(zeta <= eta && eta <= 2.0 * zeta && gap <= perce * eta)
    }

    /// The `itype == 1` cases of `IsG2`.
    fn comparable_curvatures(&self, epsnl: f64) -> bool {
        let (eta1, eta2, zeta1, zeta2) = (self.my_eta1, self.my_eta2, self.my_zeta1, self.my_zeta2);
        (zeta1.abs() < epsnl && zeta2.abs() < epsnl)
            || (eta1.abs() < epsnl && eta2.abs() < epsnl)
            || (self.my_zeta.abs() - self.my_eta.abs()).abs() < epsnl
            || (eta1 < zeta1 && eta2 < zeta2)
            || (eta1 > zeta1 && eta2 > zeta2)
    }
}
