// Rust support for the unary classes of ModelingAlgorithms/TKExpress/Expr (Expr_Sine,
// Expr_Cosh, ...) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps that the unary classes repeat natively, written once: folding a numeric
//! operand, cancelling an inverse function, comparing operands and the guard of
//! `Derivative`.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::unary_expression::ExprUnaryExpression;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};

/// `new Expr_NumericValue(f(value))` when `operand` is a numeric value, else `None`:
/// the first step of each `ShallowSimplified`; `simplified_value(&two, f64::sin)`.
pub fn simplified_value(operand: &ExprHandle, function: impl Fn(f64) -> f64) -> Option<ExprHandle> {
    let value: Handle<ExprNumericValue> = operand.cast();
    value
        .get()
        .map(|value| ExprNumericValue::expression(function(value.get_value())))
}

/// The operand of `operand` when it is of the inverse class `I` (`Sin(ASin(x))` is
/// `x`), else `None`; `cancelled_inverse::<ExprArcSine>(&operand)?`.
pub fn cancelled_inverse<I: ?Sized + RuntimeTyped>(
    operand: &ExprHandle,
) -> Result<Option<ExprHandle>, Failure> {
    if !is_kind::<I>(operand) {
        return Ok(None);
    }
    deref(operand, "ShallowSimplified")?
        .sub_expression(1)
        .map(Some)
}

/// The `ShallowSimplified` of a function with an inverse `I`: the value of a numeric
/// operand, the operand of an inverse, or the expression itself;
/// `function_shallow_simplified::<ExprArcSine, _>(&sine, f64::sin)?`.
pub fn function_shallow_simplified<I: ?Sized + RuntimeTyped, T: ExprUnaryExpression + ?Sized>(
    me: &T,
    function: impl Fn(f64) -> f64,
) -> Result<ExprHandle, Failure> {
    let operand: ExprHandle = me.operand();
    if let Some(value) = simplified_value(&operand, function) {
        return Ok(value);
    }
    if let Some(inner) = cancelled_inverse::<I>(&operand)? {
        return Ok(inner);
    }
    me.unary_core().link().me()
}

/// The `IsIdentical` of the unary classes: `other` is of class `C` and its operand is
/// identical to `operand`; `operand_is_identical::<ExprSine>(&sine.operand(), &other)?`.
pub fn operand_is_identical<C: ?Sized + RuntimeTyped>(
    operand: &ExprHandle,
    other: &ExprHandle,
) -> Result<bool, Failure> {
    let other_expression = deref(other, "IsIdentical")?;
    if !is_kind::<C>(other) {
        return Ok(false);
    }
    deref(operand, "IsIdentical")?.is_identical(&other_expression.sub_expression(1)?)
}

/// The value of the operand: the first step of each unary `Evaluate`;
/// `evaluate_operand(&sine, &vars, &vals)?.sin()`.
pub fn evaluate_operand<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    vars: &ExprVariables<'_>,
    vals: &ExprValues<'_>,
) -> Result<f64, Failure> {
    deref(&me.operand(), "Evaluate")?.evaluate(vars, vals)
}

/// `!ContainsUnknowns()`: the `IsLinear` of the transcendental functions;
/// `without_unknowns(&sine)?`.
pub fn without_unknowns<T: ExprGeneralExpression + ?Sized>(me: &T) -> Result<bool, Failure> {
    Ok(!me.contains_unknowns()?)
}

/// `if (!Contains(X)) return new Expr_NumericValue(0.0)`: the guard of most
/// `Derivative` members; `Some(zero)` when `x` does not occur;
/// `if let Some(zero) = zero_unless_contains(&sine, &x)? { return Ok(zero); }`.
pub fn zero_unless_contains<T: ExprGeneralExpression + ?Sized>(
    me: &T,
    x: &Handle<ExprNamedUnknown>,
) -> Result<Option<ExprHandle>, Failure> {
    if me.contains(&x.cast())? {
        return Ok(None);
    }
    Ok(Some(ExprNumericValue::expression(0.0)))
}

/// The operand and its derivative on `x`: the start of every chain rule
/// (`op = Operand(); derop = op->Derivative(X)`); `operand_and_derivative(&sine, &x)?`.
pub fn operand_and_derivative<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    x: &Handle<ExprNamedUnknown>,
) -> Result<(ExprHandle, ExprHandle), Failure> {
    let operand: ExprHandle = me.operand();
    let derivative: ExprHandle = deref(&operand, "Derivative")?.derivative(x)?;
    Ok((operand, derivative))
}

