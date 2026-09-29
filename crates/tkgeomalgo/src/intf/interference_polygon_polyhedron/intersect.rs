// Created on: 1992-12-21
// Created by: Didier PIFFAULT
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
// Rust translation of the Intersect members and IsInSegment of the generic
// ModelingAlgorithms/TKGeomAlgo/Intf/Intf_InterferencePolygonPolyhedron.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intersect: a segment (or an infinite line) against one triangle: the point where
//! it crosses the triangle plane, classified on a vertex, an edge or the face, then
//! the triangle edges that pass within the tolerance of the segment.
use super::{InterferencePolygonPolyhedron, Polygon3dTool, PolyhedronTool};
use crate::intf::plane_equation;
use crate::intf::section_point::{PIType, SectionPoint};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkgeombase::extrema::ExtElC;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3, Xyz};

/// `Pourcent3`: the next vertex of a triangle, cyclically.
const POURCENT3: [usize; 4] = [0, 1, 2, 0];

/// The native literal below which the plane distances are too close to divide.
const PARALLEL_GAP: f64 = 1.0e-16;

/// The angular tolerance the native code gives Extrema_ExtElC between two lines.
const EXTREMA_ANGULAR: f64 = 0.00000001;

/// Where the segment meets the triangle plane: the parameter on the segment, the type
/// of the segment point (VERTEX at an end, else EDGE) and whether it misses.
struct PlaneHit {
    param: f64,
    on_segment: PIType,
    misses: bool,
}

/// The classification of the plane point on the triangle: vertex and edge found
/// (-1 for none), the index `is` of the last hit, and the signed edge distances.
struct TriangleHit {
    vertex: i32,
    edge: i32,
    is: usize,
    edge_distances: [f64; 3],
}

impl<PT: Polygon3dTool, HT: PolyhedronTool> InterferencePolygonPolyhedron<PT, HT> {
    /// Intersect(BegO, EndO, Infinite, TTri, thePolyh): the plane of the triangle and
    /// the distances of both ends to it, then the plane-distance overload.
    pub(super) fn intersect(
        &mut self,
        beg_o: Point3,
        end_o: Point3,
        infinite: bool,
        triangle: i32,
        polyhedron: &HT::Polyhedron,
    ) -> Result<(), Failure> {
        let [t0, t1, t2] = HT::triangle(polyhedron, triangle)?;
        let (tri_nor, tri_dp) = plane_equation(
            HT::point(polyhedron, t0)?,
            HT::point(polyhedron, t1)?,
            HT::point(polyhedron, t2)?,
        );
        let d_beg: f64 = (tri_nor * beg_o.xyz()) - tri_dp;
        let d_end: f64 = (tri_nor * end_o.xyz()) - tri_dp;
        self.intersect_with_plane(
            beg_o, end_o, infinite, triangle, polyhedron, tri_nor, d_beg, d_end,
        )
    }

