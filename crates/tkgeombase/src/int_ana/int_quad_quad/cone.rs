// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of IntAna_IntQuadQuad::Perform(gp_Cone, IntAna_Quadric, double)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_IntQuadQuad::Perform(gp_Cone, ...): on the cone
//! `x = z tan(beta) cos t, y = z tan(beta) sin t, z`, the quadric is
//! `A(t) z² + B(t) z + C(t)`. The curves live where the discriminant is positive
//! and are split at the roots of A, where one branch goes to infinity.
use super::IntQuadQuad;
use super::cylinder::interval_end;
use super::trig::{SortedTrigRoots, TrigFunction};
use crate::int_ana::Quadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis3, Cone3};
use std::f64::consts::PI;

/// The native curve signs (every cone curve has one Z per angle).
const Z_POSITIVE: bool = true;
const Z_NEGATIVE: bool = false;

/// The cone, the quadric in the apex frame, the roots of A(t) and B(t).
struct ConeCase {
    cone: Cone3,
    q: [f64; 10],
    pol_z2: SortedTrigRoots,
    nbsol_z2: i32,
    b_function: TrigFunction,
}

impl IntQuadQuad {
    /// Perform(gp_Cone, IntAna_Quadric, Tol): not done when a root search is
    /// unreliable, when A and B vanish identically with `|Q1| <= 1e-8`, or when two
    /// discriminant roots are within 1e-8; otherwise the curves over the positive
    /// intervals of the discriminant, split at the roots of A (the end where B < 0
    /// goes to infinity on Z+, else on Z-), then linked end to end.
    /// `inter.perform_cone(cone, &quadric, 1e-7)?`.
    pub fn perform_cone(
        &mut self,
        cone: Cone3,
        quadric: &Quadric,
        _tol: f64,
    ) -> Result<(), Failure> {
        self.reset();
        let mut frame: Axis3 = cone.position();
        frame.set_location(cone.apex());
        let q: [f64; 10] = quadric.new_coefficients(quadric.coefficients(), frame)?;
        let [qxx, qyy, qzz, qxy, qxz, qyz, qx, qy, qz, q1] = q;
        let tg: f64 = 1. / cone.semi_angle().tan();
        // 1. A(t) = 0.
        let (z2_cte, z2_c, z2_s) = (qzz * tg * tg, tg * qxz, tg * qyz);
        let pol_z2 = SortedTrigRoots::new(
            [qxx - qyy, qxy, z2_c + z2_c, z2_s + z2_s, z2_cte + qyy],
            0.,
            PI + PI,
        )?;
        if !pol_z2.is_done() {
            self.done = false;
            return Ok(());
        }
        let nbsol_z2: i32 = pol_z2.nb_solutions()?;
        // 2. B(t) = 0.
        let z1_cte: f64 = 2. * tg * qz;
        // The native Z1CC, Z1SS and Z1SC terms are null (B has no square term).
        let (z1_cc, z1_ss, z1_sc): (f64, f64, f64) = (0.0, 0.0, 0.0);
        let pol_z1 = SortedTrigRoots::new(
            [z1_cc - z1_ss, z1_sc, qx + qx, qy + qy, z1_cte + z1_ss],
            0.,
            PI + PI,
        )?;
        if !pol_z1.is_done() {
            self.done = false;
            return Ok(());
        }
        let b_function = TrigFunction {
            cc: z1_cc,
            ss: z1_ss,
            sc: z1_sc,
            c: qx,
            s: qy,
            cte: z1_cte,
        };
        let nbsol1: i32 = pol_z1.nb_solutions()?;
        let case = ConeCase {
            cone,
            q,
            pol_z2,
            nbsol_z2,
            b_function,
        };
        if case.pol_z2.infinite_roots()? {
            // A(t) = 0 for any t: B(t) z + C(t) = 0.
            if !pol_z1.infinite_roots()? {
                if nbsol1 == 0 {
                    self.cone_curve(&case, (0., PI + PI), Z_POSITIVE)?;
                }
            } else if q1.abs() <= self.my_epsilon {
                self.done = false;
            }
            return Ok(());
        }
        self.cone_discriminant(&case, [qxx, qyy, qzz, qxy, qxz, qyz, qx, qy, qz, q1], tg)
    }

