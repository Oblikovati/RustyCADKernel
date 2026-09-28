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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx, Geom_BSplineCurve_1.cxx (poles, weights, moves) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Pole and weight edition, point moves, reversal and transformation.
use super::state::{BSplineState, rational};
use crate::geom::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{check_one_based, one_based, put, rebased};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3, resolution};

impl BSplineState {
    pub(super) fn reverse(&mut self) -> Result<(), Failure> {
        self.clear_eval_rep();
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
    pub(super) fn set_pole(&mut self, index: i32, pole: Point3) -> Result<(), Failure> {
        check_one_based(
            index,
            self.poles.len(),
            "BSpline curve: SetPole: index and #pole mismatch",
        )?;
        self.clear_eval_rep();
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
        self.clear_eval_rep();
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
        point: Point3,
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
        let mut new_poles: Array1<'static, Point3> = one_based(nbpoles)?;
        let current: Point3 = self.eval_d0(u)?;
        let displacement = Vector3::between(current, point);
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
            self.clear_eval_rep();
            self.poles = new_poles;
            self.max_deriv_inv_ok = false;
        }
        Ok(moved)
    }
    /// `MovePointAndTangent`: returns `ErrorStatus`, 0 on success; a periodic curve is
    /// made non-periodic first, whatever the outcome.
    pub(super) fn move_point_and_tangent(
        &mut self,
        u: f64,
        point: Point3,
        tangent: Vector3,
        tolerance: f64,
        conditions: (i32, i32),
    ) -> Result<i32, Failure> {
        if self.periodic {
            self.set_not_periodic()?;
        }
        let mut new_poles: Array1<'static, Point3> = one_based(self.poles.len() as i32)?;
        let current = self.eval_d1(u)?;
        let delta = Vector3::between(current.point, point);
        let delta_derivative = Vector3::new(
            tangent.x() - current.d1.x(),
            tangent.y() - current.d1.y(),
            tangent.z() - current.d1.z(),
        );
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
            self.clear_eval_rep();
            self.poles = new_poles;
            self.max_deriv_inv_ok = false;
        }
        Ok(error_status)
    }
    pub(super) fn transform(&mut self, transform: Transform3) {
        self.clear_eval_rep();
        for pole in self.poles.as_mut_slice() {
            pole.transform(transform);
        }
        self.max_deriv_inv_ok = false;
    }
}
