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
// Rust translation of VerifSurface and VerifPoints of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks of the plate surface against the constraints.
use super::GeomPlateBuildPlateSurface;
use super::errors::ConstraintGaps;
use crate::geom_plate::{GeomPlateCurveConstraint, GeomPlatePointConstraint, live};
use crate::local_analysis::{LocalAnalysisSurfaceContinuity, SurfaceContinuityTolerances};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// The relative excesses of one curve over its criteria.
#[derive(Default)]
struct Excess {
    diff_dist_max: f64,
    diff_ang_max: f64,
    ndiff_dist: i32,
    ndiff_ang: i32,
}

fn gap(array: &rustycad_tkernel::ncollection::Array1<'_, f64>, j: i32) -> Result<f64, Failure> {
    array.value(j).copied().map_err(errors::array_bounds)
}

impl GeomPlateBuildPlateSurface {
    /// The parameter at which gap `j` of curve `lin` is checked: the interval middle
    /// when the curve has more than 3 parameters, else a uniform sample.
    fn check_parameter(
        lin: &GeomPlateCurveConstraint,
        par: &[f64],
        j: i32,
        nb_pts: i32,
    ) -> Result<f64, Failure> {
        if par.len() > 3 {
            let at = |k: i32| par[(k - 1) as usize];
            return Ok((at(j) + at(j + 1)) / 2.0);
        }
        let (first, last) = (lin.first_parameter()?, lin.last_parameter()?);
        Ok(first + (last - first) * f64::from(j - 1) / f64::from(nb_pts - 2))
    }

    /// Updates the global errors with the gaps of curve `i` and measures how far the
    /// gaps exceed the criteria.
    fn curve_excess(&mut self, i: i32, lin: &GeomPlateCurveConstraint) -> Result<Excess, Failure> {
        let par: Vec<f64> = self.par_cont_of(i)?.to_vec();
        let nb_pts: i32 = if par.len() < 3 { 4 } else { par.len() as i32 };
        let mut gaps = ConstraintGaps::zeroed(nb_pts - 1)?;
        self.ecart_contraintes_mil(i, &mut gaps)?;
        let mut excess = Excess::default();
        for j in 1..nb_pts {
            let (tdist, tang, tcourb) = (gap(&gaps.d, j)?, gap(&gaps.an, j)?, gap(&gaps.courb, j)?);
            if tdist > self.g0_error {
                self.g0_error = tdist;
            }
            if tang > self.g1_error {
                self.g1_error = tang;
            }
            if tcourb > self.g2_error {
                self.g2_error = tcourb;
            }
            let u: f64 = Self::check_parameter(lin, &par, j, nb_pts)?;
            let mut diff_dist: f64 = tdist - lin.g0_criterion(u)?;
            let mut diff_ang: f64 = if lin.order() > 0 {
                tang - lin.g1_criterion(u)?
            } else {
                0.0
            };
            if diff_dist > 0.0 {
                diff_dist /= lin.g0_criterion(u)?;
                if diff_dist > excess.diff_dist_max {
                    excess.diff_dist_max = diff_dist;
                }
                excess.ndiff_dist += 1;
            } else if diff_ang > 0.0 && lin.order() == 1 {
                diff_ang /= lin.g1_criterion(u)?;
                if diff_ang > excess.diff_ang_max {
                    excess.diff_ang_max = diff_ang;
                }
                excess.ndiff_ang += 1;
            }
        }
        Ok(excess)
    }

    /// Raises the number of points of a curve whose gaps exceed the criteria; false
    /// when it does.
    fn refine_curve(lin: &GeomPlateCurveConstraint, excess: &Excess, nb_boucle: i32) -> bool {
        let nb_points: i32 = lin.nb_points();
        let mut coef: f64;
        if excess.ndiff_dist > 0 {
            // 7.4 gives the minimal coefficient 1.2 (e^1.2 / 0.6), 3.3 gives e^1.2.
            coef = if lin.order() == 0 {
                0.6 * (excess.diff_dist_max + 7.4).ln()
            } else {
                (excess.diff_dist_max + 3.3).ln()
            };
            if coef > 3.0 {
                coef = 3.0;
            }
            if nb_boucle > 1 && excess.diff_dist_max > 2.0 {
                coef = 1.6;
            }
            if f64::from(nb_points) >= (f64::from(nb_points) * coef).floor() {
                coef = 2.0;
            }
        } else if excess.ndiff_ang > 0 {
            coef = 1.5;
            if f64::from(nb_points + 1) >= (f64::from(nb_points) * coef).floor() {
                coef = 2.0;
            }
        } else {
            return true;
        }
        lin.set_nb_points((f64::from(nb_points) * coef) as i32);
        false
    }

