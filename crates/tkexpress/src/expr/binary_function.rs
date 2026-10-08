// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_BinaryFunction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_BinaryFunction`: a function of two variables applied to two operands.
use super::binary_expression::{
    ExprBinaryCore, ExprBinaryExpression, binary_expression_members, evaluate_operands,
};
use super::errors::invalid_function;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::general_function::ExprFunctionHandle;
use super::handles::{ExprHandle, deref, one_based};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::{expr_add, expr_multiply};
use super::package::copy_share;
use super::unary_kit::shallow_of;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// `f(u, v)` for a function `f` of two variables; `ExprBinaryFunction::new(&f, &u, &v)?`.
pub struct ExprBinaryFunction {
    parent: StandardTransient,
    core: ExprBinaryCore,
    function: ExprFunctionHandle,
}
impl_transient!(
    ExprBinaryFunction,
    "Expr_BinaryFunction",
    dyn ExprBinaryExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression, dyn ExprBinaryExpression]
);
impl ExprBinaryFunction {
    /// `new Expr_BinaryFunction(func, exp1, exp2)`: Expr_InvalidFunction unless `func`
    /// has two variables; `ExprBinaryFunction::new(&f, &u, &v)?`.
    pub fn new(
        func: &ExprFunctionHandle,
        exp1: &ExprHandle,
        exp2: &ExprHandle,
    ) -> Result<Handle<Self>, Failure> {
        let count: i32 = deref(func, "Expr_BinaryFunction")?.nb_of_variables();
        if count != 2 {
            return Err(invalid_function(&format!(
                "Expr_BinaryFunction: the function has {count} variables; expected 2"
            )));
        }
        let object: Self = Self {
            parent: StandardTransient::default(),
            core: ExprBinaryCore::new(exp1, exp2),
            function: func.clone(),
        };
        Ok(super::handles::adopt(object, |created| created.core.link()))
    }
    /// The applied function (`Function`); `application.function()`.
    pub fn function(&self) -> ExprFunctionHandle {
        self.function.clone()
    }
    /// `f(first, second)` on the function's own variables.
    fn apply(&self, first: f64, second: f64) -> Result<f64, Failure> {
        let function = deref(&self.function, "Expr_BinaryFunction")?;
        let vars: ExprVariables<'static> =
            one_based(vec![function.variable(1)?, function.variable(2)?]);
        function.evaluate(&vars, &one_based(vec![first, second]))
    }
    /// `(new Expr_BinaryFunction(f'_var, u, v))->ShallowSimplified() * derivative`,
    /// simplified one level: one term of the derivative.
    fn partial_term(
        &self,
        var: &Handle<ExprNamedUnknown>,
        derivative: &ExprHandle,
    ) -> Result<ExprHandle, Failure> {
        let partial: ExprFunctionHandle =
            deref(&self.function, "Expr_BinaryFunction::Derivative")?.derivative(var)?;
        let first: ExprHandle = copy_share(&self.first_operand())?;
        let applied = Self::new(&partial, &first, &copy_share(&self.second_operand())?)?;
        shallow_of(&expr_multiply(&shallow_of(&applied)?, derivative))
    }
}
impl ExprBinaryExpression for ExprBinaryFunction {
    fn binary_core(&self) -> &ExprBinaryCore {
        &self.core
    }
}
/// The value of a numeric operand, `None` otherwise.
fn value_of(operand: &ExprHandle) -> Option<f64> {
    operand
        .cast::<ExprNumericValue>()
        .get()
        .map(ExprNumericValue::get_value)
}
impl ExprGeneralExpression for ExprBinaryFunction {
    binary_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        match (
            value_of(&self.first_operand()),
            value_of(&self.second_operand()),
        ) {
            (Some(first), Some(second)) => {
                Ok(ExprNumericValue::expression(self.apply(first, second)?))
            }
            _ => self.core.link().me(),
        }
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        let first: ExprHandle = copy_share(&self.first_operand())?;
        Ok(Self::new(&self.function, &first, &copy_share(&self.second_operand())?)?.cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        deref(other, "Expr_BinaryFunction::IsIdentical")?;
        let other: Handle<ExprBinaryFunction> = other.cast();
        let Some(other) = other.get() else {
            return Ok(false);
        };
        if !deref(&other.first_operand(), "Expr_BinaryFunction::IsIdentical")?
            .is_identical(&self.first_operand())?
        {
            return Ok(false);
        }
        if !deref(&other.second_operand(), "Expr_BinaryFunction::IsIdentical")?
            .is_identical(&self.second_operand())?
        {
            return Ok(false);
        }
        deref(&self.function, "Expr_BinaryFunction::IsIdentical")?.is_identical(&other.function())
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        if !self.contains_unknowns()? {
            return Ok(true);
        }
        if !deref(&self.first_operand(), "Expr_BinaryFunction::IsLinear")?.is_linear()? {
            return Ok(false);
        }
        if !deref(&self.second_operand(), "Expr_BinaryFunction::IsLinear")?.is_linear()? {
            return Ok(false);
        }
        let function = deref(&self.function, "Expr_BinaryFunction::IsLinear")?;
        Ok(function.is_linear_on_variable(1)? && function.is_linear_on_variable(2)?)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        let function = deref(&self.function, "Expr_BinaryFunction::Derivative")?;
        let first_variable: Handle<ExprNamedUnknown> = function.variable(1)?;
        let second_variable: Handle<ExprNamedUnknown> = function.variable(2)?;
        let first_derivative: ExprHandle =
            deref(&self.first_operand(), "Expr_BinaryFunction::Derivative")?.derivative(x)?;
        let second_derivative: ExprHandle =
            deref(&self.second_operand(), "Expr_BinaryFunction::Derivative")?.derivative(x)?;
        let first_term: ExprHandle = self.partial_term(&first_variable, &first_derivative)?;
        let second_term: ExprHandle = self.partial_term(&second_variable, &second_derivative)?;
        shallow_of(&expr_add(&first_term, &second_term))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let function = deref(&self.function, "Expr_BinaryFunction::Evaluate")?;
        let variables: ExprVariables<'static> =
            one_based(vec![function.variable(1)?, function.variable(2)?]);
        let (first, second) = evaluate_operands(self, vars, vals)?;
        function.evaluate(&variables, &one_based(vec![first, second]))
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        let mut text: AsciiString =
            deref(&self.function, "Expr_BinaryFunction::String")?.get_string_name()?;
        text += "(";
        text += &deref(&self.first_operand(), "Expr_BinaryFunction::String")?.string()?;
        text += ",";
        text += &deref(&self.second_operand(), "Expr_BinaryFunction::String")?.string()?;
        text += ")";
        Ok(text)
    }
}
