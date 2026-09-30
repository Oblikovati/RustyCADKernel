// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of IntersectionWithAnArc and Recadre of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 2d intersection of an analytic line with a restriction arc, and the reframing of
//! the parameters of a point into the periods of the quadrics.
use super::super::aline::IntPatchALine;
use crate::int_start::TheArc;
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2};
use std::f64::consts::PI;

/// The parameters of `p` on the quadric (0 on an undefined one).
fn uv_on(quad: &IntSurfQuadric, p: Point3) -> (f64, f64) {
    quad.parameters(p).unwrap_or((0.0, 0.0))
}

/// The line parameter nearest to `psurf` among the samples at a hundredth of the range.
fn nearest_sample(alin: &IntPatchALine, psurf: Point3, (u0, u1): (f64, f64)) -> Result<f64, Failure> {
    let dtheta: f64 = (u1 - u0) * 0.01;
    let mut distmin: f64 = f64::MAX;
    let mut thetamin: f64 = 0.0;
    let mut theta: f64 = u0 + dtheta;
    while theta <= u1 - dtheta {
        let d: f64 = alin.value(theta)?.distance(&psurf);
        if d < distmin {
            thetamin = theta;
            distmin = d;
        }
        theta += dtheta;
    }
    Ok(thetamin)
}

/// The Newton-like step `(da, dr)` of the line and arc parameters from the 2d gap
/// `pa_pr` and the derivatives of the line (`d1a`, reversed) and of the arc.
fn step_of(d1a: Vector2, d2d: Vector2, pa_pr: Vector2) -> (f64, f64) {
    let d: f64 = d1a.x() * d2d.y() - d1a.y() * d2d.x();
    let mut da: f64 = (-pa_pr.x()) * d2d.y() - (-pa_pr.y()) * d2d.x();
    let mut dr: f64 = d1a.x() * (-pa_pr.y()) - d1a.y() * (-pa_pr.x());
    if d.abs() > 1e-15 {
        return (da / d, dr / d);
    }
    let (gap, a, r) = if pa_pr.x().abs() > pa_pr.y().abs() {
        (pa_pr.x() * 0.5, d1a.x(), d2d.x())
    } else {
        (pa_pr.y() * 0.5, d1a.y(), d2d.y())
    };
    if a != 0.0 {
        da = -gap / a;
    }
    if r != 0.0 {
        dr = -gap / r;
    }
    (da, dr)
}

/// The state of the search of `IntersectionWithAnArc`.
struct ArcSearch<'a> {
    alin: &'a IntPatchALine,
    arc: &'a TheArc,
    quad: &'a IntSurfQuadric,
    du: f64,
    bounds: (f64, f64),
    best: (f64, f64, f64),
}

impl ArcSearch<'_> {
    /// The 2d gap between the line point of `theta` (on the quadric) and the arc point
    /// of `prm`, with the derivatives of both (the line one by a finite difference).
    fn gap(&self, theta: f64, prm: f64) -> Result<(Vector2, Vector2, Vector2), Failure> {
        let (ua0, va0) = uv_on(self.quad, self.alin.value(theta)?);
        let (ua1, va1) = uv_on(self.quad, self.alin.value(theta + self.du)?);
        let d1a = Vector2::new((ua1 - ua0) / self.du, (va1 - va0) / self.du);
        let (p2d, d2d): (Point2, Vector2) = self.arc.d1(prm)?;
        Ok((d1a, d2d, Vector2::between(Point2::new(ua0, va0), p2d)))
    }

    /// Keeps the best pair `(param on arc, theta, distance)` seen so far.
    fn record(&mut self, prm: f64, theta: f64, pbd: f64) {
        if self.best.2 > pbd {
            self.best = (prm, theta, pbd);
        }
    }
}

/// `IntersectionWithAnArc(PSurf, alin, para, thearc, _theparameteronarc,
/// thepointonarc, QuadSurf, u0alin, u1alin)`: the point of the analytic line nearest to
/// the restriction arc in the parameters of the quadric, from the sample nearest to
/// `psurf` and the arc parameter `param_on_arc` (20 Newton-like steps, clamped to 5% of
/// the ranges). On success writes the line parameter, the point (also into `psurf`) and
/// the arc parameter; the native comment says the distance computation is to be
/// reviewed (lbr, 18 Nov 97).
#[allow(clippy::too_many_arguments)]
pub(super) fn intersection_with_an_arc(
    psurf: &mut Point3,
    alin: &IntPatchALine,
    para: &mut f64,
    thearc: &TheArc,
    param_on_arc: &mut f64,
    point_on_arc: &mut Point3,
    quad: &IntSurfQuadric,
    (u0alin, u1alin): (f64, f64),
) -> Result<bool, Failure> {
    let dtheta: f64 = (u1alin - u0alin) * 0.01;
    let mut du: f64 = 0.000000001;
    if du >= dtheta {
        du = dtheta / 2.0;
    }
    let thetamin: f64 = nearest_sample(alin, *psurf, (u0alin, u1alin))?;
    let mut search = ArcSearch {
        alin,
        arc: thearc,
        quad,
        du,
        bounds: (u0alin, u1alin),
        best: (0.0, 0.0, f64::MAX),
    };
    // Initial distance
    let (ua0, va0) = uv_on(quad, alin.value(thetamin)?);
    let (p2d, _) = thearc.d1(*param_on_arc)?;
    let distinit: f64 = Vector2::between(Point2::new(ua0, va0), p2d).magnitude();
    if let Some((theta, prm)) = search.iterate(thetamin, *param_on_arc)? {
        return finish(alin, (theta, prm), psurf, para, param_on_arc, point_on_arc);
    }
    let (bestpara, besttheta, bestdist) = search.best;
    if bestdist < distinit {
        return finish(alin, (besttheta, bestpara), psurf, para, param_on_arc, point_on_arc);
    }
    Ok(false)
}

