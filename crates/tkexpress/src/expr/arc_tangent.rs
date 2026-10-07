// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_ArcTangent.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_ArcTangent`: The arc tangent of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::operators::real_add;
use super::tangent::ExprTangent;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::square_of;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprArcTangent,
    "Expr_ArcTangent",
    "The arc tangent of an operand; `ExprArcTangent::new(&x).string()? == \"ATan(x)\"`."
);

impl ExprGeneralExpression for ExprArcTangent {
    unary_expression_members!();
    function_members!(ExprArcTangent, "ATan(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprTangent, _>(self, f64::atan)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, |operand| {
            shallow_of(&real_add(1.0, &square_of(operand)?))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.atan())
    }
}
