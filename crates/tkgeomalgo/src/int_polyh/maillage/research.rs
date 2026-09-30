// Created on: 1999-03-05
// Created by: Fabrice SERVANT
// Copyright (c) 1999-1999 Matra Datavision
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
// Rust translation of TestNbPoints, StartingPointsResearch and NextStartingPointsResearch
// of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection points of two triangles: each edge of one against the other, the
//! points merged as found; three or more mean coplanar triangles.
use super::edge_contact::{IntPolyhTriangleEdge, triangle_edge_contact};
use super::{IntPolyhMaillageAffinage, MY_CONFUSION_PRECISION};
use crate::int_polyh::point::IntPolyhPoint;
use crate::int_polyh::start_point::IntPolyhStartPoint;
use crate::int_polyh::triangle::IntPolyhTriangle;
use rustycad_tkernel::standard::Failure;

/// `TestNbPoints(TriSurfID, NbPoints, NbPointsTotal, Pt1, Pt2, SP1, SP2)`: merges the
/// `nb_points` new points into the `total` points found so far (3 for more than two).
fn test_nb_points(
    nb_points: i32,
    total: &mut i32,
    (pt1, pt2): (&IntPolyhStartPoint, &IntPolyhStartPoint),
    sp1: &mut IntPolyhStartPoint,
    sp2: &mut IntPolyhStartPoint,
) {
    match (nb_points, *total) {
        (1, 0) => {
            *sp1 = *pt1;
            *total = 1;
        }
        (1, 1) => {
            if pt1.check_same_sp(sp1) != 1 {
                *sp2 = *pt1;
                *total = 2;
            }
        }
        (1, 2) => {
            let known: bool = sp1.check_same_sp(pt1) != 0 || sp2.check_same_sp(pt1) != 0;
            *total = if known { 2 } else { 3 };
        }
        (2, 0) => {
            (*sp1, *sp2) = (*pt1, *pt2);
            *total = 2;
        }
        (2, 1) => {
            // Here Pt1 != Pt2.
            if sp1.check_same_sp(pt1) != 0 {
                *sp2 = *pt2;
                *total = 2;
            } else if sp1.check_same_sp(pt2) != 0 {
                *sp2 = *pt1;
                *total = 2;
            } else {
                *total = 3;
            }
        }
        (2, 2) => {
            // Here SP1 != SP2.
            let first: bool = sp1.check_same_sp(pt1) != 0 || sp1.check_same_sp(pt2) != 0;
            let second: bool =
                first && (sp2.check_same_sp(pt1) != 0 || sp2.check_same_sp(pt2) != 0);
            *total = if second { 2 } else { 3 };
        }
        _ => {}
    }
}

/// The two triangles of a couple and their geometry: the points, the sides and the
/// normals (`nn1`, `mm1`) with their moduli.
struct CouplePair<'a> {
    tri1: &'a IntPolyhTriangle,
    tri2: &'a IntPolyhTriangle,
    p: [IntPolyhPoint; 3],
    q: [IntPolyhPoint; 3],
    e: [IntPolyhPoint; 3],
    f: [IntPolyhPoint; 3],
    nn1: IntPolyhPoint,
    mm1: IntPolyhPoint,
}

impl IntPolyhMaillageAffinage {
    fn couple_pair(&self, t1: i32, t2: i32) -> CouplePair<'_> {
        let tri1 = &self.sides[0].triangles[t1];
        let tri2 = &self.sides[1].triangles[t2];
        let p = tri1.corners(&self.sides[0].points);
        let q = tri2.corners(&self.sides[1].points);
        let e = [p[1] - p[0], p[2] - p[1], p[0] - p[2]];
        let f = [q[1] - q[0], q[2] - q[1], q[0] - q[2]];
        let (mut nn1, mut mm1) = (IntPolyhPoint::default(), IntPolyhPoint::default());
        nn1.cross(&e[0], &e[1]);
        mm1.cross(&f[0], &f[1]);
        CouplePair {
            tri1,
            tri2,
            p,
            q,
            e,
            f,
            nn1,
            mm1,
        }
    }

    /// `StartingPointsResearch(T1, T2, SP1, SP2)`: the intersection points of the
    /// triangles `t1` and `t2` (0, 1, 2, or 3 and more for coplanar triangles); the
    /// points found carry the couple. `m.starting_points_research(0, 0, &mut sp1, &mut sp2)?`.
    pub fn starting_points_research(
        &self,
        t1: i32,
        t2: i32,
        sp1: &mut IntPolyhStartPoint,
        sp2: &mut IntPolyhStartPoint,
    ) -> Result<i32, Failure> {
        let pair: CouplePair<'_> = self.couple_pair(t1, t2);
        let total: i32 = research_points(&pair, [-3, -3], sp1, sp2)?;
        if total == 2 {
            sp1.set_couple_value(t1, t2);
            sp2.set_couple_value(t1, t2);
        } else if total == 1 || total == 3 {
            sp1.set_couple_value(t1, t2);
        }
        Ok(total)
    }

    /// `NextStartingPointsResearch(T1, T2, SPInit, SPNext)`: the intersection point of the
    /// triangles `t1` and `t2` other than `sp_init`, skipping the edges `sp_init` lies on;
    /// returns 1 with it in `sp_next`, 0, or more than 1 for coplanar triangles. `sp_next`
    /// always carries the couple. `m.next_starting_points_research(0, 1, &init, &mut next)?`.
    pub fn next_starting_points_research(
        &self,
        t1: i32,
        t2: i32,
        sp_init: &IntPolyhStartPoint,
        sp_next: &mut IntPolyhStartPoint,
    ) -> Result<i32, Failure> {
        let mut total: i32 = 0;
        if t1 >= 0 && t2 >= 0 {
            let pair: CouplePair<'_> = self.couple_pair(t1, t2);
            let (mut sp1, mut sp2) = (IntPolyhStartPoint::default(), IntPolyhStartPoint::default());
            total = research_points(&pair, [sp_init.e1(), sp_init.e2()], &mut sp1, &mut sp2)?;
            if total == 1 {
                if sp1.check_same_sp(sp_init) != 0 {
                    total = 0;
                } else {
                    *sp_next = sp1;
                }
            } else if total == 2 && sp1.check_same_sp(sp_init) != 0 {
                // SP1 and SPInit are the same point.
                total = 1;
                *sp_next = sp2;
            } else if total == 2 && sp2.check_same_sp(sp_init) != 0 {
                // SP2 and SPInit are the same point.
                total = 1;
                *sp_next = sp1;
            }
        }
        sp_next.set_couple_value(t1, t2);
        Ok(total)
    }
}

