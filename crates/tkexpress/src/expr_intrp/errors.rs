// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_SyntaxError.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExprIntrp_SyntaxError` (a Standard_Failure natively), as a [`Failure`] value.
use rustycad_tkernel::standard::Failure;

/// `ExprIntrp_SyntaxError`; `syntax_error("syntax error")`.
pub fn syntax_error(message: &str) -> Failure {
    Failure::with_type("ExprIntrp_SyntaxError", message, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_failure_keeps_its_native_type_and_message() {
        let failure: Failure = syntax_error("syntax error");
        assert_eq!(failure.exception_type(), "ExprIntrp_SyntaxError");
        assert_eq!(failure.message(), "syntax error");
    }
}
