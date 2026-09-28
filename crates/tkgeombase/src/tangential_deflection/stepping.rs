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
// Rust translation of the "Normal processing for a curve" loop of PerformCurve in
// ModelingData/TKGeomBase/GCPnts/GCPnts_TangentialDeflection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The marching loop of `PerformCurve`: each step proposes `U2 = U1 + Du`, corrects
//! `Du` until the chord and angular coefficients are acceptable, appends the point and
//! predicts the next step.
use super::curve::difference;
use super::march::{March, interval_index};
use super::{DeflectionCurve, TangentialDeflection, US3};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Point3, Xyz};

/// The loop state of the march besides [`March`].
struct Walk {
    more_points: bool,
    angle_max: f64,
    idx: [i32; 2],
    need_check: bool,
    prev_point: Point3,
}

/// The state of one correction loop (`Coef`, `ACoef`, `FCoef` and the flags).
#[derive(Default)]
struct Correction {
    coef: f64,
    a_coef: f64,
    f_coef: f64,
    too_large: bool,
    too_small: bool,
    running: bool,
}

impl TangentialDeflection {
    /// The `while (MorePoints)` loop.
    pub(super) fn march_points(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
    ) -> Result<(), Failure> {
        march.u2 = self.first_u;
        let lower: i32 = march.intervs.lower();
        let mut walk = Walk {
            more_points: true,
            angle_max: self.angular_deflection * 0.5, // because we take the midpoint
            idx: [lower, lower],
            need_check: false,
            prev_point: *self
                .points
                .last()
                .expect("PerformCurve appended the first point"),
        };
        while walk.more_points {
            let correction: Correction = self.march_step(curve, march, &mut walk)?;
            march.du = march.u2 - march.u1;
            if walk.more_points {
                self.predict_step(curve, march, &correction)?;
                self.finish_step(march, &mut walk);
            }
        }
        Ok(())
    }
    // One proposal of U2 and its correction loop.
    fn march_step(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        walk: &mut Walk,
    ) -> Result<Correction, Failure> {
        walk.idx[0] = interval_index(march.u1, &march.intervs, walk.idx[0]);
        march.u2 += march.du;
        if march.u2 >= self.last_u {
            // End of curve
            march.u2 = self.last_u;
            march.current_point = march.last_point;
            march.du = march.u2 - march.u1;
            march.dusave = march.du;
        } else {
            march.current_point = curve.lifted_d0(march.u2)?; // Next point
        }
        let mut correction = Correction {
            running: true,
            ..Default::default()
        };
        while correction.running {
            self.correct_step(curve, march, walk, &mut correction)?;
        }
        Ok(correction)
    }
    // One pass of `while (Correction)`: "Adjustment of Du".
    fn correct_step(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        walk: &mut Walk,
        correction: &mut Correction,
    ) -> Result<(), Failure> {
        if walk.need_check {
            self.shorten_at_interval_jump(curve, march, walk)?;
        }
        let middle_u: f64 = (march.u1 + march.u2) * 0.5; // Verification at the midpoint
        let middle_point: Point3 = curve.lifted_d0(middle_u)?;
        self.coefficients(march, walk, middle_point, correction);
        if walk.need_check && correction.coef < 0.55 {
            walk.need_check = false;
            march.du = march.dusave;
            return self.set_u2(curve, march, march.u1 + march.du);
        }
        if correction.coef <= 1.0 {
            return self.accept_step(curve, march, walk, correction);
        }
        self.reject_step(curve, march, walk, (middle_u, middle_point), correction)
    }
    // "Jump to another polynom": set Du to the smallest value and check deflection on it.
    fn shorten_at_interval_jump(
        &self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        walk: &mut Walk,
    ) -> Result<(), Failure> {
        walk.idx[1] = interval_index(march.u2, &march.intervs, walk.idx[0]);
        if walk.idx[1] <= walk.idx[0] {
            return Ok(());
        }
        let (i0, intervs) = (walk.idx[0], &march.intervs);
        let third: f64 = (intervs[i0 + 1] - intervs[i0]) * US3;
        if march.du > third {
            march.du = third;
            return self.set_u2(curve, march, march.u1 + march.du);
        }
        Ok(())
    }
    // `U2 = U; if (U2 > myLastU) U2 = myLastU; D0(theC, U2, CurrentPoint);`
    fn set_u2(
        &self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        u: f64,
    ) -> Result<(), Failure> {
        march.u2 = u;
        if march.u2 > self.last_u {
            march.u2 = self.last_u;
        }
        march.current_point = curve.lifted_d0(march.u2)?;
        Ok(())
    }
    // The deflection (FCoef) and angular (ACoef) criteria; Coef keeps the most penalizing.
    fn coefficients(
        &self,
        march: &March,
        walk: &Walk,
        middle_point: Point3,
        correction: &mut Correction,
    ) {
        let v1: Xyz = difference(march.current_point, walk.prev_point); // Deflection criterion
        let v2: Xyz = difference(middle_point, walk.prev_point);
        let l1: f64 = v1.modulus();
        correction.f_coef = if l1 > self.min_len {
            v1.cross_magnitude(v2) / (l1 * self.curvature_deflection)
        } else {
            0.0
        };
        let v1: Xyz = difference(march.current_point, middle_point); // Angular criterion
        let (l1, l2) = (v1.modulus(), v2.modulus());
        correction.a_coef = if l1 > self.min_len && l2 > self.min_len {
            let angg: f64 = v1.cross_magnitude(v2) / (l1 * l2);
            angg / walk.angle_max
        } else {
            0.0
        };
        correction.coef = std_max(correction.a_coef, correction.f_coef);
    }
    // `Coef <= 1.0`: the end of the curve, an accepted point, or a longer step.
    fn accept_step(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        walk: &mut Walk,
        correction: &mut Correction,
    ) -> Result<(), Failure> {
        if (self.last_u - march.u2).abs() < self.u_tol {
            self.append(self.last_u, march.last_point);
            walk.more_points = false;
            correction.running = false;
        } else if correction.coef >= 0.55 || correction.too_large {
            self.append(march.u2, march.current_point);
            walk.prev_point = march.current_point;
            correction.running = false;
            walk.need_check = true;
        } else if correction.too_small {
            correction.running = false;
            walk.prev_point = march.current_point;
        } else {
            correction.too_small = true;
            march.du += std_min(
                (march.u2 - march.u1) * (1.0 - correction.coef),
                march.du * US3,
            );
            return self.set_u2(curve, march, march.u1 + march.du);
        }
        Ok(())
    }
    // `Coef > 1.0`: halve at the midpoint, or shrink the step by 10%.
    fn reject_step(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        walk: &Walk,
        (middle_u, middle_point): (f64, Point3),
        correction: &mut Correction,
    ) -> Result<(), Failure> {
        if correction.coef >= 1.5 {
            let last: Point3 = *self
                .points
                .last()
                .expect("PerformCurve appended the first point");
            if !walk.prev_point.is_equal(&last, precision::confusion()) {
                self.append(march.u1, walk.prev_point);
            }
            march.u2 = middle_u;
            march.du = march.u2 - march.u1;
            march.current_point = middle_point;
            return Ok(());
        }
        march.du *= 0.9;
        march.u2 = march.u1 + march.du;
        march.current_point = curve.lifted_d0(march.u2)?;
        correction.too_large = true;
        Ok(())
    }
    // The next step from the splitting criterion of the accepted one.
    fn predict_step(
        &self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        correction: &Correction,
    ) -> Result<(), Failure> {
        // `if (U1 > myFirstu)`: nothing is predicted for the first step.
        if march.u1 > self.first_u {
            self.predict_from_criterion(curve, march, correction)?;
        }
        Ok(())
    }
    // The two splitting criteria of the step prediction.
    fn predict_from_criterion(
        &self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        correction: &Correction,
    ) -> Result<(), Failure> {
        if correction.f_coef > correction.a_coef {
            // Deflection is the splitting criterion
            let estimate = self.evaluate_du(curve, march.u2, march.du, march.not_done)?;
            (march.current_point, march.du, march.not_done) =
                (estimate.point, estimate.du, estimate.not_done);
            if march.not_done {
                extrapolate_step(march);
            }
        } else {
            // Angle is the splitting criterion
            extrapolate_step(march);
        }
        Ok(())
    }
    // A step below myUTol ends the march or shrinks to a third; then U1 moves to U2.
    fn finish_step(&mut self, march: &mut March, walk: &mut Walk) {
        if march.du < self.u_tol {
            march.du = self.last_u - march.u2;
            if march.du < self.u_tol {
                self.append(self.last_u, march.last_point);
                walk.more_points = false;
            } else if march.du * US3 > self.u_tol {
                march.du *= US3;
            }
        }
        march.u1 = march.u2;
        march.dusave = march.du;
    }
}

// `Du += (Du - Dusave) * (Du / Dusave)` kept within [0.75, 1.5] * Dusave.
fn extrapolate_step(march: &mut March) {
    march.du += (march.du - march.dusave) * (march.du / march.dusave);
    if march.du > 1.5 * march.dusave {
        march.du = 1.5 * march.dusave;
    }
    if march.du < 0.75 * march.dusave {
        march.du = 0.75 * march.dusave;
    }
}
