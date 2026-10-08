// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Exponential.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Exponential`: E raised to the power of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::log_of_e::ExprLogOfe;
use super::named_unknown::ExprNamedUnknown;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::chain;
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::{evaluate_operand, function_members};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprExponential,
    "Expr_Exponential",
    "E raised to the power of an operand; `ExprExponential::new(&x).string()? == \"Exp(x)\"`."
);

impl ExprGeneralExpression for ExprExponential {
    unary_expression_members!();
    function_members!(ExprExponential, "Exp(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprLogOfe, _>(self, f64::exp)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        chain(self, x, |_| self.copy())
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.exp())
    }
}
