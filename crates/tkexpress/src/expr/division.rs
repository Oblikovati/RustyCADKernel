// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Division.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Division`: the quotient of two operands.
use super::binary_expression::{
    ExprBinaryExpression, binary_expression_members, declare_binary_class, evaluate_operands,
    operands_are_identical,
};
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::{expr_divide, expr_multiply, expr_subtract};
use super::package::copy_share;
use super::square::ExprSquare;
use super::text::infix_text;
use super::unary_kit::{shallow_of, zero_unless_contains};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

declare_binary_class!(
    ExprDivision,
    "Expr_Division",
    "A quotient; `ExprDivision::new(&x, &y).string()? == \"x/y\"`."
);

/// The value of a numeric operand, `None` otherwise.
fn value_of(operand: &ExprHandle) -> Option<f64> {
    operand
        .cast::<ExprNumericValue>()
        .get()
        .map(ExprNumericValue::get_value)
}

impl ExprGeneralExpression for ExprDivision {
    binary_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let first: ExprHandle = self.first_operand();
        let second: ExprHandle = self.second_operand();
        deref(&first, "Expr_Division::ShallowSimplified")?;
        match (value_of(&first), value_of(&second)) {
            // `0/b` is 0 whatever b is, even 0 or NaN.
            (Some(0.0), _) => Ok(ExprNumericValue::expression(0.0)),
            (Some(a), Some(b)) => Ok(ExprNumericValue::expression(a / b)),
            (None, Some(1.0)) => Ok(first),
            _ => {
                deref(&second, "Expr_Division::ShallowSimplified")?;
                self.core.link().me()
            }
        }
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        let first: ExprHandle = copy_share(&self.first_operand())?;
        Ok(expr_divide(&first, &copy_share(&self.second_operand())?).cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        operands_are_identical::<ExprDivision, _>(self, other)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        let first: ExprHandle = self.first_operand();
        let second: ExprHandle = self.second_operand();
        let denominator = deref(&second, "Expr_Division::IsLinear")?;
        if is_kind::<ExprNamedUnknown>(&second) || denominator.contains_unknowns()? {
            return Ok(false);
        }
        if !deref(&first, "Expr_Division::IsLinear")?.is_linear()? {
            return Ok(false);
        }
        denominator.is_linear()
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        if let Some(zero) = zero_unless_contains(self, x)? {
            return Ok(zero);
        }
        let first: ExprHandle = self.first_operand();
        let second: ExprHandle = self.second_operand();
        let first_derivative: ExprHandle =
            deref(&first, "Expr_Division::Derivative")?.derivative(x)?;
        let second_derivative: ExprHandle =
            deref(&second, "Expr_Division::Derivative")?.derivative(x)?;
        // (f'*g - f*g') / g^2, each piece simplified one level.
        let first_term: ExprHandle =
            shallow_of(&expr_multiply(&first_derivative, &copy_share(&second)?))?;
        let second_term: ExprHandle =
            shallow_of(&expr_multiply(&copy_share(&first)?, &second_derivative))?;
        let numerator = expr_subtract(&first_term, &second_term);
        let denominator = ExprSquare::new(&copy_share(&second)?);
        let quotient = expr_divide(&shallow_of(&numerator)?, &shallow_of(&denominator)?);
        shallow_of(&quotient)
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let (first, second) = evaluate_operands(self, vars, vals)?;
        Ok(first / second)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        infix_text(&[self.first_operand(), self.second_operand()], "/")
    }
}
