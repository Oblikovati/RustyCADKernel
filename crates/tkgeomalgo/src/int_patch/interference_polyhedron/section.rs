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
// Rust translation of IntPatch_InterferencePolyhedron::Intersect (the edge/edge singularity
// and the section segment; IntPatch_InterferencePolyhedron.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge/edge singularity of two triangles and the section segment built from the
//! section points, sorted along the intersection of the planes.
use super::super::polyhedron_bvh::IntPatchPolyhedronTool as Tool;
use super::intersect::{FLOAT_GAP, Section, is_negative, lt_eq};
use super::{IntPatchInterferencePolyhedron, POURCENT3};
use crate::intf::{PIType, SectionPoint};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Xyz};

/// `div > floatGap || div < -floatGap`.
fn not_null(div: f64) -> bool {
    div > FLOAT_GAP || div < -FLOAT_GAP
}

/// The native persistent `piO` and `piT` of the edge/edge loop.
#[derive(Default)]
struct EdgeEdgePoints {
    pi_o: Xyz,
    pi_t: Xyz,
}

impl IntPatchInterferencePolyhedron {
    /// Singularity EDGE EDGE: an object edge and a tool edge, each crossing the other
    /// plane, that meet within `floatGap`.
    pub(super) fn edge_edge(&self, s: &mut Section<'_>) {
        let mut pts = EdgeEdgePoints::default();
        for i_obj in 0..3 {
            let inext: usize = POURCENT3[i_obj + 1];
            if s.ed_ot[i_obj] != 1 || !is_negative(s.dp_of_t[i_obj] * s.dp_of_t[inext]) {
                continue;
            }
            let lg: f64 = s.dp_of_t[i_obj] / (s.dp_of_t[i_obj] - s.dp_of_t[inext]);
            if lg > 0.0 && lg < 1.0 {
                for i_too in 0..3 {
                    self.edge_edge_pair(s, (i_obj, i_too), &mut pts);
                }
            }
        }
    }

    fn edge_edge_pair(
        &self,
        s: &mut Section<'_>,
        (i_obj, i_too): (usize, usize),
        pts: &mut EdgeEdgePoints,
    ) {
        let (inext, jnext) = (POURCENT3[i_obj + 1], POURCENT3[i_too + 1]);
        if s.ed_tt[i_too] != 1 || !is_negative(s.df_op_t[i_too] * s.df_op_t[jnext]) {
            return;
        }
        let lg: f64 = s.df_op_t[i_too] / (s.df_op_t[i_too] - s.df_op_t[jnext]);
        if !(lg > 0.0 && lg < 1.0) {
            return;
        }
        let pb_o: bool = !self.edge_edge_object(s, (i_obj, inext, i_too), pts);
        let pb_t: bool = !self.edge_edge_tool(s, (i_obj, i_too, jnext), pts);
        if pb_o || pb_t {
            return;
        }
        pts.pi_t -= pts.pi_o;
        if !lt_eq(pts.pi_t.modulus(), FLOAT_GAP) {
            return;
        }
        let (o_lo, o_hi) = (
            self.oi[i_obj].min(self.oi[inext]),
            self.oi[i_obj].max(self.oi[inext]),
        );
        let (t_lo, t_hi) = (
            self.ti[i_too].min(self.ti[jnext]),
            self.ti[i_too].max(self.ti[jnext]),
        );
        let point = SectionPoint::with_data(
            Point3::from(pts.pi_o),
            PIType::Edge,
            o_lo,
            o_hi,
            s.par_o[i_obj],
            PIType::Edge,
            t_lo,
            t_hi,
            s.par_t[i_too],
            self.incidence,
        );
        s.pi_ot.push(point);
        (s.ed_ot[i_obj], s.ed_tt[i_too]) = (0, 0);
        s.nbpi_ot += 1;
        s.nbpi_o += 1;
        s.nbpi_t += 1;
    }

    /// The point on the object edge (from its lower end); false for a null divisor.
    fn edge_edge_object(
        &self,
        s: &mut Section<'_>,
        (i_obj, inext, i_too): (usize, usize, usize),
        pts: &mut EdgeEdgePoints,
    ) -> bool {
        let (a, b) = (self.dp_oe_t[i_obj][i_too], self.dp_oe_t[inext][i_too]);
        if self.oi[i_obj] > self.oi[inext] {
            let div: f64 = b - a;
            if !not_null(div) {
                return false;
            }
            s.par_o[i_obj] = b / div;
            pts.pi_o = s.po[inext] + self.voo[i_obj].reversed() * s.par_o[i_obj];
            return true;
        }
        let div: f64 = a - b;
        if !not_null(div) {
            return false;
        }
        s.par_o[i_obj] = a / (a - b);
        pts.pi_o = s.po[i_obj] + self.voo[i_obj] * s.par_o[i_obj];
        true
    }

    /// The point on the tool edge (from its lower end); false for a null divisor.
    fn edge_edge_tool(
        &self,
        s: &mut Section<'_>,
        (i_obj, i_too, jnext): (usize, usize, usize),
        pts: &mut EdgeEdgePoints,
    ) -> bool {
        let (a, b) = (self.de_op_t[i_obj][i_too], self.de_op_t[i_obj][jnext]);
        if self.ti[i_too] > self.ti[jnext] {
            let div: f64 = b - a;
            if !not_null(div) {
                return false;
            }
            s.par_t[i_too] = b / div;
            pts.pi_t = s.pt[jnext] + self.vtt[i_too].reversed() * s.par_t[i_too];
            return true;
        }
        let div: f64 = a - b;
        if !not_null(div) {
            return false;
        }
        s.par_t[i_too] = a / div;
        pts.pi_t = s.pt[i_too] + self.vtt[i_too] * s.par_t[i_too];
        true
    }

