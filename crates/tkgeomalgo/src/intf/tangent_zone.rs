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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_TangentZone.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_TangentZone: a zone of tangency between two polygons or polyhedra, as the
//! section points bounding it and the parameter ranges they span.
use super::native_int_cast;
use super::section_point::SectionPoint;
use super::sequence_access::{index, item};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};

/// A parameter range `[min, max]`, empty (`RealLast`, `RealFirst`) until a point widens it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ParameterRange {
    min: f64,
    max: f64,
}

impl ParameterRange {
    const EMPTY: Self = Self {
        min: real_last(),
        max: real_first(),
    };

    fn widen(&mut self, value: f64) {
        if self.min > value {
            self.min = value;
        }
        if self.max < value {
            self.max = value;
        }
    }

    fn contains(&self, value: f64) -> bool {
        self.min <= value && value <= self.max
    }

    /// The native overlap test of HasCommonRange (`c` and `d` are the other range).
    fn overlaps(&self, other: &Self) -> bool {
        let (a, b, c, d) = (self.min, self.max, other.min, other.max);
        (c <= a && a <= d) || (c <= b && b <= d) || (a <= c && c <= b)
    }

    /// InfoFirst/InfoSecond: each bound split into its truncated segment index and the
    /// remaining parameter.
    fn split(&self) -> (i32, f64, i32, f64) {
        let segment_min: i32 = native_int_cast(self.min.trunc());
        let segment_max: i32 = native_int_cast(self.max.trunc());
        (
            segment_min,
            self.min - f64::from(segment_min),
            segment_max,
            self.max - f64::from(segment_max),
        )
    }
}

/// Intf_TangentZone: the section points of a zone of tangency and the ranges of their
/// cumulated parameters on both arguments.
///
/// ```
/// use rustycad_tkgeomalgo::intf::{PIType, SectionPoint, TangentZone};
/// use rustycad_tkmath::gp::Point2;
/// let mut zone = TangentZone::new();
/// zone.append(SectionPoint::with_data_2d(Point2::new(0.0, 0.0), PIType::Edge, 2, 0.5, PIType::Edge, 1, 0.0, 0.0));
/// assert_eq!(zone.param_on_first(), (1.5, 1.5));
/// ```
#[derive(Clone)]
pub struct TangentZone {
    result: Sequence<SectionPoint>,
    on_first: ParameterRange,
    on_second: ParameterRange,
}

impl Default for TangentZone {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for TangentZone {
    /// operator==: `is_equal`.
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl TangentZone {
    /// Intf_TangentZone(): an empty zone with empty ranges; `TangentZone::new()`.
    pub fn new() -> Self {
        Self {
            result: Sequence::default(),
            on_first: ParameterRange::EMPTY,
            on_second: ParameterRange::EMPTY,
        }
    }

    /// NumberOfPoints; `zone.number_of_points()`.
    pub fn number_of_points(&self) -> i32 {
        self.result.length()
    }

    /// GetPoint: the point of one-based `index`; raises Standard_OutOfRange outside
    /// the zone; `zone.get_point(1)?`.
    pub fn get_point(&self, index: i32) -> Result<&SectionPoint, Failure> {
        item(&self.result, index)
    }

    /// IsEqual: the same number of points, pairwise logically equal;
    /// `zone.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.result.len() == other.result.len()
            && self
                .result
                .iter()
                .zip(other.result.iter())
                .all(|(a, b)| a.is_equal(b))
    }

    /// Contains: whether a point of the zone is logically equal to `point`;
    /// `zone.contains(&point)`.
    pub fn contains(&self, point: &SectionPoint) -> bool {
        self.result
            .iter()
            .any(|candidate| point.is_equal(candidate))
    }

    /// ParamOnFirst: the (min, max) cumulated parameters on the first argument;
    /// `let (min, max) = zone.param_on_first();`.
    pub fn param_on_first(&self) -> (f64, f64) {
        (self.on_first.min, self.on_first.max)
    }

    /// ParamOnSecond: the (min, max) cumulated parameters on the second argument;
    /// `let (min, max) = zone.param_on_second();`.
    pub fn param_on_second(&self) -> (f64, f64) {
        (self.on_second.min, self.on_second.max)
    }

