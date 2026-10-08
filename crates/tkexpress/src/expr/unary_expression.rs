// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_UnaryExpression.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_UnaryExpression`: an expression with one operand.
//!
//! The operand (`myOperand`, private natively) lives in an [`ExprUnaryCore`] that each
//! concrete class embeds; `unary_expression_members!` writes the members that every
//! unary class inherits from `Expr_UnaryExpression` natively.
use super::errors::{invalid_operand, out_of_range};
use super::general_expression::ExprGeneralExpression;
use super::handles::{ExprHandle, ExprSelfLink, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};

/// The operand and self pointer of a unary expression; `ExprUnaryCore::new(&x)`.
#[derive(Default)]
pub struct ExprUnaryCore {
    operand: SharedState<ExprHandle>,
    link: ExprSelfLink,
}
impl ExprUnaryCore {
    /// The core of a new expression, with `exp` as operand unchecked (`CreateOperand`);
    /// `ExprUnaryCore::new(&x)`.
    pub fn new(exp: &ExprHandle) -> Self {
        Self {
            operand: SharedState::new(exp.clone()),
            link: ExprSelfLink::default(),
        }
    }
    /// The self pointer, bound by the constructor; `core.link().me::<ExprSine>()?`.
    pub fn link(&self) -> &ExprSelfLink {
        &self.link
    }
}

/// An expression with one operand; `sine.operand()`.
pub trait ExprUnaryExpression: ExprGeneralExpression {
    /// The embedded operand of the concrete object; `sine.unary_core().link()`.
    fn unary_core(&self) -> &ExprUnaryCore;
    /// The operand (`Operand`); `sine.operand()`.
    fn operand(&self) -> ExprHandle {
        self.unary_core().operand.read().clone()
    }
    /// Replace the operand (`SetOperand`); Expr_InvalidOperand when `exp` is the
    /// expression or contains it; `sine.set_operand(&y)?`.
    fn set_operand(&self, exp: &ExprHandle) -> Result<(), Failure> {
        let me: ExprHandle = self.unary_core().link.me()?;
        check_new_operand(&me, exp, "Expr_UnaryExpression::SetOperand")?;
        *self.unary_core().operand.write() = exp.clone();
        Ok(())
    }
}
impl_transient_interface!(
    dyn ExprUnaryExpression,
    "Expr_UnaryExpression",
    dyn ExprGeneralExpression
);

/// The check of every `SetOperand`: `exp` must neither be the expression `me` nor
/// contain it; `check_new_operand(&me, &exp, "SetOperand")?`.
pub fn check_new_operand(me: &ExprHandle, exp: &ExprHandle, member: &str) -> Result<(), Failure> {
    if exp == me {
        return Err(invalid_operand(&format!(
            "{member}: the operand is the expression itself; expected another expression"
        )));
    }
    if deref(exp, member)?.contains(me)? {
        return Err(invalid_operand(&format!(
            "{member}: the operand contains the expression; expected an expression without it"
        )));
    }
    Ok(())
}

/// `Expr_UnaryExpression::NbSubExpressions`: one operand; `unary_nb_sub_expressions() == 1`.
pub fn unary_nb_sub_expressions() -> i32 {
    1
}

/// `Expr_UnaryExpression::SubExpression`; `unary_sub_expression(&sine, 1)?`.
pub fn unary_sub_expression<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    index: i32,
) -> Result<ExprHandle, Failure> {
    if index != 1 {
        return Err(out_of_range(&format!(
            "Expr_UnaryExpression::SubExpression: I = {index}; expected 1"
        )));
    }
    Ok(me.operand())
}

/// `Expr_UnaryExpression::ContainsUnknowns`; `unary_contains_unknowns(&sine)?`.
pub fn unary_contains_unknowns<T: ExprUnaryExpression + ?Sized>(me: &T) -> Result<bool, Failure> {
    let operand: ExprHandle = me.operand();
    if is_kind::<ExprNamedUnknown>(&operand) {
        return Ok(true);
    }
    deref(&operand, "Expr_UnaryExpression::ContainsUnknowns")?.contains_unknowns()
}

/// `Expr_UnaryExpression::Contains`; `unary_contains(&sine, &x)?`.
pub fn unary_contains<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    exp: &ExprHandle,
) -> Result<bool, Failure> {
    let operand: ExprHandle = me.operand();
    if operand == *exp {
        return Ok(true);
    }
    deref(&operand, "Expr_UnaryExpression::Contains")?.contains(exp)
}

