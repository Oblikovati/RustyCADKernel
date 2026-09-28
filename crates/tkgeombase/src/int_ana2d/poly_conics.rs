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
// Rust translation of IntAna2d_AnaIntersection_7.cxx and IntAna2d_AnaIntersection_8.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_AnaIntersection::Perform(gp_Parab2d, IntAna2d_Conic) and
//! Perform(gp_Hypr2d, IntAna2d_Conic): quartic equations solved by
//! [`MyDirectPolynomialRoots`].
use super::{
    AnaIntersection, Conic, MyDirectPolynomialRoots, coord_ancien_repere,
    traitement_points_confondus,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2d, Hyperbola2, Parabola2};

impl AnaIntersection {
    /// Perform(gp_Parab2d, IntAna2d_Conic): the roots of the conic equation along
    /// `y = y`, `x = y^2 / (2p)` in the parabola frame (the parameter is `y`, negated
    /// for an indirect parabola), without the coincident points. Native quirk kept:
    /// this overload never returns early, so it is always done, even when the
    /// solver fails (no point) or the elements are identical (IdenticalElements
    /// and no point). `inter.perform_parabola_conic(parabola, &conic)?`.
    pub fn perform_parabola_conic(
        &mut self,
        parabola: Parabola2,
        conic: &Conic,
    ) -> Result<(), Failure> {
        let direct: bool = parabola.is_direct();
        let un_sur_2p: f64 = 0.5 / parabola.parameter();
        let axis: Axis2d = parabola.axis_of_symmetry();
        self.reset_conic_flags();
        let [a, b, c, d, e, f] = conic.new_coefficients(conic.coefficients(), axis);
        let px0: f64 = f;
        let px1: f64 = e + e;
        let px2: f64 = b + un_sur_2p * (d + d);
        let px3: f64 = (c + c) * un_sur_2p;
        let px4: f64 = a * (un_sur_2p * un_sur_2p);
        let roots = MyDirectPolynomialRoots::quartic(px4, px3, px2, px1, px0)?;
        if roots.is_done() {
            if roots.infinite_roots() {
                self.iden = true;
            }
            self.nbp = roots.nb_solutions();
            for i in 1..=self.nbp {
                let s: f64 = roots.value(i)?;
                let (tx, ty) = coord_ancien_repere(un_sur_2p * s * s, s, axis);
                let parameter: f64 = if direct { s } else { -s };
                self.lpnt[(i - 1) as usize].set_implicit_value(tx, ty, parameter);
            }
            self.nbp = traitement_points_confondus(self.nbp, &mut self.lpnt);
        }
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Hypr2d, IntAna2d_Conic): the roots of the conic equation along
    /// `x = MajorRadius Ch(t)`, `y = MinorRadius Sh(t)` rewritten in `S = Exp(t)`
    /// (multiplied by `4 S^2`); only the roots above RealEpsilon() give points, of
    /// parameter `Log(S)` (negated for an indirect hyperbola).
    /// `inter.perform_hyperbola_conic(hyperbola, &conic)?`.
    pub fn perform_hyperbola_conic(
        &mut self,
        hyperbola: Hyperbola2,
        conic: &Conic,
    ) -> Result<(), Failure> {
        let direct: bool = hyperbola.is_direct();
        let (minor, major) = (hyperbola.minor_radius(), hyperbola.major_radius());
        self.reset_conic_flags();
        let axis: Axis2d = hyperbola.x_axis();
        let [a, b, c, d, e, f] = conic.new_coefficients(conic.coefficients(), axis);
        let a_major_p2: f64 = a * major * major;
        let b_minor_p2: f64 = b * minor * minor;
        let c_2_major_minor: f64 = c * 2.0 * major * minor;
        let px0: f64 = a_major_p2 - c_2_major_minor + b_minor_p2;
        let px1: f64 = 4.0 * (d * major - e * minor);
        let px2: f64 = 2.0 * (a_major_p2 + 2.0 * f - b_minor_p2);
        let px3: f64 = 4.0 * (d * major + e * minor);
        let px4: f64 = a_major_p2 + c_2_major_minor + b_minor_p2;
        let roots = MyDirectPolynomialRoots::quartic(px4, px3, px2, px1, px0)?;
        if !roots.is_done() {
            self.done = false;
            return Ok(());
        }
        if roots.infinite_roots() {
            self.iden = true;
            self.done = true;
            return Ok(());
        }
        self.nbp = roots.nb_solutions();
        let mut valid: i32 = 0;
        for i in 1..=self.nbp {
            let s: f64 = roots.value(i)?;
            if s > f64::EPSILON {
                let tx: f64 = 0.5 * major * (s + 1.0 / s);
                let ty: f64 = 0.5 * minor * (s - 1.0 / s);
                valid += 1;
                let (tx, ty) = coord_ancien_repere(tx, ty, axis);
                let parameter: f64 = if direct { s.ln() } else { -s.ln() };
                self.lpnt[(valid - 1) as usize].set_implicit_value(tx, ty, parameter);
            }
        }
        self.nbp = traitement_points_confondus(valid, &mut self.lpnt);
        self.done = true;
        Ok(())
    }
}
