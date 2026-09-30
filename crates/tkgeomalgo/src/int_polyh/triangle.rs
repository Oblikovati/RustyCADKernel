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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_Triangle.hxx and
// IntPolyh_Triangle.cxx (all but the refinement) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_Triangle`: a triangle of the triangulation, its three edges with their
//! orientations, its deflection, flags and cached bounding box. The refinement members
//! are in `triangle_refine.rs`.
use super::array::{IntPolyhArrayOfEdges, IntPolyhArrayOfPoints};
use super::dump::c_fixed;
use super::edge::IntPolyhEdge;
use super::point::IntPolyhPoint;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use std::fmt;

/// The squared coincidence precision of the triangles (`SquareMyConfusionPrecision`).
const SQUARE_MY_CONFUSION_PRECISION: f64 = 10.0e-24;

/// `IntPolyh_Triangle`: the indices of three points and three edges (-1 when unset), the
/// edge orientations (1 when the edge runs as the triangle, -1 otherwise, 0 unset), the
/// intersection flags and the deflection.
///
/// ```
/// use rustycad_tkgeomalgo::int_polyh::IntPolyhTriangle;
/// let t = IntPolyhTriangle::new(0, 1, 2);
/// assert!(t.is_intersection_possible() && !t.has_intersection());
/// assert_eq!((t.get_edge_number(0), t.first_edge()), (0, -1));
/// ```
#[derive(Clone, Debug)]
pub struct IntPolyhTriangle {
    points: [i32; 3],
    edges: [i32; 3],
    edges_orientations: [i32; 3],
    has_intersection: bool,
    is_intersection_possible: bool,
    is_degenerated: bool,
    deflection: f64,
    bbox: BoundingBox3,
}

impl Default for IntPolyhTriangle {
    fn default() -> Self {
        Self::new(-1, -1, -1)
    }
}

impl IntPolyhTriangle {
    /// The constructor from three points; no edge, intersection possible.
    /// `IntPolyhTriangle::new(0, 1, 2)`.
    pub fn new(point1: i32, point2: i32, point3: i32) -> Self {
        Self {
            points: [point1, point2, point3],
            edges: [-1; 3],
            edges_orientations: [0; 3],
            has_intersection: false,
            is_intersection_possible: true,
            is_degenerated: false,
            deflection: 0.0,
            bbox: BoundingBox3::default(),
        }
    }

    /// `FirstPoint()`. `t.first_point()`.
    pub const fn first_point(&self) -> i32 {
        self.points[0]
    }
    /// `SecondPoint()`. `t.second_point()`.
    pub const fn second_point(&self) -> i32 {
        self.points[1]
    }
    /// `ThirdPoint()`. `t.third_point()`.
    pub const fn third_point(&self) -> i32 {
        self.points[2]
    }
    /// `FirstEdge()`. `t.first_edge()`.
    pub const fn first_edge(&self) -> i32 {
        self.edges[0]
    }
    /// `FirstEdgeOrientation()`. `t.first_edge_orientation()`.
    pub const fn first_edge_orientation(&self) -> i32 {
        self.edges_orientations[0]
    }
    /// `SecondEdge()`. `t.second_edge()`.
    pub const fn second_edge(&self) -> i32 {
        self.edges[1]
    }
    /// `SecondEdgeOrientation()`. `t.second_edge_orientation()`.
    pub const fn second_edge_orientation(&self) -> i32 {
        self.edges_orientations[1]
    }
    /// `ThirdEdge()`. `t.third_edge()`.
    pub const fn third_edge(&self) -> i32 {
        self.edges[2]
    }
    /// `ThirdEdgeOrientation()`. `t.third_edge_orientation()`.
    pub const fn third_edge_orientation(&self) -> i32 {
        self.edges_orientations[2]
    }
    /// `Deflection()`. `t.deflection()`.
    pub const fn deflection(&self) -> f64 {
        self.deflection
    }
    /// `IsIntersectionPossible()`. `t.is_intersection_possible()`.
    pub const fn is_intersection_possible(&self) -> bool {
        self.is_intersection_possible
    }
    /// `HasIntersection()`. `t.has_intersection()`.
    pub const fn has_intersection(&self) -> bool {
        self.has_intersection
    }
    /// `IsDegenerated()`. `t.is_degenerated()`.
    pub const fn is_degenerated(&self) -> bool {
        self.is_degenerated
    }