/// The edges of each triangle against the other one while fewer than three points are
/// found; an edge equal to the skipped edge of its surface (`skip = [E1, E2]` of the
/// initial point) is not tried (-3 never equals an edge number).
fn research_points(
    pair: &CouplePair<'_>,
    [skip1, skip2]: [i32; 2],
    sp1: &mut IntPolyhStartPoint,
    sp2: &mut IntPolyhStartPoint,
) -> Result<i32, Failure> {
    let mut total: i32 = 0;
    let nn1_modulus: f64 = pair.nn1.square_modulus().sqrt();
    let mm1_modulus: f64 = pair.mm1.square_modulus().sqrt();
    // The edges of T2 against T1, unless the normal of T1 is null (`abs(nn1modulus) <
    // MyConfusionPrecision`, false for NaN).
    let null_n1: bool = nn1_modulus.abs() < MY_CONFUSION_PRECISION;
    if !null_n1 {
        let n1: IntPolyhPoint = pair.nn1.divide(nn1_modulus);
        let t2_edges = [
            pair.tri2.first_edge(),
            pair.tri2.second_edge(),
            pair.tri2.third_edge(),
        ];
        for (k, &edge) in t2_edges.iter().enumerate() {
            if total < 3 && skip2 != edge {
                let (q_a, q_b) = (&pair.q[k], &pair.q[(k + 1) % 3]);
                let args = triangle_args(
                    pair,
                    1,
                    k,
                    [&pair.p, &pair.e],
                    ([q_a, q_b], &pair.f[k]),
                    &n1,
                );
                edge_points(&args, &mut total, sp1, sp2)?;
            }
        }
    }
    // The edges of T1 against T2, unless the normal of T2 is null.
    let null_m1: bool = mm1_modulus.abs() < MY_CONFUSION_PRECISION;
    if !null_m1 {
        let m1: IntPolyhPoint = pair.mm1.divide(mm1_modulus);
        let t1_edges = [
            pair.tri1.first_edge(),
            pair.tri1.second_edge(),
            pair.tri1.third_edge(),
        ];
        for (k, &edge) in t1_edges.iter().enumerate() {
            if total < 3 && skip1 != edge {
                let (p_a, p_b) = (&pair.p[k], &pair.p[(k + 1) % 3]);
                let args = triangle_args(
                    pair,
                    2,
                    k,
                    [&pair.q, &pair.f],
                    ([p_a, p_b], &pair.e[k]),
                    &m1,
                );
                edge_points(&args, &mut total, sp1, sp2)?;
            }
        }
    }
    Ok(total)
}

/// The arguments of `TriangleEdgeContact` for the edge `k` of the other triangle against
/// the triangle of the surface `tri_surf_id`.
fn triangle_args<'a>(
    pair: &'a CouplePair<'a>,
    tri_surf_id: i32,
    k: usize,
    [pts, cotes]: [&'a [IntPolyhPoint; 3]; 2],
    (pe, edge): ([&'a IntPolyhPoint; 2], &'a IntPolyhPoint),
    normale: &'a IntPolyhPoint,
) -> IntPolyhTriangleEdge<'a> {
    IntPolyhTriangleEdge {
        tri_surf_id,
        edge_index: k as i32 + 1,
        tris: (pair.tri1, pair.tri2),
        pt: [&pts[0], &pts[1], &pts[2]],
        cote: [&cotes[0], &cotes[1], &cotes[2]],
        pe,
        edge,
        normale,
    }
}

/// One `TriangleEdgeContact` and its `TestNbPoints`.
fn edge_points(
    args: &IntPolyhTriangleEdge<'_>,
    total: &mut i32,
    sp1: &mut IntPolyhStartPoint,
    sp2: &mut IntPolyhStartPoint,
) -> Result<(), Failure> {
    let (mut pt1, mut pt2) = (IntPolyhStartPoint::default(), IntPolyhStartPoint::default());
    let nb_points: i32 = triangle_edge_contact(args, &mut pt1, &mut pt2)?;
    test_nb_points(nb_points, total, (&pt1, &pt2), sp1, sp2);
    Ok(())
}