    /// 3. The discriminant `B² - 4 A C` and its positive intervals.
    fn cone_discriminant(&mut self, case: &ConeCase, q: [f64; 10], tg: f64) -> Result<(), Failure> {
        let [qxx, qyy, qzz, qxy, qxz, qyz, qx, qy, qz, q1] = q;
        let mtf = TrigFunction {
            cc: qx * qx - qxx * q1,
            ss: qy * qy - qyy * q1,
            sc: qx * qy - qxy * q1,
            c: tg * (qx * qz - qxz * q1),
            s: tg * (qy * qz - qyz * q1),
            cte: tg * tg * (qz * qz - qzz * q1),
        };
        let pol = SortedTrigRoots::new(
            [
                mtf.cc - mtf.ss,
                mtf.sc,
                mtf.c + mtf.c,
                mtf.s + mtf.s,
                mtf.cte + mtf.ss,
            ],
            0.,
            PI + PI,
        )?;
        if !pol.is_done() {
            self.done = false;
            return Ok(());
        }
        let mut nbsol: i32 = pol.nb_solutions()?;
        if pol.infinite_roots()? {
            // (z(t) - zo(t))² = 0: a null discriminant for every t.
            self.cone_curve(case, (0., PI + PI), Z_POSITIVE)?;
            return self.cone_curve(case, (0., PI + PI), Z_NEGATIVE);
        }
        if nbsol == 0 && mtf.value(PI) < 0. {
            return Ok(());
        }
        // A discriminant positive for every t is a double root at 0.
        let constant_positive: bool = nbsol == 0;
        if constant_positive {
            nbsol = 1;
        }
        for i in 1..=nbsol {
            let (theta1, theta2) = if constant_positive {
                (0., PI + PI - self.my_epsilon)
            } else {
                (pol.value(i)?, interval_end(&pol, i, nbsol)?)
            };
            if (theta2 - theta1).abs() <= self.my_epsilon {
                self.done = false;
                return Ok(());
            }
            let qwet: f64 = mtf.value(0.5 * (theta1 + theta2))
                + mtf.value(0.4 * theta1 + 0.6 * theta2)
                + mtf.value(0.6 * theta1 + 0.4 * theta2);
            if qwet < 0. {
                continue;
            }
            self.cone_interval(case, theta1, theta2)?;
        }
        self.internal_set_next_and_previous()
    }

    /// The curves of one positive interval: a Z+ and a Z- curve, split at every
    /// root of A inside the interval (only the Z- curve is affected natively, but
    /// both are split).
    fn cone_interval(&mut self, case: &ConeCase, theta1: f64, theta2: f64) -> Result<(), Failure> {
        if !a_root_inside(case, theta1, theta2)? {
            self.cone_curve(case, (theta1, theta2), Z_POSITIVE)?;
            return self.cone_curve(case, (theta1, theta2), Z_NEGATIVE);
        }
        let (mut new_min, new_max) = (theta1, theta2);
        let mut no_changes: bool = true;
        for i2 in 1..=(case.nbsol_z2 + case.nbsol_z2) {
            let to: f64 = if i2 > case.nbsol_z2 {
                case.pol_z2.value(i2 - case.nbsol_z2)? + (PI + PI)
            } else {
                case.pol_z2.value(i2)?
            };
            // At `to`, A = 0 and z = -C/B is the one solution.
            if to < new_max && to > new_min {
                no_changes = false;
                self.cone_pair(case, (new_min, to))?;
                if case.pol_z2.is_a_root(new_min) {
                    self.open_end(case, new_min, true);
                }
                self.open_end(case, to, false);
                new_min = to;
            }
        }
        let start: f64 = if no_changes { theta1 } else { new_min };
        self.cone_pair(case, (start, theta2))?;
        if case.pol_z2.is_a_root(start) {
            self.open_end(case, start, true);
        }
        if case.pol_z2.is_a_root(theta2) {
            self.open_end(case, theta2, false);
        }
        Ok(())
    }

    /// The Z- then Z+ curves of a piece.
    fn cone_pair(&mut self, case: &ConeCase, domain: (f64, f64)) -> Result<(), Failure> {
        self.cone_curve(case, domain, Z_NEGATIVE)?;
        self.cone_curve(case, domain, Z_POSITIVE)
    }

    /// Marks the infinite end at `theta` (a root of A) of the last pair: on the Z+
    /// curve when B(theta) < 0, else on the Z- curve.
    fn open_end(&mut self, case: &ConeCase, theta: f64, first: bool) {
        let count: usize = self.nb_curves as usize;
        let slot: usize = if case.b_function.value(theta) < 0. {
            count - 1
        } else {
            count - 2
        };
        if first {
            self.the_curve[slot].set_is_first_open(true);
        } else {
            self.the_curve[slot].set_is_last_open(true);
        }
    }

    /// Sets the next curve slot to a cone-quadric curve with one Z per angle.
    fn cone_curve(
        &mut self,
        case: &ConeCase,
        domain: (f64, f64),
        z_positive: bool,
    ) -> Result<(), Failure> {
        let tolerance: f64 = self.my_epsilon;
        self.new_curve_slot()?
            .set_cone_quad_values(case.cone, case.q, tolerance, domain, false, z_positive);
        Ok(())
    }
}

/// True when a root of A (or the root plus 2 PI) lies strictly inside
/// `(theta1, theta2)`.
fn a_root_inside(case: &ConeCase, theta1: f64, theta2: f64) -> Result<bool, Failure> {
    for i2 in 1..=case.nbsol_z2 {
        let r: f64 = case.pol_z2.value(i2)?;
        if (r > theta1 && r < theta2) || (r + (PI + PI) > theta1 && r + (PI + PI) < theta2) {
            return Ok(true);
        }
    }
    Ok(false)
}
