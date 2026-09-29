// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of HatchGen_PointOnHatching.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HatchGen_PointOnHatching`: an intersection point seen on a hatching, with the
//! points on the elements that meet the hatching there.
use super::point_on_element::position_orientation;
use super::text::{flag_text, index_title, position_text, sequence_slot, state_text, stream_real};
use super::{IntersectionPoint, IntersectionPointDump, PointOnElement};
use crate::int_res2d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// `HatchGen_PointOnHatching`: an [`IntersectionPoint`] on a hatching and the
/// sequence of the [`PointOnElement`]s at the same place. Points are ordered along
/// the hatching by `is_lower`/`is_equal`/`is_greater` within a confusion tolerance.
///
/// ```
/// use rustycad_tkgeomalgo::hatch_gen::{PointOnElement, PointOnHatching};
/// let mut point = PointOnHatching::new();
/// point.add_point(&PointOnElement::new(), 1e-9);
/// point.add_point(&PointOnElement::new(), 1e-9);
/// assert_eq!(point.nb_points(), 1);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointOnHatching {
    base: IntersectionPoint,
    my_points: Vec<PointOnElement>,
}

impl Deref for PointOnHatching {
    type Target = IntersectionPoint;
    fn deref(&self) -> &IntersectionPoint {
        &self.base
    }
}

impl DerefMut for PointOnHatching {
    fn deref_mut(&mut self) -> &mut IntersectionPoint {
        &mut self.base
    }
}

impl PointOnHatching {
    /// `HatchGen_PointOnHatching()`: an empty point; `PointOnHatching::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `HatchGen_PointOnHatching(Point)`: the point on the first curve (the hatching)
    /// of an intersection: its parameter and position there, UNKNOWN states and no
    /// point on element; `PointOnHatching::from_intersection_point(&point)`.
    pub fn from_intersection_point(point: &int_res2d::IntersectionPoint) -> Self {
        let mut result = Self::new();
        result.base.my_index = 0;
        result.base.my_param = point.param_on_first();
        result.base.my_posit =
            position_orientation(point.transition_of_first().position_on_curve());
        result.base.my_before = State::Unknown;
        result.base.my_after = State::Unknown;
        result.base.my_seg_beg = false;
        result.base.my_seg_end = false;
        result.my_points.clear();
        result
    }

    /// AddPoint(Point, Confusion): appends `point` unless an identical one (within
    /// `confusion`) is already there; `hatching_point.add_point(&element_point, 1e-9)`.
    pub fn add_point(&mut self, point: &PointOnElement, confusion: f64) {
        if self
            .my_points
            .iter()
            .all(|known| known.is_different(point, confusion))
        {
            self.my_points.push(*point);
        }
    }

    /// NbPoints: the number of elements meeting the hatching at this point;
    /// `point.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.my_points.len() as i32
    }

    /// Point(Index): the `index`-th point on element; Standard_OutOfRange outside
    /// [1, NbPoints]; `point.point(1)?`.
    pub fn point(&self, index: i32) -> Result<&PointOnElement, Failure> {
        let slot = sequence_slot(
            self.my_points.len(),
            index,
            "HatchGen_PointOnHatching::Point",
        )?;
        Ok(&self.my_points[slot])
    }

    /// RemPoint(Index): removes the `index`-th point on element; Standard_OutOfRange
    /// outside [1, NbPoints]; `point.rem_point(1)?`.
    pub fn rem_point(&mut self, index: i32) -> Result<(), Failure> {
        let slot = sequence_slot(
            self.my_points.len(),
            index,
            "HatchGen_PointOnHatching::RemPoint",
        )?;
        self.my_points.remove(slot);
        Ok(())
    }

    /// The points on element in their order, for the hatcher's merges and removals.
    pub(crate) fn element_points(&self) -> impl Iterator<Item = &PointOnElement> {
        self.my_points.iter()
    }

    /// ClrPoints: removes every point on element; `point.clr_points()`.
    pub fn clr_points(&mut self) {
        self.my_points.clear();
    }

    /// IsLower(Point, Confusion): `point.parameter() - self.parameter() > confusion`;
    /// `a.is_lower(&b, 1e-9)`.
    pub fn is_lower(&self, point: &PointOnHatching, confusion: f64) -> bool {
        point.base.my_param - self.base.my_param > confusion
    }

    /// IsEqual(Point, Confusion): the parameters differ by at most `confusion`;
    /// `a.is_equal(&b, 1e-9)`.
    pub fn is_equal(&self, point: &PointOnHatching, confusion: f64) -> bool {
        (point.base.my_param - self.base.my_param).abs() <= confusion
    }

    /// IsGreater(Point, Confusion): `self.parameter() - point.parameter() > confusion`;
    /// `a.is_greater(&b, 1e-9)`.
    pub fn is_greater(&self, point: &PointOnHatching, confusion: f64) -> bool {
        self.base.my_param - point.base.my_param > confusion
    }
}

impl IntersectionPointDump for PointOnHatching {
    /// Dump(Index): the native text of the point and of its points on element.
    fn dump(&self, index: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        let b = &self.base;
        writeln!(
            out,
            "--- Point on hatching {}------------------",
            index_title(index, "------")
        )?;
        writeln!(out, "    Index of the hatching = {}", b.my_index)?;
        writeln!(
            out,
            "    Parameter on hatching = {}",
            stream_real(b.my_param)
        )?;
        writeln!(
            out,
            "    Position  on hatching = {}",
            position_text(b.my_posit)
        )?;
        writeln!(
            out,
            "    State Before          = {}",
            state_text(b.my_before)
        )?;
        writeln!(
            out,
            "    State After           = {}",
            state_text(b.my_after)
        )?;
        writeln!(
            out,
            "    Beginning of segment  = {}",
            flag_text(b.my_seg_beg)
        )?;
        writeln!(
            out,
            "    End       of segment  = {}",
            flag_text(b.my_seg_end)
        )?;
        self.dump_element_points(out)?;
        writeln!(out, "----------------------------------------------")
    }
}

impl PointOnHatching {
    fn dump_element_points(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        if self.my_points.is_empty() {
            return writeln!(out, "    No points on element");
        }
        writeln!(
            out,
            "    Contains {} points on element",
            self.my_points.len()
        )?;
        for (slot, point) in self.my_points.iter().enumerate() {
            point.dump(slot as i32 + 1, out)?;
        }
        Ok(())
    }
}
