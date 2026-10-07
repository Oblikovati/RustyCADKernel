// Rust translation of DataExchange/TKXSBase/Interface/Interface_InterfaceError.hxx,
// Interface_CheckFailure.hxx and Interface_InterfaceMismatch.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exceptions of the Interface package, as [`Failure`] values with their native
//! names: `Interface_InterfaceError` (a `Standard_Failure`), and its subclasses
//! `Interface_CheckFailure` and `Interface_InterfaceMismatch`.
use rustycad_tkernel::standard::Failure;

/// `Interface_InterfaceError`; `interface_error("TypedEntity: none found")`.
pub fn interface_error(message: &str) -> Failure {
    Failure::with_type("Interface_InterfaceError", message, "")
}

/// `Interface_CheckFailure`; `check_failure("Fail on entity 3")`.
pub fn check_failure(message: &str) -> Failure {
    Failure::with_type("Interface_CheckFailure", message, "")
}

/// `Interface_InterfaceMismatch`; `interface_mismatch("model does not suit the protocol")`.
pub fn interface_mismatch(message: &str) -> Failure {
    Failure::with_type("Interface_InterfaceMismatch", message, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_names() {
        assert_eq!(
            interface_error("x").exception_type(),
            "Interface_InterfaceError"
        );
        assert_eq!(
            check_failure("x").exception_type(),
            "Interface_CheckFailure"
        );
        assert_eq!(
            interface_mismatch("x").exception_type(),
            "Interface_InterfaceMismatch"
        );
    }
}