    /// The section points sorted along `ONor ^ TNor` (insertion sort of their
    /// indices), then the section segment of a couple with two points on each triangle.
    pub(super) fn section_segment(
        &mut self,
        s: &mut Section<'_>,
        (o_nor, t_nor): (Xyz, Xyz),
    ) -> Result<(), Failure> {
        let id: Vec<usize> = sorted_indices(&s.pi_ot, s.nbpi_ot, o_nor ^ t_nor);
        if s.nbpi_o != 2 || s.nbpi_t != 2 {
            return Ok(());
        }
        let (mut ideb, mut ifin): (i32, i32) = (-1, -2);
        // An edge of one triangle in the plane of the other may have been processed on
        // a connected triangle already.
        // Both triangles are checked, as natively (no short circuit).
        let found_on_object: bool = self.already_found(s.first, s.tri[0], self.oi, s.par_o)?;
        let found_on_tool: bool = self.already_found(s.second, s.tri[1], self.ti, s.par_t)?;
        if found_on_object || found_on_tool {
            s.nbpi_ot = 0;
        }
        if s.nbpi_ot > 0 {
            (ideb, ifin) = segment_ends(&s.pi_ot, &id, s.nbpi_ot);
        }
        self.insert_segment(&s.pi_ot, ideb, ifin)
    }

    /// The pivot and edge vertices at parameter 0 of one triangle: whether the
    /// connected triangle across that edge (`TriConnex`) comes after `tri`.
    fn already_found(
        &self,
        polyhedron: &super::super::polyhedron::IntPatchPolyhedron,
        tri: i32,
        indices: [i32; 3],
        par: [f64; 3],
    ) -> Result<bool, Failure> {
        let (pivo, pedg): (i32, i32) = if par[0] == 0.0 {
            (
                0,
                if par[1] == 0.0 {
                    1
                } else if par[2] == 0.0 {
                    2
                } else {
                    -1
                },
            )
        } else if par[1] == 0.0 {
            (1, if par[2] == 0.0 { 2 } else { -1 })
        } else {
            (-1, -1)
        };
        if pivo < 0 || pedg < 0 {
            return Ok(false);
        }
        let (_, tri_con, _) = Tool::tri_connex(
            polyhedron,
            tri,
            indices[pivo as usize],
            indices[pedg as usize],
        )?;
        Ok(tri_con > tri)
    }

    /// `Insert` of the segment `ideb ifin` (negative values: the points `-ideb` and
    /// `-ifin` when they are closer than the tolerance; -999: none).
    fn insert_segment(
        &mut self,
        pi_ot: &[SectionPoint],
        ideb: i32,
        ifin: i32,
    ) -> Result<(), Failure> {
        if ideb < 0 {
            if ifin < 0 && ideb != -999 {
                let (a, b) = (pi_ot[(-ideb - 1) as usize], pi_ot[(-ifin - 1) as usize]);
                if a.pnt().distance(&b.pnt()) < self.tolerance {
                    self.insert_points(&a, &b)?;
                }
            }
            return Ok(());
        }
        if ideb != ifin {
            let (a, b) = (pi_ot[ideb as usize], pi_ot[ifin as usize]);
            self.insert_points(&a, &b)?;
        }
        Ok(())
    }
}

/// The indices of the section points sorted by their abscissa along `dir` (identity
/// when there are fewer than two points).
fn sorted_indices(pi_ot: &[SectionPoint], nbpi_ot: i32, dir: Xyz) -> Vec<usize> {
    let n: usize = nbpi_ot.max(0) as usize;
    let mut id: Vec<usize> = (0..n.max(4)).collect();
    if n <= 1 {
        return id;
    }
    let d: Vec<f64> = pi_ot[..n].iter().map(|p| dir * p.pnt().xyz()).collect();
    for i_pi in 1..n {
        id[i_pi] = i_pi;
        let mut i_ps: usize = i_pi;
        while i_ps > 0 && d[id[i_ps - 1]] > d[id[i_ps]] {
            id.swap(i_ps - 1, i_ps);
            i_ps -= 1;
        }
    }
    id
}

/// The ends of the section segment among the sorted points: the first two points,
/// or the second and third when the first is only inside a face.
fn segment_ends(pi_ot: &[SectionPoint], id: &[usize], nbpi_ot: i32) -> (i32, i32) {
    let (p0, p1) = (pi_ot[id[0]], pi_ot[id[1]]);
    let (i0, i1) = (id[0] as i32, id[1] as i32);
    let face = |on_first: bool| {
        let (t0, t1, t1_other) = if on_first {
            (p0.type_on_first(), p1.type_on_first(), p1.type_on_second())
        } else {
            (p0.type_on_second(), p1.type_on_second(), p1.type_on_first())
        };
        if t0 != PIType::Face {
            return None;
        }
        Some(if t1 == PIType::Face {
            (-i0 - 1, -i1 - 1)
        } else if t1_other != PIType::Face {
            (i1, i1)
        } else if nbpi_ot >= 3 {
            (i1, id[2] as i32)
        } else {
            (-999, -999)
        })
    };
    face(true).or_else(|| face(false)).unwrap_or((i0, i1))
}
