// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Tangent.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Tangent`: The tangent of an operand.
use super::arc_tangent::ExprArcTangent;
use super::cosine::ExprCosine;
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
    ExprTangent,
    "Expr_Tangent",
    "The tangent of an operand; `ExprTangent::new(&x).string()? == \"Tan(x)\"`."
);

impl ExprGeneralExpression for ExprTangent {
    unary_expression_members!();
    function_members!(ExprTangent, "Tan(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprArcTangent, _>(self, f64::tan)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, |operand| {
            let cosine: ExprHandle = shallow_of(&ExprCosine::new(&copy_share(operand)?))?;
            shallow_of(&ExprSquare::new(&cosine))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value.tan())
    }
}
