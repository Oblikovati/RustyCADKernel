// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Sum.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_Sum`: the sum of any number of operands.
use super::errors::out_of_range;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, deref, is_kind};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::package::copy_share;
use super::poly_expression::{
    ExprPolyCore, ExprPolyExpression, identical_in_any_order, numeric_operands, operand_list,
    poly_expression_members, spliced_operands,
};
use super::text::infix_text;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A sum; `ExprSum::new_pair(&x, &y).string()? == "x+y"`.
pub struct ExprSum {
    parent: StandardTransient,
    core: ExprPolyCore,
}
impl_transient!(
    ExprSum,
    "Expr_Sum",
    dyn ExprPolyExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression, dyn ExprPolyExpression]
);
impl ExprSum {
    /// `new Expr_Sum(exps)`: the operands in order; `ExprSum::new(&[x, y, z])`.
    pub fn new(exps: &[ExprHandle]) -> Handle<Self> {
        let object: Self = Self {
            parent: StandardTransient::default(),
            core: ExprPolyCore::new(exps),
        };
        super::handles::adopt(object, |created| created.core.link())
    }
    /// `new Expr_Sum(exp1, exp2)`; `ExprSum::new_pair(&x, &y)`.
    pub fn new_pair(exp1: &ExprHandle, exp2: &ExprHandle) -> Handle<Self> {
        Self::new(&[exp1.clone(), exp2.clone()])
    }
    /// The simplified sum of `operands`, or `None` when there is no value to fold.
    fn folded(&self, operands: &[ExprHandle]) -> Result<Option<ExprHandle>, Failure> {
        let (values, mut others) = numeric_operands(operands);
        let Some(first) = values.first() else {
            return Ok(None);
        };
        // Left to right, as `vals = vals + value`.
        let total: f64 = values[1..]
            .iter()
            .fold(*first, |total, value| total + value);
        if others.is_empty() {
            return Ok(Some(ExprNumericValue::expression(total)));
        }
        if total != 0.0 {
            if values.len() == 1 {
                return Ok(Some(self.core.link().me()?));
            }
            others.push(ExprNumericValue::expression(total));
            return Ok(Some(Self::new(&others).cast()));
        }
        if others.len() == 1 {
            return Ok(Some(others[0].clone()));
        }
        Ok(Some(Self::new(&others).cast()))
    }
}
impl ExprPolyExpression for ExprSum {
    fn poly_core(&self) -> &ExprPolyCore {
        &self.core
    }
}
impl ExprGeneralExpression for ExprSum {
    poly_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let operands: Vec<ExprHandle> = operand_list(self);
        if operands.iter().any(is_kind::<ExprSum>) {
            let spliced: Vec<ExprHandle> = spliced_operands::<ExprSum>(&operands)?;
            return deref(&Self::new(&spliced), "Expr_Sum::ShallowSimplified")?
                .shallow_simplified();
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
        deref(other, "Expr_Sum::IsIdentical")?;
        let other: Handle<ExprSum> = other.cast();
        match other.get() {
            Some(other) => identical_in_any_order(&operand_list(self), &operand_list(other)),
            None => Ok(false),
        }
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        for operand in operand_list(self) {
            if !deref(&operand, "Expr_Sum::IsLinear")?.is_linear()? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        let derivatives: Vec<ExprHandle> = operand_list(self)
            .iter()
            .map(|operand| deref(operand, "Expr_Sum::Derivative")?.derivative(x))
            .collect::<Result<_, _>>()?;
        deref(&Self::new(&derivatives), "Expr_Sum::Derivative")?.shallow_simplified()
    }
    fn n_derivative(&self, x: &Handle<ExprNamedUnknown>, n: i32) -> Result<ExprHandle, Failure> {
        if n <= 0 {
            return Err(out_of_range(&format!(
                "Expr_Sum::NDerivative: N = {n}; expected >= 1"
            )));
        }
        let derivatives: Vec<ExprHandle> = operand_list(self)
            .iter()
            .map(|operand| deref(operand, "Expr_Sum::NDerivative")?.n_derivative(x, n))
            .collect::<Result<_, _>>()?;
        deref(&Self::new(&derivatives), "Expr_Sum::NDerivative")?.shallow_simplified()
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let mut result: f64 = 0.0;
        for operand in operand_list(self) {
            result += deref(&operand, "Expr_Sum::Evaluate")?.evaluate(vars, vals)?;
        }
        Ok(result)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        // The first operand is read unconditionally, as `Operand(1)` natively.
        self.operand(1)?;
        infix_text(&operand_list(self), "+")
    }
}
