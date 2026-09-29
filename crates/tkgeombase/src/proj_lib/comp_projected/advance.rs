// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of the successful step of ProjLib_CompProjectedCurve::Init and of
// UpdateTripleByTrapCriteria at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A successful step of the walk: the new triple, the trap check, the splits of a
//! curve crossing the periodic bounds, the next step (stopped at the next split).
use super::bounds::initial_point;
use super::init::Walk;
use super::parts::Parts;
use super::splits::build_curve_splits;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};

impl Walk<'_> {
    /// The step to `t` succeeded with `solution`; returns true when the part ends
    /// (the curve end is reached, or the triple is on a split).
    pub(super) fn advance(&mut self, index: i32, solution: Point2) -> Result<bool, Failure> {
        self.prev_triple = self.triple;
        self.triple = Point3::new(self.t, solution.x(), solution.y());
        // Check for possible local traps.
        self.update_triple_by_trap_criteria()?;
        if !self.is_splits_computed && self.well_conditioned()? {
            // Protection from a curve lying entirely on a seam.
            let tol_uv: (f64, f64) = (self.state.tol_u, self.state.tol_v);
            self.splits = build_curve_splits(self.pair, &self.state.surface, tol_uv)?;
            self.is_splits_computed = true;
        }
        let parts = self.state.parts()?;
        if (self.triple.x() - parts.last_point(index)?.x()) > 1.0e-10 {
            parts.push_point(index, self.triple)?;
        }
        if self.t == self.last_u {
            self.t = self.last_u + 1.;
            return Ok(true);
        }
        self.step = self.walk_step()?;
        self.t += self.step;
        if self.t > (self.last_u - self.min_step / 2.) {
            self.step = self.step + self.last_u - self.t;
            self.t = self.last_u;
        }
        Ok(!self.stop_at_split())
    }

    // Whether the triple is off the periodic bounds in a periodic direction.
    fn well_conditioned(&self) -> Result<bool, Failure> {
        let s: &dyn Adaptor3dSurface = self.pair.surface;
        let pconf: f64 = precision::p_confusion();
        let u_possible: bool = s.is_u_periodic()?
            && ((self.triple.y() - s.first_u_parameter()?).abs() > pconf
                && (self.triple.y() - s.last_u_parameter()?).abs() > pconf);
        let v_possible: bool = s.is_v_periodic()?
            && ((self.triple.z() - s.first_v_parameter()?).abs() > pconf
                && (self.triple.z() - s.last_v_parameter()?).abs() > pconf);
        Ok(u_possible || v_possible)
    }

    // At least one cached point lies inside a split: a triple on a split ends the
    // part (returns false after moving past it); a step over one stops at it.
    fn stop_at_split(&mut self) -> bool {
        for an_idx in self.split_idx..self.splits.len() {
            let param: f64 = self.splits[an_idx];
            if (param - self.triple.x()).abs() < precision::p_confusion() {
                self.split_idx += 1;
                return false;
            }
            if param < self.t + precision::p_confusion() {
                self.t = param;
                self.step = self.t - self.prev_triple.x();
            }
        }
        true
    }

    /// `UpdateTripleByTrapCriteria(thePoint)`: on a surface of revolution near a V
    /// bound (bug 25892) or on a sphere near any bound (bug 27135, a degenerated
    /// edge), the point is replaced by the nearest normal projection, unless that
    /// only jumps a period.
    fn update_triple_by_trap_criteria(&mut self) -> Result<(), Failure> {
        if !self.trap_possible()? {
            return Ok(());
        }
        let point: Point3 = self.pair.curve.value(self.triple.x())?;
        let pconf: f64 = precision::p_confusion();
        let found = initial_point(
            point,
            self.triple.x(),
            self.pair,
            (pconf, pconf),
            self.state.max_dist,
        )?;
        let Some((mut u, mut v)) = found else {
            return Ok(());
        };
        // Restore the original position in case of a period jump.
        let s: &dyn Adaptor3dSurface = self.pair.surface;
        if s.is_u_periodic()? && ((u - self.triple.y()).abs() - s.u_period()?).abs() < pconf {
            u = self.triple.y();
        }
        if s.is_v_periodic()? && ((v - self.triple.z()).abs() - s.v_period()?).abs() < pconf {
            v = self.triple.z();
        }
        self.triple.set_y(u);
        self.triple.set_z(v);
        Ok(())
    }

    fn trap_possible(&self) -> Result<bool, Failure> {
        let s: &dyn Adaptor3dSurface = self.pair.surface;
        let kind: SurfaceType = s.get_type()?;
        let mut possible: bool = false;
        if kind == SurfaceType::SurfaceOfRevolution {
            // The maximal deviation from 3D.
            let max_tol: f64 =
                precision::p_confusion().max(s.v_resolution(precision::confusion())?);
            if (self.triple.z() - s.first_v_parameter()?).abs() < max_tol
                || (self.triple.z() - s.last_v_parameter()?).abs() < max_tol
            {
                possible = true;
            }
        }
        let pconf: f64 = precision::p_confusion();
        if kind == SurfaceType::Sphere
            && ((self.triple.z() - s.first_v_parameter()?).abs() < pconf
                || (self.triple.z() - s.last_v_parameter()?).abs() < pconf
                || (self.triple.y() - s.first_u_parameter()?).abs() < pconf
                || (self.triple.y() - s.last_u_parameter()?).abs() < pconf)
        {
            possible = true;
        }
        Ok(possible)
    }
}
