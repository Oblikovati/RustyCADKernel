// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_UnaryFunction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_UnaryFunction`: a function of one variable applied to an operand.
use super::errors::invalid_function;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::general_function::ExprFunctionHandle;
use super::handles::{ExprHandle, deref, one_based};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use super::operators::expr_multiply;
use super::package::copy_share;
use super::unary_expression::{ExprUnaryCore, ExprUnaryExpression, unary_expression_members};
use super::unary_kit::{evaluate_operand, shallow_of};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// `f(u)` for a function `f` of one variable; `ExprUnaryFunction::new(&f, &u)?`.
pub struct ExprUnaryFunction {
    parent: StandardTransient,
    core: ExprUnaryCore,
    function: ExprFunctionHandle,
}
impl_transient!(
    ExprUnaryFunction,
    "Expr_UnaryFunction",
    dyn ExprUnaryExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression, dyn ExprUnaryExpression]
);
impl ExprUnaryFunction {
    /// `new Expr_UnaryFunction(func, exp)`: Expr_InvalidFunction unless `func` has one
    /// variable; `ExprUnaryFunction::new(&f, &u)?`.
    pub fn new(func: &ExprFunctionHandle, exp: &ExprHandle) -> Result<Handle<Self>, Failure> {
        let count: i32 = deref(func, "Expr_UnaryFunction")?.nb_of_variables();
        if count != 1 {
            return Err(invalid_function(&format!(
                "Expr_UnaryFunction: the function has {count} variables; expected 1"
            )));
        }
        let object: Self = Self {
            parent: StandardTransient::default(),
            core: ExprUnaryCore::new(exp),
            function: func.clone(),
        };
        Ok(super::handles::adopt(object, |created| created.core.link()))
    }
    /// The applied function (`Function`); `application.function()`.
    pub fn function(&self) -> ExprFunctionHandle {
        self.function.clone()
    }
    /// `f(value)`: the function evaluated on its own variable set to `value`.
    fn apply(&self, value: f64) -> Result<f64, Failure> {
        let function = deref(&self.function, "Expr_UnaryFunction")?;
        let vars: ExprVariables<'static> = one_based(vec![function.variable(1)?]);
        function.evaluate(&vars, &one_based(vec![value]))
    }
}
impl ExprUnaryExpression for ExprUnaryFunction {
    fn unary_core(&self) -> &ExprUnaryCore {
        &self.core
    }
}
impl ExprGeneralExpression for ExprUnaryFunction {
    unary_expression_members!();
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let operand: ExprHandle = self.operand();
        let value: Handle<ExprNumericValue> = operand.cast();
        match value.get() {
            Some(value) => Ok(ExprNumericValue::expression(self.apply(value.get_value())?)),
            None => self.core.link().me(),
        }
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        Ok(Self::new(&self.function, &copy_share(&self.operand())?)?.cast())
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        deref(other, "Expr_UnaryFunction::IsIdentical")?;
        let other: Handle<ExprUnaryFunction> = other.cast();
        let Some(other) = other.get() else {
            return Ok(false);
        };
        let other_operand: ExprHandle = other.operand();
        if !deref(&other_operand, "Expr_UnaryFunction::IsIdentical")?
            .is_identical(&self.operand())?
        {
            return Ok(false);
        }
        deref(&self.function, "Expr_UnaryFunction::IsIdentical")?.is_identical(&other.function())
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        if !self.contains_unknowns()? {
            return Ok(true);
        }
        if !deref(&self.operand(), "Expr_UnaryFunction::IsLinear")?.is_linear()? {
            return Ok(false);
        }
        deref(&self.function, "Expr_UnaryFunction::IsLinear")?.is_linear_on_variable(1)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        let function = deref(&self.function, "Expr_UnaryFunction::Derivative")?;
        let variable: Handle<ExprNamedUnknown> = function.variable(1)?;
        let operand: ExprHandle = self.operand();
        let operand_derivative: ExprHandle =
            deref(&operand, "Expr_UnaryFunction::Derivative")?.derivative(x)?;
        let function_derivative: ExprFunctionHandle = function.derivative(&variable)?;
        let first_part = Self::new(&function_derivative, &copy_share(&operand)?)?;
        shallow_of(&expr_multiply(
            &shallow_of(&first_part)?,
            &operand_derivative,
        ))
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let function = deref(&self.function, "Expr_UnaryFunction::Evaluate")?;
        let variables: ExprVariables<'static> = one_based(vec![function.variable(1)?]);
        let values = one_based(vec![evaluate_operand(self, vars, vals)?]);
        function.evaluate(&variables, &values)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        let mut text: AsciiString =
            deref(&self.function, "Expr_UnaryFunction::String")?.get_string_name()?;
        text += "(";
        text += &deref(&self.operand(), "Expr_UnaryFunction::String")?.string()?;
        text += ")";
        Ok(text)
    }
}
