// Rust translation of the line creation of DecomposeResult of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One part of a decomposed line becomes a walking line (split at its inner vertices)
//! or a restriction line on the trimmed arc.
use super::super::super::IntPatchIType;
use super::super::super::line::IntPatchLine;
use super::super::super::point::IntPatchPoint;
use super::super::super::point_line::IntPatchPointLine;
use super::super::super::rline::IntPatchRLine;
use super::super::super::wline::{IntPatchWLType, IntPatchWLine};
use super::DecomposeCtx;
use super::smooth::{end_vertex, split_on_segments, to_smooth};
use super::vertices::{add_vertices, has_internals, put_int_vertices, verify_vertices};
use crate::int_start::TheArc;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Point2};

/// Where the parts of one line go.
pub(super) struct PartSink<'a, 'c> {
    pub(super) line: &'a Handle<dyn IntPatchPointLine>,
    pub(super) ctx: &'a DecomposeCtx<'c>,
    pub(super) vline: &'a IntSurfLineOn2S,
    pub(super) lines: &'a mut Vec<Handle<dyn IntPatchLine>>,
}

impl PartSink<'_, '_> {
    /// The part `sline` (end vertices added near the ends, end points smoothed near a
    /// pole or apex) becomes a line. Returns true when the restriction line is kept
    /// unchanged (`DecomposeResult` returns then).
    pub(super) fn emit(
        &mut self,
        sline: &Handle<IntSurfLineOn2S>,
        is_decomposited: bool,
        has_been_decomposed: bool,
    ) -> Result<bool, Failure> {
        let ctx = self.ctx;
        let mut vf = IntSurfPntOn2S::new();
        let mut vl = IntSurfPntOn2S::new();
        let tol_2ds: f64 = precision::p_confusion();
        let (add_vf, add_vl) = verify_vertices(
            sline,
            ctx.is_reversed,
            self.vline,
            tol_2ds,
            ctx.arc_tol,
            ctx.pdomain,
            &mut vf,
            &mut vl,
        )?;
        let is_internals: bool = has_internals(sline, self.vline)?;
        let (mut d3f, mut d3l) = (0.0, 0.0);
        self.smooth_ends(sline, &mut d3f, &mut d3l)?;
        if (add_vf || add_vl) && add_vertices(sline, (&vf, add_vf), (&vl, add_vl), (d3f, d3l))? {
            self.smooth_ends(sline, &mut d3f, &mut d3l)?;
        }
        if self.line.arc_type() == IntPatchIType::Walking {
            self.walking_part(sline, is_internals)?;
            return Ok(false);
        }
        if !is_decomposited && !has_been_decomposed {
            // The line has not been changed.
            let rl: Handle<IntPatchRLine> = self.line.cast();
            self.lines.push(rl.cast());
            return Ok(true);
        }
        self.restriction_part(sline, is_internals)?;
        Ok(false)
    }

    fn smooth_ends(
        &self,
        sline: &IntSurfLineOn2S,
        d3f: &mut f64,
        d3l: &mut f64,
    ) -> Result<(), Failure> {
        to_smooth(sline, self.ctx.is_reversed, self.ctx.quad, true, d3f)?;
        to_smooth(sline, self.ctx.is_reversed, self.ctx.quad, false, d3l)
    }

    /// A walking line with end vertices (and the inner ones), split at its vertices
    /// when it has more than three.
    fn walking_part(
        &mut self,
        sline: &Handle<IntSurfLineOn2S>,
        is_internals: bool,
    ) -> Result<(), Failure> {
        let arc_tol: f64 = self.ctx.arc_tol;
        let (t1, t2) = (self.line.transition_on_s1(), self.line.transition_on_s2());
        let wline = Handle::new(IntPatchWLine::new(sline, false, t1, t2));
        wline.set_creating_way_info(IntPatchWLType::ImpPrm);
        wline.add_vertex(&end_vertex(sline, 1, arc_tol, 1.0)?, false);
        wline.set_first_point(1);
        if is_internals {
            put_int_vertices(
                &*wline,
                None,
                sline,
                self.ctx.is_reversed,
                (self.vline, arc_tol),
            )?;
        }
        let n: i32 = sline.nb_points();
        wline.add_vertex(&end_vertex(sline, n, arc_tol, f64::from(n))?, false);
        wline.set_last_point(wline.nb_vertex());
        let mut segm: Vec<Handle<dyn IntPatchLine>> = Vec::new();
        if !split_on_segments(&wline, (false, t1, t2), arc_tol, &mut segm)? {
            self.lines.push(wline.cast());
            return Ok(());
        }
        self.lines.extend(segm);
        Ok(())
    }

    /// A copy of the restriction line on the part, its arc range cut to the part ends.
    fn restriction_part(
        &mut self,
        sline: &Handle<IntSurfLineOn2S>,
        is_internals: bool,
    ) -> Result<(), Failure> {
        let (arc_tol, is_reversed) = (self.ctx.arc_tol, self.ctx.is_reversed);
        let src: Handle<IntPatchRLine> = self.line.cast();
        let Some(src) = src.get() else {
            return Err(super::super::super::failures::null_object(
                "IntPatch_ImpPrmIntersection::Perform - DecomposeResult",
                "IntPatch_RLine",
            ));
        };
        let rline = Handle::new(src.duplicate());
        rline.clear_vertexes();
        rline.set_curve(sline);
        if is_internals {
            put_int_vertices(
                &*rline,
                Some(&*rline),
                sline,
                is_reversed,
                (self.vline, arc_tol),
            )?;
        }
        let arc: TheArc = if rline.is_arc_on_s1() {
            rline.arc_on_s1()
        } else {
            rline.arc_on_s2()
        };
        let (mut fpar, mut lpar) = (arc.first_parameter()?, arc.last_parameter()?);
        let ends: [IntSurfPntOn2S; 2] = [sline.value(1)?, sline.value(sline.nb_points())?];
        let lin: Line2 = arc.line()?;
        let mut tpnt = IntPatchPoint::new();
        for (fl_index, end) in ends.iter().enumerate() {
            let (u1, v1, u2, v2) = end.parameters();
            let psurf: Point2 = if is_reversed {
                Point2::new(u1, v1)
            } else {
                Point2::new(u2, v2)
            };
            let mut par: f64 = elclib::parameter(&lin, psurf)?;
            if fl_index == 0 {
                fpar = std_max(fpar, par);
                par = fpar;
            } else {
                lpar = std_min(lpar, par);
                par = lpar;
            }
            tpnt.set_parameter(par);
            tpnt.set_value_tangent(end.value(), arc_tol, false);
            tpnt.set_parameters(u1, v1, u2, v2);
            rline.add_vertex(&tpnt, false);
        }
        if lpar - fpar > precision::p_confusion() {
            rline.set_first_point(1);
            rline.set_last_point(rline.nb_vertex());
            // The trimmed arc is not kept (the native result is discarded too).
            arc.trim(fpar, lpar, arc_tol)?;
            self.lines.push(rline.cast());
        }
        Ok(())
    }
}
