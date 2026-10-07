// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Absolute.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Absolute`: the absolute value of an operand.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::operators::expr_multiply;
use super::package::copy_share;
use super::sign::ExprSign;
use super::unary_expression::{ExprUnaryExpression, declare_unary_class, unary_expression_members};
use super::unary_kit::{
    evaluate_operand, function_members, operand_and_derivative, shallow_of, simplified_value,
};
use super::unary_minus::ExprUnaryMinus;
use rustycad_tkernel::standard::{Failure, Handle};

declare_unary_class!(
    ExprAbsolute,
    "Expr_Absolute",
    "The absolute value of an operand; `ExprAbsolute::new(&x).string()? == \"Abs(x)\"`."
);

impl ExprGeneralExpression for ExprAbsolute {
    unary_expression_members!();
    function_members!(ExprAbsolute, "Abs(");
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let operand: ExprHandle = self.operand();
        deref(&operand, "Expr_Absolute::ShallowSimplified")?;
        if let Some(value) = simplified_value(&operand, f64::abs) {
            return Ok(value);
        }
        if is_kind::<ExprUnaryMinus>(&operand) {
            // `Abs(-u)` becomes a new `Abs(u)`, not simplified further.
            let inner: ExprHandle =
                deref(&operand, "Expr_Absolute::ShallowSimplified")?.sub_expression(1)?;
            return Ok(ExprAbsolute::new(&inner).cast());
        }
        self.core.link().me()
    }
    // Native quirk: no `Contains` guard, so the result is `Sign(u)*u'` simplified even
    // when `x` does not occur.
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        let (operand, derivative) = operand_and_derivative(self, x)?;
        let sign = ExprSign::new(&copy_share(&operand)?);
        shallow_of(&expr_multiply(&shallow_of(&sign)?, &derivative))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        Ok(evaluate_operand(self, vars, vals)?.abs())
    }
}
