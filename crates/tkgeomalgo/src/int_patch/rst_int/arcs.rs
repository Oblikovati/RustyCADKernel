// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of the loop over the arcs of IntPatch_RstInt::PutVertexOnLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_RstInt.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One arc of the domain against the line: the polygons, their 2d intersection over the
//! shifted copies of the arc, and the refinement of each crossing.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::super::cs_function::IntPatchCSFunction;
use super::super::h_inter_tool::IntPatchHInterTool;
use super::super::instances::IntPatchCurvIntSurf;
use super::super::polyarc::IntPatchPolyArc;
use super::super::polygo::IntPatchPolygo;
use super::super::polyline::IntPatchPolyLine;
use super::helpers::{
    FindInput, PointLineRef, find_parameter, get_line_point_2d, recadre, tol3d_arc,
};
use super::{RstInput, RstState};
use crate::int_start::TheArc;
use crate::intf::{InterferencePolygon2d, Polygon2d};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elclib::in_period;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::PI;

/// The bound used for an infinite arc bound (as in Adaptor3d_TopolTool).
const MY_INFINITE: f64 = 1.0e15;

/// An arc of the domain and what the loop derives from it.
pub(super) struct ArcRun<'a> {
    pub(super) arc: &'a TheArc,
    pub(super) edge_tol: f64,
    pub(super) brise: IntPatchPolyArc,
}