    /// InfoFirst: (segMin, paraMin, segMax, paraMax) on the first argument, each bound
    /// split into its truncated segment and the rest. For an empty zone the bounds are
    /// out of the int range; the port casts them as x86-64 does (INT_MIN), where the
    /// native cast is undefined; `let (s0, p0, s1, p1) = zone.info_first();`.
    pub fn info_first(&self) -> (i32, f64, i32, f64) {
        self.on_first.split()
    }

    /// InfoSecond: the same split on the second argument; `zone.info_second()`.
    pub fn info_second(&self) -> (i32, f64, i32, f64) {
        self.on_second.split()
    }

    /// RangeContains: whether both cumulated parameters of `point` lie in the ranges;
    /// `zone.range_contains(&point)`.
    pub fn range_contains(&self, point: &SectionPoint) -> bool {
        self.on_first.contains(point.param_on_first())
            && self.on_second.contains(point.param_on_second())
    }

    /// HasCommonRange: whether the ranges of both zones overlap on both arguments
    /// (the native three-case interval test); `zone.has_common_range(&other)`.
    pub fn has_common_range(&self, other: &Self) -> bool {
        self.on_first.overlaps(&other.on_first) && self.on_second.overlaps(&other.on_second)
    }

    /// Append(Pi): adds a point at the end and widens the ranges; `zone.append(point)`.
    pub fn append(&mut self, point: SectionPoint) {
        self.result.append(point);
        self.widen(&point);
    }

    /// Append(Tzi): inserts every point of `other` with `polygon_insert`;
    /// `zone.append_zone(&other)`.
    pub fn append_zone(&mut self, other: &Self) {
        for point in other.result.iter() {
            self.polygon_insert(*point);
        }
    }

    /// Insert: does nothing and returns false; its native body is commented out;
    /// `zone.insert(&point)`.
    pub fn insert(&mut self, _point: &SectionPoint) -> bool {
        false
    }

    /// PolygonInsert: appends the point, except that a point whose first parameter lies
    /// in `[min, max)` goes before the first point; the native sorted insertion is
    /// commented out ("too long", 1997); `zone.polygon_insert(point)`.
    pub fn polygon_insert(&mut self, point: SectionPoint) {
        let param: f64 = point.param_on_first();
        let before_first =
            !self.result.is_empty() && param < self.on_first.max && param >= self.on_first.min;
        if before_first {
            self.result.prepend(point);
            self.widen(&point);
        } else {
            self.append(point);
        }
    }

    /// InsertBefore: inserts before the one-based `index` and widens the ranges;
    /// raises Standard_OutOfRange for an index outside 1..=count+1;
    /// `zone.insert_before(1, point)?`.
    pub fn insert_before(&mut self, index_before: i32, point: SectionPoint) -> Result<(), Failure> {
        self.result.insert_before(index(index_before), point)?;
        self.widen(&point);
        Ok(())
    }

    /// InsertAfter: inserts after the one-based `index` (0 prepends) and widens the
    /// ranges; raises Standard_OutOfRange outside 0..=count;
    /// `zone.insert_after(1, point)?`.
    pub fn insert_after(&mut self, index_after: i32, point: SectionPoint) -> Result<(), Failure> {
        if index_after < 0 {
            return Err(Failure::out_of_range(&format!(
                "Intf_TangentZone::InsertAfter index {index_after}; expected 0..={}",
                self.result.len()
            )));
        }
        self.result.insert_after(index(index_after), point)?;
        self.widen(&point);
        Ok(())
    }

    /// Dump: prints nothing, as natively (its body is compiled out by DEBUG_TANGENTZONE
    /// 0); `zone.dump(2)`.
    pub fn dump(&self, _indent: i32) {}

    fn widen(&mut self, point: &SectionPoint) {
        let (first, second) = (point.param_on_first(), point.param_on_second());
        // Native order: both minima, then both maxima; the ranges are independent.
        self.on_first.widen(first);
        self.on_second.widen(second);
    }
}
