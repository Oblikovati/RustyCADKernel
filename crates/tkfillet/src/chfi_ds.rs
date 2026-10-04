//! Fillet classifications shared with offset analysis.
// ChFiDS_TypeOfConcavity.hxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.

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
