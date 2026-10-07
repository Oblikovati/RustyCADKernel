// Rust translation of ChFiDS_TypeOfConcavity.hxx, ChFiDS_State.hxx, ChFiDS_ErrorStatus.hxx,
// ChFiDS_ChamfMethod.hxx and ChFiDS_ChamfMode.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ChFiDS enumerations: connection, extremity, error and chamfer classifications.

/// Native connection classification; `TypeOfConcavity::Convex as i32 == 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TypeOfConcavity {
    /// A concave connection.
    Concave = 0,
    /// A convex connection.
    Convex = 1,
    /// A tangential connection.
    Tangential = 2,
    /// An edge bounding a single face.
    FreeBound = 3,
    /// A connection without a more specific classification.
    Other = 4,
    /// An interval containing different connection classifications.
    Mixed = 5,
}

/// The state of a spine extremity (`ChFiDS_State`); `State::Closed as i32 == 5`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum State {
    /// The extremity ends on the same face on both sides.
    OnSame = 0,
    /// The extremity ends on different faces.
    OnDiff = 1,
    /// Every edge at the vertex is filleted.
    AllSame = 2,
    /// The extremity is a break point of the guideline.
    BreakPoint = 3,
    /// The extremity lies on a free boundary.
    FreeBoundary = 4,
    /// The guideline is a closed contour.
    Closed = 5,
    /// The extremity is a tangency point.
    Tangent = 6,
}

/// The error status of a spine computation (`ChFiDS_ErrorStatus`);
/// `ErrorStatus::TwistedSurface as i32 == 4`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorStatus {
    /// The computation succeeded.
    Ok = 0,
    /// A generic error.
    Error = 1,
    /// The walking along the guideline failed.
    WalkingFailure = 2,
    /// No start solution was found.
    StartsolFailure = 3,
    /// The computed surface is twisted.
    TwistedSurface = 4,
}

/// The method that defines a chamfer (`ChFiDS_ChamfMethod`); `ChamfMethod::DistAngle as i32 == 2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ChamfMethod {
    /// One distance on both faces.
    Sym = 0,
    /// One distance on each face.
    TwoDist = 1,
    /// One distance and one angle.
    DistAngle = 2,
}

/// The mode of a chamfer (`ChFiDS_ChamfMode`); `ChamfMode::ConstThroatChamfer as i32 == 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ChamfMode {
    /// A chamfer with a constant distance from the spine to one of the two surfaces.
    ClassicChamfer = 0,
    /// A symmetric chamfer with a constant throat: the height of the isosceles section.
    ConstThroatChamfer = 1,
    /// A chamfer with a constant throat whose section is a right-angled triangle: the first
    /// surface is offset inside the solid, the apex lies on the intersection of the offset
    /// surface and the second surface, and the leg from the apex to the top is the throat.
    ConstThroatWithPenetrationChamfer = 2,
}
