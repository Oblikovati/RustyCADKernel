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
// Rust translation of IntAna2d_AnaIntersection_5.cxx and IntAna2d_AnaIntersection_6.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_AnaIntersection::Perform(gp_Circ2d, IntAna2d_Conic) and
//! Perform(gp_Elips2d, IntAna2d_Conic): the conic equation in the angular
//! parameter of the closed curve, solved by math_TrigonometricFunctionRoots.
use super::{AnaIntersection, Conic, coord_ancien_repere, traitement_points_confondus};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2d, Circle2, Ellipse2};
use rustycad_tkmath::math::TrigonometricRoots;
use std::f64::consts::PI;

/// The closed curve of a trigonometric intersection: its X axis, its sense and
/// the radii of its parametrization `x = major cos(a)`, `y = minor sin(a)`.
struct ClosedCurve {
    axis: Axis2d,
    direct: bool,
    major: f64,
    minor: f64,
}

impl AnaIntersection {
    /// Perform(gp_Circ2d, IntAna2d_Conic): the roots in `[0, 2 PI]` of the conic
    /// equation along `x = R cos(a)`, `y = R sin(a)` in the circle frame, without
    /// the coincident points; identical elements for an identity equation. The
    /// native member also prints "math_TrigonometricFunctionRoots -> NotDone" on
    /// std::cout when the solver fails; the port does not print.
    /// `inter.perform_circle_conic(circle, &conic)?`.
    pub fn perform_circle_conic(&mut self, circle: Circle2, conic: &Conic) -> Result<(), Failure> {
        let direct: bool = circle.is_direct();
        let radius: f64 = circle.radius();
        let radius_p2: f64 = radius * radius;
        self.reset_conic_flags();
        let axis: Axis2d = circle.x_axis();
        let [a, b, c, d, e, f] = conic.new_coefficients(conic.coefficients(), axis);
        let pss: f64 = b * radius_p2;
        let pcc: f64 = a * radius_p2 - pss;
        let p2sc: f64 = c * radius_p2;
        let pc: f64 = 2.0 * d * radius;
        let ps: f64 = 2.0 * e * radius;
        let pcte: f64 = f + pss;
        let roots = TrigonometricRoots::new(pcc, p2sc, pc, ps, pcte, 0.0, 2.0 * PI);
        let curve = ClosedCurve {
            axis,
            direct,
            major: radius,
            minor: radius,
        };
        self.store_angular_roots(&roots, &curve)
    }

    /// Perform(gp_Elips2d, IntAna2d_Conic): as the circle, along
    /// `x = MajorRadius cos(a)`, `y = MinorRadius sin(a)` in the ellipse frame.
    /// `inter.perform_ellipse_conic(ellipse, &conic)?`.
    pub fn perform_ellipse_conic(
        &mut self,
        ellipse: Ellipse2,
        conic: &Conic,
    ) -> Result<(), Failure> {
        let direct: bool = ellipse.is_direct();
        let (minor, major) = (ellipse.minor_radius(), ellipse.major_radius());
        self.reset_conic_flags();
        let axis: Axis2d = ellipse.x_axis();
        let [a, b, c, d, e, f] = conic.new_coefficients(conic.coefficients(), axis);
        let pss: f64 = b * minor * minor;
        let pcc: f64 = a * major * major - pss;
        let p2sc: f64 = c * major * minor;
        let pc: f64 = 2.0 * d * major;
        let ps: f64 = 2.0 * e * minor;
        let pcte: f64 = f + pss;
        let roots = TrigonometricRoots::new(pcc, p2sc, pc, ps, pcte, 0.0, 2.0 * PI);
        self.store_angular_roots(
            &roots,
            &ClosedCurve {
                axis,
                direct,
                major,
                minor,
            },
        )
    }

    /// The flags every conic Perform clears before solving.
    pub(super) fn reset_conic_flags(&mut self) {
        self.done = false;
        self.nbp = 0;
        self.para = false;
        self.empt = false;
        self.iden = false;
    }

    /// Stores the points of the angular roots, back in the absolute frame; an
    /// indirect curve maps the angle `a` to `2 PI - a`.
    fn store_angular_roots(
        &mut self,
        roots: &TrigonometricRoots,
        curve: &ClosedCurve,
    ) -> Result<(), Failure> {
        if !roots.is_done() {
            self.done = false;
            return Ok(());
        }
        if roots.infinite_roots() {
            self.iden = true;
            self.done = true;
            return Ok(());
        }
        self.nbp = roots.solutions()?;
        for i in 1..=self.nbp {
            let mut s: f64 = roots.value(i)?;
            let (tx, ty) =
                coord_ancien_repere(curve.major * s.cos(), curve.minor * s.sin(), curve.axis);
            if !curve.direct {
                s = PI + PI - s;
            }
            self.lpnt[(i - 1) as usize].set_implicit_value(tx, ty, s);
        }
        self.nbp = traitement_points_confondus(self.nbp, &mut self.lpnt);
        self.done = true;
        Ok(())
    }
}
