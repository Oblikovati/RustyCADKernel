// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_PolyExpression.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_PolyExpression`: an expression with any number of operands.
//!
//! The operands (an `NCollection_Sequence`, private natively) live in an
//! [`ExprPolyCore`] that each concrete class embeds; `poly_expression_members!`
//! writes the inherited members.
use super::errors::{dimension_mismatch, out_of_range};
use super::general_expression::ExprGeneralExpression;
use super::handles::{ExprHandle, ExprSelfLink, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::unary_expression::{check_new_operand, replace_inside};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, SharedState};

/// The operands and the self pointer of a poly expression; `ExprPolyCore::new(&[x, y])`.
#[derive(Default)]
pub struct ExprPolyCore {
    operands: SharedState<Vec<ExprHandle>>,
    link: ExprSelfLink,
}
impl ExprPolyCore {
    /// The core of a new expression, `AddOperand` of each of `exps` in order;
    /// `ExprPolyCore::new(&[x, y])`.
    pub fn new(exps: &[ExprHandle]) -> Self {
        Self {
            operands: SharedState::new(exps.to_vec()),
            link: ExprSelfLink::default(),
        }
    }
    /// The self pointer, bound by the constructor; `core.link().me::<ExprSum>()?`.
    pub fn link(&self) -> &ExprSelfLink {
        &self.link
    }
}

/// The 0-based slot of the 1-based `index` among `count` operands, or the
/// Standard_OutOfRange of the checked sequence access.
fn slot_of(index: i32, count: usize, member: &str) -> Result<usize, Failure> {
    usize::try_from(index)
        .ok()
        .filter(|&position| (1..=count).contains(&position))
        .map(|position| position - 1)
        .ok_or_else(|| {
            out_of_range(&format!(
                "{member}: index = {index}; expected in [1, {count}]"
            ))
        })
}

/// An expression with a sequence of operands; `sum.nb_operands()`.
pub trait ExprPolyExpression: ExprGeneralExpression {
    /// The embedded operands of the concrete object; `sum.poly_core()`.
    fn poly_core(&self) -> &ExprPolyCore;
    /// Number of operands (`NbOperands`); `sum.nb_operands()`.
    fn nb_operands(&self) -> i32 {
        i32::try_from(self.poly_core().operands.read().len()).unwrap_or(i32::MAX)
    }
    /// The `index`-th operand, 1-based (`Operand`); Standard_OutOfRange outside
    /// `1..=nb_operands()`; `sum.operand(1)?`.
    fn operand(&self, index: i32) -> Result<ExprHandle, Failure> {
        let operands = self.poly_core().operands.read();
        let slot: usize = slot_of(index, operands.len(), "Expr_PolyExpression::Operand")?;
        Ok(operands[slot].clone())
    }
    /// Replace the `index`-th operand (`SetOperand`); Expr_InvalidOperand when `exp` is
    /// the expression or contains it; `sum.set_operand(&y, 2)?`.
    fn set_operand(&self, exp: &ExprHandle, index: i32) -> Result<(), Failure> {
        let me: ExprHandle = self.poly_core().link.me()?;
        check_new_operand(&me, exp, "Expr_PolyExpression::SetOperand")?;
        let mut operands = self.poly_core().operands.write();
        let slot: usize = slot_of(index, operands.len(), "Expr_PolyExpression::SetOperand")?;
        operands[slot] = exp.clone();
        Ok(())
    }
    /// Append an operand (`AddOperand`, protected natively); `product.add_operand(&z)`.
    fn add_operand(&self, exp: &ExprHandle) {
        self.poly_core().operands.write().push(exp.clone());
    }
    /// Remove the `index`-th operand (`RemoveOperand`, protected natively);
    /// Standard_DimensionMismatch with 2 operands or fewer; `sum.remove_operand(1)?`.
    fn remove_operand(&self, index: i32) -> Result<(), Failure> {
        let mut operands = self.poly_core().operands.write();
        if operands.len() <= 2 {
            return Err(dimension_mismatch(&format!(
                "Expr_PolyExpression::RemoveOperand: {} operands; expected more than 2",
                operands.len()
            )));
        }
        let slot: usize = slot_of(index, operands.len(), "Expr_PolyExpression::RemoveOperand")?;
        operands.remove(slot);
        Ok(())
    }
}
impl_transient_interface!(
    dyn ExprPolyExpression,
    "Expr_PolyExpression",
    dyn ExprGeneralExpression
);

/// The operands in order, copied out of the lock so that calls on them cannot meet it;
/// `operand_list(&sum).len() == 2`.
pub fn operand_list<T: ExprPolyExpression + ?Sized>(me: &T) -> Vec<ExprHandle> {
    me.poly_core().operands.read().clone()
}

/// Split `operands` into the values of the numeric ones and the others, both in
/// order: the scan of `Expr_Sum` and `Expr_Product::ShallowSimplified`;
/// `numeric_operands(&[two, x]) == (vec![2.0], vec![x])`.
pub fn numeric_operands(operands: &[ExprHandle]) -> (Vec<f64>, Vec<ExprHandle>) {
    let mut values: Vec<f64> = Vec::new();
    let mut others: Vec<ExprHandle> = Vec::new();
    for operand in operands {
        let numeric: Handle<ExprNumericValue> = operand.cast();
        match numeric.get() {
            Some(value) => values.push(value.get_value()),
            None => others.push(operand.clone()),
        }
    }
    (values, others)
}

