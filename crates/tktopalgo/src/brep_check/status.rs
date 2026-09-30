// Created on: 1995-12-06
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Status.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Status`: the defects a checker records, and the list type of the results.
use rustycad_tkernel::ncollection::List;

/// `BRepCheck_Status`: one defect of a shape (or `NoError`), in the native order, so
/// that `status as i32` is the native enumerator value.
///
/// ```
/// use rustycad_tktopalgo::brep_check::BRepCheckStatus;
/// assert_eq!(BRepCheckStatus::CheckFail as i32, 36);
/// assert_eq!(BRepCheckStatus::from_index(13), Some(BRepCheckStatus::FreeEdge));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRepCheckStatus {
    /// `BRepCheck_NoError`.
    NoError,
    /// `BRepCheck_InvalidPointOnCurve`.
    InvalidPointOnCurve,
    /// `BRepCheck_InvalidPointOnCurveOnSurface`.
    InvalidPointOnCurveOnSurface,
    /// `BRepCheck_InvalidPointOnSurface`.
    InvalidPointOnSurface,
    /// `BRepCheck_No3DCurve`.
    No3DCurve,
    /// `BRepCheck_Multiple3DCurve`.
    Multiple3DCurve,
    /// `BRepCheck_Invalid3DCurve`.
    Invalid3DCurve,
    /// `BRepCheck_NoCurveOnSurface`.
    NoCurveOnSurface,
    /// `BRepCheck_InvalidCurveOnSurface`.
    InvalidCurveOnSurface,
    /// `BRepCheck_InvalidCurveOnClosedSurface`.
    InvalidCurveOnClosedSurface,
    /// `BRepCheck_InvalidSameRangeFlag`.
    InvalidSameRangeFlag,
    /// `BRepCheck_InvalidSameParameterFlag`.
    InvalidSameParameterFlag,
    /// `BRepCheck_InvalidDegeneratedFlag`.
    InvalidDegeneratedFlag,
    /// `BRepCheck_FreeEdge`.
    FreeEdge,
    /// `BRepCheck_InvalidMultiConnexity`.
    InvalidMultiConnexity,
    /// `BRepCheck_InvalidRange`.
    InvalidRange,
    /// `BRepCheck_EmptyWire`.
    EmptyWire,
    /// `BRepCheck_RedundantEdge`.
    RedundantEdge,
    /// `BRepCheck_SelfIntersectingWire`.
    SelfIntersectingWire,
    /// `BRepCheck_NoSurface`.
    NoSurface,
    /// `BRepCheck_InvalidWire`.
    InvalidWire,
    /// `BRepCheck_RedundantWire`.
    RedundantWire,
    /// `BRepCheck_IntersectingWires`.
    IntersectingWires,
    /// `BRepCheck_InvalidImbricationOfWires`.
    InvalidImbricationOfWires,
    /// `BRepCheck_EmptyShell`.
    EmptyShell,
    /// `BRepCheck_RedundantFace`.
    RedundantFace,
    /// `BRepCheck_InvalidImbricationOfShells`.
    InvalidImbricationOfShells,
    /// `BRepCheck_UnorientableShape`.
    UnorientableShape,
    /// `BRepCheck_NotClosed`.
    NotClosed,
    /// `BRepCheck_NotConnected`.
    NotConnected,
    /// `BRepCheck_SubshapeNotInShape`.
    SubshapeNotInShape,
    /// `BRepCheck_BadOrientation`.
    BadOrientation,
    /// `BRepCheck_BadOrientationOfSubshape`.
    BadOrientationOfSubshape,
    /// `BRepCheck_InvalidPolygonOnTriangulation`.
    InvalidPolygonOnTriangulation,
    /// `BRepCheck_InvalidToleranceValue`.
    InvalidToleranceValue,
    /// `BRepCheck_EnclosedRegion`.
    EnclosedRegion,
    /// `BRepCheck_CheckFail`.
    CheckFail,
}

/// `NCollection_List<BRepCheck_Status>`: the statuses of one shape in one context.
pub type ListOfStatus = List<BRepCheckStatus>;

