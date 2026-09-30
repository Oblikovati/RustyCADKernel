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
// Rust translation of ApproxInt_MultiLine::MakeMLOneMorePoint of
// ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_MultiLine.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MakeMLOneMorePoint`: the sub-line with one more point, found by the SvSurfaces
//! between a bad point and its predecessor.
use super::ApproxIntMultiLine;
use crate::approx_int::line::ApproxIntLine;
use crate::approx_int::sv_surfaces::SvSurfacesRef;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;

/// The tolerance of the parameter tests (`tolerance(1) = tolerance(2) = 1.e-8`).
const PARAM_TOL: f64 = 1.0e-8;

/// The two points around the insertion: the 3d points and parameters of the points
/// `indbad - 1` and `indbad`, and the middle parameters.
struct Around {
    prev_pnt: Point3,
    cur_pnt: Point3,
    cur: [f64; 4],
    mid: [f64; 4],
}

impl Around {
    /// Whether the middle is within the tolerance of the current point on the surface
    /// of the parameters `k`, `k + 1` (both tests of the native code).
    fn mid_on_current(&self, k: usize) -> bool {
        (self.cur[k] - self.mid[k]).abs() <= PARAM_TOL
            && (self.cur[k + 1] - self.mid[k + 1]).abs() <= PARAM_TOL
    }

    /// Whether `new` is further from the middle than the current point is, on the
    /// surface of the parameters `k`, `k + 1`.
    fn new_too_far(&self, new: [f64; 4], k: usize) -> bool {
        let (c, m) = (self.cur, self.mid);
        let sq_dist_cur_mid: f64 =
            (c[k] - m[k]) * (c[k] - m[k]) + (c[k + 1] - m[k + 1]) * (c[k + 1] - m[k + 1]);
        let sq_dist_mid_new: f64 =
            (m[k] - new[k]) * (m[k] - new[k]) + (m[k + 1] - new[k + 1]) * (m[k + 1] - new[k + 1]);
        sq_dist_mid_new > sq_dist_cur_mid
    }
}

impl<L: ApproxIntLine> ApproxIntMultiLine<L> {
    /// `MakeMLOneMorePoint(Low, High, indbad, OtherLine)`: the points `low..=high` with
    /// one more point, the solution of the SvSurfaces (solver switched on) near the
    /// middle of the points `index_bad - 1` and `index_bad`; None without SvSurfaces or
    /// when the new point is too close to its neighbours or further from the middle than
    /// the bad point; `line.make_ml_one_more_point(1, 20, 7)?`.
    pub fn make_ml_one_more_point(
        &self,
        low: i32,
        high: i32,
        index_bad: i32,
    ) -> Result<Option<Self>, Failure> {
        let Some(sv) = self.ptr_on_sv_surfaces.clone() else {
            return Ok(None);
        };
        let save_use_solver: bool = sv.borrow().get_use_solver();
        if !save_use_solver {
            sv.borrow_mut().set_use_solver(true);
        }
        let line: &L = self.line("MakeMLOneMorePoint")?;
        let result = Handle::new(IntSurfLineOn2S::new(None));
        for indice in low..=high {
            result.add(&line.point(indice)?);
        }
        let around: Around = around(line, index_bad)?;
        let made: bool = match self.new_point(&sv, &around)? {
            Some(mid_point) => {
                result.insert_before(index_bad - low + 1, &mid_point)?;
                true
            }
            None => false,
        };
        sv.borrow_mut().set_use_solver(save_use_solver);
        if !made {
            return Ok(None);
        }
        let temp = Handle::new(L::from_points(&result, false));
        let layout = self.layout(1, result.nb_points());
        Ok(Some(Self::with_sv_surfaces(temp, Some(sv), layout)))
    }

    /// The point to insert, None when it is invalid (the native `IsNewPointInvalid`).
    fn new_point(
        &self,
        sv: &SvSurfacesRef,
        around: &Around,
    ) -> Result<Option<IntSurfPntOn2S>, Failure> {
        if self.approx_u1v1 && around.mid_on_current(0) {
            return Ok(None);
        }
        if self.approx_u2v2 && around.mid_on_current(2) {
            return Ok(None);
        }
        let Some(mid_point) = sv.borrow_mut().seek_point(around.mid)? else {
            return Ok(None);
        };
        let sq_tol_3d: f64 = precision::square_confusion();
        let new_pnt: Point3 = mid_point.value();
        if new_pnt.square_distance(&around.prev_pnt) <= sq_tol_3d
            || new_pnt.square_distance(&around.cur_pnt) <= sq_tol_3d
        {
            return Ok(None);
        }
        let (u1, v1, u2, v2) = mid_point.parameters();
        let new: [f64; 4] = [u1, v1, u2, v2];
        if self.approx_u1v1 && around.new_too_far(new, 0) {
            return Ok(None);
        }
        if self.approx_u2v2 && around.new_too_far(new, 2) {
            return Ok(None);
        }
        Ok(Some(mid_point))
    }
}

/// The points `index_bad - 1` and `index_bad` and their middle parameters.
fn around<L: ApproxIntLine>(line: &L, index_bad: i32) -> Result<Around, Failure> {
    let prev = line.point(index_bad - 1)?;
    let cur = line.point(index_bad)?;
    let (up1, vp1, up2, vp2) = prev.parameters();
    let (uc1, vc1, uc2, vc2) = cur.parameters();
    Ok(Around {
        prev_pnt: prev.value(),
        cur_pnt: cur.value(),
        cur: [uc1, vc1, uc2, vc2],
        mid: [
            (up1 + uc1) / 2.0,
            (vp1 + vc1) / 2.0,
            (up2 + uc2) / 2.0,
            (vp2 + vc2) / 2.0,
        ],
    })
}
