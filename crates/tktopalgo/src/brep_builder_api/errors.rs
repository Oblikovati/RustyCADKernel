// Rust translation of BRepBuilderAPI_EdgeError.hxx, BRepBuilderAPI_WireError.hxx,
// BRepBuilderAPI_FaceError.hxx, BRepBuilderAPI_ShellError.hxx, BRepBuilderAPI_PipeError.hxx,
// BRepBuilderAPI_TransitionMode.hxx and the Error() switches of the makers at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The status enumerations of the BRepBuilderAPI makers, with the native discriminants,
//! and the conversions from the BRepLib statuses that the `Error()` members make.
use crate::brep_lib::{BRepLibEdgeError, BRepLibFaceError, BRepLibShellError, BRepLibWireError};

/// `BRepBuilderAPI_EdgeError`: the status of `BRepBuilderAPI_MakeEdge` and
/// `BRepBuilderAPI_MakeEdge2d`;
/// `assert_eq!(BRepBuilderAPIEdgeError::LineThroughIdenticPoints as i32, 6)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepBuilderAPIEdgeError {
    /// `BRepBuilderAPI_EdgeDone`.
    EdgeDone = 0,
    /// `BRepBuilderAPI_PointProjectionFailed`: a point does not project on the curve.
    PointProjectionFailed = 1,
    /// `BRepBuilderAPI_ParameterOutOfRange`: a parameter is outside the curve range.
    ParameterOutOfRange = 2,
    /// `BRepBuilderAPI_DifferentPointsOnClosedCurve`: two points on a closed curve.
    DifferentPointsOnClosedCurve = 3,
    /// `BRepBuilderAPI_PointWithInfiniteParameter`: a point at an infinite parameter.
    PointWithInfiniteParameter = 4,
    /// `BRepBuilderAPI_DifferentsPointAndParameter`: a point away from its parameter.
    DifferentsPointAndParameter = 5,
    /// `BRepBuilderAPI_LineThroughIdenticPoints`: a line through two identical points.
    LineThroughIdenticPoints = 6,
}

/// `BRepBuilderAPI_WireError`: the status of `BRepBuilderAPI_MakeWire`;
/// `assert_eq!(BRepBuilderAPIWireError::NonManifoldWire as i32, 3)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepBuilderAPIWireError {
    /// `BRepBuilderAPI_WireDone`.
    WireDone = 0,
    /// `BRepBuilderAPI_EmptyWire`: nothing was added.
    EmptyWire = 1,
    /// `BRepBuilderAPI_DisconnectedWire`: the last edge does not touch the wire.
    DisconnectedWire = 2,
    /// `BRepBuilderAPI_NonManifoldWire`: the wire branches.
    NonManifoldWire = 3,
}

/// `BRepBuilderAPI_FaceError`: the status of `BRepBuilderAPI_MakeFace`;
/// `assert_eq!(BRepBuilderAPIFaceError::ParametersOutOfRange as i32, 4)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepBuilderAPIFaceError {
    /// `BRepBuilderAPI_FaceDone`.
    FaceDone = 0,
    /// `BRepBuilderAPI_NoFace`: only the empty constructor was used.
    NoFace = 1,
    /// `BRepBuilderAPI_NotPlanar`: no surface was given and the wire is not planar.
    NotPlanar = 2,
    /// `BRepBuilderAPI_CurveProjectionFailed` (not used natively).
    CurveProjectionFailed = 3,
    /// `BRepBuilderAPI_ParametersOutOfRange`: bounds outside the surface domain.
    ParametersOutOfRange = 4,
}

/// `BRepBuilderAPI_ShellError`: the status of `BRepBuilderAPI_MakeShell`;
/// `assert_eq!(BRepBuilderAPIShellError::ShellParametersOutOfRange as i32, 3)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepBuilderAPIShellError {
    /// `BRepBuilderAPI_ShellDone`.
    ShellDone = 0,
    /// `BRepBuilderAPI_EmptyShell`: nothing was built.
    EmptyShell = 1,
    /// `BRepBuilderAPI_DisconnectedShell` (not used natively).
    DisconnectedShell = 2,
    /// `BRepBuilderAPI_ShellParametersOutOfRange`: bounds outside the surface domain.
    ShellParametersOutOfRange = 3,
}

