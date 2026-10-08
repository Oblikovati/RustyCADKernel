// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Difference.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Difference`: the difference of two operands.
use super::binary_expression::{
    ExprBinaryExpression, binary_expression_members, declare_binary_class, evaluate_operands,
    operands_are_identical,
};
use super::errors::out_of_range;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::{expr_add, expr_negate, expr_subtract};
use super::package::copy_share;
use super::text::infix_text;
use super::unary_kit::{shallow_of, zero_unless_contains};
use super::unary_minus::ExprUnaryMinus;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

declare_binary_class!(
    ExprDifference,
    "Expr_Difference",
    "A difference; `ExprDifference::new(&x, &y).string()? == \"x-y\"`."
);

/// The value of a numeric operand, `None` otherwise.
fn value_of(operand: &ExprHandle) -> Option<f64> {
    operand
        .cast::<ExprNumericValue>()
        .get()
        .map(ExprNumericValue::get_value)
}

/// The operand of an opposite (`-x` gives `x`), `None` otherwise.
fn negated_operand(operand: &ExprHandle) -> Result<Option<ExprHandle>, Failure> {
    if !is_kind::<ExprUnaryMinus>(operand) {
        return Ok(None);
    }
    deref(operand, "Expr_Difference::ShallowSimplified")?
        .sub_expression(1)
        .map(Some)
}

impl ExprDifference {
    /// The value rules of `ShallowSimplified`: two values fold, `0-b` is `-b` and `a-0`
    /// is `a`; `None` when no rule applies.
    fn value_rule(first: &ExprHandle, second: &ExprHandle) -> Option<ExprHandle> {
        match (value_of(first), value_of(second)) {
            (Some(a), Some(b)) => Some(ExprNumericValue::expression(a - b)),
            (Some(0.0), None) => Some(expr_negate(second).cast()),
            (None, Some(0.0)) => Some(first.clone()),
            _ => None,
        }
    }
    /// The opposite rules: `-a-(-b)` is `b-a`, `-a-b` is `-(a+b)`, `a-(-b)` is `a+b`.
    fn opposite_rule(
        first: &ExprHandle,
        second: &ExprHandle,
    ) -> Result<Option<ExprHandle>, Failure> {
        let rule: Option<ExprHandle> = match (negated_operand(first)?, negated_operand(second)?) {
            (Some(a), Some(b)) => Some(expr_subtract(&b, &a).cast()),
            (Some(a), None) => Some(expr_negate(&expr_add(&a, second).cast()).cast()),
            (None, Some(b)) => Some(expr_add(first, &b).cast()),
            (None, None) => None,
        };
        Ok(rule)
    }
}

impl ExprGeneralExpression for ExprDifference {
    binary_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let first: ExprHandle = self.first_operand();
        let second: ExprHandle = self.second_operand();
        deref(&first, "Expr_Difference::ShallowSimplified")?;
        deref(&second, "Expr_Difference::ShallowSimplified")?;
        if let Some(result) = Self::value_rule(&first, &second) {
            return Ok(result);
        }
        if let Some(result) = Self::opposite_rule(&first, &second)? {
            return Ok(result);
        }
        self.core.link().me()
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        let first: ExprHandle = copy_share(&self.first_operand())?;
        Ok(expr_subtract(&first, &copy_share(&self.second_operand())?).cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        operands_are_identical::<ExprDifference, _>(self, other)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        if !deref(&self.first_operand(), "Expr_Difference::IsLinear")?.is_linear()? {
            return Ok(false);
        }
        deref(&self.second_operand(), "Expr_Difference::IsLinear")?.is_linear()
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        if let Some(zero) = zero_unless_contains(self, x)? {
            return Ok(zero);
        }
        let first: ExprHandle =
            deref(&self.first_operand(), "Expr_Difference::Derivative")?.derivative(x)?;
        let second: ExprHandle =
            deref(&self.second_operand(), "Expr_Difference::Derivative")?.derivative(x)?;
        shallow_of(&expr_subtract(&first, &second))
    }
    fn n_derivative(&self, x: &Handle<ExprNamedUnknown>, n: i32) -> Result<ExprHandle, Failure> {
        if n <= 0 {
            return Err(out_of_range(&format!(
                "Expr_Difference::NDerivative: N = {n}; expected >= 1"
            )));
        }
        if let Some(zero) = zero_unless_contains(self, x)? {
            return Ok(zero);
        }
        let first: ExprHandle =
            deref(&self.first_operand(), "Expr_Difference::NDerivative")?.n_derivative(x, n)?;
        let second: ExprHandle =
            deref(&self.second_operand(), "Expr_Difference::NDerivative")?.n_derivative(x, n)?;
        shallow_of(&expr_subtract(&first, &second))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let (first, second) = evaluate_operands(self, vars, vals)?;
        Ok(first - second)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        infix_text(&[self.first_operand(), self.second_operand()], "-")
    }
}