    /// Checks the plate against the curve constraints at the middles of their
    /// intervals (`VerifSurface`): records the greatest gaps and, for each curve out
    /// of its criteria, raises its number of points; false when one is. The plate is
    /// then reset (kept as the previous plate after the first loop of a free plate).
    pub(super) fn verif_surface(&mut self, nb_boucle: i32) -> Result<bool, Failure> {
        let mut result: bool = true;
        (self.g0_error, self.g1_error, self.g2_error) = (0.0, 0.0, 0.0);
        for i in 1..=self.nt_lin_cont() {
            let lin_handle = self.lin_cont[(i - 1) as usize].clone();
            let lin = live(&lin_handle, "a curve constraint")?;
            if lin.order() == -1 {
                continue;
            }
            let excess: Excess = self.curve_excess(i, lin)?;
            if !Self::refine_curve(lin, &excess, nb_boucle) {
                result = false;
            }
        }
        if !result {
            if self.free && nb_boucle == 1 {
                self.prev_plate.copy_from(&self.plate);
            }
            self.plate.init();
        }
        Ok(result)
    }

    /// The gaps of point constraint `pnt` of order G0, G1 or G2 (`VerifPoints`, one
    /// point).
    fn point_gaps(
        &self,
        pnt: &GeomPlatePointConstraint,
        gaps: &mut (f64, f64, f64),
    ) -> Result<(), Failure> {
        let plate = live(&self.geom_plate_surface, "the plate surface")?;
        match pnt.order() {
            0 => {
                let p2d: Point2 = pnt.pnt2d_on_surf();
                gaps.0 = plate.d0(p2d.x(), p2d.y())?.distance(&pnt.d0());
            }
            1 => {
                let (pi, v1i, v2i) = pnt.d1();
                let p2d: Point2 = pnt.pnt2d_on_surf();
                let df = plate.d1(p2d.x(), p2d.y())?;
                gaps.0 = df.point.distance(&pi);
                let mut ang: f64 = (df.d1u ^ df.d1v).angle(v1i ^ v2i)?;
                if ang > PI / 2.0 {
                    ang = PI - ang;
                }
                gaps.1 = ang;
            }
            2 => {
                let splate: Handle<dyn GeomSurface> = self.geom_plate_surface.cast();
                let mut cg2 = LocalAnalysisSurfaceContinuity::with_tolerances(
                    &SurfaceContinuityTolerances::default(),
                );
                let p2d: Point2 = pnt.pnt2d_on_surf();
                let mut prop = SLProps::with_parameters(splate, p2d.x(), p2d.y(), 2, 0.001)?;
                cg2.compute_analysis(&mut prop, &mut *pnt.lprop_surf()?, Shape::G2)?;
                *gaps = (cg2.c0_value()?, cg2.g1_angle()?, cg2.g2_curvature_gap()?);
            }
            _ => {}
        }
        Ok(())
    }

    /// The gaps `(Dist, Ang, Curv)` of the last point constraint of order 0 to 2 that
    /// sets them (`VerifPoints`): each point overwrites the values it computes, so
    /// only the last ones remain, as natively.
    pub(super) fn verif_points(&self) -> Result<(f64, f64, f64), Failure> {
        let mut gaps: (f64, f64, f64) = (0.0, 0.0, 0.0);
        for pnt in &self.pnt_cont {
            self.point_gaps(live(pnt, "a point constraint")?, &mut gaps)?;
        }
        Ok(gaps)
    }
}
