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
// Rust translation of MaxParamRatio and ApproxInt_KnotTools::DefineParType of
// ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_KnotTools.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DefineParType`: the chord-length parametrization, or the centripetal one for a
//! line whose curvature has a high peak.
use super::{ApproxIntKnotPointSets, ApproxIntKnotTools};
use crate::approx_int::multi_line::ApproxIntMultiLineLayout;
use crate::approx_int::test_points::{TestPoints, bounded};
use crate::geom_int::{GeomIntTheMultiLineOfWLApprox, GeomIntWLApprox};
use crate::int_patch::IntPatchWLine;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::math::Vector;

/// `aCritRat`: the peak ratio of the curvature above which the centripetal
/// parametrization is considered.
const CRIT_RAT: f64 = 500.0;
/// `aCritParRat`: the ratio of consecutive parameter steps above which it is chosen.
const CRIT_PAR_RAT: f64 = 100.0;

/// `MaxParamRatio(thePars)`: the largest ratio of consecutive parameter steps (inverted
/// when below 1), skipping the steps below `Precision::Computational()`.
fn max_param_ratio(pars: &Vector<'_>) -> f64 {
    let mut max_ratio: f64 = 0.0;
    for i in pars.lower() + 1..pars.upper() {
        let denom: f64 = pars[i] - pars[i - 1];
        if denom.abs() < precision::computational() {
            continue;
        }
        let mut rat: f64 = (pars[i + 1] - pars[i]) / denom;
        if rat > 0.0 && rat < 1.0 {
            rat = 1.0 / rat;
        }
        max_ratio = std_max(max_ratio, rat);
    }
    max_ratio
}

impl ApproxIntKnotTools {
    /// `DefineParType(theWL, theFpar, theLpar, theApproxXYZ, theApproxU1V1,
    /// theApproxU2V2)`: the preferable parametrization of the points `fpar..=lpar` of
    /// the line: iso-parametric for two points, centripetal when the peak of the
    /// discrete curvature is more than 2500 times its mean, or more than 500 times with
    /// a parameter step ratio above 100, else chord length;
    /// `ApproxIntKnotTools::define_par_type(&wl, 1, 20, (true, true, false))?`.
    pub fn define_par_type(
        wl: &Handle<IntPatchWLine>,
        fpar: i32,
        lpar: i32,
        (approx_xyz, approx_u1v1, approx_u2v2): (bool, bool, bool),
    ) -> Result<ParametrizationType, Failure> {
        if lpar - fpar == 1 {
            return Ok(ParametrizationType::IsoParametric);
        }
        let nbp3d: i32 = i32::from(approx_xyz);
        let nbp2d: i32 = i32::from(approx_u1v1) + i32::from(approx_u2v2);
        let layout = ApproxIntMultiLineLayout {
            nb_p3d: nbp3d,
            nb_p2d: nbp2d,
            approx_u1v1,
            approx_u2v2,
            p2d_on_first: approx_u1v1,
            ind_min: fpar,
            ind_max: lpar,
            ..Default::default()
        };
        let test_line = GeomIntTheMultiLineOfWLApprox::with_line(wl.clone(), layout);
        let points = TestPoints::read(&test_line, (fpar, lpar), approx_u1v1)?;
        let sets = ApproxIntKnotPointSets {
            xyz: &points.xyz,
            u1v1: &points.u1v1,
            u2v2: &points.u2v2,
            approx_xyz,
            approx_u1v1,
            approx_u2v2,
        };
        let coords: Vec<f64> = sets.coords(fpar, lpar)?;
        // Analysis of the curvature.
        let par_type = ParametrizationType::ChordLength;
        let mut pars = Vector::filled(fpar, lpar, 0.0)?;
        GeomIntWLApprox::parameters(&test_line, fpar, lpar, par_type, &mut pars)?;
        let mut curv: Array1<'static, f64> = bounded(pars.lower(), pars.upper())?;
        let max_curv: f64 = Self::build_curvature(&coords, sets.dim() as i32, &pars, &mut curv)?;
        Ok(Self::choose_par_type(max_curv, &curv, &pars))
    }

    /// The choice of `DefineParType` from the discrete curvature.
    fn choose_par_type(
        max_curv: f64,
        curv: &Array1<'_, f64>,
        pars: &Vector<'_>,
    ) -> ParametrizationType {
        let chord = ParametrizationType::ChordLength;
        if max_curv < precision::p_confusion() || precision::is_positive_infinite(max_curv) {
            // Linear case.
            return chord;
        }
        let eps: f64 = epsilon(1.0);
        let mut mid_curv: f64 = 0.0;
        let mut j: i32 = 0;
        for value in curv.iter() {
            if max_curv - value < eps {
                continue;
            }
            j += 1;
            mid_curv += value;
        }
        if j > 1 {
            mid_curv /= f64::from(j);
        }
        if mid_curv <= eps {
            return chord;
        }
        let rat: f64 = max_curv / mid_curv;
        if rat > CRIT_RAT && (rat > 5.0 * CRIT_RAT || max_param_ratio(pars) > CRIT_PAR_RAT) {
            return ParametrizationType::Centripetal;
        }
        chord
    }
}
