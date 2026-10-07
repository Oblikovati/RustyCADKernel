// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Sinh.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Sinh`: The hyperbolic sine of an operand.
use super::arg_sinh::ExprArgSinh;
use super::cosh::ExprCosh;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::ExprHandle;
use super::named_unknown::ExprNamedUnknown;
use super::package::copy_share;
use super::unary_expression::{declare_unary_class, unary_expression_members};
use super::unary_kit::chain;
use super::unary_kit::function_shallow_simplified;
use super::unary_kit::{evaluate_operand, function_members, shallow_of};
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprSinh,
    "Expr_Sinh",
    "The hyperbolic sine of an operand; `ExprSinh::new(&x).string()? == \"Sinh(x)\"`."
);

impl ExprGeneralExpression for ExprSinh {
    unary_expression_members!();
    function_members!(ExprSinh, "Sinh(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        function_shallow_simplified::<ExprArgSinh, _>(self, f64::sinh)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        chain(self, x, |operand| {
            shallow_of(&ExprCosh::new(&copy_share(operand)?))
        })
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok((value.exp() - (-value).exp()) / 2.0)
    }
}
