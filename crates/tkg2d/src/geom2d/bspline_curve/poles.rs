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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx (InsertPoleAfter, RemovePole) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pole insertion and removal that only Geom2d_BSplineCurve declares.
use super::state::BSplineState;
use crate::geom2d::arrays::{
    at, check_one_based, insert_at, one_based, put, remove_at, stored_or_unit_weights,
};
use crate::geom2d::errors;
use crate::geom2d::spline_data::bspline_rational as rational;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::BSplKnotDistribution;
use rustycad_tkmath::gp::{Point2, resolution};

impl BSplineState {
    fn check_knot_set(&self, member: &str) -> Result<(), Failure> {
        if matches!(
            self.knot_set,
            BSplKnotDistribution::NonUniform | BSplKnotDistribution::PiecewiseBezier
        ) {
            return Err(errors::construction(&format!(
                "BSpline curve: {member}: bad knotSet type {:?}; expected Uniform or QuasiUniform",
                self.knot_set
            )));
        }
        Ok(())
    }
    pub(super) fn insert_pole_after(
        &mut self,
        index: i32,
        pole: Point2,
        weight: f64,
    ) -> Result<(), Failure> {
        let nbpoles: i32 = self.poles.len() as i32;
        if index < 0 || index > nbpoles {
            return Err(errors::out_of_range(&format!(
                "BSpline curve: InsertPoleAfter: Index = {index}; expected in [0, {nbpoles}]"
            )));
        }
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "BSpline curve: InsertPoleAfter: Weight = {weight} too small; expected > gp::Resolution()"
            )));
        }
        self.check_knot_set("InsertPoleAfter")?;
        let (knots, mults) = self.appended_knot()?;
        let mut new_poles: Array1<'static, Point2> = one_based(nbpoles + 1)?;
        insert_at(&self.poles, &mut new_poles, index, pole)?;
        let rational_now: bool = self.rational || (weight - 1.).abs() > resolution();
        self.weights = if rational_now {
            let mut new_weights: Array1<'static, f64> = one_based(nbpoles + 1)?;
            let old: Array1<'static, f64> =
                stored_or_unit_weights(self.rational, &self.weights, self.poles.len() as i32)?;
            insert_at(&old, &mut new_weights, index, weight)?;
            new_weights
        } else {
            bsplclib::unit_weights(nbpoles + 1)?
        };
        self.poles = new_poles;
        self.rational = rational_now;
        (self.knots, self.mults) = (knots, mults);
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    // InsertPoleAfter's knots: one more knot `2*K(n) - K(n-1)`, interior multiplicities 1.
    fn appended_knot(&self) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let nbknots: i32 = self.knots.len() as i32;
        let mut knots: Array1<'static, f64> = one_based(nbknots + 1)?;
        for i in 1..=nbknots {
            put(&mut knots, i, at(&self.knots, i)?)?;
        }
        let extrapolated: f64 = 2. * at(&knots, nbknots)? - at(&knots, nbknots - 1)?;
        put(&mut knots, nbknots + 1, extrapolated)?;
        let mut mults: Array1<'static, i32> = one_based(nbknots + 1)?;
        for i in 2..=nbknots {
            put(&mut mults, i, 1)?;
        }
        put(&mut mults, 1, at(&self.mults, 1)?)?;
        put(&mut mults, nbknots + 1, at(&self.mults, nbknots)?)?;
        Ok((knots, mults))
    }
    pub(super) fn remove_pole(&mut self, index: i32) -> Result<(), Failure> {
        check_one_based(
            index,
            self.poles.len(),
            "BSpline curve: RemovePole: Index and #pole mismatch",
        )?;
        if self.poles.len() <= 2 {
            return Err(errors::construction(&format!(
                "BSpline curve: RemovePole: {} poles; #pole is already minimum",
                self.poles.len()
            )));
        }
        self.check_knot_set("RemovePole")?;
        let (knots, mults) = self.dropped_knot()?;
        let mut new_poles: Array1<'static, Point2> = one_based(self.poles.upper() - 1)?;
        remove_at(&self.poles, &mut new_poles, index)?;
        let mut new_weights: Array1<'static, f64> = Array1::default();
        if self.rational {
            new_weights = one_based(new_poles.len() as i32)?;
            remove_at(&self.weights, &mut new_weights, index)?;
        }
        self.poles = new_poles;
        self.rational = self.rational && rational(&new_weights)?;
        self.weights = new_weights;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        (self.knots, self.mults) = (knots, mults);
        self.update_knots()
    }
    // RemovePole's knots: the last interior knot is dropped, interior multiplicities 1.
    fn dropped_knot(&self) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let count: i32 = self.knots.len() as i32 - 1;
        let mut knots: Array1<'static, f64> = one_based(count)?;
        let mut mults: Array1<'static, i32> = one_based(self.mults.len() as i32 - 1)?;
        for i in 1..count {
            put(&mut knots, i, at(&self.knots, i)?)?;
            put(&mut mults, i, 1)?;
        }
        put(&mut mults, 1, at(&self.mults, 1)?)?;
        let (knots_upper, mults_upper) = (knots.upper(), mults.upper());
        put(
            &mut knots,
            knots_upper,
            at(&self.knots, self.knots.upper())?,
        )?;
        let last_mult: i32 = at(&self.mults, self.mults.upper())?;
        put(&mut mults, mults_upper, last_mult)?;
        Ok((knots, mults))
    }
}
