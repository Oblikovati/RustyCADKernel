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
// Rust translation of IntAna_IntQuadQuad::Perform(gp_Cylinder, IntAna_Quadric, double)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_IntQuadQuad::Perform(gp_Cylinder, ...): on the cylinder
//! `X = R cos t, Y = R sin t, Z`, the quadric is quadratic in Z; the curves live
//! where the discriminant DIS(t) is positive.
use super::IntQuadQuad;
use super::trig::{SortedTrigRoots, TrigFunction, add_special_points};
use crate::int_ana::Quadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Cylinder3;
use std::f64::consts::PI;

/// The native flags of the curve kinds: one Z per angle (with the sign of the
/// root) or two Z per angle.
const ONE_Z: bool = false;
const TWO_Z: bool = true;
const Z_POSITIVE: bool = true;
const Z_NEGATIVE: bool = false;
const Z_ANY: bool = true;

/// The cylinder, the quadric in the cylinder frame and the source quadric (for
/// its special points).
struct CylinderCase<'a> {
    cylinder: Cylinder3,
    q: [f64; 10],
    quadric: &'a Quadric,
}

impl IntQuadQuad {
    /// Perform(gp_Cylinder, IntAna_Quadric, Tol): not done when the quadric is
    /// linear in Z (|Qzz| < 1e-8) or the discriminant roots are unreliable; two
    /// full curves (Z+ and Z-) when the discriminant is null or never negative,
    /// none when it is negative, else the curves over its positive intervals.
    /// Native quirk kept: this overload never links the curves (no
    /// InternalSetNextAndPrevious), so HasNextCurve is always false.
    /// `inter.perform_cylinder(cylinder, &quadric, 1e-7)?`.
    pub fn perform_cylinder(
        &mut self,
        cylinder: Cylinder3,
        quadric: &Quadric,
        _tol: f64,
    ) -> Result<(), Failure> {
        self.reset();
        let rcyl: f64 = cylinder.radius();
        let q: [f64; 10] = quadric.new_coefficients(quadric.coefficients(), cylinder.position())?;
        let [qxx, qyy, qzz, qxy, qxz, qyz, qx, qy, qz, q1] = q;
        if qzz.abs() < self.my_epsilon_coeff_poly_null {
            self.done = false;
            return Ok(());
        }
        let r2: f64 = rcyl * rcyl;
        let discriminant = TrigFunction {
            cc: r2 * (qxz * qxz - qxx * qzz),
            ss: r2 * (qyz * qyz - qyy * qzz),
            sc: r2 * (qxz * qyz - qxy * qzz),
            c: rcyl * (qxz * qz - qx * qzz),
            s: rcyl * (qyz * qz - qy * qzz),
            cte: qz * qz - qzz * q1,
        };
        let d = &discriminant;
        let roots = SortedTrigRoots::new(
            [d.cc - d.ss, d.sc, d.c + d.c, d.s + d.s, d.cte + d.ss],
            0.,
            PI + PI,
        )?;
        if !roots.is_done() {
            self.done = false;
            return Ok(());
        }
        let case = CylinderCase {
            cylinder,
            q,
            quadric,
        };
        if roots.infinite_roots()? {
            return self.full_cylinder_curves(&case);
        }
        match roots.nb_solutions()? {
            0 if discriminant.value(PI) >= -f64::EPSILON => self.full_cylinder_curves(&case),
            1 if discriminant.value(roots.value(1)? + PI) >= -f64::EPSILON => {
                self.full_cylinder_curves(&case)
            }
            0 | 1 => Ok(()),
            count => self.cylinder_intervals(&case, &roots, &discriminant, count),
        }
    }

