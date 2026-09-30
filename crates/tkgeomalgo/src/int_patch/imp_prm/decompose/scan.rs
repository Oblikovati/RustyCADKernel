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
// Rust translation of the "while (flNextLine)" loop of DecomposeResult of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walk along the points of the line that cuts it into parts at the special points
//! and at the bounds of the quadric.
use super::super::super::IntPatchSpecPntType as Spt;
use super::super::super::point::IntPatchPoint;
use super::super::super::special_points::IntPatchSpecialPoints;
use super::super::setup::is_seam_or_pole;
use super::DecomposeCtx;
use super::emit::PartSink;
use super::smooth::detect_of_boundary_achievement;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::math::Vector;
use std::f64::consts::FRAC_PI_2;

/// The locals of `DecomposeResult` kept from one part of the line to the next.
pub(super) struct DecomposeState {
    lindex: i32,
    findex: i32,
    bindex: i32,
    has_been_decomposed: bool,
    pre_exist: Spt,
    pre_point: IntSurfPntOn2S,
}

impl DecomposeState {
    /// The state before the first part of a line of `lindex` points.
    pub(super) fn new(lindex: i32) -> Self {
        Self {
            lindex,
            findex: 1,
            bindex: 0,
            has_been_decomposed: false,
            pre_exist: Spt::SPntNone,
            pre_point: IntSurfPntOn2S::new(),
        }
    }

    /// The parts of the line, from the first point (or the point after the special point
    /// of the previous part) to the next special point or bound. A part of one point is
    /// skipped (the walk goes on after it). Returns whether the line was decomposed.
    pub(super) fn run(
        &mut self,
        ssline: &IntSurfLineOn2S,
        ctx: &DecomposeCtx<'_>,
        mut sink: PartSink<'_, '_>,
    ) -> Result<bool, Failure> {
        let mut fl_next_line: bool = true;
        while fl_next_line {
            fl_next_line = false;
            let sline = Handle::new(IntSurfLineOn2S::new(None));
            if self.lindex <= self.findex && self.pre_exist == Spt::SPntNone {
                break;
            }
            if self.pre_exist != Spt::SPntNone && !self.continue_after(ssline, ctx, &sline)? {
                break;
            }
            self.pre_exist = Spt::SPntNone;
            let is_decomposited: bool = self.scan(ssline, ctx, &sline)?;
            if sline.nb_points() == 1 {
                // A line of a single point cannot be made: the walk goes on.
                fl_next_line = true;
                if self.findex < self.bindex {
                    self.findex = self.bindex;
                }
                continue;
            }
            if sink.emit(&sline, is_decomposited, self.has_been_decomposed)? {
                return Ok(self.has_been_decomposed);
            }
            if is_decomposited {
                self.findex = self.bindex;
                fl_next_line = true;
                self.has_been_decomposed = true;
            }
        }
        Ok(self.has_been_decomposed)
    }

    /// The part starts at the point after the special point; false when there is none.
    fn continue_after(
        &mut self,
        ssline: &IntSurfLineOn2S,
        ctx: &DecomposeCtx<'_>,
        sline: &IntSurfLineOn2S,
    ) -> Result<bool, Failure> {
        let ref_pt: IntSurfPntOn2S = ssline.value(self.findex)?;
        let ures: f64 = ctx.q_surf.u_resolution(ctx.arc_tol)?;
        let vres: f64 = ctx.q_surf.v_resolution(ctx.arc_tol)?;
        let tol_2d: f64 = match self.pre_exist {
            Spt::SPntPole => -1.0,
            Spt::SPntSeamV => vres,
            Spt::SPntSeamUV => std_max(ures, vres),
            _ => ures,
        };
        if !IntPatchSpecialPoints::continue_after_special_point(
            ctx.q_surf,
            ctx.p_surf,
            &ref_pt,
            self.pre_exist,
            tol_2d,
            &mut self.pre_point,
            ctx.is_reversed,
        )? {
            return Ok(false);
        }
        sline.add(&self.pre_point);
        // Avoid adding duplicate points.
        while self.findex <= self.lindex {
            if !self
                .pre_point
                .is_same(&ssline.value(self.findex)?, ctx.tol_tang, -1.0)
            {
                break;
            }
            self.findex += 1;
        }
        Ok(true)
    }

