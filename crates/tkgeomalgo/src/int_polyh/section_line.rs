// Created on: 1999-04-06
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_SectionLine.hxx and
// IntPolyh_SectionLine.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_SectionLine`: a chain of start points, 0-based, over a sequence that always
//! holds one spare point after the last one.
use super::array::IntPolyhDumpItem;
use super::start_point::IntPolyhStartPoint;
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// `IntPolyh_SectionLine`: the start points of one section line. `Init` puts the spare
/// point in an empty line; `NbStartPoints()` is the sequence length minus one, so the
/// spare point is `Value(NbStartPoints())`, written then followed by a new spare one.
///
/// ```
/// use rustycad_tkgeomalgo::int_polyh::IntPolyhSectionLine;
/// let mut line = IntPolyhSectionLine::with_size(10);
/// assert_eq!((line.nb_start_points(), line.get_n()), (0, 1));
/// line.increment_nb_start_points();
/// assert_eq!(line.nb_start_points(), 1);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IntPolyhSectionLine {
    seq_of_s_points: Vec<IntPolyhStartPoint>,
}

impl IntPolyhSectionLine {
    /// The empty constructor: no point, not even the spare one (`NbStartPoints()` is -1).
    /// `IntPolyhSectionLine::new().nb_start_points() == -1`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `IntPolyh_SectionLine(nn)`: `Init(nn)`. `IntPolyhSectionLine::with_size(10)`.
    pub fn with_size(nn: i32) -> Self {
        let mut line: Self = Self::new();
        line.init(nn);
        line
    }

    /// `Init(nn)`: the spare point of an empty line; `nn` is not read.
    /// `line.init(10)`.
    pub fn init(&mut self, _nn: i32) {
        if self.seq_of_s_points.is_empty() {
            self.increment_nb_start_points();
        }
    }

    /// `Value(nn)` (also `operator[] const`): Standard_OutOfRange outside
    /// `0..=NbStartPoints()`. `line.value(0)?`.
    pub fn value(&self, index: i32) -> Result<&IntPolyhStartPoint, Failure> {
        let len: usize = self.seq_of_s_points.len();
        usize::try_from(index)
            .ok()
            .and_then(|i| self.seq_of_s_points.get(i))
            .ok_or_else(|| range_error("Value", index, len))
    }

    /// `ChangeValue(nn)` (also `operator[]`): Standard_OutOfRange outside
    /// `0..=NbStartPoints()`. `*line.change_value(0)? = point`.
    pub fn change_value(&mut self, index: i32) -> Result<&mut IntPolyhStartPoint, Failure> {
        let len: usize = self.seq_of_s_points.len();
        usize::try_from(index)
            .ok()
            .and_then(|i| self.seq_of_s_points.get_mut(i))
            .ok_or_else(|| range_error("ChangeValue", index, len))
    }

    /// `Copy(Other)` (also `operator=`). `line.copy_from(&other)`.
    pub fn copy_from(&mut self, other: &IntPolyhSectionLine) -> &mut Self {
        self.seq_of_s_points.clone_from(&other.seq_of_s_points);
        self
    }

    /// `GetN()`: the sequence length. `line.get_n()`.
    pub fn get_n(&self) -> i32 {
        self.seq_of_s_points.len() as i32
    }

    /// `NbStartPoints()`: the sequence length minus the spare point. `line.nb_start_points()`.
    pub fn nb_start_points(&self) -> i32 {
        self.seq_of_s_points.len() as i32 - 1
    }

    /// `IncrementNbStartPoints()`: appends a default spare point.
    /// `line.increment_nb_start_points()`.
    pub fn increment_nb_start_points(&mut self) {
        self.seq_of_s_points.push(IntPolyhStartPoint::default());
    }

    /// `Destroy()`: does nothing (the former manual storage is gone). `line.destroy()`.
    pub fn destroy(&mut self) {}

    /// `Prepend(SP)`. `line.prepend(&point)`.
    pub fn prepend(&mut self, sp: &IntPolyhStartPoint) {
        self.seq_of_s_points.insert(0, *sp);
    }

    /// `Dump()`: the native `printf` text of every start point (not the spare one).
    /// `line.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        write!(out, "\n SectionLine 0-> {}", self.nb_start_points() - 1)?;
        for i in 0..self.nb_start_points() {
            self.seq_of_s_points[i as usize].dump_index(i, out)?;
        }
        writeln!(out)
    }
}

/// The Standard_OutOfRange of `NCollection_Sequence::Value(nn + 1)`.
fn range_error(member: &str, index: i32, len: usize) -> Failure {
    Failure::out_of_range(&format!(
        "IntPolyh_SectionLine::{member}() - index {index} is outside the range 0..{len}"
    ))
}

impl IntPolyhDumpItem for IntPolyhSectionLine {
    fn dump_item(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        self.dump(out)
    }
}
