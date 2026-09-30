// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of EvalCurv and ApproxInt_KnotTools::BuildCurvature of
// ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_KnotTools.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The discrete curvature of a point set: the Lagrange parabola through each point and
//! its neighbours, and the curvature of its derivatives.
use super::ApproxIntKnotTools;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::plib;

/// The largest dimension of the point sets (one 3d and two 2d curves).
const MAX_DIM: usize = 7;

/// `EvalCurv(dim, V1, V2)`: the curvature `|V1 ^ V2| / |V1|^3` of a curve of dimension
/// `dim` from its first and second derivatives, with the outer product
/// `P(i, j) = V1(i) V2(j) - V1(j) V2(i)`; `Precision::Infinite()` for a null first
/// derivative (a knot is wanted at such a singularity).
pub(super) fn eval_curv(dim: usize, v1: &[f64], v2: &[f64]) -> f64 {
    let mut mp: f64 = 0.0;
    for i in 1..dim {
        for j in 0..i {
            let p: f64 = v1[i] * v2[j] - v1[j] * v2[i];
            mp += p * p;
        }
    }
    let mut q: f64 = 0.0;
    for value in v1.iter().take(dim) {
        q += value * value;
    }
    if q < 1.0 / precision::infinite() {
        // "0/0": the local curvature is unknown; a curvature jump is imitated.
        return precision::infinite();
    }
    q = std_min(q, precision::infinite());
    // `q *= q * q`: the square first.
    q = q * (q * q);
    (mp / q).sqrt()
}

/// The persistent buffers of `BuildCurvature` (`Val[21]`, `Par[3]`, `Res[21]`): a
/// Lagrange evaluation that fails on coincident parameters leaves `Res` as it was.
struct LagrangeBuffers {
    val: [f64; 3 * MAX_DIM],
    par: [f64; 3],
    res: [f64; 3 * MAX_DIM],
}

impl LagrangeBuffers {
    /// Loads the points `first..first + 2` and evaluates the parabola at `Par[at]`.
    fn eval(
        &mut self,
        (coords, dim, pars): (&[f64], usize, &Vector<'_>),
        (lower, first, at): (i32, i32, usize),
    ) -> Result<f64, Failure> {
        for j in 0..3 {
            let k: i32 = first + j as i32;
            let ic: usize = (k - lower) as usize * dim;
            self.val[dim * j..dim * j + dim].copy_from_slice(&coords[ic..ic + dim]);
            self.par[j] = *pars.value(k)?;
        }
        plib::eval_lagrange(
            self.par[at],
            2,
            2,
            dim as i32,
            &self.val,
            &self.par,
            &mut self.res,
        )?;
        Ok(eval_curv(dim, &self.res[dim..], &self.res[2 * dim..]))
    }
}

impl ApproxIntKnotTools {
    /// `BuildCurvature(theCoords, theDim, thePars, theCurv, theMaxCurv)`: the discrete
    /// curvature at each point (the parabola of the first three points at the first
    /// point, of the neighbours inside, of the last three at the last point) and its
    /// maximum; zeros for less than three points;
    /// `ApproxIntKnotTools::build_curvature(&coords, 3, &pars, &mut curv)?`.
    pub fn build_curvature(
        coords: &[f64],
        dim: i32,
        pars: &Vector<'_>,
        curv: &mut Array1<'_, f64>,
    ) -> Result<f64, Failure> {
        let mut max_curv: f64 = 0.0;
        if curv.length() < 3 {
            curv.init(0.0);
            return Ok(max_curv);
        }
        let d: usize = dim as usize;
        let mut buffers = LagrangeBuffers {
            val: [0.0; 3 * MAX_DIM],
            par: [0.0; 3],
            res: [0.0; 3 * MAX_DIM],
        };
        let lower: i32 = curv.lower();
        let upper: i32 = curv.upper();
        for i in lower..=upper {
            let (first, at): (i32, usize) = if i == lower {
                (i, 0)
            } else if i == upper {
                (i - 2, 2)
            } else {
                (i - 1, 1)
            };
            let value: f64 = buffers.eval((coords, d, pars), (lower, first, at))?;
            curv[i] = value;
            if value > max_curv {
                max_curv = value;
            }
        }
        Ok(max_curv)
    }
}
