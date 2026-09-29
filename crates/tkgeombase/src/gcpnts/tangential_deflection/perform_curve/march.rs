// Created on: 1996-11-08
// Created by: Jean Claude VAUTHIER
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
// Rust translation of the "Normal processing for a curve" loop of
// GCPnts_TangentialDeflection::PerformCurve (GCPnts_TangentialDeflection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The march of `PerformCurve`: each step is adjusted until the deflection and angle
//! coefficients of its span are between 0.55 and 1, then the next step is predicted.
use super::{CurveMarch, US3, difference, interval_idx};
use crate::cpnts::PntsCurve;
use crate::gcpnts::TangentialDeflection;
use crate::gcpnts::intervals::bound;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Point3, Xyz};

impl TangentialDeflection {
    /// The march from the first point to the end of the range.
    pub(super) fn march(
        &mut self,
        curve: PntsCurve<'_>,
        state: &mut CurveMarch,
    ) -> Result<(), Failure> {
        // because we take the midpoint
        let angle_max: f64 = self.my_angular_deflection * 0.5;
        state.prev_point = self.my_points.last_point()?;
        while state.more_points {
            state.idx[0] = interval_idx(state.u1, &state.intervs, state.idx[0])?;
            state.u2 += state.du;
            if state.u2 >= self.my_last_u {
                // End of curve
                state.u2 = self.my_last_u;
                state.current_point = state.last_point;
                state.du = state.u2 - state.u1;
                state.dusave = state.du;
            } else {
                // Next point
                state.current_point = curve.lifted_d0(state.u2)?;
            }
            let coefficients: (f64, f64) = self.adjust_step(curve, state, angle_max)?;
            state.du = state.u2 - state.u1;
            if state.more_points {
                self.predict_step(curve, state, coefficients)?;
            }
        }
        Ok(())
    }

    // `U2 = U1 + Du`, not beyond the end of the range, and its point.
    fn advance(&self, curve: PntsCurve<'_>, state: &mut CurveMarch) -> Result<(), Failure> {
        state.u2 = state.u1 + state.du;
        if state.u2 > self.my_last_u {
            state.u2 = self.my_last_u;
        }
        state.current_point = curve.lifted_d0(state.u2)?;
        Ok(())
    }