impl BRepCheckStatus {
    /// Every enumerator in native order: `ALL[i] as i32 == i`.
    pub const ALL: [BRepCheckStatus; 37] = [
        Self::NoError,
        Self::InvalidPointOnCurve,
        Self::InvalidPointOnCurveOnSurface,
        Self::InvalidPointOnSurface,
        Self::No3DCurve,
        Self::Multiple3DCurve,
        Self::Invalid3DCurve,
        Self::NoCurveOnSurface,
        Self::InvalidCurveOnSurface,
        Self::InvalidCurveOnClosedSurface,
        Self::InvalidSameRangeFlag,
        Self::InvalidSameParameterFlag,
        Self::InvalidDegeneratedFlag,
        Self::FreeEdge,
        Self::InvalidMultiConnexity,
        Self::InvalidRange,
        Self::EmptyWire,
        Self::RedundantEdge,
        Self::SelfIntersectingWire,
        Self::NoSurface,
        Self::InvalidWire,
        Self::RedundantWire,
        Self::IntersectingWires,
        Self::InvalidImbricationOfWires,
        Self::EmptyShell,
        Self::RedundantFace,
        Self::InvalidImbricationOfShells,
        Self::UnorientableShape,
        Self::NotClosed,
        Self::NotConnected,
        Self::SubshapeNotInShape,
        Self::BadOrientation,
        Self::BadOrientationOfSubshape,
        Self::InvalidPolygonOnTriangulation,
        Self::InvalidToleranceValue,
        Self::EnclosedRegion,
        Self::CheckFail,
    ];

    /// The enumerator of native value `index`, `None` outside `[0, 36]`;
    /// `BRepCheckStatus::from_index(0) == Some(BRepCheckStatus::NoError)`.
    pub fn from_index(index: i32) -> Option<Self> {
        usize::try_from(index)
            .ok()
            .and_then(|i| Self::ALL.get(i).copied())
    }

    /// The enumerator name `BRepCheck::Print` writes, without the line end;
    /// `BRepCheckStatus::NotClosed.name() == "BRepCheck_NotClosed"`.
    pub fn name(self) -> &'static str {
        STATUS_NAMES[self as usize]
    }
}

/// The names of the enumerators, in native order.
const STATUS_NAMES: [&str; 37] = [
    "BRepCheck_NoError",
    "BRepCheck_InvalidPointOnCurve",
    "BRepCheck_InvalidPointOnCurveOnSurface",
    "BRepCheck_InvalidPointOnSurface",
    "BRepCheck_No3DCurve",
    "BRepCheck_Multiple3DCurve",
    "BRepCheck_Invalid3DCurve",
    "BRepCheck_NoCurveOnSurface",
    "BRepCheck_InvalidCurveOnSurface",
    "BRepCheck_InvalidCurveOnClosedSurface",
    "BRepCheck_InvalidSameRangeFlag",
    "BRepCheck_InvalidSameParameterFlag",
    "BRepCheck_InvalidDegeneratedFlag",
    "BRepCheck_FreeEdge",
    "BRepCheck_InvalidMultiConnexity",
    "BRepCheck_InvalidRange",
    "BRepCheck_EmptyWire",
    "BRepCheck_RedundantEdge",
    "BRepCheck_SelfIntersectingWire",
    "BRepCheck_NoSurface",
    "BRepCheck_InvalidWire",
    "BRepCheck_RedundantWire",
    "BRepCheck_IntersectingWires",
    "BRepCheck_InvalidImbricationOfWires",
    "BRepCheck_EmptyShell",
    "BRepCheck_RedundantFace",
    "BRepCheck_InvalidImbricationOfShells",
    "BRepCheck_UnorientableShape",
    "BRepCheck_NotClosed",
    "BRepCheck_NotConnected",
    "BRepCheck_SubshapeNotInShape",
    "BRepCheck_BadOrientation",
    "BRepCheck_BadOrientationOfSubshape",
    "BRepCheck_InvalidPolygonOnTriangulation",
    "BRepCheck_InvalidToleranceValue",
    "BRepCheck_EnclosedRegion",
    "BRepCheck_CheckFail",
];
