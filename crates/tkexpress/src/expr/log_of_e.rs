// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_LogOfe.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_LogOfe`: The natural logarithm of an operand.
use super::exponential::ExprExponential;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::package::copy_share;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::{evaluate_operand, function_members};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprLogOfe,
    "Expr_LogOfe",
    "The natural logarithm of an operand; `ExprLogOfe::new(&x).string()? == \"Ln(x)\"`."
);

impl ExprGeneralExpression for ExprLogOfe {
    unary_expression_members!();
    function_members!(ExprLogOfe, "Ln(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprExponential, _>(self, f64::ln)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, copy_share)
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.ln())
    }
}
