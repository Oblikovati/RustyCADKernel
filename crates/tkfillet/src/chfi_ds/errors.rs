//! The native exceptions that the ChFiDS classes raise, as named failures.
use rustycad_tkernel::standard::Failure;

/// `Standard_DomainError(message)`; `domain_error("CommonPoint not on Arc")`.
pub(crate) fn domain_error(message: &str) -> Failure {
    Failure::with_message("Standard_DomainError", message)
}

/// `Standard_Failure(message)`; `standard_failure("Non-periodic Spine")`.
pub(crate) fn standard_failure(message: &str) -> Failure {
    Failure::new(message)
}

/// `Standard_OutOfRange(message)` for a 1-based index outside `[1, length]`;
/// `out_of_range("ChFiDS_Spine::Edges", 3, 2)`.
pub(crate) fn out_of_range(member: &str, index: i32, length: i32) -> Failure {
    Failure::out_of_range(&format!(
        "{member}: index {index}; expected an index in [1, {length}]"
    ))
}

/// `Standard_NullObject` for a member that dereferences a null handle;
/// `null_object("ChFiDS_Stripe::Reset: null spine")`.
pub(crate) fn null_object(message: &str) -> Failure {
    Failure::with_message("Standard_NullObject", message)
}
