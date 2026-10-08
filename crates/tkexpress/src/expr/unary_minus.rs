// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_UnaryMinus.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_UnaryMinus`: the opposite of an operand.
use super::errors::out_of_range;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::operators::expr_negate;
use super::package::copy_share;
use super::unary_expression::{ExprUnaryExpression, declare_unary_class, unary_expression_members};
use super::unary_kit::{evaluate_operand, operand_is_identical, simplified_value};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

declare_unary_class!(
    ExprUnaryMinus,
    "Expr_UnaryMinus",
    "The opposite of an operand; `ExprUnaryMinus::new(&x).string()? == \"-x\"`."
);

/// `resu->ShallowSimplified()` of the opposite of `derivative`.
fn negated(derivative: &ExprHandle) -> Result<ExprHandle, Failure> {
    deref(&expr_negate(derivative), "Expr_UnaryMinus::Derivative")?.shallow_simplified()
}

impl ExprGeneralExpression for ExprUnaryMinus {
    unary_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let operand: ExprHandle = self.operand();
        if let Some(value) = simplified_value(&operand, |value| -value) {
            return Ok(value);
        }
        if is_kind::<ExprUnaryMinus>(&operand) {
            return deref(&operand, "Expr_UnaryMinus::ShallowSimplified")?.sub_expression(1);
        }
        self.core.link().me()
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        Ok(expr_negate(&copy_share(&self.operand())?).cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        operand_is_identical::<ExprUnaryMinus>(&self.operand(), other)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        deref(&self.operand(), "Expr_UnaryMinus::IsLinear")?.is_linear()
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        negated(&deref(&self.operand(), "Expr_UnaryMinus::Derivative")?.derivative(x)?)
    }
    fn n_derivative(&self, x: &Handle<ExprNamedUnknown>, n: i32) -> Result<ExprHandle, Failure> {
        if n <= 0 {
            return Err(out_of_range(&format!(
                "Expr_UnaryMinus::NDerivative: N = {n}; expected >= 1"
            )));
        }
        negated(&deref(&self.operand(), "Expr_UnaryMinus::NDerivative")?.n_derivative(x, n)?)
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        Ok(-evaluate_operand(self, vars, vals)?)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        let operand: ExprHandle = self.operand();
        let operand = deref(&operand, "Expr_UnaryMinus::String")?;
        let mut text: AsciiString = AsciiString::new("-");
        if operand.nb_sub_expressions() > 1 {
            text += "(";
            text += &operand.string()?;
            text += ")";
            return Ok(text);
        }
        text += &operand.string()?;
        Ok(text)
    }
}