/// Writes the found line parameter, its point and the arc parameter.
fn finish(
    alin: &IntPatchALine,
    (theta, prm): (f64, f64),
    psurf: &mut Point3,
    para: &mut f64,
    param_on_arc: &mut f64,
    point_on_arc: &mut Point3,
) -> Result<bool, Failure> {
    *para = theta;
    *psurf = alin.value(theta)?;
    *param_on_arc = prm;
    *point_on_arc = alin.value(theta)?;
    Ok(true)
}

impl ArcSearch<'_> {
    /// The 20 steps from `(theta, prm)`: the converged pair, or `None`.
    fn iterate(&mut self, mut theta: f64, mut prm: f64) -> Result<Option<(f64, f64)>, Failure> {
        let (u0, u1) = self.bounds;
        let (af, al) = (self.arc.first_parameter()?, self.arc.last_parameter()?);
        let drmax: f64 = (al - af) * 0.05;
        let damax: f64 = (u1 - u0) * 0.05;
        for _ in 0..20 {
            let (d1a, d2d, pa_pr) = self.gap(theta, prm)?;
            self.record(prm, theta, pa_pr.magnitude());
            let (da, dr) = step_of(Vector2::new(-d1a.x(), -d1a.y()), d2d, pa_pr);
            let da: f64 = clamp_step(da, damax);
            let dr: f64 = clamp_step(dr, drmax);
            if da.abs() < 1e-10 && dr.abs() < 1e-10 {
                return Ok(Some((theta, prm)));
            }
            theta += da;
            prm += dr;
            (theta, prm) = self.clamp_params(theta, prm, (af, al))?;
        }
        Ok(None)
    }

    /// The parameters brought back into the arc range and the line range.
    fn clamp_params(&self, mut theta: f64, mut prm: f64, (af, al): (f64, f64)) -> Result<(f64, f64), Failure> {
        let (u0, u1) = self.bounds;
        if prm > al {
            prm = al;
        }
        if prm < af {
            prm = af;
        }
        if theta < u0 {
            theta = u0;
        }
        if theta > u1 - self.du {
            theta = u1 - self.du - self.du;
        }
        Ok((theta, prm))
    }
}

/// `if (x < -max) x = -max; else if (x > max) x = max;`.
fn clamp_step(x: f64, max: f64) -> f64 {
    if x < -max {
        return -max;
    }
    if x > max {
        return max;
    }
    x
}

/// The periodicity of the parametrisation that `Recadre` assumes for each surface type
/// (the doubly periodic parametric surfaces are handled upstream).
fn periodic_flags(s: &Adaptor3dSurfaceHandle) -> Result<(bool, bool), Failure> {
    Ok(match s.get_type()? {
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere => (true, false),
        SurfaceType::Torus => (true, true),
        _ => (false, false),
    })
}

/// Moves `u` by turns of 2 PI toward the middle of `[f, l]` while it is outside and a
/// turn brings it nearer.
fn reframe(mut u: f64, f: f64, l: f64) -> f64 {
    let lmf: f64 = PI + PI;
    let fpls2: f64 = 0.5 * (f + l);
    while u < f && (fpls2 - u) > (u + lmf - fpls2) {
        u += lmf;
    }
    while u > l && (u - fpls2) > (fpls2 - (u - lmf)) {
        u -= lmf;
    }
    u
}

/// `Recadre(myHS1, myHS2, u1, v1, u2, v2)`: brings the periodic parameters of the two
/// quadrics near the middle of their ranges.
pub(super) fn recadre(
    hs1: &Adaptor3dSurfaceHandle,
    hs2: &Adaptor3dSurfaceHandle,
    [u1, v1, u2, v2]: [f64; 4],
) -> Result<[f64; 4], Failure> {
    let (u1_periodic, v1_periodic) = periodic_flags(hs1)?;
    let (u2_periodic, v2_periodic) = periodic_flags(hs2)?;
    let mut out: [f64; 4] = [u1, v1, u2, v2];
    if u1_periodic {
        out[0] = reframe(u1, hs1.first_u_parameter()?, hs1.last_u_parameter()?);
    }
    if v1_periodic {
        out[1] = reframe(v1, hs1.first_v_parameter()?, hs1.last_v_parameter()?);
    }
    if u2_periodic {
        out[2] = reframe(u2, hs2.first_u_parameter()?, hs2.last_u_parameter()?);
    }
    if v2_periodic {
        out[3] = reframe(v2, hs2.first_v_parameter()?, hs2.last_v_parameter()?);
    }
    Ok(out)
}