impl RstInput<'_> {
    /// The ends of the arc (infinite ones at 1e15) and its end points at its vertices
    /// (computed and unused natively).
    fn arc_bounds(&self, arc: &TheArc) -> Result<(f64, f64), Failure> {
        let (mut pfirst, mut plast) = IntPatchHInterTool::bounds(arc)?;
        if precision::is_negative_infinite(pfirst) {
            pfirst = -MY_INFINITE;
        }
        if precision::is_positive_infinite(plast) {
            plast = MY_INFINITE;
        }
        self.domain.initialize_arc(arc)?;
        self.domain.init_vertex_iterator();
        while self.domain.more_vertex() {
            let vtx = self.domain.vertex()?;
            let prm: f64 = IntPatchHInterTool::parameter(&vtx, arc)?;
            if (prm - pfirst).abs() < precision::p_confusion() {
                arc.d0(pfirst)?;
            } else if (prm - plast).abs() < precision::p_confusion() {
                arc.d0(plast)?;
            }
            self.domain.next_vertex();
        }
        Ok((pfirst, plast))
    }

    /// The number of samples of the arc polygon and the offsets that bring a line arc
    /// (an axis-aligned restriction) into the period of the line polygon box.
    fn samples(
        &self,
        arc: &TheArc,
        bplin: &BoundingBox2,
        (pfirst, plast): (f64, f64),
    ) -> Result<(i32, f64, f64), Failure> {
        let clamp = |n: i32| n.clamp(10, 50);
        match arc.get_type()? {
            CurveType::Line => {
                let [xmin, ymin, _, _] = bplin.get()?;
                let lin = arc.line()?;
                let (loc, dir) = (lin.location(), lin.direction());
                // Here, rectangular axis-aligned domains only.
                let is_along_u: bool = dir.x().abs() > dir.y().abs();
                let (mut ou, mut ov) = (0.0, 0.0);
                if self.periodic && !is_along_u {
                    ou = in_period(loc.x(), xmin, xmin + PI + PI) - loc.x();
                } else if self.bi_periodic && is_along_u {
                    ov = in_period(loc.y(), ymin, ymin + PI + PI) - loc.y();
                }
                Ok((10, ou, ov))
            }
            CurveType::BezierCurve => Ok((clamp(3 + arc.nb_poles()?), 0.0, 0.0)),
            CurveType::BSplineCurve => {
                let nbs: f64 = f64::from(arc.nb_knots()? * arc.degree()?) * (plast - pfirst)
                    / (arc.last_parameter()? - arc.first_parameter()?);
                Ok((clamp(if nbs < 2.0 { 2 } else { nbs as i32 }), 0.0, 0.0))
            }
            _ => Ok((25, 0.0, 0.0)),
        }
    }

    /// The line polygon box widened by a period in U (and V) on a periodic surface.
    fn widened_box(&self, mut bplin: BoundingBox2) -> Result<BoundingBox2, Failure> {
        if self.periodic {
            let [xmin, ymin, xmax, ymax] = bplin.get()?;
            let g: f64 = bplin.gap();
            bplin.set_void();
            bplin.update_bounds([xmin - PI - PI, ymin], [xmax + PI + PI, ymax]);
            bplin.set_gap(g);
        }
        if self.bi_periodic {
            let [xmin, ymin, xmax, ymax] = bplin.get()?;
            let g: f64 = bplin.gap();
            bplin.set_void();
            bplin.update_bounds([xmin, ymin - PI - PI], [xmax, ymax + PI + PI]);
            bplin.set_gap(g);
        }
        Ok(bplin)
    }

    /// One arc of the domain: its polygon is intersected with the line one, shifted by
    /// -2PI, 0 and +2PI in U (and V) on a periodic surface.
    pub(super) fn process_arc(
        &self,
        st: &mut RstState,
        plin: &IntPatchPolyLine,
        arc: &TheArc,
    ) -> Result<(), Failure> {
        // MSV Oct 15, 2001: the tolerance of this edge if possible.
        let edge_tol: f64 = tol3d_arc(arc, self.domain, self.tol)?;
        let bounds = self.arc_bounds(arc)?;
        let bplin: BoundingBox2 = plin.bounding().clone();
        let (nb_echant, ref_ou, ref_ov) = self.samples(arc, &bplin, bounds)?;
        let bplin = self.widened_box(bplin)?;
        let brise = IntPatchPolyArc::new(arc, nb_echant, bounds.0, bounds.1, &bplin)?;
        let mut run = ArcRun {
            arc,
            edge_tol,
            brise,
        };
        // The periodic counter is not reset for the shifts in V (a torus runs the three
        // U shifts with the first V shift only), as natively.
        let (mut ibi, mut ip) = (0, 0);
        let (mut ou, mut ov) = (ref_ou, ref_ov);
        loop {
            if ibi == 1 {
                ov = ref_ov - PI - PI;
            } else if ibi == 2 {
                ov = ref_ov + PI + PI;
            }
            loop {
                if ip == 1 {
                    ou = ref_ou - PI - PI;
                } else if ip == 2 {
                    ou = ref_ou + PI + PI;
                }
                run.brise.set_offset(ou, ov)?;
                self.section_points(st, plin, &run)?;
                ip += 1;
                if !(self.periodic && ip <= 2) {
                    break;
                }
            }
            ibi += 1;
            if !(self.bi_periodic && ibi <= 2) {
                break;
            }
        }
        Ok(())
    }

    /// The crossings of the line and arc polygons (only the section points: the
    /// tangent zones need no vertex, #29494).
    fn section_points(
        &self,
        st: &mut RstState,
        plin: &IntPatchPolyLine,
        run: &ArcRun<'_>,
    ) -> Result<(), Failure> {
        let mut commun = InterferencePolygon2d::new();
        commun.perform(plin as &dyn Polygon2d, &run.brise as &dyn Polygon2d)?;
        st.locpt.clear();
        st.locpt2.clear();
        for i in 1..=commun.nb_section_points() {
            let sp = commun.pnt_value(i)?;
            let (aw1, aw2) = (sp.param_on_first(), sp.param_on_second());
            self.section_point(st, run, (aw1, aw2))?;
        }
        Ok(())
    }

    /// The arc parameter of the polygon parameter `aw2`.
    fn arc_parameter(brise: &IntPatchPolyArc, aw2: f64) -> Result<f64, Failure> {
        let mut par: f64 = aw2.trunc();
        let mut irang: i32 = par as i32 + 1;
        if irang == brise.nb_points()? {
            irang -= 1;
            par = 1.0;
        } else {
            par = (aw2 - par).abs();
        }
        Ok((1.0 - par) * brise.parameter(irang)? + par * brise.parameter(irang + 1)?)
    }

    /// One crossing: the approximate 2d point is refined by the curve-surface
    /// intersection, then put on the line unless a point near it was already found.
    fn section_point(
        &self,
        st: &mut RstState,
        run: &ArcRun<'_>,
        (aw1, aw2): (f64, f64),
    ) -> Result<(), Failure> {
        (st.u, st.v) = get_line_point_2d(&self.line, aw1 + 1.0, !self.on_first)?;
        st.w = Self::arc_parameter(&run.brise, aw2)?;
        if !self.refine(st, run)? {
            return Ok(());
        }
        if self.is_duplicate(st, run.edge_tol) {
            return Ok(());
        }
        let param_approx: i32 = aw1 as i32 + 1;
        let (p2d, d2d) = run.arc.d1(st.paramarc)?;
        (st.u1, st.v1, st.d2d) = (p2d.x(), p2d.y(), d2d);
        if let (PointLineRef::Walking(w), true) = (&self.line, self.periodic) {
            let p = if self.on_first {
                [st.u1, st.v1, st.u2, st.v2]
            } else {
                [st.u2, st.v2, st.u1, st.v1]
            };
            let [a, b, c, d] = recadre(self.types, w, param_approx, p)?;
            if self.on_first {
                (st.u1, st.v1, st.u2, st.v2) = (a, b, c, d);
            } else {
                (st.u2, st.v2, st.u1, st.v1) = (a, b, c, d);
            }
        }
        st.locpt.push(st.ptsommet);
        st.locpt2.push(Point2::new(st.u2, st.v2));
        if !self.found_on_line(st, run.edge_tol, param_approx)? {
            return Ok(());
        }
        self.put_vertex(st, run)
    }

    /// IntPatch_CurvIntSurf from the approximate point (the UV bounds extended not to
    /// miss a solution near the boundary); the surface parameters are kept when the
    /// refined point is within the tolerance of the first one.
    fn refine(&self, st: &mut RstState, run: &ArcRun<'_>) -> Result<bool, Failure> {
        let thefunc = IntPatchCSFunction::new(self.other_surf, run.arc, self.surf);
        let marg_coef: f64 = 0.004;
        let int_cs =
            IntPatchCurvIntSurf::new([st.u, st.v, st.w], &thefunc, run.edge_tol, marg_coef)?;
        if !int_cs.is_done() || int_cs.is_empty()? {
            return Ok(false);
        }
        st.ptsommet = int_cs.point()?;
        (st.u2, st.v2) = int_cs.parameter_on_surface()?;
        let old_pnt: Point3 = self.other_surf.d0(st.u, st.v)?;
        let new_pnt: Point3 = self.other_surf.d0(st.u2, st.v2)?;
        let tol_conf: f64 = std_max(precision::confusion(), run.edge_tol);
        if old_pnt.square_distance(&new_pnt) < tol_conf * tol_conf {
            (st.u2, st.v2) = (st.u, st.v);
        }
        st.paramarc = int_cs.parameter_on_curve()?;
        Ok(true)
    }

    /// Whether the point was already found on this arc (on a closed other surface, the
    /// 2d parameters must be near too).
    fn is_duplicate(&self, st: &mut RstState, edge_tol: f64) -> bool {
        for j in 0..st.locpt.len() {
            if !(st.ptsommet.distance(&st.locpt[j]) <= edge_tol) {
                continue;
            }
            if self.possibly_closed() {
                (st.u, st.v) = (st.locpt2[j].x(), st.locpt2[j].y());
                if (self.closed[2] && (st.u - st.u2).abs() > self.tol_closed[2])
                    || (self.closed[3] && (st.v - st.v2).abs() > self.tol_closed[3])
                {
                    continue;
                }
            }
            return true;
        }
        false
    }

    /// The parameter of the point on the line and the tangent there; on a closed
    /// surface of a walking line, the 2d parameters of the line point must match.
    fn found_on_line(
        &self,
        st: &mut RstState,
        edge_tol: f64,
        param_approx: i32,
    ) -> Result<bool, Failure> {
        let input = FindInput {
            other_surf: self.other_surf,
            tol: edge_tol,
            ptsom: st.ptsommet,
            ptsom2d: Point2::new(st.u2, st.v2),
            param_approche: param_approx,
            on_first: self.on_first,
        };
        let mut found = find_parameter(&self.line, &input, &mut st.paramline, &mut st.tgline)?;
        if !(matches!(self.line, PointLineRef::Walking(_)) && found && self.possibly_closed()) {
            return Ok(found);
        }
        if self.closed[0] || self.closed[1] {
            (st.u, st.v) = get_line_point_2d(&self.line, st.paramline, self.on_first)?;
            if (self.closed[0] && (st.u - st.u1).abs() > self.tol_closed[0])
                || (self.closed[1] && (st.v - st.v1).abs() > self.tol_closed[1])
            {
                found = false;
            }
        }
        if found && (self.closed[2] || self.closed[3]) {
            (st.u, st.v) = get_line_point_2d(&self.line, st.paramline, !self.on_first)?;
            if (self.closed[2] && (st.u - st.u2).abs() > self.tol_closed[2])
                || (self.closed[3] && (st.v - st.v2).abs() > self.tol_closed[3])
            {
                found = false;
            }
        }
        Ok(found)
    }
}
