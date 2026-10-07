// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_BinaryExpression.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_BinaryExpression`: an expression with two operands.
//!
//! The operands (private natively) live in an [`ExprBinaryCore`] that each concrete
//! class embeds; [`binary_expression_members!`] writes the inherited members.
use super::errors::out_of_range;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, ExprSelfLink, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::unary_expression::{check_new_operand, replace_inside};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, SharedState};

/// The two operands and the self pointer of a binary expression;
/// `ExprBinaryCore::new(&x, &y)`.
#[derive(Default)]
pub struct ExprBinaryCore {
    operands: SharedState<[ExprHandle; 2]>,
    link: ExprSelfLink,
}
impl ExprBinaryCore {
    /// The core of a new expression with unchecked operands (`CreateFirstOperand`,
    /// `CreateSecondOperand`); `ExprBinaryCore::new(&x, &y)`.
    pub fn new(exp1: &ExprHandle, exp2: &ExprHandle) -> Self {
        Self {
            operands: SharedState::new([exp1.clone(), exp2.clone()]),
            link: ExprSelfLink::default(),
        }
    }
    /// The self pointer, bound by the constructor; `core.link().me::<ExprDivision>()?`.
    pub fn link(&self) -> &ExprSelfLink {
        &self.link
    }
    /// Store `exp` at `slot` (0 first, 1 second) after the `SetOperand` checks.
    fn set_checked(&self, slot: usize, exp: &ExprHandle, member: &str) -> Result<(), Failure> {
        let me: ExprHandle = self.link.me()?;
        check_new_operand(&me, exp, member)?;
        self.operands.write()[slot] = exp.clone();
        Ok(())
    }
}

/// An expression with two operands; `difference.first_operand()`.
pub trait ExprBinaryExpression: ExprGeneralExpression {
    /// The embedded operands of the concrete object; `difference.binary_core()`.
    fn binary_core(&self) -> &ExprBinaryCore;
    /// The first operand (`FirstOperand`); `difference.first_operand()`.
    fn first_operand(&self) -> ExprHandle {
        self.binary_core().operands.read()[0].clone()
    }
    /// The second operand (`SecondOperand`); `difference.second_operand()`.
    fn second_operand(&self) -> ExprHandle {
        self.binary_core().operands.read()[1].clone()
    }
    /// Replace the first operand (`SetFirstOperand`); Expr_InvalidOperand when `exp`
    /// is the expression or contains it; `difference.set_first_operand(&y)?`.
    fn set_first_operand(&self, exp: &ExprHandle) -> Result<(), Failure> {
        self.binary_core()
            .set_checked(0, exp, "Expr_BinaryExpression::SetFirstOperand")
    }
    /// Replace the second operand (`SetSecondOperand`); Expr_InvalidOperand when `exp`
    /// is the expression or contains it; `difference.set_second_operand(&y)?`.
    fn set_second_operand(&self, exp: &ExprHandle) -> Result<(), Failure> {
        self.binary_core()
            .set_checked(1, exp, "Expr_BinaryExpression::SetSecondOperand")
    }
}
impl_transient_interface!(
    dyn ExprBinaryExpression,
    "Expr_BinaryExpression",
    dyn ExprGeneralExpression
);

/// `Expr_BinaryExpression::SubExpression`; `binary_sub_expression(&difference, 2)?`.
pub fn binary_sub_expression<T: ExprBinaryExpression + ?Sized>(
    me: &T,
    index: i32,
) -> Result<ExprHandle, Failure> {
    match index {
        1 => Ok(me.first_operand()),
        2 => Ok(me.second_operand()),
        _ => Err(out_of_range(&format!(
            "Expr_BinaryExpression::SubExpression: I = {index}; expected 1 or 2"
        ))),
    }
}

/// `Expr_BinaryExpression::ContainsUnknowns`: both operands are tested for being an
/// unknown before either is asked; `binary_contains_unknowns(&difference)?`.
pub fn binary_contains_unknowns<T: ExprBinaryExpression + ?Sized>(me: &T) -> Result<bool, Failure> {
    let first: ExprHandle = me.first_operand();
    let second: ExprHandle = me.second_operand();
    if is_kind::<ExprNamedUnknown>(&first) || is_kind::<ExprNamedUnknown>(&second) {
        return Ok(true);
    }
    if deref(&first, "Expr_BinaryExpression::ContainsUnknowns")?.contains_unknowns()? {
        return Ok(true);
    }
    deref(&second, "Expr_BinaryExpression::ContainsUnknowns")?.contains_unknowns()
}

/// `Expr_BinaryExpression::Contains`; `binary_contains(&difference, &x)?`.
pub fn binary_contains<T: ExprBinaryExpression + ?Sized>(
    me: &T,
    exp: &ExprHandle,
) -> Result<bool, Failure> {
    let first: ExprHandle = me.first_operand();
    let second: ExprHandle = me.second_operand();
    if first == *exp || second == *exp {
        return Ok(true);
    }
    if deref(&first, "Expr_BinaryExpression::Contains")?.contains(exp)? {
        return Ok(true);
    }
    deref(&second, "Expr_BinaryExpression::Contains")?.contains(exp)
}

/// `Expr_BinaryExpression::Replace`: the first operand, then the second;
/// `binary_replace(&difference, &x, &y)?`.
pub fn binary_replace<T: ExprBinaryExpression + ?Sized>(
    me: &T,
    var: &Handle<ExprNamedUnknown>,
    with: &ExprHandle,
) -> Result<(), Failure> {
    let first: ExprHandle = me.first_operand();
    if first == *var {
        me.set_first_operand(with)?;
    } else {
        replace_inside(&first, var, with)?;
    }
    let second: ExprHandle = me.second_operand();
    if second == *var {
        return me.set_second_operand(with);
    }
    replace_inside(&second, var, with)
}