/// `operands` with every operand of kind `T` (a poly class) replaced by its own
/// operands, one level deep: the flattening of nested sums and products;
/// `spliced_operands::<ExprSum>(&[x, y_plus_z])? == [x, y, z]`.
pub fn spliced_operands<T: ?Sized + RuntimeTyped>(
    operands: &[ExprHandle],
) -> Result<Vec<ExprHandle>, Failure> {
    let mut spliced: Vec<ExprHandle> = Vec::new();
    for operand in operands {
        if !is_kind::<T>(operand) {
            spliced.push(operand.clone());
            continue;
        }
        let inner: Handle<dyn ExprPolyExpression> = operand.cast();
        spliced.extend(operand_list(deref(&inner, "ShallowSimplified")?));
    }
    Ok(spliced)
}

/// The `IsIdentical` of `Expr_Sum` and `Expr_Product`: each of `mine` must match an
/// operand of `his` not matched yet, scanned in order (an identical operand already
/// taken is re-marked and the scan goes on, as the native `tab` loop does);
/// `identical_in_any_order(&[x, y], &[y, x])? == true`.
pub fn identical_in_any_order(mine: &[ExprHandle], his: &[ExprHandle]) -> Result<bool, Failure> {
    if his.len() != mine.len() {
        return Ok(false);
    }
    let mut taken: Vec<usize> = vec![0; his.len()];
    for (position, my_operand) in mine.iter().enumerate() {
        let my_operand = deref(my_operand, "IsIdentical")?;
        let mut found: bool = false;
        for (slot, his_operand) in his.iter().enumerate() {
            if !my_operand.is_identical(his_operand)? {
                continue;
            }
            found = taken[slot] == 0;
            taken[slot] = position + 1;
            if found {
                break;
            }
        }
        if !found {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `Expr_PolyExpression::ContainsUnknowns`: every operand is tested for being an
/// unknown before any is asked; `poly_contains_unknowns(&sum)?`.
pub fn poly_contains_unknowns<T: ExprPolyExpression + ?Sized>(me: &T) -> Result<bool, Failure> {
    let operands: Vec<ExprHandle> = operand_list(me);
    if operands.iter().any(is_kind::<ExprNamedUnknown>) {
        return Ok(true);
    }
    for operand in &operands {
        if deref(operand, "Expr_PolyExpression::ContainsUnknowns")?.contains_unknowns()? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Expr_PolyExpression::Contains`: identity over every operand first, then inside each;
/// `poly_contains(&sum, &x)?`.
pub fn poly_contains<T: ExprPolyExpression + ?Sized>(
    me: &T,
    exp: &ExprHandle,
) -> Result<bool, Failure> {
    let operands: Vec<ExprHandle> = operand_list(me);
    if operands.iter().any(|operand| operand == exp) {
        return Ok(true);
    }
    for operand in &operands {
        if deref(operand, "Expr_PolyExpression::Contains")?.contains(exp)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Expr_PolyExpression::Replace`; `poly_replace(&sum, &x, &y)?`.
pub fn poly_replace<T: ExprPolyExpression + ?Sized>(
    me: &T,
    var: &Handle<ExprNamedUnknown>,
    with: &ExprHandle,
) -> Result<(), Failure> {
    let count: i32 = me.nb_operands();
    for index in 1..=count {
        let operand: ExprHandle = me.operand(index)?;
        if operand == *var {
            me.set_operand(with, index)?;
        } else {
            replace_inside(&operand, var, with)?;
        }
    }
    Ok(())
}

/// `Expr_PolyExpression::Simplified`: a copy whose operands are simplified one by one,
/// then simplified itself; `poly_simplified(&sum)?`.
pub fn poly_simplified<T: ExprPolyExpression + ?Sized>(me: &T) -> Result<ExprHandle, Failure> {
    let copy: Handle<dyn ExprPolyExpression> = me.copy()?.cast();
    let copy: &dyn ExprPolyExpression = deref(&copy, "Expr_PolyExpression::Simplified")?;
    for index in 1..=copy.nb_operands() {
        let operand: ExprHandle = copy.operand(index)?;
        let simplified: ExprHandle =
            deref(&operand, "Expr_PolyExpression::Simplified")?.simplified()?;
        copy.set_operand(&simplified, index)?;
    }
    copy.shallow_simplified()
}

/// The members every poly class inherits from `Expr_PolyExpression`: inside an
/// `impl ExprGeneralExpression for Class` block, `poly_expression_members!();`.
macro_rules! poly_expression_members {
    () => {
        fn nb_sub_expressions(&self) -> i32 {
            $crate::expr::ExprPolyExpression::nb_operands(self)
        }
        fn sub_expression(
            &self,
            index: i32,
        ) -> Result<$crate::expr::ExprHandle, rustycad_tkernel::standard::Failure> {
            $crate::expr::ExprPolyExpression::operand(self, index)
        }
        fn contains_unknowns(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::poly_expression::poly_contains_unknowns(self)
        }
        fn contains(
            &self,
            exp: &$crate::expr::ExprHandle,
        ) -> Result<bool, rustycad_tkernel::standard::Failure> {
            $crate::expr::poly_expression::poly_contains(self, exp)
        }
        fn replace(
            &self,
            var: &rustycad_tkernel::standard::Handle<$crate::expr::ExprNamedUnknown>,
            with: &$crate::expr::ExprHandle,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::expr::poly_expression::poly_replace(self, var, with)
        }
        fn simplified(
            &self,
        ) -> Result<$crate::expr::ExprHandle, rustycad_tkernel::standard::Failure> {
            $crate::expr::poly_expression::poly_simplified(self)
        }
    };
}
pub(crate) use poly_expression_members;
