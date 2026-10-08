// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_ArgCosh.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_ArgCosh`: The inverse hyperbolic cosine of an operand.
use super::cosh::ExprCosh;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::operators::subtract_real;
use super::square_root::ExprSquareRoot;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::square_of;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprArgCosh,
    "Expr_ArgCosh",
    "The inverse hyperbolic cosine of an operand; `ExprArgCosh::new(&x).string()? == \"ACosh(x)\"`."
);

impl ExprGeneralExpression for ExprArgCosh {
    unary_expression_members!();
    function_members!(ExprArgCosh, "ACosh(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprCosh, _>(self, rustycad_cmath::acosh)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, |operand| {
            let difference = subtract_real(&square_of(operand)?, 1.0);
            shallow_of(&ExprSquareRoot::new(&shallow_of(&difference)?))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok((value + (value * value - 1.0).sqrt()).ln())
    }
}
