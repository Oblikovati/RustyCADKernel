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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx, Geom2d_BSplineCurve_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Pole and weight edition, point moves, reversal and transformation.
use super::state::{BSplineState, rational};
use crate::geom2d::arrays::{
    at, check_one_based, insert_at, one_based, put, rebased, remove_at, stored_or_unit_weights,
};
use crate::geom2d::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::BSplKnotDistribution;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2, resolution};

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
    pub(super) fn reverse(&mut self) -> Result<(), Failure> {
        bsplclib::reverse_knots(&mut self.knots)?;
        bsplclib::reverse_mults(&mut self.mults);
        let last: i32 = if self.periodic {
            self.flat_knots.upper() - self.deg - 1
        } else {
            self.poles.upper()
        };
        bsplclib::reverse_poles(&mut self.poles, last)?;
        if self.rational {
            bsplclib::reverse_poles(&mut self.weights, last)?;
        }
        self.update_knots()
    }
    pub(super) fn set_pole(&mut self, index: i32, pole: Point2) -> Result<(), Failure> {
        check_one_based(
            index,
            self.poles.len(),
            "BSpline curve: SetPole: index and #pole mismatch",
        )?;
        put(&mut self.poles, index, pole)?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    pub(super) fn set_weight(&mut self, index: i32, weight: f64) -> Result<(), Failure> {
        check_one_based(
            index,
            self.poles.len(),
            "BSpline curve: SetWeight: Index and #pole mismatch",
        )?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "BSpline curve: SetWeight: Weight = {weight} too small; expected > gp::Resolution()"
            )));
        }
        let mut rational_now: bool = self.rational || (weight - 1.).abs() > resolution();
        if rational_now {
            if !self.rational {
                // Detach the shared unit weights before writing, like the native copy.
                self.weights = rebased(&self.weights)?;
            }
            put(&mut self.weights, index, weight)?;
            if self.rational {
                rational_now = rational(&self.weights)?;
                if !rational_now {
                    self.reset_unit_weights()?;
                }
            }
            self.rational = rational_now;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `MovePoint`: returns `(FirstModifiedPole, LastModifiedPole)`, both 0 when nothing moved.
    pub(super) fn move_point(
        &mut self,
        u: f64,
        point: Point2,
        index1: i32,
        index2: i32,
    ) -> Result<(i32, i32), Failure> {
        let nbpoles: i32 = self.poles.len() as i32;
        if index1 < 1 || index1 > nbpoles || index2 < 1 || index2 > nbpoles || index1 > index2 {
            return Err(errors::out_of_range(&format!(
                "BSpline curve: MovePoint: Index1 = {index1}, Index2 = {index2}; \
                 expected 1 <= Index1 <= Index2 <= {nbpoles}"
            )));
        }
        let mut new_poles: Array1<'static, Point2> = one_based(nbpoles)?;
        let current: Point2 = self.eval_d0(u)?;
        let displacement = Vector2::between(current, point);
        let moved: (i32, i32) = bsplclib::move_point(
            u,
            displacement,
            index1,
            index2,
            self.deg,
            &self.poles,
            self.weights_view(),
            &self.flat_knots,
            &mut new_poles,
        )?;
        if moved.0 != 0 {
            self.poles = new_poles;
            self.max_deriv_inv_ok = false;
        }
        Ok(moved)
    }
    /// `MovePointAndTangent`: returns `ErrorStatus`, 0 on success.
    pub(super) fn move_point_and_tangent(
        &mut self,
        u: f64,
        point: Point2,
        tangent: Vector2,
        tolerance: f64,
        conditions: (i32, i32),
    ) -> Result<i32, Failure> {
        if self.periodic {
            self.set_not_periodic()?;
        }
        let mut new_poles: Array1<'static, Point2> = one_based(self.poles.len() as i32)?;
        let current = self.eval_d1(u)?;
        let delta = Vector2::between(current.point, point);
        let delta_derivative =
            Vector2::new(tangent.x() - current.d1.x(), tangent.y() - current.d1.y());
        let mut error_status: i32 = 0;
        bsplclib::move_point_and_tangent(
            u,
            delta,
            delta_derivative,
            tolerance,
            self.deg,
            conditions.0,
            conditions.1,
            &self.poles,
            self.weights_view(),
            &self.flat_knots,
            &mut new_poles,
            &mut error_status,
        )?;
        if error_status == 0 {
            self.poles = new_poles;
            self.max_deriv_inv_ok = false;
        }
        Ok(error_status)
    }
    pub(super) fn transform(&mut self, transform: Transform2) {
        for pole in self.poles.as_mut_slice() {
            pole.transform(transform);
        }
        self.max_deriv_inv_ok = false;
    }
}
