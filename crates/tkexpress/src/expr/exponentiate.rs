// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Exponentiate.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Exponentiate`: an operand raised to the power of another.
use super::binary_expression::{
    ExprBinaryExpression, binary_expression_members, declare_binary_class, evaluate_operands,
    operands_are_identical,
};
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref};
use super::log_of_e::ExprLogOfe;
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::{expr_add, subtract_real};
use super::package::copy_share;
use super::product::ExprProduct;
use super::text::infix_text;
use super::unary_kit::{shallow_of, without_unknowns, zero_unless_contains};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

declare_binary_class!(
    ExprExponentiate,
    "Expr_Exponentiate",
    "A power; `ExprExponentiate::new(&x, &two).string()? == \"x^2\"`."
);

impl ExprExponentiate {
    /// The rules on a numeric exponent: `a^0` is 1, `a^1` is `a`, and two values fold
    /// with `pow`; `None` when no rule applies.
    fn exponent_rule(first: &ExprHandle, second: &ExprHandle) -> Option<ExprHandle> {
        let exponent: f64 = second.cast::<ExprNumericValue>().get()?.get_value();
        if exponent == 0.0 {
            return Some(ExprNumericValue::expression(1.0));
        }
        if exponent == 1.0 {
            return Some(first.clone());
        }
        let base: f64 = first.cast::<ExprNumericValue>().get()?.get_value();
        Some(ExprNumericValue::expression(base.powf(exponent)))
    }
    /// `h * g^(h-1) * g'`, the first member of the derivative of `g^h`.
    fn base_member(
        first: &ExprHandle,
        second: &ExprHandle,
        first_derivative: ExprHandle,
    ) -> Result<Handle<ExprProduct>, Failure> {
        let exponent: ExprHandle = copy_share(second)?; // h(X)
        let lowered = subtract_real(&copy_share(second)?, 1.0); // h(X)-1
        let power = Self::new(&copy_share(first)?, &shallow_of(&lowered)?);
        let factors: [ExprHandle; 3] = [exponent, shallow_of(&power)?, first_derivative];
        Ok(ExprProduct::new(&factors))
    }
    /// `g^h * Ln(g) * h'`, the second member of the derivative of `g^h`.
    fn exponent_member(
        first: &ExprHandle,
        second: &ExprHandle,
        second_derivative: ExprHandle,
    ) -> Result<Handle<ExprProduct>, Failure> {
        let power = Self::new(&copy_share(first)?, &copy_share(second)?);
        let logarithm = ExprLogOfe::new(&copy_share(first)?);
        let factors: [ExprHandle; 3] = [
            shallow_of(&power)?,
            shallow_of(&logarithm)?,
            second_derivative,
        ];
        Ok(ExprProduct::new(&factors))
    }
}

impl ExprGeneralExpression for ExprExponentiate {
    binary_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let first: ExprHandle = self.first_operand();
        let second: ExprHandle = self.second_operand();
        deref(&second, "Expr_Exponentiate::ShallowSimplified")?;
        if let Some(result) = Self::exponent_rule(&first, &second) {
            return Ok(result);
        }
        deref(&first, "Expr_Exponentiate::ShallowSimplified")?;
        // `1^h` is the base itself, the same numeric value object.
        let base: Handle<ExprNumericValue> = first.cast();
        if base.get().is_some_and(|base| base.get_value() == 1.0) {
            return Ok(first);
        }
        self.core.link().me()
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        let first: ExprHandle = copy_share(&self.first_operand())?;
        Ok(Self::new(&first, &copy_share(&self.second_operand())?).cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        operands_are_identical::<ExprExponentiate, _>(self, other)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        without_unknowns(self)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        if let Some(zero) = zero_unless_contains(self, x)? {
            return Ok(zero);
        }
        let first: ExprHandle = self.first_operand();
        let second: ExprHandle = self.second_operand();
        let first_derivative: ExprHandle =
            deref(&first, "Expr_Exponentiate::Derivative")?.derivative(x)?;
        let second_derivative: ExprHandle =
            deref(&second, "Expr_Exponentiate::Derivative")?.derivative(x)?;
        let base_member = Self::base_member(&first, &second, first_derivative)?;
        let exponent_member = Self::exponent_member(&first, &second, second_derivative)?;
        shallow_of(&expr_add(
            &shallow_of(&base_member)?,
            &shallow_of(&exponent_member)?,
        ))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let (base, exponent) = evaluate_operands(self, vars, vals)?;
        Ok(base.powf(exponent))
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        infix_text(&[self.first_operand(), self.second_operand()], "^")
    }
}
