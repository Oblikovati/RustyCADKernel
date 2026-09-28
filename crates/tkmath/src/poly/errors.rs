//! Native exception types for the Poly package, carrying the offending value.
use rustycad_tkernel::{ncollection::ArrayError, standard::Failure};
/// Maps an `NCollection_Array1` error to the exception the native array raises;
/// `array_failure("Poly_Triangulation::Triangle", error)`.
pub(crate) fn array_failure(member: &str, error: ArrayError) -> Failure {
    let kind = match error {
        ArrayError::Bounds { .. } => "Standard_RangeError",
        ArrayError::Index { .. } => "Standard_OutOfRange",
        ArrayError::ShapeMismatch { .. } | ArrayError::DimensionMismatch { .. } => {
            "Standard_DimensionMismatch"
        }
    };
    Failure::with_type(kind, &format!("{member}: {error}"), "")
}
/// Prefixes a failure from a nested array with the Poly member that indexed it;
/// `in_member("Poly_Triangulation::Node", failure)`.
pub(crate) fn in_member(member: &str, failure: Failure) -> Failure {
    Failure::with_type(
        failure.exception_type(),
        &format!("{member}: {}", failure.message()),
        "",
    )
}
/// Converts a native one-based index to the zero-based offset of an aliased array; a
/// non-positive index becomes an offset that the array rejects; `zero_based(1) == 0`.
pub(crate) fn zero_based(index: i32) -> usize {
    usize::try_from(i64::from(index) - 1).unwrap_or(usize::MAX)
}
