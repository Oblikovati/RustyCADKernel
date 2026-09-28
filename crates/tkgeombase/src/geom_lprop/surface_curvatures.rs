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
// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp_SurfaceUtils.hxx
// (ComputeSurfCurvatures) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LProp_SurfaceUtils::ComputeSurfCurvatures`: principal curvatures from the
//! first and second fundamental forms.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Vector3};
use rustycad_tkmath::math::DirectPolynomialRoots;

/// The curvature fields of a surface property object (`myMinCurv`, `myMaxCurv`,
/// `myDirMinCurv`, `myDirMaxCurv`, `myMeanCurv`, `myGausCurv`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfaceCurvatures {
    /// Minimum principal curvature.
    pub min_curv: f64,
    /// Maximum principal curvature.
    pub max_curv: f64,
    /// Direction of the minimum curvature.
    pub dir_min: Direction3,
    /// Direction of the maximum curvature.
    pub dir_max: Direction3,
    /// Mean curvature.
    pub mean_curv: f64,
    /// Gaussian curvature.
    pub gaus_curv: f64,
}

// The fundamental form coefficients E, F, G (first) and L, M, N (second).
#[derive(Clone, Copy)]
struct Forms {
    e: f64,
    f: f64,
    g: f64,
    l: f64,
    m: f64,
    n: f64,
}

// Two candidate curvatures and their directions (before the min/max sort).
struct Candidates {
    curv1: f64,
    curv2: f64,
    vect1: Vector3,
    vect2: Vector3,
}

/// `ComputeSurfCurvatures`: solve the shape-operator eigenproblem for
/// `derivs = [D1U, D1V, D2U, D2V, DUV]` and the unit `normal`, writing `out` in
/// the native order; false when undefined (a null G at an umbilic point, or not
/// two roots). A null direction raises Standard_ConstructionError after the
/// curvature values were written, as natively;
/// `compute_surf_curvatures([d1u, d1v, d2u, d2v, duv], n, &mut out)?`.
pub fn compute_surf_curvatures(
    derivs: [Vector3; 5],
    normal: Direction3,
    out: &mut SurfaceCurvatures,
) -> Result<bool, Failure> {
    let [d1u, d1v, d2u, d2v, duv] = derivs;
    let norm: Vector3 = Vector3::from(normal);
    let forms = Forms {
        e: d1u.square_magnitude(),
        f: d1u.dot(d1v),
        g: d1v.square_magnitude(),
        l: norm.dot(d2u),
        m: norm.dot(duv),
        n: norm.dot(d2v),
    };
    let a0: f64 = forms.e * forms.m - forms.f * forms.l;
    let b0: f64 = forms.e * forms.n - forms.g * forms.l;
    let c0: f64 = forms.f * forms.n - forms.g * forms.m;
    let max_abc: f64 = a0.abs().max(b0.abs()).max(c0.abs());
    if max_abc < f64::EPSILON {
        return umbilic(forms, d1u, norm, out);
    }
    let Some(candidates) = candidates(forms, [a0, b0, c0].map(|x| x / max_abc), d1u, d1v)? else {
        return Ok(false);
    };
    sorted_extremes(candidates, out)?;
    let eg_ff: f64 = (forms.e * forms.g) - (forms.f * forms.f);
    out.mean_curv =
        ((forms.n * forms.e) - (2.0 * forms.m * forms.f) + (forms.l * forms.g)) / (2.0 * eg_ff);
    out.gaus_curv = ((forms.l * forms.n) - (forms.m * forms.m)) / eg_ff;
    Ok(true)
}

// The umbilic branch: every direction is principal, curvature N / G.
fn umbilic(
    forms: Forms,
    d1u: Vector3,
    norm: Vector3,
    out: &mut SurfaceCurvatures,
) -> Result<bool, Failure> {
    if forms.g < f64::EPSILON {
        return Ok(false);
    }
    out.min_curv = forms.n / forms.g;
    out.max_curv = out.min_curv;
    out.dir_min = Direction3::from_vector(d1u)?;
    out.dir_max = Direction3::from_vector(d1u.crossed(norm))?;
    out.mean_curv = out.min_curv;
    out.gaus_curv = out.min_curv * out.min_curv;
    Ok(true)
}

// The two principal curvatures: roots of A t^2 + B t + C (direction t D1U + D1V),
// of C t^2 + B t + A (direction D1U + t D1V), or the iso directions when both A
// and C vanish. None when the quadratic has not two roots.
fn candidates(
    forms: Forms,
    [a, b, c]: [f64; 3],
    d1u: Vector3,
    d1v: Vector3,
) -> Result<Option<Candidates>, Failure> {
    let Forms { e, f, g, l, m, n } = forms;
    if a.abs() > f64::EPSILON {
        let Some([r1, r2]) = two_roots(a, b, c)? else {
            return Ok(None);
        };
        return Ok(Some(Candidates {
            curv1: ((l * r1 + 2.0 * m) * r1 + n) / ((e * r1 + 2.0 * f) * r1 + g),
            curv2: ((l * r2 + 2.0 * m) * r2 + n) / ((e * r2 + 2.0 * f) * r2 + g),
            vect1: r1 * d1u + d1v,
            vect2: r2 * d1u + d1v,
        }));
    }
    if c.abs() > f64::EPSILON {
        let Some([r1, r2]) = two_roots(c, b, a)? else {
            return Ok(None);
        };
        return Ok(Some(Candidates {
            curv1: ((n * r1 + 2.0 * m) * r1 + l) / ((g * r1 + 2.0 * f) * r1 + e),
            curv2: ((n * r2 + 2.0 * m) * r2 + l) / ((g * r2 + 2.0 * f) * r2 + e),
            vect1: d1u + r1 * d1v,
            vect2: d1u + r2 * d1v,
        }));
    }
    Ok(Some(Candidates {
        curv1: l / e,
        curv2: n / g,
        vect1: d1u,
        vect2: d1v,
    }))
}

// `math_DirectPolynomialRoots(A, B, C)` when it has exactly two solutions.
fn two_roots(a: f64, b: f64, c: f64) -> Result<Option<[f64; 2]>, Failure> {
    let roots = DirectPolynomialRoots::quadratic(a, b, c);
    if roots.solutions()? != 2 {
        return Ok(None);
    }
    Ok(Some([roots.value(1)?, roots.value(2)?]))
}

// The min/max assignment, each curvature written before its direction.
fn sorted_extremes(candidates: Candidates, out: &mut SurfaceCurvatures) -> Result<(), Failure> {
    let Candidates {
        curv1,
        curv2,
        vect1,
        vect2,
    } = candidates;
    if curv1 < curv2 {
        (out.min_curv, out.max_curv) = (curv1, curv2);
        out.dir_min = Direction3::from_vector(vect1)?;
        out.dir_max = Direction3::from_vector(vect2)?;
        return Ok(());
    }
    (out.min_curv, out.max_curv) = (curv2, curv1);
    out.dir_min = Direction3::from_vector(vect2)?;
    out.dir_max = Direction3::from_vector(vect1)?;
    Ok(())
}
