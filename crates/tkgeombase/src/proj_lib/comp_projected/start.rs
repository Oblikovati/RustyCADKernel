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
// Rust translation of the search for the beginning of a continuous part in
// ProjLib_CompProjectedCurve::Init at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start of a continuous part: the curve is sampled from `t` by `Step` until a
//! point projects; on a periodic surface a start on the seam is moved to the side
//! the projection goes to, and a start after the curve start is moved back to the
//! exact bound where the projection enters the surface.
use super::FUNC_TOL;
use super::bounds::initial_point;
use super::derivatives::d1;
use super::init::Walk;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Point3, Vector2};

/// The result of the start search.
pub(super) enum StartSearch {
    /// A start was found (the walk triple).
    Found,
    /// No more start on the curve.
    NotFound,
    /// The start is farther than MaxDist from the surface: the parts are cleared.
    Cleared,
}

impl Walk<'_> {
    /// The loop `while (t <= LastU && !new_part && !FromLastU && !SameDeb)`.
    pub(super) fn search_start(&mut self) -> Result<StartSearch, Failure> {
        while self.t <= self.last_u && !self.from_last_u && !self.same_deb {
            self.prev_deb = self.t;
            if self.t == self.last_u {
                self.from_last_u = true;
            }
            let Some((u, v, c_point)) = self.initial_candidate()? else {
                if self.t == self.last_u {
                    break;
                }
                self.t += self.step;
                if self.t > self.last_u {
                    self.step = self.step + self.last_u - self.t;
                    self.t = self.last_u;
                }
                continue;
            };
            let (u, v) = self.off_seam(u, v)?;
            if self.state.max_dist > 0. {
                // Stop when the projection is farther than MaxDist from the curve.
                let on_surface: Point3 = self.pair.surface.d0(u, v)?;
                if c_point.distance(&on_surface) > self.state.max_dist {
                    return Ok(StartSearch::Cleared);
                }
            }
            self.triple = Point3::new(self.t, u, v);
            if self.t != self.first_u {
                // Search for the exact boundary point.
                self.exact_boundary(self.t - self.step)?;
            }
            return Ok(StartSearch::Found);
        }
        Ok(StartSearch::NotFound)
    }

    // The initial point at `t`: an extremum curve-surface in `]t - Step, t]` that
    // projects (it moves `t`), else the nearest normal projection of `C(t)`.
    fn initial_candidate(&mut self) -> Result<Option<(f64, f64, Point3)>, Failure> {
        if self.nend != 0 && !self.ext.is_parallel()? {
            let lower: f64 = self
                .first_u
                .max(self.t - self.step + precision::p_confusion());
            for i in 1..=self.nend {
                let (p1, p2) = self.ext.points(i)?;
                let par_t: f64 = p1.parameter();
                let (par_u, par_v) = p2.parameter();
                let mut resolve = self.resolver()?;
                resolve.perform(par_t, par_u, par_v, self.uv_box(), FUNC_TOL, true)?;
                if resolve.is_done() && p1.parameter() > lower && p1.parameter() <= self.t {
                    self.t = par_t;
                    return Ok(Some((par_u, par_v, *p1.value())));
                }
            }
        }
        let c_point: Point3 = self.pair.curve.d0(self.t)?;
        // PConfusion: geometric tolerances in extrema and optimization.
        let tol_uv: (f64, f64) = (self.state.tol_u, self.state.tol_v);
        let found = initial_point(c_point, self.t, self.pair, tol_uv, self.state.max_dist)?;
        Ok(found.map(|(u, v)| (u, v, c_point)))
    }

    // A start on the seam of a periodic surface (whose bounds are one period)
    // cannot start the resolution: it moves to the bound the projection goes to.
    fn off_seam(&self, mut u: f64, mut v: f64) -> Result<(f64, f64), Failure> {
        let s: &dyn Adaptor3dSurface = self.pair.surface;
        let (low, upp) = (self.low, self.upp);
        let u_seam: bool = s.is_u_periodic()?
            && (upp.x() - low.x() - s.u_period()?).abs() < precision::confusion();
        let v_seam: bool = s.is_v_periodic()?
            && (upp.y() - low.y() - s.v_period()?).abs() < precision::confusion();
        if !(u_seam || v_seam) {
            return Ok((u, v));
        }
        let u_res: f64 = s.u_resolution(precision::p_confusion())?;
        if (u - low.x()).abs() < u_res && s.is_u_periodic()? {
            if self.direction(u, v)?.x() < 0. {
                u = upp.x();
            }
        } else if (u - upp.x()).abs() < s.u_resolution(precision::p_confusion())?
            && s.is_u_periodic()?
        {
            if self.direction(u, v)?.x() > 0. {
                u = low.x();
            }
        }
        let v_res: f64 = s.v_resolution(precision::p_confusion())?;
        if (v - low.y()).abs() < v_res && s.is_v_periodic()? {
            if self.direction(u, v)?.y() < 0. {
                v = upp.y();
            }
        } else if (v - upp.y()).abs() <= s.v_resolution(precision::p_confusion())?
            && s.is_v_periodic()?
        {
            if self.direction(u, v)?.y() > 0. {
                v = low.y();
            }
        }
        Ok((u, v))
    }

    fn direction(&self, u: f64, v: f64) -> Result<Vector2, Failure> {
        d1(self.t, u, v, self.pair)
    }
}
