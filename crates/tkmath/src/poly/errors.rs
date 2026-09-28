//! Native exception types for the Poly package, carrying the offending value.
use rustycad_tkernel::standard::Failure;
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
