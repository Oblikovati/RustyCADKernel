// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Square.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Square`: the square of an operand.
use super::exponentiate::ExprExponentiate;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::real_multiply;
use super::package::copy_share;
use super::product::ExprProduct;
use super::square_root::ExprSquareRoot;
use super::unary_expression::{ExprUnaryExpression, declare_unary_class, unary_expression_members};
use super::unary_kit::{
    cancelled_inverse, evaluate_operand, operand_is_identical, shallow_of, simplified_value,
    without_unknowns, zero_unless_contains,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

declare_unary_class!(
    ExprSquare,
    "Expr_Square",
    "The square of an operand; `ExprSquare::new(&x).string()? == \"x^2\"`."
);

impl ExprSquare {
    /// The rules on powers: `(u^2)^2` is `u^4` and `(u^h)^2` is `u^(2*h)`; `None` when
    /// the operand is not a power.
    fn power_rule(operand: &ExprHandle) -> Result<Option<ExprHandle>, Failure> {
        if is_kind::<ExprSquare>(operand) {
            let base: ExprHandle =
                deref(operand, "Expr_Square::ShallowSimplified")?.sub_expression(1)?;
            let four: ExprHandle = ExprNumericValue::expression(4.0);
            return Ok(Some(ExprExponentiate::new(&base, &four).cast()));
        }
        if !is_kind::<ExprExponentiate>(operand) {
            return Ok(None);
        }
        let power = deref(operand, "Expr_Square::ShallowSimplified")?;
        let base: ExprHandle = power.sub_expression(1)?;
        let doubled = real_multiply(2.0, &power.sub_expression(2)?);
        shallow_of(&ExprExponentiate::new(&base, &shallow_of(&doubled)?)).map(Some)
    }
}

impl ExprGeneralExpression for ExprSquare {
    unary_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let operand: ExprHandle = self.operand();
        deref(&operand, "Expr_Square::ShallowSimplified")?;
        if let Some(value) = simplified_value(&operand, |value| value * value) {
            return Ok(value);
        }
        if let Some(inner) = cancelled_inverse::<ExprSquareRoot>(&operand)? {
            return Ok(inner);
        }
        if let Some(power) = Self::power_rule(&operand)? {
            return Ok(power);
        }
        self.core.link().me()
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        Ok(Self::new(&copy_share(&self.operand())?).cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        operand_is_identical::<ExprSquare>(&self.operand(), other)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        without_unknowns(self)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        if let Some(zero) = zero_unless_contains(self, x)? {
            return Ok(zero);
        }
        let derivative: ExprHandle =
            deref(&self.operand(), "Expr_Square::Derivative")?.derivative(x)?;
        let factors: [ExprHandle; 3] = [
            ExprNumericValue::expression(2.0),
            derivative,
            copy_share(&self.operand())?,
        ];
        shallow_of(&ExprProduct::new(&factors))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let value: f64 = evaluate_operand(self, vars, vals)?;
        Ok(value * value)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        let operand: ExprHandle = self.operand();
        let operand = deref(&operand, "Expr_Square::String")?;
        let mut text: AsciiString = AsciiString::new("");
        if operand.nb_sub_expressions() > 1 {
            text += "(";
            text += &operand.string()?;
            text += ")^2";
            return Ok(text);
        }
        text += &operand.string()?;
        text += "^2";
        Ok(text)
    }
}
