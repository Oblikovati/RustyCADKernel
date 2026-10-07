// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_PolyFunction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_PolyFunction`: a function of several variables applied to as many operands.
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::general_function::ExprFunctionHandle;
use super::handles::{ExprHandle, deref, one_based};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::expr_multiply;
use super::package::copy_share;
use super::poly_expression::{
    ExprPolyCore, ExprPolyExpression, numeric_operands, operand_list, poly_expression_members,
};
use super::sum::ExprSum;
use super::unary_kit::shallow_of;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// `f(u1, ..., un)`; `ExprPolyFunction::new(&f, &[u, v, w])`.
pub struct ExprPolyFunction {
    parent: StandardTransient,
    core: ExprPolyCore,
    function: ExprFunctionHandle,
}
impl_transient!(
    ExprPolyFunction,
    "Expr_PolyFunction",
    dyn ExprPolyExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression, dyn ExprPolyExpression]
);
impl ExprPolyFunction {
    /// `new Expr_PolyFunction(func, exps)`: the operands in order; the number of
    /// variables is not checked natively; `ExprPolyFunction::new(&f, &[u, v, w])`.
    pub fn new(func: &ExprFunctionHandle, exps: &[ExprHandle]) -> Handle<Self> {
        let object: Self = Self {
            parent: StandardTransient::default(),
            core: ExprPolyCore::new(exps),
            function: func.clone(),
        };
        super::handles::adopt(object, |created| created.core.link())
    }
    /// The applied function (`Function`); `application.function()`.
    pub fn function(&self) -> ExprFunctionHandle {
        self.function.clone()
    }
    /// The function's first `count` variables, in order.
    fn function_variables(&self, count: usize) -> Result<Vec<Handle<ExprNamedUnknown>>, Failure> {
        let function = deref(&self.function, "Expr_PolyFunction")?;
        (1..=i32::try_from(count).unwrap_or(i32::MAX))
            .map(|index| function.variable(index))
            .collect()
    }
    /// `(f'_i(u1, ..., un))->ShallowSimplified() * ui'`, simplified one level: the
    /// `index`-th term of the derivative; the operands are shared, not copied.
    fn partial_term(
        &self,
        index: i32,
        operands: &[ExprHandle],
        x: &Handle<ExprNamedUnknown>,
    ) -> Result<ExprHandle, Failure> {
        let function = deref(&self.function, "Expr_PolyFunction::Derivative")?;
        let variable: Handle<ExprNamedUnknown> = function.variable(index)?;
        let operand: ExprHandle = self.operand(index)?;
        let partial: ExprFunctionHandle = function.derivative(&variable)?;
        let applied = Self::new(&partial, operands);
        let derivative: ExprHandle =
            deref(&operand, "Expr_PolyFunction::Derivative")?.derivative(x)?;
        shallow_of(&expr_multiply(&shallow_of(&applied)?, &derivative))
    }
}
impl ExprPolyExpression for ExprPolyFunction {
    fn poly_core(&self) -> &ExprPolyCore {
        &self.core
    }
}
impl ExprGeneralExpression for ExprPolyFunction {
    poly_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let operands: Vec<ExprHandle> = operand_list(self);
        let (values, others) = numeric_operands(&operands);
        if !others.is_empty() {
            return self.core.link().me();
        }
        let vars: ExprVariables<'static> = one_based(self.function_variables(values.len())?);
        let result: f64 = deref(&self.function, "Expr_PolyFunction::ShallowSimplified")?
            .evaluate(&vars, &one_based(values))?;
        Ok(ExprNumericValue::expression(result))
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        let copies: Vec<ExprHandle> = operand_list(self)
            .iter()
            .map(copy_share)
            .collect::<Result<_, _>>()?;
        Ok(Self::new(&self.function, &copies).cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        let other_expression = deref(other, "Expr_PolyFunction::IsIdentical")?;
        let other: Handle<ExprPolyFunction> = other.cast();
        let Some(other) = other.get() else {
            return Ok(false);
        };
        if other_expression.nb_sub_expressions() != self.nb_sub_expressions() {
            return Ok(false);
        }
        if !deref(&other.function(), "Expr_PolyFunction::IsIdentical")?
            .is_identical(&self.function)?
        {
            return Ok(false);
        }
        for (mine, his) in operand_list(self).iter().zip(operand_list(other)) {
            if !deref(&his, "Expr_PolyFunction::IsIdentical")?.is_identical(mine)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        if !self.contains_unknowns()? {
            return Ok(true);
        }
        let function = deref(&self.function, "Expr_PolyFunction::IsLinear")?;
        for index in 1..=self.nb_operands() {
            if !deref(&self.operand(index)?, "Expr_PolyFunction::IsLinear")?.is_linear()? {
                return Ok(false);
            }
            if !function.is_linear_on_variable(index)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        let operands: Vec<ExprHandle> = operand_list(self);
        let terms: Vec<ExprHandle> = (1..=self.nb_operands())
            .map(|index| self.partial_term(index, &operands, x))
            .collect::<Result<_, _>>()?;
        shallow_of(&ExprSum::new(&terms))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let function = deref(&self.function, "Expr_PolyFunction::Evaluate")?;
        let mut variables: Vec<Handle<ExprNamedUnknown>> = Vec::new();
        let mut values: Vec<f64> = Vec::new();
        for (index, operand) in (1..).zip(operand_list(self)) {
            variables.push(function.variable(index)?);
            values.push(deref(&operand, "Expr_PolyFunction::Evaluate")?.evaluate(vars, vals)?);
        }
        function.evaluate(&one_based(variables), &one_based(values))
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        let mut text: AsciiString =
            deref(&self.function, "Expr_PolyFunction::String")?.get_string_name()?;
        text += "(";
        for (index, operand) in operand_list(self).iter().enumerate() {
            if index > 0 {
                text += ",";
            }
            text += &deref(operand, "Expr_PolyFunction::String")?.string()?;
        }
        text += ")";
        Ok(text)
    }
}
