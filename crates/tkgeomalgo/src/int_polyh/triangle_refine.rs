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
// Rust translation of MiddleRefinement, MultipleMiddleRefinement and their helpers
// (GetInfoTA, NewTriangle, NewEdge, OldEdge) of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_Triangle.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The refinement of a triangle: split at the middle of its longest side, with the
//! adjacent triangle across that side split too.
use super::array::{IntPolyhArrayOfEdges, IntPolyhArrayOfPoints, IntPolyhArrayOfTriangles};
use super::triangle::IntPolyhTriangle;
use super::triangle_split::Splitter;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::bnd::BoundingBox3;

/// The three arrays of one surface's triangulation.
pub(crate) struct Mesh<'a> {
    pub(crate) points: &'a mut IntPolyhArrayOfPoints,
    pub(crate) triangles: &'a mut IntPolyhArrayOfTriangles,
    pub(crate) edges: &'a mut IntPolyhArrayOfEdges,
}

impl IntPolyhTriangle {
    /// `MiddleRefinement(NumTri, MySurface, TPoints, TTriangles, TEdges)`: splits this
    /// triangle (number `num_tri`) at the middle of its longest side and the adjacent
    /// triangle across that side, adding the middle point, two or four triangles and three
    /// or four edges; this triangle and the adjacent one become obsolete.
    /// `t.middle_refinement(0, &surface, &mut points, &mut triangles, &mut edges)?`.
    pub fn middle_refinement(
        &mut self,
        num_tri: i32,
        surface: &dyn Adaptor3dSurface,
        points: &mut IntPolyhArrayOfPoints,
        triangles: &mut IntPolyhArrayOfTriangles,
        edges: &mut IntPolyhArrayOfEdges,
    ) -> Result<(), Failure> {
        let mut mesh = Mesh {
            points,
            triangles,
            edges,
        };
        self.refine(num_tri, surface, &mut mesh)?;
        mesh.points.increment_nb_items();
        // The triangle is obsolete.
        self.set_deflection(-1.0);
        self.set_intersection_possible(false);
        Ok(())
    }

    /// `MultipleMiddleRefinement(theRefineCriterion, theBox, theTriangleNumber, ...)`:
    /// splits this triangle, then the new triangles while their deflection is above the
    /// criterion, marking those out of `the_box` (at most twice the initial triangle
    /// count plus 1000). `t.multiple_middle_refinement(0.1, &b, 0, &s, &mut p, &mut t2, &mut e)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn multiple_middle_refinement(
        &mut self,
        refine_criterion: f64,
        the_box: &BoundingBox3,
        triangle_number: i32,
        surface: &dyn Adaptor3dSurface,
        points: &mut IntPolyhArrayOfPoints,
        triangles: &mut IntPolyhArrayOfTriangles,
        edges: &mut IntPolyhArrayOfEdges,
    ) -> Result<(), Failure> {
        let fin_tt_init: i32 = triangles.nb_items();
        // Allow each triangle to be split at least once, plus some checks of the splits.
        let max_nb_tt: i32 = 2 * fin_tt_init + 1000;
        self.middle_refinement(triangle_number, surface, points, triangles, edges)?;
        let mut i: i32 = fin_tt_init;
        while i < triangles.nb_items() && i < max_nb_tt {
            let out: bool = the_box.is_out_box(triangles[i].bounding_box(points));
            if out {
                triangles[i].set_intersection_possible(false);
            } else if triangles[i].deflection() > refine_criterion {
                refine_in_array(i, surface, points, triangles, edges)?;
            }
            i += 1;
        }
        Ok(())
    }

    /// The body of `MiddleRefinement` before its last writes: the middle point of the
    /// longest side (the third when no side is strictly longest), then the split.
    fn refine(
        &self,
        num_tri: i32,
        surface: &dyn Adaptor3dSurface,
        mesh: &mut Mesh<'_>,
    ) -> Result<(), Failure> {
        let [p1, p2, p3] = self.corners(mesh.points);
        let l12: f64 = p1.square_distance(&p2);
        let l23: f64 = p2.square_distance(&p3);
        let l31: f64 = p3.square_distance(&p1);
        let side: i32 = if l12 > l23 && l12 > l31 {
            1
        } else if l23 > l31 && l23 > l12 {
            2
        } else {
            3
        };
        let fin_tp: i32 = mesh.points.nb_items();
        let (ends_a, ends_b) = match side {
            1 => (p1, p2),
            2 => (p2, p3),
            _ => (p3, p1),
        };
        let mut split = Splitter {
            fin_tp,
            points: self.points_ids(),
            my_edges: self.edge_ids(),
            num_tri,
            surface,
            mesh,
        };
        split.mesh.points[fin_tp].middle(surface, &ends_a, &ends_b)?;
        let num_ta: i32 = self.get_next_triangle(num_tri, side, split.mesh.edges);
        split.run(side, num_ta)
    }
}

/// Runs `MiddleRefinement` on the triangle `index` of `triangles`, as the native code
/// calls it on an item of the array it grows: the item is refined through a copy that is
/// written back (docs/geomalgo-design.md section 12).
pub(crate) fn refine_in_array(
    index: i32,
    surface: &dyn Adaptor3dSurface,
    points: &mut IntPolyhArrayOfPoints,
    triangles: &mut IntPolyhArrayOfTriangles,
    edges: &mut IntPolyhArrayOfEdges,
) -> Result<(), Failure> {
    let mut triangle: IntPolyhTriangle = triangles[index].clone();
    let result = triangle.middle_refinement(index, surface, points, triangles, edges);
    triangles[index] = triangle;
    result
}

/// Runs `MultipleMiddleRefinement` on the triangle `index` of `triangles`.
pub(crate) fn multiple_refine_in_array(
    (criterion, the_box): (f64, &BoundingBox3),
    index: i32,
    surface: &dyn Adaptor3dSurface,
    mesh: &mut Mesh<'_>,
) -> Result<(), Failure> {
    let mut triangle: IntPolyhTriangle = mesh.triangles[index].clone();
    let result = triangle.multiple_middle_refinement(
        criterion,
        the_box,
        index,
        surface,
        mesh.points,
        mesh.triangles,
        mesh.edges,
    );
    // The loop of the native member refines the new triangles only (indices past the
    // initial count), so the triangle itself is written back as the copy left it.
    mesh.triangles[index] = triangle;
    result
}
