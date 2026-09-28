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
// Rust translation of gce_ErrorType.hxx and gce_Root.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction status shared by the gce and GC algorithms.
use rustycad_tkernel::standard::Failure;

/// gce_ErrorType: why a construction algorithm failed, or `Done`.
///
/// The discriminants are the native enumerator values.
/// `assert_eq!(ErrorType::NullAxis as i32, 5)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ErrorType {
    /// gce_Done: the construction succeeded. It is also the value that an
    /// algorithm whose native constructor never assigns the status reads (see
    /// [`Root`]).
    #[default]
    Done = 0,
    /// gce_ConfusedPoints: two points coincide.
    ConfusedPoints = 1,
    /// gce_NegativeRadius: a radius is negative.
    NegativeRadius = 2,
    /// gce_ColinearPoints: three points are collinear.
    ColinearPoints = 3,
    /// gce_IntersectionError: an intersection has no result.
    IntersectionError = 4,
    /// gce_NullAxis: an axis has a null length.
    NullAxis = 5,
    /// gce_NullAngle: an angle is null.
    NullAngle = 6,
    /// gce_NullRadius: a radius is null.
    NullRadius = 7,
    /// gce_InvertAxis: the axes are inverted.
    InvertAxis = 8,
    /// gce_BadAngle: an angle is out of range.
    BadAngle = 9,
    /// gce_InvertRadius: the major radius is smaller than the minor one.
    InvertRadius = 10,
    /// gce_NullFocusLength: the focal length is negative.
    NullFocusLength = 11,
    /// gce_NullVector: a vector has a null length.
    NullVector = 12,
    /// gce_BadEquation: the plane equation has no normal.
    BadEquation = 13,
}

impl ErrorType {
    /// Every enumerator in native order; `ErrorType::ALL[13] == ErrorType::BadEquation`.
    pub const ALL: [Self; 14] = [
        Self::Done,
        Self::ConfusedPoints,
        Self::NegativeRadius,
        Self::ColinearPoints,
        Self::IntersectionError,
        Self::NullAxis,
        Self::NullAngle,
        Self::NullRadius,
        Self::InvertAxis,
        Self::BadAngle,
        Self::InvertRadius,
        Self::NullFocusLength,
        Self::NullVector,
        Self::BadEquation,
    ];

    /// The native enumerator name; `ErrorType::Done.name() == "gce_Done"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Done => "gce_Done",
            Self::ConfusedPoints => "gce_ConfusedPoints",
            Self::NegativeRadius => "gce_NegativeRadius",
            Self::ColinearPoints => "gce_ColinearPoints",
            Self::IntersectionError => "gce_IntersectionError",
            Self::NullAxis => "gce_NullAxis",
            Self::NullAngle => "gce_NullAngle",
            Self::NullRadius => "gce_NullRadius",
            Self::InvertAxis => "gce_InvertAxis",
            Self::BadAngle => "gce_BadAngle",
            Self::InvertRadius => "gce_InvertRadius",
            Self::NullFocusLength => "gce_NullFocusLength",
            Self::NullVector => "gce_NullVector",
            Self::BadEquation => "gce_BadEquation",
        }
    }
}

/// gce_Root: the status of a gce construction algorithm.
///
/// Every native constructor assigns the status except in the paths that
/// docs/tkgeombase-progress.md lists; there the port reads [`ErrorType::Done`],
/// the value that the zeroing OCCT allocator gives a heap-allocated algorithm.
/// `assert!(MakeDir::from_coordinates(1.0, 0.0, 0.0)?.is_done())`.
pub trait Root {
    /// gce_Root::Status: `Done`, or the reason why the construction failed.
    fn status(&self) -> ErrorType;
    /// gce_Root::IsDone: true when the construction succeeded.
    fn is_done(&self) -> bool {
        self.status() == ErrorType::Done
    }
    /// gce_Root::IsError: true when the construction failed.
    fn is_error(&self) -> bool {
        self.status() != ErrorType::Done
    }
}

/// The StdFail_NotDone check of every `Value()`: `member` is the native member
/// name, such as `gce_MakeCirc::Value()`, and `status` the construction status.
pub(crate) fn check_done(status: ErrorType, member: &str) -> Result<(), Failure> {
    if status == ErrorType::Done {
        return Ok(());
    }
    let message = format!(
        "{member} - no result (status {}; expected gce_Done)",
        status.name()
    );
    Err(Failure::with_type("StdFail_NotDone", &message, ""))
}
