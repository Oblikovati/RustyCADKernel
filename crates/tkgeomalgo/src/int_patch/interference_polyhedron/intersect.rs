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
// Rust translation of IntPatch_InterferencePolyhedron::Intersect (the singularities and the
// crossings; IntPatch_InterferencePolyhedron.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_InterferencePolyhedron::Intersect`: the intersection of a triangle of each
//! polyhedron. The section points come from the vertex/vertex, vertex/edge,
//! edge/vertex, face/vertex, vertex/face and edge/edge singularities, then from the
//! edge/face and face/edge crossings.
use super::super::polyhedron::IntPatchPolyhedron;
use super::super::polyhedron_bvh::IntPatchPolyhedronTool as Tool;
use super::{IntPatchInterferencePolyhedron, POURCENT3};
use crate::intf::{PIType, SectionPoint, TangentZone, plane_equation};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Xyz};

/// `floatGap`: the precision of the section computation (instead of Epsilon(1000.)).
pub(super) const FLOAT_GAP: f64 = 1e-13; // tol:length

/// The section points of one couple and their bookkeeping (`piOT`, `nbpiOT`,
/// `nbpiO`, `nbpiT`, `parO`, `parT`, `edOT`, `edTT`) with the plane distances of the
/// vertices (`dfOpT`, `dpOfT`) and the two triangles.
pub(super) struct Section<'p> {
    pub(super) first: &'p IntPatchPolyhedron,
    pub(super) second: &'p IntPatchPolyhedron,
    pub(super) tri: [i32; 2],
    pub(super) po: [Xyz; 3],
    pub(super) pt: [Xyz; 3],
    pub(super) df_op_t: [f64; 3],
    pub(super) dp_of_t: [f64; 3],
    pub(super) pi_ot: Vec<SectionPoint>,
    pub(super) nbpi_ot: i32,
    pub(super) nbpi_o: i32,
    pub(super) nbpi_t: i32,
    pub(super) par_o: [f64; 3],
    pub(super) par_t: [f64; 3],
    pub(super) ed_ot: [i32; 3],
    pub(super) ed_tt: [i32; 3],
}

impl Section<'_> {
    /// Appends a section point counted on the object (`o`) and the tool (`t`).
    fn push(&mut self, point: SectionPoint, o: bool, t: bool) {
        self.pi_ot.push(point);
        self.nbpi_ot += 1;
        self.nbpi_o += i32::from(o);
        self.nbpi_t += i32::from(t);
    }
}

/// `a <= b`, false for a NaN operand (the native tests, whose negation must not
/// accept a NaN).
pub(super) fn lt_eq(a: f64, b: f64) -> bool {
    a <= b
}

/// `x < 0.`, false for a NaN.
pub(super) fn is_negative(x: f64) -> bool {
    x < 0.0
}

fn point_of(xyz: Xyz) -> Point3 {
    Point3::from(xyz)
}

impl IntPatchInterferencePolyhedron {
    /// `Intersect(Tri1, FirstPol, Tri2, SeconPol)`: the tangent zone of two coplanar
    /// triangles, or the section points and the section segment of two crossing ones.
    /// In a self interference, triangles sharing a vertex are skipped.
    pub(super) fn intersect(
        &mut self,
        tri1: i32,
        first: &IntPatchPolyhedron,
        tri2: i32,
        second: &IntPatchPolyhedron,
    ) -> Result<(), Failure> {
        self.oi = Tool::triangle(first, tri1);
        self.ti = Tool::triangle(second, tri2);
        if self.self_interference && self.oi.iter().any(|o| self.ti.contains(o)) {
            return Ok(());
        }
        let (po, pt) = (self.object_points(first)?, self.tool_points(second)?);
        let (o_nor, odp) = plane_equation(point_of(po[0]), point_of(po[1]), point_of(po[2]));
        let (t_nor, tdp) = plane_equation(point_of(pt[0]), point_of(pt[1]), point_of(pt[2]));
        // The scalar product of the unit normals: the cosine of the angle.
        self.incidence = (t_nor * o_nor).abs();
        let df_op_t: [f64; 3] = pt.map(|p| o_nor * p - odp);
        let dp_of_t: [f64; 3] = po.map(|p| t_nor * p - tdp);
        self.couple_characteristics(first, second)?;
        if self.is_tangent_couple(df_op_t, dp_of_t) {
            let mut zone = TangentZone::new();
            if self.tangent_zone_value(&mut zone, first, tri1, second, tri2)?
                && !self.insert_zone(&zone)?
            {
                self.tangent_zones.append(zone);
            }
            return Ok(());
        }
        let mut s = Section {
            first,
            second,
            tri: [tri1, tri2],
            po,
            pt,
            df_op_t,
            dp_of_t,
            pi_ot: Vec::new(),
            nbpi_ot: 0,
            nbpi_o: 0,
            nbpi_t: 0,
            par_o: [-1.0; 3],
            par_t: [-1.0; 3],
            ed_ot: [1; 3],
            ed_tt: [1; 3],
        };
        self.singularities(&mut s);
        self.edge_edge(&mut s);
        self.edge_face(&mut s);
        self.face_edge(&mut s);
        self.section_segment(&mut s, (o_nor, t_nor))
    }

