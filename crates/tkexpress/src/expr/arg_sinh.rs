// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_ArgSinh.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_ArgSinh`: The inverse hyperbolic sine of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::operators::add_real;
use super::sinh::ExprSinh;
use super::square_root::ExprSquareRoot;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::quotient_rule;
use super::unary_kit::square_of;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprArgSinh,
    "Expr_ArgSinh",
    "The inverse hyperbolic sine of an operand; `ExprArgSinh::new(&x).string()? == \"ASinh(x)\"`."
);

impl ExprGeneralExpression for ExprArgSinh {
    unary_expression_members!();
    function_members!(ExprArgSinh, "ASinh(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprSinh, _>(self, rustycad_cmath::asinh)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        quotient_rule(self, x, |operand| {
            // `sq + 1.0` is `operator+(x, double)`, which builds `1 + sq`.
            let sum = add_real(&square_of(operand)?, 1.0);
            shallow_of(&ExprSquareRoot::new(&shallow_of(&sum)?))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok((value + (value * value + 1.0).sqrt()).ln())
    }
}
