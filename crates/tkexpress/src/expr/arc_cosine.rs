// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_ArcCosine.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_ArcCosine`: The arc cosine of an operand.
use super::cosine::ExprCosine;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::operators::{expr_negate, real_divide};
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::chain;
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::root_of_one_minus_square;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprArcCosine,
    "Expr_ArcCosine",
    "The arc cosine of an operand; `ExprArcCosine::new(&x).string()? == \"ACos(x)\"`."
);

impl ExprGeneralExpression for ExprArcCosine {
    unary_expression_members!();
    function_members!(ExprArcCosine, "ACos(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprCosine, _>(self, f64::acos)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        chain(self, x, |operand| {
            let root: ExprHandle = root_of_one_minus_square(operand)?;
            let inverse = real_divide(1.0, &root);
            shallow_of(&expr_negate(&inverse.cast()))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.acos())
    }
}