    /// Intersect(BegO, EndO, Infinite, TTri, thePolyh, TriNormal, TriDp, dBegTri,
    /// dEndTri): the plane point, when the finite segment reaches the plane (a line
    /// always does), then the near triangle edges. The unused TriDp is not taken.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn intersect_with_plane(
        &mut self,
        beg_o: Point3,
        end_o: Point3,
        infinite: bool,
        triangle: i32,
        polyhedron: &HT::Polyhedron,
        tri_nor: Xyz,
        d_beg: f64,
        d_end: f64,
    ) -> Result<(), Failure> {
        let p_tri: [i32; 3] = HT::triangle(polyhedron, triangle)?;
        let hit: PlaneHit = plane_hit(d_beg, d_end, infinite);
        if !hit.misses {
            let location: Xyz = beg_o.xyz() + ((end_o.xyz() - beg_o.xyz()) * hit.param);
            self.classify_plane_point(location, &hit, triangle, p_tri, polyhedron, tri_nor)?;
        }
        self.near_triangle_edges(beg_o, end_o, hit.on_segment, p_tri, polyhedron)
    }

    /// The section point of the plane point on a vertex, an edge (after the native
    /// TriConnex call, whose result is unused), inside the face, or inside a boundary
    /// edge's border deflection (S. Khromov, 7 Dec 2001: the triangulation may not
    /// cover the whole face).
    fn classify_plane_point(
        &mut self,
        location: Xyz,
        hit: &PlaneHit,
        triangle: i32,
        p_tri: [i32; 3],
        polyhedron: &HT::Polyhedron,
        tri_nor: Xyz,
    ) -> Result<(), Failure> {
        let corners = [
            HT::point(polyhedron, p_tri[0])?,
            HT::point(polyhedron, p_tri[1])?,
            HT::point(polyhedron, p_tri[2])?,
        ];
        let found: TriangleHit = triangle_hit(location, corners, tri_nor);
        let at = Point3::new(location.x(), location.y(), location.z());
        let on_face: SectionPoint = self.section_point(at, hit, (PIType::Face, triangle, 0));
        if found.vertex > -1 {
            let point = self.section_point(at, hit, (PIType::Vertex, p_tri[found.is], 0));
            self.base.section_points.append(point);
        } else if found.edge > -1 {
            let (a, b) = (
                p_tri[found.edge as usize],
                p_tri[POURCENT3[found.edge as usize + 1]],
            );
            HT::tri_connex(polyhedron, triangle, a, b)?;
            let point = self.section_point(at, hit, (PIType::Edge, a.min(b), a.max(b)));
            self.base.section_points.append(point);
        } else if found.edge_distances.iter().all(|&d| d > 0.0) {
            self.base.section_points.append(on_face);
        } else {
            for _ in 0..self.boundary_hits(at, p_tri, polyhedron)? {
                self.base.section_points.append(on_face);
            }
        }
        Ok(())
    }

    /// The boundary edges (in the native order) whose border deflection holds the
    /// point within the edge's extent; one face point is added per such edge.
    fn boundary_hits(
        &self,
        at: Point3,
        p_tri: [i32; 3],
        polyhedron: &HT::Polyhedron,
    ) -> Result<usize, Failure> {
        let mut hits: usize = 0;
        for i in 1..=3 {
            let index1: i32 = if i == 3 { p_tri[0] } else { p_tri[i] };
            let index2: i32 = p_tri[i - 1];
            if HT::is_on_bound(polyhedron, index1, index2)?
                && near_boundary::<HT>(at, index1, index2, polyhedron)?
            {
                hits += 1;
            }
        }
        Ok(hits)
    }

    /// The triangle edges that pass within the tolerance of the segment, each as an
    /// EDGE section point at the closest point of the edge.
    fn near_triangle_edges(
        &mut self,
        beg_o: Point3,
        end_o: Point3,
        on_segment: PIType,
        p_tri: [i32; 3],
        polyhedron: &HT::Polyhedron,
    ) -> Result<(), Failure> {
        let vec_pol: Vector3 = Vector3::between(beg_o, end_o);
        let n_vec_pol: f64 = vec_pol.magnitude();
        if n_vec_pol < precision::computational() {
            return Ok(());
        }
        let line_pol = Line3::new(beg_o, Direction3::from_vector(vec_pol)?);
        for i in 0..3 {
            let (next, current) = (p_tri[POURCENT3[i + 1]], p_tri[i]);
            let (beg_t, end_t) = (
                HT::point(polyhedron, next)?,
                HT::point(polyhedron, current)?,
            );
            let Some((par_o, par_t)) =
                self.near_edge(line_pol, (vec_pol, n_vec_pol), beg_o, beg_t, end_t)?
            else {
                continue;
            };
            let location: Xyz = beg_t.xyz() + ((end_t.xyz() - beg_t.xyz()) * par_t);
            let (tmin, tmax) = if current > next {
                (next, current)
            } else {
                (current, next)
            };
            let point = SectionPoint::with_data(
                Point3::new(location.x(), location.y(), location.z()),
                on_segment,
                0,
                self.i_lin,
                par_o,
                PIType::Edge,
                tmin,
                tmax,
                0.0,
                1.0,
            );
            self.base.section_points.append(point);
        }
        Ok(())
    }

    /// The parameters on the segment and on the edge `beg_t end_t` of their closest
    /// points, when the lines are not parallel, within the tolerance, and the closest
    /// points lie on both (IsInSegment).
    fn near_edge(
        &self,
        line_pol: Line3,
        (vec_pol, n_vec_pol): (Vector3, f64),
        beg_o: Point3,
        beg_t: Point3,
        end_t: Point3,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let vec_tri: Vector3 = Vector3::between(beg_t, end_t);
        let n_vec_tri: f64 = vec_tri.magnitude();
        if n_vec_tri < precision::computational() {
            return Ok(None);
        }
        let line_tri = Line3::new(beg_t, Direction3::from_vector(vec_tri)?);
        let extrema = ExtElC::lines(line_pol, line_tri, EXTREMA_ANGULAR);
        if !extrema.is_done() || extrema.is_parallel()? || extrema.nb_ext()? == 0 {
            return Ok(None);
        }
        let tolerance: f64 = self.base.tolerance;
        // The native positive test, so that a NaN distance is not near.
        let near: bool = extrema.square_distance(1)? <= tolerance * tolerance;
        if !near {
            return Ok(None);
        }
        let (on_pol, on_tri) = extrema.points(1)?;
        let Some(par_o) = in_segment(
            vec_pol,
            Vector3::between(beg_o, *on_pol.value()),
            n_vec_pol,
            tolerance,
        ) else {
            return Ok(None);
        };
        let par_t = in_segment(
            vec_tri,
            Vector3::between(beg_t, *on_tri.value()),
            n_vec_tri,
            tolerance,
        );
        Ok(par_t.map(|par_t| (par_o, par_t)))
    }

    /// A section point on the segment `iLin` (first address 0) and on the polyhedron
    /// element `(type, first, second)` with a null parameter and incidence 1.
    fn section_point(
        &self,
        at: Point3,
        hit: &PlaneHit,
        element: (PIType, i32, i32),
    ) -> SectionPoint {
        SectionPoint::with_data(
            at,
            hit.on_segment,
            0,
            self.i_lin,
            hit.param,
            element.0,
            element.1,
            element.2,
            0.0,
            1.0,
        )
    }
}