    /// The three vertices of each triangle are in the plane of the other within the
    /// tolerance, and neither triangle lies on one side of the other's plane.
    fn is_tangent_couple(&self, df_op_t: [f64; 3], dp_of_t: [f64; 3]) -> bool {
        let tol: f64 = self.tolerance;
        let within = |d: [f64; 3]| d.iter().all(|x| x.abs() <= tol);
        let crossing =
            |d: [f64; 3]| (d[0] + d[1] + d[2]).abs() != d[0].abs() + d[1].abs() + d[2].abs();
        within(df_op_t) && within(dp_of_t) && crossing(df_op_t) && crossing(dp_of_t)
    }

    fn singularities(&self, s: &mut Section<'_>) {
        self.vertex_vertex(s);
        self.vertex_edge(s);
        self.edge_vertex(s);
        self.face_vertex(s);
        self.vertex_face(s);
    }

    /// Singularity VERTEX VERTEX.
    fn vertex_vertex(&self, s: &mut Section<'_>) {
        for i_obj in 0..3 {
            for i_too in 0..3 {
                if !lt_eq(self.dp_op_t[i_obj][i_too], FLOAT_GAP) {
                    continue;
                }
                let point = SectionPoint::with_data(
                    point_of(s.po[i_obj]),
                    PIType::Vertex,
                    self.oi[i_obj],
                    0,
                    0.0,
                    PIType::Vertex,
                    self.ti[i_too],
                    0,
                    0.0,
                    self.incidence,
                );
                s.push(point, true, true);
                (s.par_o[i_obj], s.par_t[i_too]) = (0.0, 0.0);
                (s.ed_ot[POURCENT3[i_obj + 2]], s.ed_ot[i_obj]) = (0, 0);
                (s.ed_tt[POURCENT3[i_too + 2]], s.ed_tt[i_too]) = (0, 0);
            }
        }
    }

    /// Singularity VERTEX EDGE: an object vertex on a tool edge.
    fn vertex_edge(&self, s: &mut Section<'_>) {
        for i_obj in 0..3 {
            if s.par_o[i_obj] != -1.0 {
                continue;
            }
            for i_too in 0..3 {
                let inext: usize = POURCENT3[i_too + 1];
                let d: f64 = self.dp_oe_t[i_obj][i_too];
                let (a, b) = (self.dp_op_t[i_obj][i_too], self.dp_op_t[i_obj][inext]);
                let on_edge: bool =
                    (-FLOAT_GAP..=FLOAT_GAP).contains(&d) && a + b < self.vtt[i_too].modulus();
                if s.ed_tt[i_too] != 1 || !on_edge {
                    continue;
                }
                s.par_t[i_too] = a / (a + b);
                if self.ti[i_too] > self.ti[inext] {
                    s.par_t[i_too] = 1.0 - s.par_t[i_too];
                }
                let (lo, hi) = (
                    self.ti[i_too].min(self.ti[inext]),
                    self.ti[i_too].max(self.ti[inext]),
                );
                let point = SectionPoint::with_data(
                    point_of(s.po[i_obj]),
                    PIType::Vertex,
                    self.oi[i_obj],
                    0,
                    0.0,
                    PIType::Edge,
                    lo,
                    hi,
                    s.par_t[i_too],
                    self.incidence,
                );
                s.push(point, true, true);
                s.par_o[i_obj] = 0.0;
                s.ed_ot[POURCENT3[i_obj + 2]] = 0;
                s.ed_ot[i_obj] = 0;
                s.ed_tt[i_too] = 0;
            }
        }
    }

    /// Singularity EDGE VERTEX: a tool vertex on an object edge.
    fn edge_vertex(&self, s: &mut Section<'_>) {
        for i_too in 0..3 {
            if s.par_t[i_too] != -1.0 {
                continue;
            }
            for i_obj in 0..3 {
                let inext: usize = POURCENT3[i_obj + 1];
                let (a, b) = (self.dp_op_t[i_obj][i_too], self.dp_op_t[inext][i_too]);
                let on_edge: bool = self.de_op_t[i_obj][i_too].abs() <= FLOAT_GAP
                    && a + b < self.voo[i_obj].modulus();
                if s.ed_ot[i_obj] != 1 || !on_edge {
                    continue;
                }
                s.par_o[i_obj] = a / (a + b);
                if self.oi[i_obj] > self.oi[inext] {
                    s.par_o[i_obj] = 1.0 - s.par_o[i_obj];
                }
                let (lo, hi) = (
                    self.oi[i_obj].min(self.oi[inext]),
                    self.oi[i_obj].max(self.oi[inext]),
                );
                let point = SectionPoint::with_data(
                    point_of(s.pt[i_too]),
                    PIType::Edge,
                    lo,
                    hi,
                    s.par_o[i_obj],
                    PIType::Vertex,
                    self.ti[i_too],
                    0,
                    0.0,
                    self.incidence,
                );
                s.push(point, true, true);
                s.par_t[i_too] = 0.0;
                (
                    s.ed_ot[i_obj],
                    s.ed_tt[POURCENT3[i_too + 2]],
                    s.ed_tt[i_too],
                ) = (0, 0, 0);
            }
        }
    }