    // The "Adjustment of Du" loop; returns the last deflection and angle coefficients.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn adjust_step(
        &mut self,
        curve: PntsCurve<'_>,
        state: &mut CurveMarch,
        angle_max: f64,
    ) -> Result<(f64, f64), Failure> {
        let (mut too_large, mut too_small) = (false, false);
        loop {
            if state.is_need_to_check {
                self.limit_at_interval_jump(curve, state)?;
            }
            // Verification at the midpoint
            let middle_u: f64 = (state.u1 + state.u2) * 0.5;
            let middle_point: Point3 = curve.lifted_d0(middle_u)?;
            let (f_coef, a_coef) = self.coefficients(state, middle_point, angle_max);
            // Keep the most penalizing coefficient
            let coef: f64 = std_max(a_coef, f_coef);
            if state.is_need_to_check && coef < 0.55 {
                state.is_need_to_check = false;
                state.du = state.dusave;
                self.advance(curve, state)?;
                continue;
            }
            // Negated as the native `if (Coef <= 1.0) ... else`, so a NaN shrinks.
            if !(coef <= 1.0) {
                too_large |= self.shrink(curve, state, coef, (middle_u, middle_point))?;
                continue;
            }
            if self.accept(state, coef, (too_large, too_small)) {
                return Ok((f_coef, a_coef));
            }
            too_small = true;
            state.du += std_min((state.u2 - state.u1) * (1.0 - coef), state.du * US3);
            self.advance(curve, state)?;
        }
    }

    // A span of coefficient at most 1: the end point is reached, or the point is
    // stored (coefficient from 0.55, or after a shrink), or kept only as previous point
    // after one growth; true when the adjustment is over.
    fn accept(
        &mut self,
        state: &mut CurveMarch,
        coef: f64,
        (too_large, too_small): (bool, bool),
    ) -> bool {
        if (self.my_last_u - state.u2).abs() < self.my_u_tol {
            self.my_points.append(self.my_last_u, state.last_point);
            state.more_points = false;
            return true;
        }
        if coef >= 0.55 || too_large {
            self.my_points.append(state.u2, state.current_point);
            state.prev_point = state.current_point;
            state.is_need_to_check = true;
            return true;
        }
        if too_small {
            state.prev_point = state.current_point;
            return true;
        }
        false
    }

    // A span of coefficient above 1: from 1.5 the span is halved (the previous point is
    // stored first when it is not the last stored one); below, the step shrinks by a
    // tenth. Returns TooLarge for the second case.
    fn shrink(
        &mut self,
        curve: PntsCurve<'_>,
        state: &mut CurveMarch,
        coef: f64,
        (middle_u, middle_point): (f64, Point3),
    ) -> Result<bool, Failure> {
        if coef >= 1.5 {
            if !state
                .prev_point
                .is_equal(&self.my_points.last_point()?, precision::confusion())
            {
                self.my_points.append(state.u1, state.prev_point);
            }
            state.u2 = middle_u;
            state.du = state.u2 - state.u1;
            state.current_point = middle_point;
            return Ok(false);
        }
        state.du *= 0.9;
        state.u2 = state.u1 + state.du;
        state.current_point = curve.lifted_d0(state.u2)?;
        Ok(true)
    }

    // After a stored point, a step reaching another CN interval ("Jump to another
    // polynom") is limited to a third of the current interval.
    fn limit_at_interval_jump(
        &self,
        curve: PntsCurve<'_>,
        state: &mut CurveMarch,
    ) -> Result<(), Failure> {
        state.idx[1] = interval_idx(state.u2, &state.intervs, state.idx[0])?;
        if state.idx[1] <= state.idx[0] {
            return Ok(());
        }
        let low: f64 = bound(&state.intervs, state.idx[0])?;
        let high: f64 = bound(&state.intervs, state.idx[0] + 1)?;
        // Set Du to the smallest value and check deflection on it.
        if state.du > (high - low) * US3 {
            state.du = (high - low) * US3;
            self.advance(curve, state)?;
        }
        Ok(())
    }

    // The deflection coefficient (distance of the midpoint to the chord from the
    // previous point, over the linear deflection) and the angle coefficient (sine of
    // the chord angle at the midpoint, over half the angular deflection); each is 0
    // below the minimal length.
    fn coefficients(&self, state: &CurveMarch, middle_point: Point3, angle_max: f64) -> (f64, f64) {
        let mut v1: Xyz = difference(state.current_point, state.prev_point); // Deflection criterion
        let v2: Xyz = difference(middle_point, state.prev_point);
        let mut l1: f64 = v1.modulus();
        let f_coef: f64 = if l1 > self.my_min_len {
            v1.cross_magnitude(v2) / (l1 * self.my_curvature_deflection)
        } else {
            0.0
        };
        v1 = difference(state.current_point, middle_point); // Angular criterion
        l1 = v1.modulus();
        let l2: f64 = v2.modulus();
        if l1 > self.my_min_len && l2 > self.my_min_len {
            let angg: f64 = v1.cross_magnitude(v2) / (l1 * l2);
            return (f_coef, angg / angle_max);
        }
        (f_coef, 0.0)
    }

    // The next step: from the second span on, EvaluateDu when the deflection decided
    // (until it succeeds once), else an extrapolation bounded to [0.75, 1.5] times the
    // previous step; a step below UTol takes the rest of the range, ends the march, or
    // a third of the rest.
    fn predict_step(
        &mut self,
        curve: PntsCurve<'_>,
        state: &mut CurveMarch,
        (f_coef, a_coef): (f64, f64),
    ) -> Result<(), Failure> {
        if state.u1 > self.my_firstu {
            let extrapolate: bool = if f_coef > a_coef {
                // Deflection is the splitting criterion
                state.current_point =
                    self.evaluate_du(curve, state.u2, &mut state.du, &mut state.not_done)?;
                state.not_done
            } else {
                // Angle is the splitting criterion
                true
            };
            if extrapolate {
                state.du += (state.du - state.dusave) * (state.du / state.dusave);
                state.du = bounded_step(state.du, state.dusave);
            }
        }
        if state.du < self.my_u_tol {
            state.du = self.my_last_u - state.u2;
            if state.du < self.my_u_tol {
                self.my_points.append(self.my_last_u, state.last_point);
                state.more_points = false;
            } else if state.du * US3 > self.my_u_tol {
                state.du *= US3;
            }
        }
        state.u1 = state.u2;
        state.dusave = state.du;
        Ok(())
    }
}

// `if (Du > 1.5 Dusave) Du = 1.5 Dusave; if (Du < 0.75 Dusave) Du = 0.75 Dusave;`
fn bounded_step(du: f64, dusave: f64) -> f64 {
    let mut du: f64 = du;
    if du > 1.5 * dusave {
        du = 1.5 * dusave;
    }
    if du < 0.75 * dusave {
        du = 0.75 * dusave;
    }
    du
}
