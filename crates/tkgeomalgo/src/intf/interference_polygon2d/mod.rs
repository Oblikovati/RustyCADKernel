// Created on: 1991-06-24
// Created by: Didier PIFFAULT
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_InterferencePolygon2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_InterferencePolygon2d: the intersection points and tangent zones of two
//! polygons, or the self intersections of one polygon, in 2d.
mod clean;
mod intersect;
mod radius;

use super::interference::Interference;
use super::nonzero_tolerance;
use super::polygon2d::Polygon2d;
use super::sequence_access::item;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point2;
use std::ops::{Deref, DerefMut};

/// Intf_InterferencePolygon2d: an Intf_Interference computed between the segments of
/// two polygons (or of one polygon with itself), whose results the `Interference`
/// accessors read through `Deref`.
///
/// ```
/// use rustycad_tkgeomalgo::intf::InterferencePolygon2d;
/// let empty = InterferencePolygon2d::new();
/// assert_eq!(empty.nb_section_points(), 0);
/// ```
#[derive(Clone)]
pub struct InterferencePolygon2d {
    base: Interference,
    o_clos: bool,
    // Stored as natively (tClos) and never read there either.
    #[allow(dead_code)]
    t_clos: bool,
    nbso: i32,
}

impl Default for InterferencePolygon2d {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for InterferencePolygon2d {
    type Target = Interference;
    fn deref(&self) -> &Interference {
        &self.base
    }
}

impl DerefMut for InterferencePolygon2d {
    /// The public members of Intf_Interference (Insert) apply to the derived object.
    fn deref_mut(&mut self) -> &mut Interference {
        &mut self.base
    }
}

impl InterferencePolygon2d {
    /// Intf_InterferencePolygon2d(): an empty interference; `InterferencePolygon2d::new()`.
    pub fn new() -> Self {
        Self {
            base: Interference::new(false),
            o_clos: false,
            t_clos: false,
            nbso: 0,
        }
    }

    /// Intf_InterferencePolygon2d(Obje1, Obje2): computes the interference of two
    /// polygons; an error of a polygon's `segment` propagates;
    /// `InterferencePolygon2d::between(&first, &second)?`.
    pub fn between(first: &dyn Polygon2d, second: &dyn Polygon2d) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.compute_pair(first, second)?;
        Ok(interference)
    }

    /// Intf_InterferencePolygon2d(Obje): computes the self interference of a polygon;
    /// `InterferencePolygon2d::of_self(&polygon)?`.
    pub fn of_self(polygon: &dyn Polygon2d) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.base.self_interference = true;
        interference.compute_self(polygon)?;
        Ok(interference)
    }

    /// Perform(Obje1, Obje2): clears the results and computes the interference of two
    /// polygons; `interference.perform(&first, &second)?`.
    pub fn perform(
        &mut self,
        first: &dyn Polygon2d,
        second: &dyn Polygon2d,
    ) -> Result<(), Failure> {
        self.base.self_interference(false);
        self.compute_pair(first, second)
    }

    /// Perform(Obje): clears the results and computes the self interference of a
    /// polygon. As natively the segment count used by the closed-polygon test keeps
    /// its value from the last two-polygon computation; `interference.perform_self(&polygon)?`.
    pub fn perform_self(&mut self, polygon: &dyn Polygon2d) -> Result<(), Failure> {
        self.base.self_interference(true);
        self.compute_self(polygon)
    }

    /// Pnt2dValue: the 2d location of the section point of one-based `index`; raises
    /// Standard_OutOfRange outside the points; `interference.pnt2d_value(1)?`.
    pub fn pnt2d_value(&self, index: i32) -> Result<Point2, Failure> {
        let location = item(&self.base.section_points, index)?.pnt();
        Ok(Point2::new(location.x(), location.y()))
    }

    /// The shared body of the two-polygon constructor and Perform: nothing is computed
    /// (not even the tolerance) when the polygon boxes are apart.
    fn compute_pair(
        &mut self,
        first: &dyn Polygon2d,
        second: &dyn Polygon2d,
    ) -> Result<(), Failure> {
        if first.bounding().is_out_box(second.bounding()) {
            return Ok(());
        }
        let tolerance: f64 =
            first.deflection_over_estimation() + second.deflection_over_estimation();
        self.base.tolerance = nonzero_tolerance(tolerance);
        self.nbso = first.nb_segments();
        self.o_clos = first.closed();
        self.t_clos = second.closed();
        self.interfere_pair(first, second)?;
        self.clean()
    }

    fn compute_self(&mut self, polygon: &dyn Polygon2d) -> Result<(), Failure> {
        self.base.tolerance = nonzero_tolerance(polygon.deflection_over_estimation() * 2.0);
        self.o_clos = polygon.closed();
        self.t_clos = self.o_clos;
        self.interfere_self(polygon)?;
        self.clean()
    }

    /// Interference(Obje1, Obje2): intersects every pair of segments whose enlarged
    /// boxes meet.
    fn interfere_pair(
        &mut self,
        first: &dyn Polygon2d,
        second: &dyn Polygon2d,
    ) -> Result<(), Failure> {
        let (d1, d2) = (
            first.deflection_over_estimation(),
            second.deflection_over_estimation(),
        );
        for i1 in 1..=self.nbso {
            let (p1b, p1e, box1) = segment_box(first, i1, d1)?;
            if second.bounding().is_out_box(&box1) {
                continue;
            }
            for i2 in 1..=second.nb_segments() {
                let (p2b, p2e, box2) = segment_box(second, i2, d2)?;
                if !box1.is_out_box(&box2) {
                    self.intersect([i1, i2], [p1b, p1e], [p2b, p2e]);
                }
            }
        }
        Ok(())
    }

    /// Interference(Obje): intersects every pair of distinct segments of one polygon.
    fn interfere_self(&mut self, polygon: &dyn Polygon2d) -> Result<(), Failure> {
        let (count, deflection) = (polygon.nb_segments(), polygon.deflection_over_estimation());
        for i1 in 1..=count {
            let (p1b, p1e, box1) = segment_box(polygon, i1, deflection)?;
            if polygon.bounding().is_out_box(&box1) {
                continue;
            }
            for i2 in i1 + 1..=count {
                let (p2b, p2e, box2) = segment_box(polygon, i2, deflection)?;
                if !box1.is_out_box(&box2) {
                    self.intersect([i1, i2], [p1b, p1e], [p2b, p2e]);
                }
            }
        }
        Ok(())
    }
}

/// The segment `index` of `polygon` with its box enlarged by `deflection`.
fn segment_box(
    polygon: &dyn Polygon2d,
    index: i32,
    deflection: f64,
) -> Result<(Point2, Point2, BoundingBox2), Failure> {
    let (begin, end) = polygon.segment(index)?;
    let mut bounds = BoundingBox2::default();
    bounds.add_point(begin);
    bounds.add_point(end);
    bounds.enlarge(deflection);
    Ok((begin, end, bounds))
}
