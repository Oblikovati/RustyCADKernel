// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Tanh.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Tanh`: The hyperbolic tangent of an operand.
use super::arg_tanh::ExprArgTanh;
use super::cosh::ExprCosh;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::package::copy_share;
use super::square::ExprSquare;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprTanh,
    "Expr_Tanh",
    "The hyperbolic tangent of an operand; `ExprTanh::new(&x).string()? == \"Tanh(x)\"`."
);

impl ExprGeneralExpression for ExprTanh {
    unary_expression_members!();
    function_members!(ExprTanh, "Tanh(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprArgTanh, _>(self, f64::tanh)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, |operand| {
            let cosh: ExprHandle = shallow_of(&ExprCosh::new(&copy_share(operand)?))?;
            shallow_of(&ExprSquare::new(&cosh))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok((value.exp() - (-value).exp()) / (value.exp() + (-value).exp()))
    }
}