    /// `SetFirstPoint(thePoint)`. `t.set_first_point(0)`.
    pub fn set_first_point(&mut self, point: i32) {
        self.points[0] = point;
    }
    /// `SetSecondPoint(thePoint)`. `t.set_second_point(1)`.
    pub fn set_second_point(&mut self, point: i32) {
        self.points[1] = point;
    }
    /// `SetThirdPoint(thePoint)`. `t.set_third_point(2)`.
    pub fn set_third_point(&mut self, point: i32) {
        self.points[2] = point;
    }
    /// `SetFirstEdge(theEdge, theEdgeOrientation)`. `t.set_first_edge(0, 1)`.
    pub fn set_first_edge(&mut self, edge: i32, orientation: i32) {
        (self.edges[0], self.edges_orientations[0]) = (edge, orientation);
    }
    /// `SetSecondEdge(theEdge, theEdgeOrientation)`. `t.set_second_edge(1, -1)`.
    pub fn set_second_edge(&mut self, edge: i32, orientation: i32) {
        (self.edges[1], self.edges_orientations[1]) = (edge, orientation);
    }
    /// `SetThirdEdge(theEdge, theEdgeOrientation)`. `t.set_third_edge(2, 1)`.
    pub fn set_third_edge(&mut self, edge: i32, orientation: i32) {
        (self.edges[2], self.edges_orientations[2]) = (edge, orientation);
    }
    /// `SetDeflection(theDeflection)`. `t.set_deflection(0.1)`.
    pub fn set_deflection(&mut self, deflection: f64) {
        self.deflection = deflection;
    }
    /// `SetIntersectionPossible(theIP)`. `t.set_intersection_possible(false)`.
    pub fn set_intersection_possible(&mut self, ip: bool) {
        self.is_intersection_possible = ip;
    }
    /// `SetIntersection(theInt)`. `t.set_intersection(true)`.
    pub fn set_intersection(&mut self, int: bool) {
        self.has_intersection = int;
    }
    /// `SetDegenerated(theDegFlag)`. `t.set_degenerated(true)`.
    pub fn set_degenerated(&mut self, deg_flag: bool) {
        self.is_degenerated = deg_flag;
    }

    /// `GetEdgeNumber(theEdgeIndex)`: the edge 1..=3, 0 for another index.
    /// `t.get_edge_number(1)`.
    pub const fn get_edge_number(&self, edge_index: i32) -> i32 {
        if edge_index >= 1 && edge_index <= 3 {
            return self.edges[(edge_index - 1) as usize];
        }
        0
    }
    /// `SetEdge(theEdgeIndex, theEdgeNumber)`: ignored for an index outside 1..=3.
    /// `t.set_edge(1, 5)`.
    pub fn set_edge(&mut self, edge_index: i32, edge_number: i32) {
        if (1..=3).contains(&edge_index) {
            self.edges[(edge_index - 1) as usize] = edge_number;
        }
    }
    /// `GetEdgeOrientation(theEdgeIndex)`: 0 for an index outside 1..=3.
    /// `t.get_edge_orientation(1)`.
    pub const fn get_edge_orientation(&self, edge_index: i32) -> i32 {
        if edge_index >= 1 && edge_index <= 3 {
            return self.edges_orientations[(edge_index - 1) as usize];
        }
        0
    }
    /// `SetEdgeOrientation(theEdgeIndex, theEdgeOrientation)`: ignored outside 1..=3.
    /// `t.set_edge_orientation(1, -1)`.
    pub fn set_edge_orientation(&mut self, edge_index: i32, orientation: i32) {
        if (1..=3).contains(&edge_index) {
            self.edges_orientations[(edge_index - 1) as usize] = orientation;
        }
    }

    /// `myPoints`.
    pub(crate) const fn points_ids(&self) -> [i32; 3] {
        self.points
    }

    /// `myEdges`.
    pub(crate) const fn edge_ids(&self) -> [i32; 3] {
        self.edges
    }

    /// The three points of the triangle in `points`.
    pub(crate) fn corners(&self, points: &IntPolyhArrayOfPoints) -> [IntPolyhPoint; 3] {
        [
            points[self.points[0]],
            points[self.points[1]],
            points[self.points[2]],
        ]
    }

