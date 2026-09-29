// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of the mySequence member of ProjLib_CompProjectedCurve
// (NCollection_HSequence<occ::handle<NCollection_HSequence<gp_Pnt>>>)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuous parts of the projection: a handle-shared sequence of
//! handle-shared sequences of `(t, u, v)` triples stored as gp_Pnt, with the
//! 1-based accessors the algorithm uses.
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;

/// `NCollection_HSequence<occ::handle<NCollection_HSequence<gp_Pnt>>>` (formerly
/// `ProjLib_HSequenceOfHSequenceOfPnt`): one sequence of `(t, u, v)` per part.
pub type SequenceOfSequenceOfPnt = HSequence<Handle<HSequence<Point3>>>;

fn out_of_range(what: &str, index: i32, length: i32) -> Failure {
    failure(
        "Standard_OutOfRange",
        &format!("ProjLib_CompProjectedCurve: {what} index {index}; expected 1..{length}"),
    )
}

/// The 1-based accessors of the part sequence.
pub(super) trait Parts {
    /// The number of parts.
    fn nb_parts(&self) -> i32;
    /// The part of index `i` (1-based).
    fn part(&self, i: i32) -> Result<Handle<HSequence<Point3>>, Failure>;
    /// The triples of part `i`.
    fn points(&self, i: i32) -> Result<Vec<Point3>, Failure>;
    /// Every part as plain triples.
    fn all_points(&self) -> Result<Vec<Vec<Point3>>, Failure>;
    /// Appends a new empty part.
    fn push_part(&self);
    /// Appends a triple to part `i`.
    fn push_point(&self, i: i32, point: Point3) -> Result<(), Failure>;
    /// The last triple of part `i`.
    fn last_point(&self, i: i32) -> Result<Point3, Failure>;
    /// Sets the parameter of the first triple of part `i`.
    fn set_first_x(&self, i: i32, x: f64) -> Result<(), Failure>;
    /// Removes part `i`.
    fn remove_part(&self, i: i32) -> Result<(), Failure>;
    /// Removes every part.
    fn clear_parts(&self);
}

impl Parts for SequenceOfSequenceOfPnt {
    fn nb_parts(&self) -> i32 {
        self.sequence(|parts| parts.length())
    }
    fn part(&self, i: i32) -> Result<Handle<HSequence<Point3>>, Failure> {
        let length: i32 = self.nb_parts();
        if i < 1 || i > length {
            return Err(out_of_range("part", i, length));
        }
        self.sequence(|parts| parts.value(i as usize).cloned())
    }
    fn points(&self, i: i32) -> Result<Vec<Point3>, Failure> {
        let part: Handle<HSequence<Point3>> = self.part(i)?;
        let seq: &HSequence<Point3> = part.get().ok_or_else(|| out_of_range("null part", i, 0))?;
        Ok(seq.sequence(|points| points.iter().copied().collect()))
    }
    fn all_points(&self) -> Result<Vec<Vec<Point3>>, Failure> {
        (1..=self.nb_parts()).map(|i| self.points(i)).collect()
    }
    fn push_part(&self) {
        self.append(Handle::new(HSequence::<Point3>::default()));
    }
    fn push_point(&self, i: i32, point: Point3) -> Result<(), Failure> {
        let part: Handle<HSequence<Point3>> = self.part(i)?;
        let seq: &HSequence<Point3> = part.get().ok_or_else(|| out_of_range("null part", i, 0))?;
        seq.append(point);
        Ok(())
    }
    fn last_point(&self, i: i32) -> Result<Point3, Failure> {
        let points: Vec<Point3> = self.points(i)?;
        points
            .last()
            .copied()
            .ok_or_else(|| out_of_range("empty part", i, 0))
    }
    fn set_first_x(&self, i: i32, x: f64) -> Result<(), Failure> {
        let part: Handle<HSequence<Point3>> = self.part(i)?;
        let seq: &HSequence<Point3> = part.get().ok_or_else(|| out_of_range("null part", i, 0))?;
        seq.change_sequence(|points| points.change_value(1).map(|point| point.set_x(x)))
    }
    fn remove_part(&self, i: i32) -> Result<(), Failure> {
        let length: i32 = self.nb_parts();
        if i < 1 || i > length {
            return Err(out_of_range("part", i, length));
        }
        self.change_sequence(|parts| parts.remove(i as usize))
    }
    fn clear_parts(&self) {
        self.change_sequence(|parts| parts.clear(None));
    }
}
