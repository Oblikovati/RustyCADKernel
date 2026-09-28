// Created on: 2016-06-07
// Created by: Nikolai BUKHALOV
// Copyright (c) 2016 OPEN CASCADE SAS
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

//! Bnd_Range intervals, including native void and strict boundary behavior.
use rustycad_tkernel::{
    ncollection::List,
    standard::{
        Failure,
        dump::{self, DumpOutput},
    },
};
use std::io;
mod periodic;
/// Relationship to a value or periodic value sequence; `IntersectStatus::Boundary`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum IntersectStatus {
    /// No intersection.
    Out = 0,
    /// At least one strictly interior intersection.
    In = 1,
    /// Native boundary classification (can take precedence over interior hits).
    Boundary = 2,
}
/// A closed one-dimensional range, initially void; `Range1::new(3.0,15.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range1 {
    first: f64,
    last: f64,
}
impl Default for Range1 {
    fn default() -> Self {
        Self {
            first: 0.0,
            last: -1.0,
        }
    }
}
impl Range1 {
    /// Rejects reversed bounds with the native construction error; `Range1::new(3.0,15.0)?`.
    pub fn new(first: f64, last: f64) -> Result<Self, Failure> {
        if last < first {
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                "Last < First",
                "",
            ));
        }
        Ok(Self { first, last })
    }
    /// Uses the stored order to detect void ranges; `range.is_void()`.
    pub const fn is_void(&self) -> bool {
        self.last < self.first
    }
    /// Restores the canonical void state; `range.set_void()`.
    pub fn set_void(&mut self) {
        *self = Self::default();
    }
    /// Returns signed span, including a negative void span; `range.delta()`.
    pub fn delta(&self) -> f64 {
        self.last - self.first
    }
    /// Returns both bounds or no value for void; `range.get()`.
    pub fn get(&self) -> Option<[f64; 2]> {
        (!self.is_void()).then_some([self.first, self.last])
    }
    /// Returns minimum or no value for void; `range.min()`.
    pub fn min(&self) -> Option<f64> {
        (!self.is_void()).then_some(self.first)
    }
    /// Returns maximum or no value for void; `range.max()`.
    pub fn max(&self) -> Option<f64> {
        (!self.is_void()).then_some(self.last)
    }
    /// Computes half of the sum, preserving native overflow; `range.center()`.
    pub fn center(&self) -> Option<f64> {
        (!self.is_void()).then_some(0.5 * (self.first + self.last))
    }
    /// Writes minimum only when nonvoid; `range.get_min(&mut minimum)`.
    pub fn get_min(&self, value: &mut f64) -> bool {
        write_optional(self.min(), value)
    }
    /// Writes maximum only when nonvoid; `range.get_max(&mut maximum)`.
    pub fn get_max(&self, value: &mut f64) -> bool {
        write_optional(self.max(), value)
    }
    /// Writes both outputs only when nonvoid; `range.get_bounds(&mut min,&mut max)`.
    pub fn get_bounds(&self, first: &mut f64, last: &mut f64) -> bool {
        if self.is_void() {
            return false;
        }
        *first = self.first;
        *last = self.last;
        true
    }
    /// Uses the native interpolation expression, including extrapolation; `range.get_intermediate_point(0.5,&mut middle)`.
    pub fn get_intermediate_point(&self, lambda: f64, value: &mut f64) -> bool {
        if self.is_void() {
            return false;
        }
        *value = self.first + lambda * (self.last - self.first);
        true
    }
    /// Adds one value with native NaN comparison order; `range.add(2.0)`.
    pub fn add(&mut self, value: f64) {
        if self.is_void() {
            self.first = value;
            self.last = value;
            return;
        }
        self.first = native_min(self.first, value);
        self.last = native_max(self.last, value);
    }
    /// Unconditionally encloses another nonvoid range; `range.add_range(other)`.
    pub fn add_range(&mut self, other: Self) {
        if other.is_void() {
            return;
        }
        if self.is_void() {
            *self = other;
            return;
        }
        self.first = native_min(self.first, other.first);
        self.last = native_max(self.last, other.last);
    }
    /// Keeps the common interval, retaining native noncanonical void bounds; `range.common(other)`.
    pub fn common(&mut self, other: Self) {
        if other.is_void() {
            self.set_void();
            return;
        }
        if self.is_void() {
            return;
        }
        self.first = native_max(self.first, other.first);
        self.last = native_min(self.last, other.last);
    }
    /// Merges only intersecting nonvoid intervals; `range.union(other)`.
    pub fn union(&mut self, other: Self) -> bool {
        if self.is_out_range(other) {
            return false;
        }
        self.first = native_min(self.first, other.first);
        self.last = native_max(self.last, other.last);
        true
    }
    /// Expands both bounds; negative values can make the interval void; `range.enlarge(1.0)`.
    pub fn enlarge(&mut self, delta: f64) {
        if !self.is_void() {
            self.first -= delta;
            self.last += delta;
        }
    }
    /// Copies and shifts nonvoid bounds; `range.shifted(2.0)?`.
    pub fn shifted(&self, value: f64) -> Result<Self, Failure> {
        if self.is_void() {
            return Ok(Self::default());
        }
        Self::new(self.first + value, self.last + value)
    }
    /// Shifts this range without canonicalizing void; `range.shift(2.0)`.
    pub fn shift(&mut self, value: f64) {
        if !self.is_void() {
            self.first += value;
            self.last += value;
        }
    }
    /// Raises the lower bound; `range.trim_from(3.0)`.
    pub fn trim_from(&mut self, value: f64) {
        if !self.is_void() {
            self.first = native_max(self.first, value);
        }
    }
    /// Lowers the upper bound; `range.trim_to(7.0)`.
    pub fn trim_to(&mut self, value: f64) {
        if !self.is_void() {
            self.last = native_min(self.last, value);
        }
    }
    /// Rejects outside values; NaN follows native comparisons; `range.is_out(3.0)`.
    pub fn is_out(&self, value: f64) -> bool {
        self.is_void() || value < self.first || value > self.last
    }
    /// Checks disjointness including void intervals; `range.is_out_range(other)`.
    pub fn is_out_range(&self, other: Self) -> bool {
        self.is_void() || other.is_void() || other.last < self.first || other.first > self.last
    }
    /// Includes both boundaries; `range.contains(3.0)`.
    pub fn contains(&self, value: f64) -> bool {
        !self.is_out(value)
    }
    /// Treats touching intervals as intersecting; `range.intersects(other)`.
    pub fn intersects(&self, other: Self) -> bool {
        !self.is_out_range(other)
    }
    /// Classifies a value with optional period; `range.is_intersected(5.0,4.0)`.
    pub fn is_intersected(&self, value: f64, period: f64) -> IntersectStatus {
        periodic::classify(*self, value, period)
    }
    /// Appends subranges without clearing the supplied list; `range.split(5.0,&mut list,4.0)?`.
    pub fn split(&self, value: f64, list: &mut List<Self>, period: f64) -> Result<(), Failure> {
        periodic::split(*self, value, list, period)
    }
    /// Writes native class and bound fields; `range.dump_json(&mut stream)?`.
    pub fn dump_json(&self, stream: &mut impl DumpOutput) -> io::Result<()> {
        dump::add_values_separator(stream)?;
        stream.write_fragment(b"\"className\": \"Bnd_Range\"")?;
        dump::dump_real(stream, "First", self.first)?;
        dump::dump_real(stream, "Last", self.last)
    }
}
fn write_optional(value: Option<f64>, output: &mut f64) -> bool {
    if let Some(value) = value {
        *output = value;
        true
    } else {
        false
    }
}
fn native_min(left: f64, right: f64) -> f64 {
    if right < left { right } else { left }
}
fn native_max(left: f64, right: f64) -> f64 {
    if left < right { right } else { left }
}