    /// Singularity FACE VERTEX: a tool vertex in the object plane.
    fn face_vertex(&self, s: &mut Section<'_>) {
        for i_too in 0..3 {
            if s.par_t[i_too] == 0.0 || !lt_eq(s.df_op_t[i_too].abs(), FLOAT_GAP) {
                continue;
            }
            let point = SectionPoint::with_data(
                point_of(s.pt[i_too]),
                PIType::Face,
                s.tri[0],
                0,
                0.0,
                PIType::Vertex,
                self.ti[i_too],
                0,
                0.0,
                self.incidence,
            );
            s.push(point, false, true);
            s.par_t[i_too] = 0.0;
            (s.ed_tt[POURCENT3[i_too + 2]], s.ed_tt[i_too]) = (0, 0);
        }
    }

    /// Singularity VERTEX FACE: an object vertex in the tool plane.
    fn vertex_face(&self, s: &mut Section<'_>) {
        for i_obj in 0..3 {
            if s.par_o[i_obj] == 0.0 || !lt_eq(s.dp_of_t[i_obj].abs(), FLOAT_GAP) {
                continue;
            }
            let point = SectionPoint::with_data(
                point_of(s.po[i_obj]),
                PIType::Vertex,
                self.oi[i_obj],
                0,
                0.0,
                PIType::Face,
                s.tri[1],
                0,
                0.0,
                self.incidence,
            );
            s.push(point, true, false);
            s.par_o[i_obj] = 0.0;
            (s.ed_ot[POURCENT3[i_obj + 2]], s.ed_ot[i_obj]) = (0, 0);
        }
    }

    /// Intersection EDGE FACE: an object edge crossing the tool plane.
    fn edge_face(&self, s: &mut Section<'_>) {
        for i_obj in 0..3 {
            let inext: usize = POURCENT3[i_obj + 1];
            if s.ed_ot[i_obj] != 1 || !is_negative(s.dp_of_t[i_obj] * s.dp_of_t[inext]) {
                continue;
            }
            let lg: f64 = s.dp_of_t[i_obj] / (s.dp_of_t[i_obj] - s.dp_of_t[inext]);
            if !(lg > 0.0 && lg < 1.0) {
                continue;
            }
            s.par_o[i_obj] = lg;
            let pi_o: Xyz = s.po[i_obj] + self.voo[i_obj] * s.par_o[i_obj];
            if self.oi[i_obj] > self.oi[inext] {
                s.par_o[i_obj] = 1.0 - s.par_o[i_obj];
            }
            let (lo, hi) = (
                self.oi[i_obj].min(self.oi[inext]),
                self.oi[i_obj].max(self.oi[inext]),
            );
            let point = SectionPoint::with_data(
                point_of(pi_o),
                PIType::Edge,
                lo,
                hi,
                s.par_o[i_obj],
                PIType::Face,
                s.tri[1],
                0,
                0.0,
                self.incidence,
            );
            s.push(point, true, false);
        }
    }

    /// Intersection FACE EDGE: a tool edge crossing the object plane.
    fn face_edge(&self, s: &mut Section<'_>) {
        for i_too in 0..3 {
            let jnext: usize = POURCENT3[i_too + 1];
            if s.ed_tt[i_too] != 1 || !is_negative(s.df_op_t[i_too] * s.df_op_t[jnext]) {
                continue;
            }
            let lg: f64 = s.df_op_t[i_too] / (s.df_op_t[i_too] - s.df_op_t[jnext]);
            if !(lg > 0.0 && lg < 1.0) {
                continue;
            }
            s.par_t[i_too] = lg;
            let pi_o: Xyz = s.pt[i_too] + self.vtt[i_too] * s.par_t[i_too];
            if self.ti[i_too] > self.ti[jnext] {
                s.par_t[i_too] = 1.0 - s.par_t[i_too];
            }
            let (lo, hi) = (
                self.ti[i_too].min(self.ti[jnext]),
                self.ti[i_too].max(self.ti[jnext]),
            );
            let point = SectionPoint::with_data(
                point_of(pi_o),
                PIType::Face,
                s.tri[0],
                0,
                0.0,
                PIType::Edge,
                lo,
                hi,
                s.par_t[i_too],
                self.incidence,
            );
            s.push(point, false, true);
        }
    }
}
