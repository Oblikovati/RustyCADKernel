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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_Edge.hxx,
// IntPolyh_Edge.cxx, IntPolyh_Couple.hxx and IntPolyh_Couple.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_Edge` (an edge of the triangulation and its two triangles) and
//! `IntPolyh_Couple` (two interfering triangles of the two surfaces).
use super::dump::c_fixed;
use std::fmt;
use std::hash::{Hash, Hasher};

/// `IntPolyh_Edge`: the indices of its two points and of its (up to) two triangles, -1
/// when unset.
///
/// ```
/// use rustycad_tkgeomalgo::int_polyh::IntPolyhEdge;
/// let e = IntPolyhEdge::new(0, 1, 2, -1);
/// assert_eq!((e.first_point(), e.second_triangle()), (0, -1));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntPolyhEdge {
    point1: i32,
    point2: i32,
    triangle1: i32,
    triangle2: i32,
}

impl Default for IntPolyhEdge {
    fn default() -> Self {
        Self::new(-1, -1, -1, -1)
    }
}

impl IntPolyhEdge {
    /// The constructor from points and triangles. `IntPolyhEdge::new(0, 1, 2, 3)`.
    pub const fn new(point1: i32, point2: i32, triangle1: i32, triangle2: i32) -> Self {
        Self {
            point1,
            point2,
            triangle1,
            triangle2,
        }
    }
    /// `FirstPoint()`. `e.first_point()`.
    pub const fn first_point(&self) -> i32 {
        self.point1
    }
    /// `SecondPoint()`. `e.second_point()`.
    pub const fn second_point(&self) -> i32 {
        self.point2
    }
    /// `FirstTriangle()`. `e.first_triangle()`.
    pub const fn first_triangle(&self) -> i32 {
        self.triangle1
    }
    /// `SecondTriangle()`. `e.second_triangle()`.
    pub const fn second_triangle(&self) -> i32 {
        self.triangle2
    }
    /// `SetFirstPoint(thePoint)`. `e.set_first_point(0)`.
    pub fn set_first_point(&mut self, point: i32) {
        self.point1 = point;
    }
    /// `SetSecondPoint(thePoint)`. `e.set_second_point(1)`.
    pub fn set_second_point(&mut self, point: i32) {
        self.point2 = point;
    }
    /// `SetFirstTriangle(theTriangle)`. `e.set_first_triangle(0)`.
    pub fn set_first_triangle(&mut self, triangle: i32) {
        self.triangle1 = triangle;
    }
    /// `SetSecondTriangle(theTriangle)`. `e.set_second_triangle(1)`.
    pub fn set_second_triangle(&mut self, triangle: i32) {
        self.triangle2 = triangle;
    }
    /// `Dump(v)`: the native `printf` text. `e.dump(3, &mut text)?`.
    pub fn dump(&self, i: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "\nEdge({i:3}) : P1:{:5}   P2:{:5}   T1:{:5}   T2:{:5}",
            self.point1, self.point2, self.triangle1, self.triangle2
        )
    }
}

/// `IntPolyh_Couple`: two triangles (of the first and of the second surface), the
/// analyzed flag and the cosine of the angle between their normals (-2 when unknown).
/// Two couples are equal when they hold the same two indices in either order.
///
/// ```
/// use rustycad_tkgeomalgo::int_polyh::IntPolyhCouple;
/// assert!(IntPolyhCouple::new(1, 2, 0.5).is_equal(&IntPolyhCouple::new(2, 1, -2.0)));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct IntPolyhCouple {
    index1: i32,
    index2: i32,
    analyzed: bool,
    angle: f64,
}

impl Default for IntPolyhCouple {
    fn default() -> Self {
        Self::new(-1, -1, -2.0)
    }
}

impl IntPolyhCouple {
    /// The constructor (`theAngle` is -2 by default natively). `IntPolyhCouple::new(1, 2, -2.0)`.
    pub const fn new(triangle1: i32, triangle2: i32, angle: f64) -> Self {
        Self {
            index1: triangle1,
            index2: triangle2,
            analyzed: false,
            angle,
        }
    }
    /// `FirstValue()`. `c.first_value()`.
    pub const fn first_value(&self) -> i32 {
        self.index1
    }
    /// `SecondValue()`. `c.second_value()`.
    pub const fn second_value(&self) -> i32 {
        self.index2
    }
    /// `IsAnalyzed()`. `c.is_analyzed()`.
    pub const fn is_analyzed(&self) -> bool {
        self.analyzed
    }
    /// `Angle()`. `c.angle()`.
    pub const fn angle(&self) -> f64 {
        self.angle
    }
    /// `SetCoupleValue(theInd1, theInd2)`. `c.set_couple_value(1, 2)`.
    pub fn set_couple_value(&mut self, ind1: i32, ind2: i32) {
        (self.index1, self.index2) = (ind1, ind2);
    }
    /// `SetAnalyzed(theAnalyzed)`. `c.set_analyzed(true)`.
    pub fn set_analyzed(&mut self, analyzed: bool) {
        self.analyzed = analyzed;
    }
    /// `SetAngle(theAngle)`. `c.set_angle(0.5)`.
    pub fn set_angle(&mut self, angle: f64) {
        self.angle = angle;
    }
    /// `IsEqual(theOther)` (also `operator==`): the same indices in either order.
    /// `a.is_equal(&b)`.
    pub const fn is_equal(&self, other: &IntPolyhCouple) -> bool {
        (self.index1 == other.index1 && self.index2 == other.index2)
            || (self.index1 == other.index2 && self.index2 == other.index1)
    }
    /// `Dump(v)`: the native `printf` text. `c.dump(3, &mut text)?`.
    pub fn dump(&self, i: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "\nCouple({i:3}) : {:5} {:5} {:3} {}",
            self.index1,
            self.index2,
            i32::from(self.analyzed),
            c_fixed(self.angle, 5, 6)
        )
    }
}

impl PartialEq for IntPolyhCouple {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl Eq for IntPolyhCouple {}

impl Hash for IntPolyhCouple {
    /// `std::hash<IntPolyh_Couple>`: the two indices in increasing order, so that equal
    /// couples hash alike.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let pair: (i32, i32) = if self.index1 > self.index2 {
            (self.index2, self.index1)
        } else {
            (self.index1, self.index2)
        };
        pair.hash(state);
    }
}
