// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_SquareRoot.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_SquareRoot`: The square root of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::operators::real_multiply;
use super::package::copy_share;
use super::square::ExprSquare;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprSquareRoot,
    "Expr_SquareRoot",
    "The square root of an operand; `ExprSquareRoot::new(&x).string()? == \"Sqrt(x)\"`."
);

impl ExprGeneralExpression for ExprSquareRoot {
    unary_expression_members!();
    function_members!(ExprSquareRoot, "Sqrt(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprSquare, _>(self, f64::sqrt)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, |operand| {
            let root: ExprHandle = ExprSquareRoot::new(&copy_share(operand)?).cast();
            shallow_of(&real_multiply(2.0, &root))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.sqrt())
    }
}
