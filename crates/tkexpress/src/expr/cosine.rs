// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Cosine.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Cosine`: The cosine of an operand.
use super::arc_cosine::ExprArcCosine;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::operators::expr_negate;
use super::package::copy_share;
use super::sine::ExprSine;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::chain;
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprCosine,
    "Expr_Cosine",
    "The cosine of an operand; `ExprCosine::new(&x).string()? == \"Cos(x)\"`."
);

impl ExprGeneralExpression for ExprCosine {
    unary_expression_members!();
    function_members!(ExprCosine, "Cos(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprArcCosine, _>(self, f64::cos)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        chain(self, x, |operand| {
            let sine: ExprHandle = shallow_of(&ExprSine::new(&copy_share(operand)?))?;
            shallow_of(&expr_negate(&sine))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.cos())
    }
}