/// `BRepBuilderAPI_PipeError`: the status of the (shell) pipe constructions of
/// BRepOffsetAPI; `assert_eq!(BRepBuilderAPIPipeError::ImpossibleContact as i32, 3)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepBuilderAPIPipeError {
    /// `BRepBuilderAPI_PipeDone`.
    PipeDone = 0,
    /// `BRepBuilderAPI_PipeNotDone`.
    PipeNotDone = 1,
    /// `BRepBuilderAPI_PlaneNotIntersectGuide`.
    PlaneNotIntersectGuide = 2,
    /// `BRepBuilderAPI_ImpossibleContact`.
    ImpossibleContact = 3,
}

/// `BRepBuilderAPI_TransitionMode`: how a sweep manages the discontinuities of its spine;
/// `assert_eq!(BRepBuilderAPITransitionMode::RoundCorner as i32, 2)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepBuilderAPITransitionMode {
    /// `BRepBuilderAPI_Transformed`.
    Transformed = 0,
    /// `BRepBuilderAPI_RightCorner`.
    RightCorner = 1,
    /// `BRepBuilderAPI_RoundCorner`.
    RoundCorner = 2,
}

impl From<BRepLibEdgeError> for BRepBuilderAPIEdgeError {
    /// The switch of `BRepBuilderAPI_MakeEdge::Error()` and `MakeEdge2d::Error()`;
    /// `BRepBuilderAPIEdgeError::from(BRepLibEdgeError::EdgeDone)`.
    fn from(error: BRepLibEdgeError) -> Self {
        match error {
            BRepLibEdgeError::EdgeDone => Self::EdgeDone,
            BRepLibEdgeError::PointProjectionFailed => Self::PointProjectionFailed,
            BRepLibEdgeError::ParameterOutOfRange => Self::ParameterOutOfRange,
            BRepLibEdgeError::DifferentPointsOnClosedCurve => Self::DifferentPointsOnClosedCurve,
            BRepLibEdgeError::PointWithInfiniteParameter => Self::PointWithInfiniteParameter,
            BRepLibEdgeError::DifferentsPointAndParameter => Self::DifferentsPointAndParameter,
            BRepLibEdgeError::LineThroughIdenticPoints => Self::LineThroughIdenticPoints,
        }
    }
}

impl From<BRepLibWireError> for BRepBuilderAPIWireError {
    /// The switch of `BRepBuilderAPI_MakeWire::Error()`;
    /// `BRepBuilderAPIWireError::from(BRepLibWireError::WireDone)`.
    fn from(error: BRepLibWireError) -> Self {
        match error {
            BRepLibWireError::WireDone => Self::WireDone,
            BRepLibWireError::EmptyWire => Self::EmptyWire,
            BRepLibWireError::DisconnectedWire => Self::DisconnectedWire,
            BRepLibWireError::NonManifoldWire => Self::NonManifoldWire,
        }
    }
}

impl From<BRepLibFaceError> for BRepBuilderAPIFaceError {
    /// The switch of `BRepBuilderAPI_MakeFace::Error()`;
    /// `BRepBuilderAPIFaceError::from(BRepLibFaceError::NoFace)`.
    fn from(error: BRepLibFaceError) -> Self {
        match error {
            BRepLibFaceError::FaceDone => Self::FaceDone,
            BRepLibFaceError::NoFace => Self::NoFace,
            BRepLibFaceError::NotPlanar => Self::NotPlanar,
            BRepLibFaceError::CurveProjectionFailed => Self::CurveProjectionFailed,
            BRepLibFaceError::ParametersOutOfRange => Self::ParametersOutOfRange,
        }
    }
}

impl From<BRepLibShellError> for BRepBuilderAPIShellError {
    /// The switch of `BRepBuilderAPI_MakeShell::Error()`;
    /// `BRepBuilderAPIShellError::from(BRepLibShellError::EmptyShell)`.
    fn from(error: BRepLibShellError) -> Self {
        match error {
            BRepLibShellError::ShellDone => Self::ShellDone,
            BRepLibShellError::EmptyShell => Self::EmptyShell,
            BRepLibShellError::DisconnectedShell => Self::DisconnectedShell,
            BRepLibShellError::ShellParametersOutOfRange => Self::ShellParametersOutOfRange,
        }
    }
}