    /// The curves over the positive intervals of the discriminant: a double root
    /// among two roots gives the whole period, else each positive interval gives
    /// one two-Z curve, or a Z+ and a Z- curve when the next root is within 5e-8.
    fn cylinder_intervals(
        &mut self,
        case: &CylinderCase<'_>,
        roots: &SortedTrigRoots,
        discriminant: &TrigFunction,
        count: i32,
    ) -> Result<(), Failure> {
        if count == 2 && self.cylinder_double_root(case, roots, discriminant)? {
            return Ok(());
        }
        for i in 1..=count {
            let mut theta1: f64 = roots.value(i)?;
            let mut theta2: f64 = interval_end(roots, i, count)?;
            if (theta2 - theta1).abs() <= 1e-12 {
                continue;
            }
            // Everything has the same sign between the roots: sample three points.
            let qwet: f64 = discriminant.value(0.5 * (theta1 + theta2))
                + discriminant.value(0.4 * theta1 + 0.6 * theta2)
                + discriminant.value(0.6 * theta1 + 0.4 * theta2);
            if qwet.is_nan() || qwet < 0. {
                continue;
            }
            // A root within 5e-8 after the interval (a double tangency point) splits
            // the curve in its Z+ and Z- halves; the test precedes the widening.
            let theta3: f64 = interval_end(roots, i + 1, count)?;
            let split: bool = (theta3 - theta2) < 5.0e-8;
            add_special_points(case.quadric, &case.cylinder, &mut theta1, &mut theta2);
            if split {
                self.cylinder_curve(case, (theta1, theta2), ONE_Z, Z_POSITIVE)?;
                self.cylinder_curve(case, (theta1, theta2), ONE_Z, Z_NEGATIVE)?;
            } else {
                self.cylinder_curve(case, (theta1, theta2), TWO_Z, Z_ANY)?;
            }
        }
        Ok(())
    }

    /// The two-root case whose roots coincide within RealEpsilon(): a tangency
    /// point, with the two curves over the whole period when the discriminant is
    /// positive 0.1 away. Returns whether a double root was found.
    fn cylinder_double_root(
        &mut self,
        case: &CylinderCase<'_>,
        roots: &SortedTrigRoots,
        discriminant: &TrigFunction,
    ) -> Result<bool, Failure> {
        let mut tangency: bool = false;
        for i in 1..=2 {
            let mut theta1: f64 = roots.value(i)?;
            let theta2: f64 = interval_end(roots, i, 2)?;
            let gap: f64 = (theta2 - theta1).abs();
            if gap.is_nan() || gap > f64::EPSILON {
                continue;
            }
            tangency = true;
            let mut other: f64 = theta1 - 0.1;
            if other < 0. {
                other = theta1 + 0.1;
            }
            if discriminant.value(other) >= 0. {
                let mut end: f64 = theta1 + PI + PI;
                add_special_points(case.quadric, &case.cylinder, &mut theta1, &mut end);
                self.cylinder_curve(case, (theta1, end), ONE_Z, Z_POSITIVE)?;
                self.cylinder_curve(case, (theta1, end), ONE_Z, Z_NEGATIVE)?;
            }
        }
        Ok(tangency)
    }

    /// The Z+ and Z- curves over `[0, 2 PI]`.
    fn full_cylinder_curves(&mut self, case: &CylinderCase<'_>) -> Result<(), Failure> {
        self.cylinder_curve(case, (0.0, PI + PI), ONE_Z, Z_POSITIVE)?;
        self.cylinder_curve(case, (0.0, PI + PI), ONE_Z, Z_NEGATIVE)
    }

    /// Sets the next curve slot to a cylinder-quadric curve.
    fn cylinder_curve(
        &mut self,
        case: &CylinderCase<'_>,
        domain: (f64, f64),
        two_curves: bool,
        z_positive: bool,
    ) -> Result<(), Failure> {
        let tolerance: f64 = self.my_epsilon;
        self.new_curve_slot()?.set_cylinder_quad_values(
            case.cylinder,
            case.q,
            tolerance,
            domain,
            two_curves,
            z_positive,
        );
        Ok(())
    }
}

/// The root after root `i` (1-based), or the first root plus 2 PI after the last.
pub(super) fn interval_end(roots: &SortedTrigRoots, i: i32, count: i32) -> Result<f64, Failure> {
    if i < count {
        roots.value(i + 1)
    } else {
        Ok(roots.value(1)? + (PI + PI))
    }
}
