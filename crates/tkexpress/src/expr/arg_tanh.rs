// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_ArgTanh.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_ArgTanh`: The inverse hyperbolic tangent of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::operators::real_subtract;
use super::tanh::ExprTanh;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::square_of;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprArgTanh,
    "Expr_ArgTanh",
    "The inverse hyperbolic tangent of an operand; `ExprArgTanh::new(&x).string()? == \"ATanh(x)\"`."
);

impl ExprGeneralExpression for ExprArgTanh {
    unary_expression_members!();
    function_members!(ExprArgTanh, "ATanh(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprTanh, _>(self, rustycad_cmath::atanh)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, |operand| {
            shallow_of(&real_subtract(1.0, &square_of(operand)?))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(((1.0 + value) / (1.0 - value)).ln() / 2.0)
    }
}