/// The parameter where the segment meets the plane: distances too close to divide
/// give the first distance itself, as natively. A finite segment snaps to an end
/// within `Epsilon(1000.)` of the plane (a VERTEX there) and misses the plane outside
/// [0, 1]. The native `if (BeginOfClosedPolygon)` assignment keeps the flag as it is,
/// so it has no effect.
fn plane_hit(d_beg: f64, d_end: f64, infinite: bool) -> PlaneHit {
    let t: f64 = d_beg - d_end;
    let mut hit = PlaneHit {
        param: if t >= PARALLEL_GAP || t <= -PARALLEL_GAP {
            d_beg / t
        } else {
            d_beg
        },
        on_segment: PIType::Edge,
        misses: false,
    };
    if infinite {
        return hit;
    }
    let floatgap: f64 = epsilon(1000.0);
    let within = |d: f64| d <= floatgap && d >= -floatgap;
    if within(d_beg) {
        (hit.param, hit.on_segment) = (0.0, PIType::Vertex);
    } else if within(d_end) {
        (hit.param, hit.on_segment) = (1.0, PIType::Vertex);
    }
    hit.misses = hit.param < 0.0 || hit.param > 1.0;
    hit
}

/// The native three edge tests of the plane point (edges 0-1, 1-2, 2-0): the first
/// two stop at a vertex or an edge hit; the last records a vertex hit and still
/// measures its edge distance.
fn triangle_hit(location: Xyz, corners: [Point3; 3], tri_nor: Xyz) -> TriangleHit {
    let floatgap: f64 = epsilon(1000.0);
    let mut found = TriangleHit {
        vertex: -1,
        edge: -1,
        is: 0,
        edge_distances: [0.0; 3],
    };
    for k in 0..3 {
        let from: Xyz = corners[k].xyz();
        let seg_t: Xyz = corners[(k + 1) % 3].xyz() - from;
        let vec_p: Xyz = location - from;
        if vec_p.modulus() <= floatgap {
            (found.vertex, found.is) = (k as i32, k);
            if k < 2 {
                return found;
            }
        }
        found.edge_distances[k] = signed_edge_distance(seg_t, vec_p, tri_nor, floatgap);
        let d: f64 = found.edge_distances[k];
        if d <= floatgap && d >= -floatgap {
            (found.edge, found.is) = (k as i32, k);
            if k < 2 {
                return found;
            }
        }
    }
    found
}

/// The distance of the point to the edge line, signed by the side of the triangle
/// normal (0 within `floatgap`).
fn signed_edge_distance(seg_t: Xyz, vec_p: Xyz, tri_nor: Xyz, floatgap: f64) -> f64 {
    let cross: Xyz = seg_t ^ vec_p;
    let modulus: f64 = cross.modulus();
    let mut sigd: f64 = cross * tri_nor;
    sigd = if sigd > floatgap {
        1.0
    } else if sigd < -floatgap {
        -1.0
    } else {
        0.0
    };
    sigd * (modulus / seg_t.modulus())
}

/// Whether the point is within the border deflection of the boundary edge
/// `index1 index2` and projects inside it; a null edge raises Standard_ConstructionError
/// (gp_Dir of a null vector), as natively.
fn near_boundary<HT: PolyhedronTool>(
    at: Point3,
    index1: i32,
    index2: i32,
    polyhedron: &HT::Polyhedron,
) -> Result<bool, Failure> {
    let deflection: f64 = HT::get_border_deflection(polyhedron);
    let (beg_p, end_p) = (
        HT::point(polyhedron, index1)?,
        HT::point(polyhedron, index2)?,
    );
    let vec_tri: Vector3 = Vector3::between(beg_p, end_p);
    let dir_tri: Direction3 = Direction3::from_vector(vec_tri)?;
    let near: bool = Line3::new(beg_p, dir_tri).distance(at) <= deflection;
    if !near {
        return Ok(false);
    }
    let parameter: f64 = Vector3::between(beg_p, at) * Vector3::from(dir_tri);
    Ok(parameter >= 0.0 && parameter <= vec_tri.magnitude())
}

/// IsInSegment: the parameter in [0, 1] of the projection of `p1p` on `p1p2`, None
/// when the segment is null or the projection is beyond an end by more than the
/// tolerance.
fn in_segment(p1p2: Vector3, p1p: Vector3, np1p2: f64, tolerance: f64) -> Option<f64> {
    if np1p2 < precision::computational() {
        return None;
    }
    let mut param: f64 = p1p2.dot(p1p);
    param /= np1p2;
    if param > (np1p2 + tolerance) || param < (-tolerance) {
        return None;
    }
    param /= np1p2;
    Some(param.clamp(0.0, 1.0))
}
