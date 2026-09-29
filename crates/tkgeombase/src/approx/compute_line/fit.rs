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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeLine.gxx (Compute, Parameters)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fit of one piece of `Approx_ComputeLine` and its parametrization.
use super::{ComputeLine, owned_array};
use crate::app_par_curves::arrays::owned_vector;
use crate::app_par_curves::{Gradient, LeastSquare, LineTool, MultiCurve};
use crate::approx::check_multi_curve::check_multi_curve;
use crate::approx::status::ParametrizationType;
use crate::approx::tangency::cumulated_chords;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkmath::math::Vector;

/// The errors of one fit, `TheTol3d` and `TheTol2d`.
pub(super) type FitTolerances = [f64; 2];

impl ComputeLine {
    /// `Parameters(Line, firstP, LastP, TheParameters)`: the chord-length or
    /// centripetal parameters scaled by the last one (the first becomes `0 / L`, NaN
    /// for coincident points), or `(i - firstP) / (lastP - firstP)`.
    pub(super) fn compute_parameters<L: LineTool>(
        &self,
        line: &L,
        first_p: i32,
        last_p: i32,
        parameters: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        if self.par == ParametrizationType::IsoParametric {
            for i in first_p..=last_p {
                *parameters.change_value(i)? =
                    (f64::from(i) - f64::from(first_p)) / (f64::from(last_p) - f64::from(first_p));
            }
            return Ok(());
        }
        *parameters.change_value(first_p)? = 0.0;
        let centripetal: bool = self.par == ParametrizationType::Centripetal;
        cumulated_chords(line, [first_p, last_p], centripetal, parameters)?;
        for i in first_p..=last_p {
            *parameters.change_value(i)? /= *parameters.value(last_p)?;
        }
        Ok(())
    }

    /// `Compute(Line, fpt, lpt, Para, TheTol3d, TheTol2d, indbad)`: fits the degrees
    /// from `min(nbp - 1, degreemin)` and stores the first piece within the
    /// tolerances that does not loop; otherwise keeps the best fit in
    /// `TheMultiCurve` (and its parameters) and returns false. `index_bad` receives
    /// the loop point of a rejected fit.
    pub(super) fn compute<L: LineTool + Clone>(
        &mut self,
        line: &L,
        [fpt, lpt]: [i32; 2],
        para: &mut Vector<'_>,
        tolerances: &mut FitTolerances,
        index_bad: &mut i32,
    ) -> Result<bool, Failure> {
        *index_bad = 0;
        let nbp: i32 = lpt - fpt + 1;
        let par_sav: Vector<'static> = owned_vector(para)?;
        let mut mdegmax: i32 = self.options.degree_max;
        if nbp < mdegmax + 5 && self.options.cutting {
            mdegmax = nbp - 5;
        }
        mdegmax = mdegmax.max(self.options.degree_min);
        (self.currenttol3d, self.currenttol2d) = (real_last(), real_last());
        for deg in (nbp - 1).min(self.options.degree_min)..=mdegmax {
            let Some((curve, done)) = self.fit_degree(line, [fpt, lpt], para, deg, tolerances)?
            else {
                continue;
            };
            if !increasing(para)? {
                para.initialized(&par_sav)?;
            }
            if done && self.fits(*tolerances) {
                self.tolreached = true;
                if !check_multi_curve(&curve, line, [fpt, lpt], index_bad)? {
                    return Ok(false);
                }
                self.store(curve, owned_array(para)?, *tolerances);
                return Ok(true);
            }
            if tolerances[0] <= self.currenttol3d && tolerances[1] <= self.currenttol2d {
                self.the_multi_curve = curve;
                (self.currenttol3d, self.currenttol2d) = (tolerances[0], tolerances[1]);
                self.my_parameters = Some(owned_array(para)?);
            }
        }
        Ok(false)
    }

    // One degree: the plain least squares or the optimizing gradient; None when
    // the gradient returns no curve (the native `continue`).
    fn fit_degree<L: LineTool + Clone>(
        &self,
        line: &L,
        [fpt, lpt]: [i32; 2],
        para: &mut Vector<'_>,
        deg: i32,
        tolerances: &mut FitTolerances,
    ) -> Result<Option<(MultiCurve, bool)>, Failure> {
        if self.options.squares {
            let (first_c, last_c) = (self.first_c, self.last_c);
            let mut sq = LeastSquare::new(line, fpt, lpt, first_c, last_c, para, deg + 1)?;
            let done: bool = sq.is_done();
            let curve: MultiCurve = sq.bezier_value()?;
            (_, tolerances[0], tolerances[1]) = sq.error()?;
            return Ok(Some((curve, done)));
        }
        let [tol3d, tol2d] = [self.options.tolerance_3d, self.options.tolerance_2d];
        let iterations: i32 = self.options.nb_iterations;
        let constraints = self.constraints();
        let grad = Gradient::new(
            line,
            fpt,
            lpt,
            &constraints,
            para,
            deg,
            tol3d,
            tol2d,
            iterations,
        )?;
        let curve: MultiCurve = grad.value();
        if curve.nb_curves()? == 0 {
            return Ok(None);
        }
        *tolerances = [grad.max_error_3d(), grad.max_error_2d()];
        Ok(Some((curve, grad.is_done())))
    }

    pub(super) fn fits(&self, [tol3d, tol2d]: FitTolerances) -> bool {
        tol3d <= self.options.tolerance_3d && tol2d <= self.options.tolerance_2d
    }

    // Stores a piece that fits ("Stockage de la multicurve approximee").
    pub(super) fn store(
        &mut self,
        curve: MultiCurve,
        parameters: rustycad_tkernel::ncollection::Array1<'static, f64>,
        [tol3d, tol2d]: FitTolerances,
    ) {
        self.multi_curves.push(curve);
        self.par_seq.push(parameters);
        self.tolers3d.push(tol3d);
        self.tolers2d.push(tol2d);
    }
}

// The optimized parameters must increase strictly (`restau` restores them otherwise).
fn increasing(para: &Vector<'_>) -> Result<bool, Failure> {
    let mut uu1: f64 = *para.value(para.lower())?;
    for i in para.lower() + 1..=para.upper() {
        let uu2: f64 = *para.value(i)?;
        if uu2 <= uu1 {
            return Ok(false);
        }
        uu1 = uu2;
    }
    Ok(true)
}
