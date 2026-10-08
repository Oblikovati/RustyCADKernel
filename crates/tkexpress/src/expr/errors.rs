// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_ExprFailure.hxx, Expr_NotEvaluable.hxx,
// Expr_NotAssigned.hxx, Expr_InvalidAssignment.hxx, Expr_InvalidFunction.hxx and
// Expr_InvalidOperand.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exceptions of the Expr package, as [`Failure`] values.
//!
//! `Expr_ExprFailure` derives from Standard_Failure natively and the five others derive
//! from `Expr_ExprFailure`; the failure keeps the most derived name. OCCT throws them
//! without a message; the port adds the offending value to the message.
use rustycad_tkernel::standard::Failure;

/// `Expr_ExprFailure`; `expr_failure("Expr: unexpected state")`.
pub fn expr_failure(message: &str) -> Failure {
    Failure::with_type("Expr_ExprFailure", message, "")
}
/// `Expr_NotEvaluable`; `not_evaluable("Expr_NamedUnknown::Evaluate: x not in vars")`.
pub fn not_evaluable(message: &str) -> Failure {
    Failure::with_type("Expr_NotEvaluable", message, "")
}
/// `Expr_NotAssigned`; `not_assigned("Expr_NamedUnknown::AssignedExpression: x")`.
pub fn not_assigned(message: &str) -> Failure {
    Failure::with_type("Expr_NotAssigned", message, "")
}
/// `Expr_InvalidAssignment`; `invalid_assignment("Expr_NamedUnknown::Assign: x+1 contains x")`.
pub fn invalid_assignment(message: &str) -> Failure {
    Failure::with_type("Expr_InvalidAssignment", message, "")
}
/// `Expr_InvalidFunction`; `invalid_function("Expr_UnaryFunction: 2 variables; expected 1")`.
pub fn invalid_function(message: &str) -> Failure {
    Failure::with_type("Expr_InvalidFunction", message, "")
}
/// `Expr_InvalidOperand`; `invalid_operand("SetOperand: the operand contains the expression")`.
pub fn invalid_operand(message: &str) -> Failure {
    Failure::with_type("Expr_InvalidOperand", message, "")
}
/// `Standard_OutOfRange`; `out_of_range("SubExpression: I = 3; expected 1")`.
pub fn out_of_range(message: &str) -> Failure {
    Failure::out_of_range(message)
}
/// `Standard_DimensionMismatch`; `dimension_mismatch("RemoveOperand: 2 operands; expected > 2")`.
pub fn dimension_mismatch(message: &str) -> Failure {
    Failure::with_type("Standard_DimensionMismatch", message, "")
}
/// `Standard_NoSuchObject`; `no_such_object("Expr_RelationIterator::Value: no more relation")`.
pub fn no_such_object(message: &str) -> Failure {
    Failure::with_type("Standard_NoSuchObject", message, "")
}
/// `Standard_NoMoreObject`; `no_more_object("Expr_UnknownIterator::Next: past the end")`.
pub fn no_more_object(message: &str) -> Failure {
    Failure::with_type("Standard_NoMoreObject", message, "")
}
/// `Standard_NullObject`: a member called through a null handle, which dereferences a
/// null pointer natively; `null_object("Expr_Sum::Evaluate: null operand 2")`.
pub fn null_object(message: &str) -> Failure {
    Failure::with_type("Standard_NullObject", message, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_constructor_keeps_its_native_type() {
        let cases: [(Failure, &str); 11] = [
            (expr_failure("a"), "Expr_ExprFailure"),
            (not_evaluable("a"), "Expr_NotEvaluable"),
            (not_assigned("a"), "Expr_NotAssigned"),
            (invalid_assignment("a"), "Expr_InvalidAssignment"),
            (invalid_function("a"), "Expr_InvalidFunction"),
            (invalid_operand("a"), "Expr_InvalidOperand"),
            (out_of_range("a"), "Standard_OutOfRange"),
            (dimension_mismatch("a"), "Standard_DimensionMismatch"),
            (no_such_object("a"), "Standard_NoSuchObject"),
            (no_more_object("a"), "Standard_NoMoreObject"),
            (null_object("a"), "Standard_NullObject"),
        ];
        for (failure, name) in cases {
            assert_eq!(failure.exception_type(), name);
            assert_eq!(failure.message(), "a");
        }
    }
}
