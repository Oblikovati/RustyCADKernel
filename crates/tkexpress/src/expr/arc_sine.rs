// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_ArcSine.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_ArcSine`: The arc sine of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::sine::ExprSine;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::root_of_one_minus_square;
use super::unary_kit::{evaluate_operand, function_members};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprArcSine,
    "Expr_ArcSine",
    "The arc sine of an operand; `ExprArcSine::new(&x).string()? == \"ASin(x)\"`."
);

impl ExprGeneralExpression for ExprArcSine {
    unary_expression_members!();
    function_members!(ExprArcSine, "ASin(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprSine, _>(self, f64::asin)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, root_of_one_minus_square)
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.asin())
    }
}