/// `handle->ShallowSimplified()` of a freshly built expression of any class;
/// `shallow_of(&ExprSine::new(&x))?`.
pub fn shallow_of<T: ?Sized + RuntimeTyped + ExprGeneralExpression>(
    handle: &Handle<T>,
) -> Result<ExprHandle, Failure> {
    deref(handle, "ShallowSimplified")?.shallow_simplified()
}

/// The members that the standard functions (`Expr_Sine`, `Expr_Cosh`, ...) write the
/// same way natively: `Copy` (a new object on a shared copy of the operand),
/// `IsIdentical` (same class, identical operands), `IsLinear` (no unknowns) and
/// `String` (`prefix`, the operand, `)`). Inside `impl ExprGeneralExpression for
/// Class`, `function_members!(ExprSine, "Sin(");`.
macro_rules! function_members {
    ($class:ident, $prefix:literal) => {
        fn copy(&self) -> Result<$crate::expr::ExprHandle, rustycad_tkernel::standard::Failure> {
            let operand = $crate::expr::package::copy_share(
                &$crate::expr::ExprUnaryExpression::operand(self),
            )?;
            Ok($class::new(&operand).cast())
        }
        fn is_identical(
            &self,
            other: &$crate::expr::ExprHandle,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::unary_kit::operand_is_identical::<$class>(
                &$crate::expr::ExprUnaryExpression::operand(self),
                other,
            )
        }
        fn is_linear(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::unary_kit::without_unknowns(self)
        }
        fn string(
            &self,
        ) -> Result<rustycad_tkernel::tcollection::AsciiString, rustycad_tkernel::standard::Failure>
        {
            $crate::expr::text::call_text(
                $prefix,
                &$crate::expr::ExprUnaryExpression::operand(self),
            )
        }
    };
}
pub(crate) use function_members;

/// The `ShallowSimplified` of a function without an inverse (`Expr_LogOf10`,
/// `Expr_Sign`): the value of a numeric operand, or the expression itself;
/// `value_shallow_simplified(&logarithm, f64::log10)?`.
pub fn value_shallow_simplified<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    function: impl Fn(f64) -> f64,
) -> Result<ExprHandle, Failure> {
    let operand: ExprHandle = me.operand();
    deref(&operand, "ShallowSimplified")?;
    match simplified_value(&operand, function) {
        Some(value) => Ok(value),
        None => me.unary_core().link().me(),
    }
}

/// The chain rule `f'(u) * u'` of the standard functions: 0 when `x` does not occur,
/// else `factor(u) * u'` simplified one level, where `factor` builds `f'(u)` from the
/// operand once its derivative is known; `chain(&sine, &x, cosine_factor)?`.
pub fn chain<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    x: &Handle<ExprNamedUnknown>,
    factor: impl FnOnce(&ExprHandle) -> Result<ExprHandle, Failure>,
) -> Result<ExprHandle, Failure> {
    if let Some(zero) = zero_unless_contains(me, x)? {
        return Ok(zero);
    }
    let (operand, derivative) = operand_and_derivative(me, x)?;
    let factor: ExprHandle = factor(&operand)?;
    shallow_of(&super::operators::expr_multiply(&factor, &derivative))
}

/// The chain rule written as a quotient `u' / g(u)`: 0 when `x` does not occur, else
/// `u' / denominator(u)` simplified one level; `quotient_rule(&logarithm, &x, copy_share)?`.
pub fn quotient_rule<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    x: &Handle<ExprNamedUnknown>,
    denominator: impl FnOnce(&ExprHandle) -> Result<ExprHandle, Failure>,
) -> Result<ExprHandle, Failure> {
    if let Some(zero) = zero_unless_contains(me, x)? {
        return Ok(zero);
    }
    let (operand, derivative) = operand_and_derivative(me, x)?;
    let denominator: ExprHandle = denominator(&operand)?;
    shallow_of(&super::operators::expr_divide(&derivative, &denominator))
}

/// `(new Expr_Square(Expr::CopyShare(u)))->ShallowSimplified()`; `square_of(&u)?`.
pub fn square_of(operand: &ExprHandle) -> Result<ExprHandle, Failure> {
    shallow_of(&super::square::ExprSquare::new(
        &super::package::copy_share(operand)?,
    ))
}

/// `Sqrt(1 - u^2)`, each step simplified one level: the denominator of the
/// derivatives of the arc sine and the arc cosine; `root_of_one_minus_square(&u)?`.
pub fn root_of_one_minus_square(operand: &ExprHandle) -> Result<ExprHandle, Failure> {
    let difference = super::operators::real_subtract(1.0, &square_of(operand)?);
    shallow_of(&super::square_root::ExprSquareRoot::new(&shallow_of(
        &difference,
    )?))
}