    /// `ComputeDeflection(theSurface, thePoints)`: the distance from the plane of the
    /// triangle to the surface point at the barycenter of its parameters; 0 and the
    /// degenerated flag for a triangle with two degenerated points or a null normal.
    /// `t.compute_deflection(&surface, &points)?`.
    pub fn compute_deflection(
        &mut self,
        surface: &dyn Adaptor3dSurface,
        points: &IntPolyhArrayOfPoints,
    ) -> Result<f64, Failure> {
        self.deflection = 0.;
        let [p1, p2, p3] = self.corners(points);
        // The triangle is degenerated when more than one point is.
        let i_deg: i32 = [p1, p2, p3]
            .iter()
            .map(|p| i32::from(p.degenerated()))
            .sum();
        if i_deg > 1 {
            self.is_degenerated = true;
            return Ok(self.deflection);
        }
        let mut normale_tri = IntPolyhPoint::default();
        normale_tri.cross(&(p2 - p1), &(p3 - p1));
        let sq_norm: f64 = normale_tri.square_modulus();
        if sq_norm < SQUARE_MY_CONFUSION_PRECISION {
            self.is_degenerated = true;
            return Ok(self.deflection);
        }
        let gu: f64 = (p1.u() + p2.u() + p3.u()) / 3.0;
        let gv: f64 = (p1.v() + p2.v() + p3.v()) / 3.0;
        let pt_xyz: Point3 = surface.value(gu, gv)?;
        let barycentre_reel = IntPolyhPoint::new(pt_xyz.x(), pt_xyz.y(), pt_xyz.z(), gu, gv);
        normale_tri = normale_tri / sq_norm.sqrt();
        self.deflection = normale_tri.dot(&(barycentre_reel - p1)).abs();
        Ok(self.deflection)
    }

    /// `GetNextTriangle(theTriangle, theEdgeNum, TEdges)`: the other triangle of the edge
    /// 1..=3 of this triangle (-1 for another edge number or a border edge).
    /// `t.get_next_triangle(0, 1, &edges)`.
    pub fn get_next_triangle(
        &self,
        triangle: i32,
        edge_num: i32,
        edges: &IntPolyhArrayOfEdges,
    ) -> i32 {
        if !(1..=3).contains(&edge_num) {
            return -1;
        }
        let edge: &IntPolyhEdge = &edges[self.edges[(edge_num - 1) as usize]];
        if edge.first_triangle() == triangle {
            return edge.second_triangle();
        }
        edge.first_triangle()
    }

    /// `LinkEdges2Triangle(TEdges, theEdge1, theEdge2, theEdge3)`: sets the three edges
    /// and their orientations (1 when the edge starts at the point of the same rank);
    /// ignored when an edge is negative. `t.link_edges_2_triangle(&edges, 0, 1, 2)`.
    pub fn link_edges_2_triangle(
        &mut self,
        edges: &IntPolyhArrayOfEdges,
        edge1: i32,
        edge2: i32,
        edge3: i32,
    ) {
        if edge1 < 0 || edge2 < 0 || edge3 < 0 {
            return;
        }
        self.edges = [edge1, edge2, edge3];
        for k in 0..3 {
            let starts_here: bool = edges[self.edges[k]].first_point() == self.points[k];
            self.edges_orientations[k] = if starts_here { 1 } else { -1 };
        }
    }

    /// `SetEdgeAndOrientation(theEdge, theEdgeIndex)`: stores `edge_index` as the edge of
    /// the triangle whose points are the edge's (e1 = p1p2, e2 = p2p3, e3 = p3p1), with
    /// orientation 1 when they run the same way. `t.set_edge_and_orientation(&e, 4)`.
    pub fn set_edge_and_orientation(&mut self, edge: &IntPolyhEdge, edge_index: i32) {
        let (pe1, pe2) = (edge.first_point(), edge.second_point());
        for i in 0..3 {
            let i1: usize = (i + 1) % 3;
            if pe1 == self.points[i] && pe2 == self.points[i1] {
                (self.edges[i], self.edges_orientations[i]) = (edge_index, 1);
                return;
            }
            if pe1 == self.points[i1] && pe2 == self.points[i] {
                (self.edges[i], self.edges_orientations[i]) = (edge_index, -1);
                return;
            }
        }
    }

    /// `BoundingBox(thePoints)`: the box of the three points enlarged by the deflection
    /// plus Precision::Confusion, computed once and kept. `t.bounding_box(&points)`.
    pub fn bounding_box(&mut self, points: &IntPolyhArrayOfPoints) -> &BoundingBox3 {
        if self.bbox.is_void() {
            for p in self.corners(points) {
                self.bbox.add_point(Point3::new(p.x(), p.y(), p.z()));
            }
            self.bbox.set_gap(self.deflection + precision::confusion());
        }
        &self.bbox
    }

    /// `Dump(v)`: the native `printf` text. `t.dump(3, &mut text)?`.
    pub fn dump(&self, i: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "\nTriangle({i:3}) : Points {:5} {:5} {:5} Edges {:5} {:5} {:5} deflection: {} \
             intersection possible {:8}  intersection: {:5}",
            self.points[0],
            self.points[1],
            self.points[2],
            self.edges[0],
            self.edges[1],
            self.edges[2],
            c_fixed(self.deflection, 8, 6),
            i32::from(self.is_intersection_possible),
            i32::from(self.has_intersection)
        )
    }
}