    /// The points of the part up to the first special point or bound; returns whether
    /// the line is cut there.
    fn scan(
        &mut self,
        ssline: &IntSurfLineOn2S,
        ctx: &DecomposeCtx<'_>,
        sline: &IntSurfLineOn2S,
    ) -> Result<bool, Failure> {
        for k in self.findex..=self.lindex {
            if k == self.findex {
                self.pre_point = ssline.value(k)?;
                sline.add(&self.pre_point);
                continue;
            }
            // A point on a bound of the quadric (not a seam or pole) cuts the line simply.
            let mut is_on_boundary: bool = false;
            detect_of_boundary_achievement(
                ctx.q_surf,
                ctx.is_reversed,
                ssline,
                k,
                precision::p_confusion(),
                sline,
                &mut is_on_boundary,
            )?;
            self.pre_exist = is_seam_or_pole(
                ctx.q_surf,
                ssline,
                ctx.is_reversed,
                k - 1,
                (ctx.tol_tang, FRAC_PI_2),
            )?;
            if is_on_boundary && self.pre_exist != Spt::SPntPoleSeamU {
                // On the seam the parameters are known within the period; a pole or apex
                // on the bound is computed below (bugs modalg_6 bug26684_2).
                self.pre_exist = Spt::SPntNone;
            }
            if self.pre_exist != Spt::SPntNone {
                self.bindex = k;
                self.special_point(ssline, ctx, sline, (k, is_on_boundary))?;
                return Ok(true);
            }
            self.pre_point = ssline.value(k)?;
            if is_on_boundary {
                self.bindex = k;
                self.pre_exist = Spt::SPntNone;
                return Ok(true);
            }
            sline.add(&ssline.value(k)?);
        }
        Ok(false)
    }

    /// The special point between the points `k - 1` and `k` is computed and ends the
    /// part (unless it is the reference point).
    fn special_point(
        &mut self,
        ssline: &IntSurfLineOn2S,
        ctx: &DecomposeCtx<'_>,
        sline: &IntSurfLineOn2S,
        (k, mut is_on_boundary): (i32, bool),
    ) -> Result<(), Failure> {
        let ref_pt: IntSurfPntOn2S = ssline.value(self.bindex - 1)?;
        let compare_tol_3d: f64 = precision::confusion();
        let mut compare_tol_2d: f64 = precision::p_confusion();
        let mut new_point: IntSurfPntOn2S = ref_pt;
        let mut last_type: Spt = Spt::SPntNone;
        let iso = IsoInput {
            ssline,
            ctx,
            ref_pt: &ref_pt,
            k,
        };
        match self.pre_exist {
            Spt::SPntSeamUV => {
                self.pre_exist = Spt::SPntNone;
                last_type = Spt::SPntSeamUV;
                IntPatchSpecialPoints::add_cross_uv_iso_point(
                    ctx.q_surf,
                    ctx.p_surf,
                    &ref_pt,
                    ctx.tol_tang,
                    &mut new_point,
                    ctx.is_reversed,
                )?;
            }
            Spt::SPntSeamV => {
                // The line goes through the seam.
                self.pre_exist = Spt::SPntNone;
                last_type = Spt::SPntSeamV;
                iso.add(false, &mut new_point)?;
            }
            Spt::SPntPoleSeamU => {
                self.pre_exist = Spt::SPntNone;
                let mut vert = IntPatchPoint::new();
                vert.set_pnt_on_2s(&ref_pt);
                vert.set_tolerance(ctx.tol_tang);
                if IntPatchSpecialPoints::add_singular_pole(
                    ctx.q_surf,
                    ctx.p_surf,
                    &ref_pt,
                    &mut vert,
                    &mut new_point,
                    ctx.is_reversed,
                    false,
                )? {
                    self.pre_exist = Spt::SPntPole;
                    last_type = Spt::SPntPole;
                    if is_on_boundary {
                        // The point added on the bound is replaced by the pole.
                        is_on_boundary = false;
                        sline.remove_point(sline.nb_points())?;
                    }
                    compare_tol_2d = -1.0;
                } else {
                    // The pole is not an intersection point.
                    self.pre_exist = Spt::SPntSeamU;
                }
            }
            _ => {}
        }
        if self.pre_exist == Spt::SPntSeamU {
            // The line goes through the seam.
            self.pre_exist = Spt::SPntNone;
            last_type = Spt::SPntSeamU;
            iso.add(true, &mut new_point)?;
        }
        if !new_point.is_same(&ref_pt, compare_tol_3d, compare_tol_2d) {
            if is_on_boundary {
                return Ok(());
            }
            sline.add(&new_point);
            self.pre_exist = last_type;
            self.pre_point = new_point;
        } else if is_on_boundary || sline.nb_points() == 1 {
            // The first point of the part is the pole, or the point on the bound is
            // already a vertex: no decomposition.
            self.pre_point = ref_pt;
            self.pre_exist = if is_on_boundary {
                Spt::SPntNone
            } else {
                last_type
            };
        }
        Ok(())
    }
}

