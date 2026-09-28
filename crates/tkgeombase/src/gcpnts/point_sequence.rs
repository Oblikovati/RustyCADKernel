// Created on: 1995-11-02
// Created by: Jacques GOUSSARD
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the `NCollection_Sequence<double> myParams` and
// `NCollection_Sequence<gp_Pnt> myPoints` pair of GCPnts_UniformDeflection,
// GCPnts_QuasiUniformDeflection and GCPnts_TangentialDeflection at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameters and points of a deflection distribution, edited in step.
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// Two sequences indexed from 1: the parameters and their points (2D points in the
/// plane Z = 0); `let mut s = PointSequence::default(); s.append(0.0, point);`.
#[derive(Default)]
pub(super) struct PointSequence {
    params: Sequence<f64>,
    points: Sequence<Point3>,
}

// A C++ int index as a Sequence index; a negative one is Standard_OutOfRange.
fn position(index: i32, member: &str) -> Result<usize, Failure> {
    usize::try_from(index).map_err(|_| {
        let message = format!("NCollection_Sequence::{member}: index {index}, expected >= 0");
        Failure::with_type("Standard_OutOfRange", &message, "")
    })
}

impl PointSequence {
    /// `Length()`; `sequence.length()`.
    pub fn length(&self) -> i32 {
        self.params.length()
    }
    /// `Clear()` of both; `sequence.clear()`.
    pub fn clear(&mut self) {
        self.params.clear(None);
        self.points.clear(None);
    }
    /// `Append` to both; `sequence.append(0.5, point)`.
    pub fn append(&mut self, param: f64, point: Point3) {
        self.params.append(param);
        self.points.append(point);
    }
    /// `Remove(Length())` of both; `sequence.remove_last()?`.
    pub fn remove_last(&mut self) -> Result<(), Failure> {
        self.params.remove(self.params.len())?;
        self.points.remove(self.points.len())
    }
    /// `myParams(Index)`; `sequence.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        self.params.value(position(index, "Value")?).copied()
    }
    /// `myPoints(Index)`; `sequence.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        self.points.value(position(index, "Value")?).copied()
    }
    /// `myPoints.Last()`; Standard_NoSuchObject when empty; `sequence.last_point()?`.
    pub fn last_point(&self) -> Result<Point3, Failure> {
        self.points.last().copied().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "NCollection_Sequence::Last: empty sequence, expected at least one point",
                "",
            )
        })
    }
    /// `SetValue(Index, ...)` of both; `sequence.set(1, 0.5, point)?`.
    pub fn set(&mut self, index: i32, param: f64, point: Point3) -> Result<(), Failure> {
        let at: usize = position(index, "SetValue")?;
        self.params.set_value(at, param)?;
        self.points.set_value(at, point)
    }
    /// `InsertBefore(Index, ...)` of both; `sequence.insert_before(2, 0.5, point)?`.
    pub fn insert_before(&mut self, index: i32, param: f64, point: Point3) -> Result<(), Failure> {
        let at: usize = position(index, "InsertBefore")?;
        self.params.insert_before(at, param)?;
        self.points.insert_before(at, point)?;
        Ok(())
    }
    /// `InsertAfter(Index, ...)` of both; `sequence.insert_after(1, 0.5, point)?`.
    pub fn insert_after(&mut self, index: i32, param: f64, point: Point3) -> Result<(), Failure> {
        let at: usize = position(index, "InsertAfter")?;
        self.params.insert_after(at, param)?;
        self.points.insert_after(at, point)?;
        Ok(())
    }
}
