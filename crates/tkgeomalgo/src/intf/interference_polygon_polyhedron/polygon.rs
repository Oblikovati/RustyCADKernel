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
// Rust translation of the Interference members of the generic
// ModelingAlgorithms/TKGeomAlgo/Intf/Intf_InterferencePolygonPolyhedron.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Interference(thePolyg, thePolyh[, theBoundSB]): each polygon segment against the
//! triangles whose boxes meet the segment's box.
use super::{InterferencePolygonPolyhedron, Polygon3dTool, PolyhedronTool, polyhedron_grid};
use crate::intf::plane_equation;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::gp::{self, Point3, Vector3, Xyz};

/// The norm below which a triangle is skipped as degenerate in the grid-less
/// Interference (the native literal).
const DEGENERATE_NORMAL: f64 = 1e-14;

impl<PT: Polygon3dTool, HT: PolyhedronTool> InterferencePolygonPolyhedron<PT, HT> {
    /// Interference(thePolyg, thePolyh): builds a grid of the polyhedron, then
    /// intersects each segment, moved by the polyhedron deflection along the triangle
    /// normal to both sides (two crossed segments), with each selected triangle;
    /// `interference.interference(&pg, &ph)?`.
    pub fn interference(
        &mut self,
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
    ) -> Result<(), Failure> {
        let mut grid: BoundSortBox = polyhedron_grid::<HT>(polyhedron)?;
        self.begin_of_closed_polygon = PT::closed(polygon);
        let deflection: f64 = HT::deflection_over_estimation(polyhedron);
        self.i_lin = 1;
        while self.i_lin <= PT::nb_segments(polygon) {
            for triangle in self.segment_triangles(polygon, &mut grid)? {
                self.intersect_offset_segments(polygon, polyhedron, triangle, deflection)?;
            }
            self.begin_of_closed_polygon = false;
            self.i_lin += 1;
        }
        Ok(())
    }

    /// Interference(thePolyg, thePolyh, theBoundSB): intersects each segment with each
    /// triangle the given grid selects; the first and last segments are extended by the
    /// polyhedron deflection over the incidence (MKK, 25 Oct 2007);
    /// `interference.interference_grid(&pg, &ph, &mut grid)?`.
    pub fn interference_grid(
        &mut self,
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        self.begin_of_closed_polygon = PT::closed(polygon);
        self.i_lin = 1;
        while self.i_lin <= PT::nb_segments(polygon) {
            let deflection: f64 = HT::deflection_over_estimation(polyhedron);
            let triangles: Vec<i32> = self.segment_triangles(polygon, grid)?;
            // The segment ends default to the origin and are read only for a nonempty
            // selection, as natively.
            let (mut p1, mut p2) = (Point3::default(), Point3::default());
            if !triangles.is_empty() {
                p1 = PT::begin_of_seg(polygon, self.i_lin)?;
                p2 = PT::end_of_seg(polygon, self.i_lin)?;
            }
            let (mut beg0, mut end0) = (p1, p2);
            for triangle in triangles {
                self.intersect_extended_segment(
                    polygon,
                    polyhedron,
                    triangle,
                    deflection,
                    (p1, p2),
                    (&mut beg0, &mut end0),
                )?;
            }
            self.begin_of_closed_polygon = false;
            self.i_lin += 1;
        }
        Ok(())
    }

    /// The triangles whose boxes meet the box of segment `iLin`, enlarged by the
    /// polygon deflection.
    fn segment_triangles(
        &self,
        polygon: &PT::Polygon,
        grid: &mut BoundSortBox,
    ) -> Result<Vec<i32>, Failure> {
        let mut segment_box = BoundingBox3::default();
        segment_box.add_point(PT::begin_of_seg(polygon, self.i_lin)?);
        segment_box.add_point(PT::end_of_seg(polygon, self.i_lin)?);
        segment_box.enlarge(PT::deflection_over_estimation(polygon));
        Ok(grid.compare_box(&segment_box)?.iter().copied().collect())
    }

    /// The grid-less case for one triangle: skipped when degenerate, else the segment
    /// crossed from one side of the triangle plane to the other, both ways.
    fn intersect_offset_segments(
        &mut self,
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
        triangle: i32,
        deflection: f64,
    ) -> Result<(), Failure> {
        let p1: Point3 = PT::begin_of_seg(polygon, self.i_lin)?;
        let p2: Point3 = PT::end_of_seg(polygon, self.i_lin)?;
        let [t0, t1, t2] = HT::triangle(polyhedron, triangle)?;
        let pa: Point3 = HT::point(polyhedron, t0)?;
        let pb: Point3 = HT::point(polyhedron, t1)?;
        let pc: Point3 = HT::point(polyhedron, t2)?;
        let mut normal: Vector3 = Vector3::between(pa, pb).crossed(Vector3::between(pa, pc));
        let length: f64 = normal.magnitude();
        if length < DEGENERATE_NORMAL {
            return Ok(());
        }
        normal *= deflection / length;
        let (p1m, p1p) = (p1.translated(-normal), p1.translated(normal));
        let (p2m, p2p) = (p2.translated(-normal), p2.translated(normal));
        self.intersect(p1m, p2p, false, triangle, polyhedron)?;
        self.intersect(p1p, p2m, false, triangle, polyhedron)
    }

    /// The grid case for one triangle: extends the first (or last) segment beyond its
    /// start (or end), then intersects with the plane distances precomputed.
    fn intersect_extended_segment(
        &mut self,
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
        triangle: i32,
        deflection: f64,
        (p1, p2): (Point3, Point3),
        (beg0, end0): (&mut Point3, &mut Point3),
    ) -> Result<(), Failure> {
        let [t0, t1, t2] = HT::triangle(polyhedron, triangle)?;
        let points = [
            HT::point(polyhedron, t0)?,
            HT::point(polyhedron, t1)?,
            HT::point(polyhedron, t2)?,
        ];
        let (tri_nor, tri_dp) = plane_equation(points[0], points[1], points[2]);
        if self.i_lin == 1 {
            extend_end(beg0, p1, p2, tri_nor, deflection);
        } else if self.i_lin == PT::nb_segments(polygon) {
            extend_end(end0, p2, p1, tri_nor, deflection);
        }
        let d_beg: f64 = (tri_nor * beg0.xyz()) - tri_dp;
        let d_end: f64 = (tri_nor * end0.xyz()) - tri_dp;
        self.intersect_with_plane(
            *beg0, *end0, false, triangle, polyhedron, tri_nor, d_beg, d_end,
        )
    }
}

/// Moves `end` to `from` pushed away from `other` by deflection / |cos| of the segment
/// with the triangle normal, when neither is below `gp::Resolution`.
fn extend_end(end: &mut Point3, from: Point3, other: Point3, tri_nor: Xyz, deflection: f64) {
    let mut dif: Xyz = from.xyz() - other.xyz();
    let dist: f64 = dif.modulus();
    // The native positive test, so that a NaN length leaves the end.
    let long_enough: bool = dist > gp::resolution();
    if !long_enough {
        return;
    }
    dif /= dist;
    let cos: f64 = (dif * tri_nor).abs();
    if cos > gp::resolution() {
        let shift: f64 = deflection / cos;
        end.set_xyz(from.xyz() + dif * shift);
    }
}
