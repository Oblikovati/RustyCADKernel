// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Product.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Product`: the product of any number of operands.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::{expr_add, expr_multiply, expr_negate};
use super::package::copy_share;
use super::poly_expression::{
    ExprPolyCore, ExprPolyExpression, identical_in_any_order, numeric_operands, operand_list,
    poly_expression_members, spliced_operands,
};
use super::text::infix_text;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A product; `ExprProduct::new_pair(&x, &y).string()? == "x*y"`.
pub struct ExprProduct {
    parent: StandardTransient,
    core: ExprPolyCore,
}
impl_transient!(
    ExprProduct,
    "Expr_Product",
    dyn ExprPolyExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression, dyn ExprPolyExpression]
);
impl ExprProduct {
    /// `new Expr_Product(exps)`: the operands in order; `ExprProduct::new(&[x, y, z])`.
    pub fn new(exps: &[ExprHandle]) -> Handle<Self> {
        let object: Self = Self {
            parent: StandardTransient::default(),
            core: ExprPolyCore::new(exps),
        };
        super::handles::adopt(object, |created| created.core.link())
    }
    /// `new Expr_Product(exp1, exp2)`; `ExprProduct::new_pair(&x, &y)`.
    pub fn new_pair(exp1: &ExprHandle, exp2: &ExprHandle) -> Handle<Self> {
        Self::new(&[exp1.clone(), exp2.clone()])
    }
    /// The simplified product of `operands`, or `None` when there is no value to fold.
    fn folded(&self, operands: &[ExprHandle]) -> Result<Option<ExprHandle>, Failure> {
        let (values, mut others) = numeric_operands(operands);
        let Some(first) = values.first() else {
            return Ok(None);
        };
        // Left to right, as `vals = vals * value`.
        let factor: f64 = values[1..]
            .iter()
            .fold(*first, |factor, value| factor * value);
        if others.is_empty() || factor == 0.0 {
            // A lone value, or the zero that absorbs the other operands.
            return Ok(Some(ExprNumericValue::expression(factor)));
        }
        if factor == 1.0 || factor == -1.0 {
            let rest: ExprHandle = match others.len() {
                1 => others[0].clone(),
                _ => Self::new(&others).cast(),
            };
            return Ok(Some(if factor == 1.0 {
                rest
            } else {
                expr_negate(&rest).cast()
            }));
        }
        if values.len() == 1 {
            return Ok(Some(self.core.link().me()?));
        }
        others.push(ExprNumericValue::expression(factor));
        Ok(Some(Self::new(&others).cast()))
    }
    /// `V`, the product of the operands after the first, of the product rule `U*V`.
    fn tail(&self) -> Result<ExprHandle, Failure> {
        if self.nb_operands() == 2 {
            return copy_share(&self.operand(2)?);
        }
        let tail: Handle<ExprProduct> = expr_multiply(
            &copy_share(&self.operand(2)?)?,
            &copy_share(&self.operand(3)?)?,
        );
        let tail_product: &ExprProduct = deref(&tail, "Expr_Product::Derivative")?;
        for index in 4..=self.nb_operands() {
            tail_product.add_operand(&copy_share(&self.operand(index)?)?);
        }
        Ok(tail.cast())
    }
}
impl ExprPolyExpression for ExprProduct {
    fn poly_core(&self) -> &ExprPolyCore {
        &self.core
    }
}
/// `member->ShallowSimplified()` of a freshly built product.
fn shallow(product: &Handle<ExprProduct>) -> Result<ExprHandle, Failure> {
    deref(product, "Expr_Product::ShallowSimplified")?.shallow_simplified()
}
impl ExprGeneralExpression for ExprProduct {
    poly_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let operands: Vec<ExprHandle> = operand_list(self);
        if operands.iter().any(is_kind::<ExprProduct>) {
            return shallow(&Self::new(&spliced_operands::<ExprProduct>(&operands)?));
        }
        match self.folded(&operands)? {
            Some(result) => Ok(result),
            None => self.core.link().me(),
        }
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        let copies: Vec<ExprHandle> = operand_list(self)
            .iter()
            .map(copy_share)
            .collect::<Result<_, _>>()?;
        Ok(Self::new(&copies).cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        deref(other, "Expr_Product::IsIdentical")?;
        let other: Handle<ExprProduct> = other.cast();
        match other.get() {
            Some(other) => identical_in_any_order(&operand_list(self), &operand_list(other)),
            None => Ok(false),
        }
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        // Linear when at most one operand holds unknowns, and that one is linear.
        let mut seen_unknowns: bool = false;
        for operand in operand_list(self) {
            let expression = deref(&operand, "Expr_Product::IsLinear")?;
            if !is_kind::<ExprNamedUnknown>(&operand) && !expression.contains_unknowns()? {
                continue;
            }
            if seen_unknowns || !expression.is_linear()? {
                return Ok(false);
            }
            seen_unknowns = true;
        }
        Ok(true)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        if !self.contains(&x.cast())? {
            return Ok(ExprNumericValue::expression(0.0));
        }
        let first: ExprHandle = copy_share(&self.operand(1)?)?; // U
        let tail: ExprHandle = self.tail()?; // V
        let first_derivative: ExprHandle =
            deref(&first, "Expr_Product::Derivative")?.derivative(x)?;
        let tail_derivative: ExprHandle =
            deref(&tail, "Expr_Product::Derivative")?.derivative(x)?;
        let first_member: Handle<ExprProduct> = expr_multiply(&first, &tail_derivative); // U*V'
        let second_member: Handle<ExprProduct> = expr_multiply(&first_derivative, &tail); // U'*V
        let result = expr_add(&shallow(&first_member)?, &shallow(&second_member)?);
        deref(&result, "Expr_Product::Derivative")?.shallow_simplified()
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let mut result: f64 = 1.0;
        for operand in operand_list(self) {
            result *= deref(&operand, "Expr_Product::Evaluate")?.evaluate(vars, vals)?;
        }
        Ok(result)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        // The first operand is read unconditionally, as `Operand(1)` natively.
        self.operand(1)?;
        infix_text(&operand_list(self), "*")
    }
}
