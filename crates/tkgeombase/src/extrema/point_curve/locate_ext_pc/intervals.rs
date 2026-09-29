// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of the B-spline, offset and other-curve case of
// Extrema_GLocateExtPC::Perform and of its accessors at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The interval search of Extrema_GLocateExtPC: the C2 interval holding the start,
//! then the intervals at increasing distance on both sides, where a sign change of
//! `(C - P).C'` at a shared bound is an extremum at that bound.
use super::GLocateExtPC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::not_done;
use crate::extrema::gp_space::GpPoint;
use crate::extrema::pon::POnCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;

/// The signs kept across the neighbour search (`s1sup`, `s2inf`).
struct BoundSigns {
    s1_sup: f64,
    s2_inf: f64,
}

impl<'a, T: ExtremaCurveTool> GLocateExtPC<'a, T> {
    /// The GeomAbs_OtherCurve, GeomAbs_OffsetCurve and GeomAbs_BSplineCurve case.
    pub(super) fn perform_intervals(&mut self, point: T::Point, u0: f64) -> Result<(), Failure> {
        let bounds: Vec<f64> = Self::c2_bounds(self.curve()?)?;
        let n: i32 = bounds.len() as i32 - 1;
        // be gentle with the caller
        let u0: f64 = if u0 < self.umin {
            self.umin
        } else if u0 > self.usup {
            self.usup
        } else {
            u0
        };
        let (mut found, mut inter) = (false, 1);
        let (mut int_uinf, mut int_usup) = (0.0, 0.0);
        while !found && inter <= n {
            (int_uinf, int_usup) = self.clipped(&bounds, inter);
            if (u0 >= int_uinf) && (u0 < int_usup) {
                found = true;
            }
            inter += 1;
        }
        if found {
            inter -= 1; // IFV 16.06.00 - inter is increased after found!
        }
        let tol: f64 = self.tol;
        self.loc_ext_pc
            .initialize(self.curve()?, (int_uinf, int_usup), tol)?;
        self.loc_ext_pc.perform(point, u0)?;
        self.done = self.loc_ext_pc.is_done();
        if self.done {
            return self.take_local();
        }
        let mut signs = BoundSigns {
            s2_inf: self.projection_sign(point, int_uinf)?.0,
            s1_sup: self.projection_sign(point, int_usup)?.0,
        };
        self.search_neighbours(point, &bounds, inter, &mut signs)
    }

    // Tries the intervals `inter + k` and `inter - k` for k = 1, 2, ...
    fn search_neighbours(
        &mut self,
        point: T::Point,
        bounds: &[f64],
        inter: i32,
        signs: &mut BoundSigns,
    ) -> Result<(), Failure> {
        let n: i32 = bounds.len() as i32 - 1;
        let (mut i1, mut i2, mut k) = (inter, inter, 1);
        while !self.done && (i2 > 0) && (i1 <= n) {
            i1 = inter + k;
            i2 = inter - k;
            if i1 <= n && self.try_upper(point, bounds, i1, signs)? {
                break;
            }
            if i2 > 0 && self.try_lower(point, bounds, i2, signs)? {
                break;
            }
            k += 1;
        }
        Ok(())
    }

    // The interval after the start one; true to stop the search (an extremum at its
    // lower bound, a local result, or an infinite sign).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn try_upper(
        &mut self,
        point: T::Point,
        bounds: &[f64],
        i1: i32,
        signs: &mut BoundSigns,
    ) -> Result<bool, Failure> {
        let (int_uinf, int_usup) = self.clipped(bounds, i1);
        if !(int_uinf < int_usup) {
            return Ok(false);
        }
        let (s2_sup, p1) = self.projection_sign(point, int_uinf)?;
        if precision::is_infinite(s2_sup) || precision::is_infinite(signs.s1_sup) {
            return Ok(true);
        }
        if signs.s1_sup * s2_sup <= f64::EPSILON {
            // extremum:
            self.done = true;
            self.pp.set_values(int_uinf, p1);
            self.is_min = signs.s1_sup <= 0.0;
            self.dist2 = point.square_distance_to(&p1);
            return Ok(true);
        }
        signs.s1_sup = self.projection_sign(point, int_usup)?.0;
        self.local_search(point, (int_uinf, int_usup))
    }

    // The interval before the start one, symmetrically.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn try_lower(
        &mut self,
        point: T::Point,
        bounds: &[f64],
        i2: i32,
        signs: &mut BoundSigns,
    ) -> Result<bool, Failure> {
        let (int_uinf, int_usup) = self.clipped(bounds, i2);
        if !(int_uinf < int_usup) {
            return Ok(false);
        }
        let (s1_inf, p1) = self.projection_sign(point, int_usup)?;
        if precision::is_infinite(signs.s2_inf) || precision::is_infinite(s1_inf) {
            return Ok(true);
        }
        if s1_inf * signs.s2_inf <= f64::EPSILON {
            // extremum:
            self.done = true;
            self.pp.set_values(int_usup, p1);
            self.is_min = s1_inf <= 0.0;
            self.dist2 = point.square_distance_to(&p1);
            return Ok(true);
        }
        signs.s2_inf = self.projection_sign(point, int_uinf)?.0;
        self.local_search(point, (int_uinf, int_usup))
    }

    // The local search from the middle of an interval; true when it succeeds.
    fn local_search(
        &mut self,
        point: T::Point,
        (int_uinf, int_usup): (f64, f64),
    ) -> Result<bool, Failure> {
        let tol: f64 = self.tol;
        self.loc_ext_pc
            .initialize(self.curve()?, (int_uinf, int_usup), tol)?;
        self.loc_ext_pc
            .perform(point, (int_uinf + int_usup) * 0.5)?;
        self.done = self.loc_ext_pc.is_done();
        if self.done {
            self.take_local()?;
        }
        Ok(self.done)
    }
}

impl<T: ExtremaCurveTool> GLocateExtPC<'_, T> {
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(not_done(&format!("Extrema_GLocateExtPC::{member}()")))
    }
    /// `SquareDistance()`: StdFail_NotDone when not done; `extrema.square_distance()?`.
    pub fn square_distance(&self) -> Result<f64, Failure> {
        self.check_done("SquareDistance")?;
        match self.kind {
            CurveType::BezierCurve => self.loc_ext_pc.square_distance(),
            CurveType::BSplineCurve | CurveType::OffsetCurve | CurveType::OtherCurve => {
                Ok(self.dist2)
            }
            _ if self.number_ext != 0 => self.extrem_pc.square_distance(self.number_ext),
            _ => Ok(0.0),
        }
    }
    /// `IsMin()`; `extrema.is_min()?`.
    pub fn is_min(&self) -> Result<bool, Failure> {
        self.check_done("IsMin")?;
        match self.kind {
            CurveType::BezierCurve => self.loc_ext_pc.is_min(),
            CurveType::BSplineCurve | CurveType::OffsetCurve | CurveType::OtherCurve => {
                Ok(self.is_min)
            }
            _ if self.number_ext != 0 => self.extrem_pc.is_min(self.number_ext),
            _ => Ok(false),
        }
    }
    /// `Point()`; `extrema.point()?.parameter()`.
    pub fn point(&self) -> Result<POnCurve<T::Point>, Failure> {
        self.check_done("Point")?;
        match self.kind {
            CurveType::BezierCurve => self.loc_ext_pc.point(),
            CurveType::BSplineCurve | CurveType::OffsetCurve | CurveType::OtherCurve => Ok(self.pp),
            _ => self.extrem_pc.point(self.number_ext),
        }
    }
}