/// The inputs of the seam point between the points `k - 1` and `k`.
struct IsoInput<'a> {
    ssline: &'a IntSurfLineOn2S,
    ctx: &'a DecomposeCtx<'a>,
    ref_pt: &'a IntSurfPntOn2S,
    k: i32,
}

impl IsoInput<'_> {
    /// The point on the U seam (`is_u`, the unknown is the quadric V) or the V seam (the
    /// unknown is the quadric U), started from the middle of the two points.
    fn add(&self, is_u: bool, new_point: &mut IntSurfPntOn2S) -> Result<(), Failure> {
        let (p, q, arc_tol) = (self.ctx.p_surf, self.ctx.q_surf, self.ctx.arc_tol);
        let (a, b, c, d) = self.ref_pt.parameters();
        // The parametric point and the quadric point.
        let ((u0, v0), (uq_ref, vq_ref)) = if self.ctx.is_reversed {
            ((a, b), (c, d))
        } else {
            ((c, d), (a, b))
        };
        let (a, b, c, d) = self.ssline.value(self.k)?.parameters();
        let ((up, vp), (uq, vq)) = if self.ctx.is_reversed {
            ((a, b), (c, d))
        } else {
            ((c, d), (a, b))
        };
        let mut tol = Vector::with_bounds(1, 3)?;
        let mut start = Vector::with_bounds(1, 3)?;
        let mut inf = Vector::with_bounds(1, 3)?;
        let mut sup = Vector::with_bounds(1, 3)?;
        tol[1] = p.u_resolution(arc_tol)?;
        tol[2] = p.v_resolution(arc_tol)?;
        tol[3] = if is_u {
            q.v_resolution(arc_tol)?
        } else {
            q.u_resolution(arc_tol)?
        };
        start[1] = 0.5 * (u0 + up);
        start[2] = 0.5 * (v0 + vp);
        start[3] = if is_u {
            0.5 * (vq_ref + vq)
        } else {
            0.5 * (uq_ref + uq)
        };
        inf[1] = p.first_u_parameter()?;
        inf[2] = p.first_v_parameter()?;
        inf[3] = if is_u {
            q.first_v_parameter()?
        } else {
            q.first_u_parameter()?
        };
        sup[1] = p.last_u_parameter()?;
        sup[2] = p.last_v_parameter()?;
        sup[3] = if is_u {
            q.last_v_parameter()?
        } else {
            q.last_u_parameter()?
        };
        IntPatchSpecialPoints::add_point_on_u_or_v_iso(
            q,
            p,
            self.ref_pt,
            (is_u, 0.0),
            [&tol, &start, &inf, &sup],
            new_point,
            self.ctx.is_reversed,
        )?;
        Ok(())
    }
}
