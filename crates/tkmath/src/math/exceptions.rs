//! The package's `DEFINE_STANDARD_EXCEPTION` classes as typed `Failure` constructors.
use rustycad_tkernel::standard::Failure;

/// `math_NotSquare` (a `Standard_DimensionError`), raised for non-square input;
/// `not_square("math_Matrix::Invert() - matrix is not square")`.
pub fn not_square(message: &str) -> Failure {
    Failure::with_type("math_NotSquare", message, "")
}

/// `math_SingularMatrix` (a `Standard_Failure`), raised when a factorization has no pivot;
/// `singular_matrix("")`.
pub fn singular_matrix(message: &str) -> Failure {
    Failure::with_type("math_SingularMatrix", message, "")
}