/// `Expr_BinaryExpression::Simplified`: a copy whose operands are simplified, then
/// simplified itself; `binary_simplified(&difference)?`.
pub fn binary_simplified<T: ExprBinaryExpression + ?Sized>(me: &T) -> Result<ExprHandle, Failure> {
    let copy: Handle<dyn ExprBinaryExpression> = me.copy()?.cast();
    let copy: &dyn ExprBinaryExpression = deref(&copy, "Expr_BinaryExpression::Simplified")?;
    let first: ExprHandle = copy.first_operand();
    let second: ExprHandle = copy.second_operand();
    copy.set_first_operand(&deref(&first, "Expr_BinaryExpression::Simplified")?.simplified()?)?;
    copy.set_second_operand(&deref(&second, "Expr_BinaryExpression::Simplified")?.simplified()?)?;
    copy.shallow_simplified()
}

/// The `IsIdentical` of the binary classes: `other` is of class `C` and its operands
/// are identical to those of `me`, the first compared before the second;
/// `operands_are_identical::<ExprDivision, _>(&quotient, &other)?`.
pub fn operands_are_identical<C: ?Sized + RuntimeTyped, T: ExprBinaryExpression + ?Sized>(
    me: &T,
    other: &ExprHandle,
) -> Result<bool, Failure> {
    let other_expression = deref(other, "IsIdentical")?;
    if !is_kind::<C>(other) {
        return Ok(false);
    }
    let first: ExprHandle = me.first_operand();
    if !deref(&first, "IsIdentical")?.is_identical(&other_expression.sub_expression(1)?)? {
        return Ok(false);
    }
    deref(&me.second_operand(), "IsIdentical")?.is_identical(&other_expression.sub_expression(2)?)
}

/// The values of both operands, the first evaluated first: the start of each binary
/// `Evaluate`; `let (a, b) = evaluate_operands(&quotient, &vars, &vals)?;`.
pub fn evaluate_operands<T: ExprBinaryExpression + ?Sized>(
    me: &T,
    vars: &ExprVariables<'_>,
    vals: &ExprValues<'_>,
) -> Result<(f64, f64), Failure> {
    let first: f64 = deref(&me.first_operand(), "Evaluate")?.evaluate(vars, vals)?;
    let second: f64 = deref(&me.second_operand(), "Evaluate")?.evaluate(vars, vals)?;
    Ok((first, second))
}

/// The members every binary class inherits from `Expr_BinaryExpression`: inside an
/// `impl ExprGeneralExpression for Class` block, `binary_expression_members!();`.
macro_rules! binary_expression_members {
    () => {
        fn nb_sub_expressions(&self) -> i32 {
            2
        }
        fn sub_expression(
            &self,
            index: i32,
        ) -> Result<$crate::expr::ExprHandle, rustycad_tkernel::standard::Failure> {
            $crate::expr::binary_expression::binary_sub_expression(self, index)
        }
        fn contains_unknowns(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::binary_expression::binary_contains_unknowns(self)
        }
        fn contains(
            &self,
            exp: &$crate::expr::ExprHandle,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::binary_expression::binary_contains(self, exp)
        }
        fn replace(
            &self,
            var: &rustycad_tkernel::standard::Handle<$crate::expr::ExprNamedUnknown>,
            with: &$crate::expr::ExprHandle,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::expr::binary_expression::binary_replace(self, var, with)
        }
        fn simplified(
            &self,
        ) -> Result<$crate::expr::ExprHandle, rustycad_tkernel::standard::Failure> {
            $crate::expr::binary_expression::binary_simplified(self)
        }
    };
}
pub(crate) use binary_expression_members;

/// Declare a binary class whose only state is its operands: the struct, its runtime
/// type, its constructor and its `ExprBinaryExpression` impl;
/// `declare_binary_class!(ExprDivision, "Expr_Division", "A quotient.");`.
macro_rules! declare_binary_class {
    ($class:ident, $name:literal, $doc:literal) => {
        #[doc = $doc]
        pub struct $class {
            parent: rustycad_tkernel::standard::StandardTransient,
            core: $crate::expr::binary_expression::ExprBinaryCore,
        }
        rustycad_tkernel::impl_transient!(
            $class,
            $name,
            dyn $crate::expr::ExprBinaryExpression,
            parent,
            interfaces: [dyn $crate::expr::ExprGeneralExpression, dyn $crate::expr::ExprBinaryExpression]
        );
        impl $class {
            #[doc = concat!("`new ", $name, "(exp1, exp2)`; `", stringify!($class), "::new(&x, &y)`.")]
            pub fn new(
                exp1: &$crate::expr::ExprHandle,
                exp2: &$crate::expr::ExprHandle,
            ) -> rustycad_tkernel::standard::Handle<Self> {
                let object: Self = Self {
                    parent: rustycad_tkernel::standard::StandardTransient::default(),
                    core: $crate::expr::binary_expression::ExprBinaryCore::new(exp1, exp2),
                };
                $crate::expr::handles::adopt(object, |created| created.core.link())
            }
        }
        impl $crate::expr::ExprBinaryExpression for $class {
            fn binary_core(&self) -> &$crate::expr::binary_expression::ExprBinaryCore {
                &self.core
            }
        }
    };
}
pub(crate) use declare_binary_class;