/// `Expr_UnaryExpression::Replace`; `unary_replace(&sine, &x, &y)?`.
pub fn unary_replace<T: ExprUnaryExpression + ?Sized>(
    me: &T,
    var: &Handle<ExprNamedUnknown>,
    with: &ExprHandle,
) -> Result<(), Failure> {
    let operand: ExprHandle = me.operand();
    if operand == *var {
        return me.set_operand(with);
    }
    replace_inside(&operand, var, with)
}

/// `if (operand->Contains(var)) operand->Replace(var, with)`, the replacement inside an
/// operand that is not `var` itself; `replace_inside(&operand, &x, &y)?`.
pub fn replace_inside(
    operand: &ExprHandle,
    var: &Handle<ExprNamedUnknown>,
    with: &ExprHandle,
) -> Result<(), Failure> {
    let operand: &dyn ExprGeneralExpression = deref(operand, "Replace")?;
    if operand.contains(&var.cast())? {
        operand.replace(var, with)?;
    }
    Ok(())
}

/// `Expr_UnaryExpression::Simplified`: a copy whose operand is simplified, then
/// simplified itself; `unary_simplified(&sine)?`.
pub fn unary_simplified<T: ExprUnaryExpression + ?Sized>(me: &T) -> Result<ExprHandle, Failure> {
    let copy: Handle<dyn ExprUnaryExpression> = me.copy()?.cast();
    let copy: &dyn ExprUnaryExpression = deref(&copy, "Expr_UnaryExpression::Simplified")?;
    let operand: ExprHandle = copy.operand();
    copy.set_operand(&deref(&operand, "Expr_UnaryExpression::Simplified")?.simplified()?)?;
    copy.shallow_simplified()
}

/// The members every unary class inherits from `Expr_UnaryExpression`: inside an
/// `impl ExprGeneralExpression for Class` block, `unary_expression_members!();`.
macro_rules! unary_expression_members {
    () => {
        fn nb_sub_expressions(&self) -> i32 {
            $crate::expr::unary_expression::unary_nb_sub_expressions()
        }
        fn sub_expression(
            &self,
            index: i32,
        ) -> Result<$crate::expr::ExprHandle, rustycad_tkernel::standard::Failure> {
            $crate::expr::unary_expression::unary_sub_expression(self, index)
        }
        fn contains_unknowns(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::unary_expression::unary_contains_unknowns(self)
        }
        fn contains(
            &self,
            exp: &$crate::expr::ExprHandle,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::unary_expression::unary_contains(self, exp)
        }
        fn replace(
            &self,
            var: &rustycad_tkernel::standard::Handle<$crate::expr::ExprNamedUnknown>,
            with: &$crate::expr::ExprHandle,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::expr::unary_expression::unary_replace(self, var, with)
        }
        fn simplified(
            &self,
        ) -> Result<$crate::expr::ExprHandle, rustycad_tkernel::standard::Failure> {
            $crate::expr::unary_expression::unary_simplified(self)
        }
    };
}
pub(crate) use unary_expression_members;

/// Declare a unary class whose only state is its operand: the struct, its runtime
/// type, its constructor and its `ExprUnaryExpression` impl;
/// `declare_unary_class!(ExprSine, "Expr_Sine", "The sine; `ExprSine::new(&x)`.");`.
macro_rules! declare_unary_class {
    ($class:ident, $name:literal, $doc:literal) => {
        #[doc = $doc]
        pub struct $class {
            parent: rustycad_tkernel::standard::StandardTransient,
            core: $crate::expr::unary_expression::ExprUnaryCore,
        }
        rustycad_tkernel::impl_transient!(
            $class,
            $name,
            dyn $crate::expr::ExprUnaryExpression,
            parent,
            interfaces: [dyn $crate::expr::ExprGeneralExpression, dyn $crate::expr::ExprUnaryExpression]
        );
        impl $class {
            #[doc = concat!("`new ", $name, "(exp)`; `", stringify!($class), "::new(&x)`.")]
            pub fn new(
                exp: &$crate::expr::ExprHandle,
            ) -> rustycad_tkernel::standard::Handle<Self> {
                let object: Self = Self {
                    parent: rustycad_tkernel::standard::StandardTransient::default(),
                    core: $crate::expr::unary_expression::ExprUnaryCore::new(exp),
                };
                $crate::expr::handles::adopt(object, |created| created.core.link())
            }
        }
        impl $crate::expr::ExprUnaryExpression for $class {
            fn unary_core(&self) -> &$crate::expr::unary_expression::ExprUnaryCore {
                &self.core
            }
        }
    };
}
pub(crate) use declare_unary_class;
