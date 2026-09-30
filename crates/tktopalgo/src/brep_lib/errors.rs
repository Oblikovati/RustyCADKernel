// Created on: 1993-12-15
// Created by: Remi LEQUETTE
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
// Rust translation of BRepLib_EdgeError.hxx, BRepLib_WireError.hxx, BRepLib_FaceError.hxx,
// BRepLib_ShellError.hxx and BRepLib_ShapeModification.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The status enumerations of the BRepLib makers, with the native discriminants, and
//! the exceptions the package raises.
use rustycad_tkernel::standard::Failure;

/// `BRepLib_EdgeError`: the status of `BRepLib_MakeEdge` and `BRepLib_MakeEdge2d`;
/// `assert_eq!(BRepLibEdgeError::LineThroughIdenticPoints as i32, 6)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepLibEdgeError {
    /// `BRepLib_EdgeDone`.
    EdgeDone = 0,
    /// `BRepLib_PointProjectionFailed`: a vertex does not project on the curve.
    PointProjectionFailed = 1,
    /// `BRepLib_ParameterOutOfRange`: a parameter is outside the curve range.
    ParameterOutOfRange = 2,
    /// `BRepLib_DifferentPointsOnClosedCurve`: two vertices on a closed curve.
    DifferentPointsOnClosedCurve = 3,
    /// `BRepLib_PointWithInfiniteParameter`: a vertex at an infinite parameter.
    PointWithInfiniteParameter = 4,
    /// `BRepLib_DifferentsPointAndParameter`: a vertex away from its parameter point.
    DifferentsPointAndParameter = 5,
    /// `BRepLib_LineThroughIdenticPoints`: a line through two identical points.
    LineThroughIdenticPoints = 6,
}

/// `BRepLib_WireError`: the status of `BRepLib_MakeWire`;
/// `assert_eq!(BRepLibWireError::NonManifoldWire as i32, 3)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepLibWireError {
    /// `BRepLib_WireDone`.
    WireDone = 0,
    /// `BRepLib_EmptyWire`: nothing was added.
    EmptyWire = 1,
    /// `BRepLib_DisconnectedWire`: an edge does not touch the wire.
    DisconnectedWire = 2,
    /// `BRepLib_NonManifoldWire`: the wire branches.
    NonManifoldWire = 3,
}

/// `BRepLib_FaceError`: the status of `BRepLib_MakeFace`;
/// `assert_eq!(BRepLibFaceError::ParametersOutOfRange as i32, 4)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepLibFaceError {
    /// `BRepLib_FaceDone`.
    FaceDone = 0,
    /// `BRepLib_NoFace`: nothing was built.
    NoFace = 1,
    /// `BRepLib_NotPlanar`: no surface was found through the wire.
    NotPlanar = 2,
    /// `BRepLib_CurveProjectionFailed` (never set by the package).
    CurveProjectionFailed = 3,
    /// `BRepLib_ParametersOutOfRange`: bounds outside the surface domain.
    ParametersOutOfRange = 4,
}

/// `BRepLib_ShellError`: the status of `BRepLib_MakeShell`;
/// `assert_eq!(BRepLibShellError::ShellParametersOutOfRange as i32, 3)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepLibShellError {
    /// `BRepLib_ShellDone`.
    ShellDone = 0,
    /// `BRepLib_EmptyShell`: the surface has no C2 interval.
    EmptyShell = 1,
    /// `BRepLib_DisconnectedShell` (never set by the package).
    DisconnectedShell = 2,
    /// `BRepLib_ShellParametersOutOfRange` (never set by the package).
    ShellParametersOutOfRange = 3,
}

/// `BRepLib_ShapeModification`: the status of a face after a construction;
/// `assert_eq!(BRepLibShapeModification::Trimmed as i32, 2)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepLibShapeModification {
    /// `BRepLib_Preserved`.
    Preserved = 0,
    /// `BRepLib_Deleted`.
    Deleted = 1,
    /// `BRepLib_Trimmed`.
    Trimmed = 2,
    /// `BRepLib_Merged`.
    Merged = 3,
    /// `BRepLib_BoundaryModified`.
    BoundaryModified = 4,
}

/// StdFail_NotDone: a result read from a maker that is not done.
pub(crate) fn not_done(message: &str) -> Failure {
    Failure::with_message("StdFail_NotDone", message)
}

/// Standard_NullObject: a null handle or shape that the native code dereferences (a
/// crash natively).
pub(crate) fn null_object(message: &str) -> Failure {
    Failure::with_message("Standard_NullObject", message)
}
