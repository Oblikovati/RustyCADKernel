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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_AppSurf.gxx (the end of
// both Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surface of an approximated multi-curve: the homogeneous poles divided by
//! their weights, the V knots and multiplicities, and the 2d curves.
use super::AppBlendAppSurf;
use super::sections::SectionArrays;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{arrays::one_based, errors};
use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurveEval};
use rustycad_tkmath::gp::{self, Point2, Point3};

impl AppBlendAppSurf {
    /// Stores the surface of `multc`; `knot_rule` adjusts the V knots (the known
    /// parameters). A weight below `gp::Resolution()` leaves the approximation not
    /// done, with the poles stored so far.
    pub(super) fn store_solution(
        &mut self,
        multc: &MultiBSpCurve,
        shape: &SectionArrays,
        knot_rule: impl FnOnce(&mut Array1<'static, f64>) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        let (nb_u, nb_2d) = (shape.nb_u_poles, shape.nb_poles2d);
        self.vdeg = MultiCurveEval::degree(multc)?;
        let nb_v: i32 = MultiCurveEval::nb_poles(multc);
        let table = |e| errors::array_bounds(e);
        self.tab_poles = Array2::with_bounds(1, nb_u, 1, nb_v).map_err(table)?;
        self.tab_weights = Array2::with_bounds(1, nb_u, 1, nb_v).map_err(table)?;
        self.tab_v_knots = multc.knots()?;
        knot_rule(&mut self.tab_v_knots)?;
        self.tab_v_mults = multc.multiplicities()?;
        if !self.store_rational_poles(multc, [nb_u, nb_v, nb_2d])? {
            self.done = false;
            return Ok(());
        }
        for j in 1..=nb_2d {
            let mut poles2d: Array1<'static, Point2> = one_based(nb_v)?;
            multc.curve_2d(nb_u + j, &mut poles2d)?;
            self.seq_poles2d.push(poles2d);
        }
        self.done = true;
        Ok(())
    }
    /// The poles and weights from the homogeneous curves (the weight is the X of the
    /// 2d curve after the 2d poles); false at the first weight below the resolution.
    fn store_rational_poles(
        &mut self,
        multc: &MultiBSpCurve,
        [nb_u, nb_v, nb_2d]: [i32; 3],
    ) -> Result<bool, Failure> {
        let mut new_p: Array1<'static, Point3> = one_based(nb_v)?;
        let mut new_p2d: Array1<'static, Point2> = one_based(nb_v)?;
        for j in 1..=nb_u {
            multc.curve_3d(j, &mut new_p)?;
            multc.curve_2d(j + nb_u + nb_2d, &mut new_p2d)?;
            for k in 1..=nb_v {
                let weight: f64 = new_p2d[k].x();
                self.tab_poles[(j, k)] = Point3::from(new_p[k].xyz() / weight);
                if weight < gp::resolution() {
                    return Ok(false);
                }
                self.tab_weights[(j, k)] = weight;
            }
        }
        Ok(true)
    }
}
