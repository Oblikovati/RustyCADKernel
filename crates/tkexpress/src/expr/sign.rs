// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Sign.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Sign`: the sign (+1 or -1) of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::package::sign;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::{evaluate_operand, function_members, value_shallow_simplified};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprSign,
    "Expr_Sign",
    "The sign of an operand, as `Expr::Sign`; `ExprSign::new(&x).string()? == \"Sign(x)\"`."
);

impl ExprGeneralExpression for ExprSign {
    unary_expression_members!();
    function_members!(ExprSign, "Sign(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        value_shallow_simplified(self, sign)
    }
    fn derivative(&self, _x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        Ok(ExprNumericValue::expression(0.0))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        Ok(sign(evaluate_operand(self, vars, vals)?))
    }
}
