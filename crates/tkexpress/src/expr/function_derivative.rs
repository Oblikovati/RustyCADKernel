// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_FunctionDerivative.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_FunctionDerivative`: the derivative of a function on one of its variables.
use super::errors::{null_object, out_of_range};
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::general_function::{ExprFunctionHandle, ExprGeneralFunction};
use super::handles::{ExprHandle, ExprSelfLink, adopt, deref};
use super::named_function::ExprNamedFunction;
use super::named_unknown::ExprNamedUnknown;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// The `deg`-th derivative of a function on one variable, with its expression;
/// `ExprFunctionDerivative::new(&f, &x, 1)?.degree() == 1`.
pub struct ExprFunctionDerivative {
    parent: StandardTransient,
    function: ExprFunctionHandle,
    derivate: Handle<ExprNamedUnknown>,
    degree: i32,
    expression: SharedState<ExprHandle>,
    link: ExprSelfLink,
}
impl_transient!(
    ExprFunctionDerivative,
    "Expr_FunctionDerivative",
    dyn ExprGeneralFunction,
    parent,
    interfaces: [dyn ExprGeneralFunction]
);
impl ExprFunctionDerivative {
    /// `new Expr_FunctionDerivative(func, withX, deg)`: Standard_OutOfRange for
    /// `deg <= 0`; the expression is derived at once (`UpdateExpression`);
    /// `ExprFunctionDerivative::new(&f, &x, 2)?`.
    pub fn new(
        func: &ExprFunctionHandle,
        with_x: &Handle<ExprNamedUnknown>,
        deg: i32,
    ) -> Result<Handle<Self>, Failure> {
        if deg <= 0 {
            return Err(out_of_range(&format!(
                "Expr_FunctionDerivative: degree = {deg}; expected >= 1"
            )));
        }
        let object: Self = Self {
            parent: StandardTransient::default(),
            function: func.clone(),
            derivate: with_x.clone(),
            degree: deg,
            expression: SharedState::new(Handle::default()),
            link: ExprSelfLink::default(),
        };
        let derivative: Handle<Self> = adopt(object, |derivative| &derivative.link);
        deref(&derivative, "Expr_FunctionDerivative")?.update_expression()?;
        Ok(derivative)
    }
    /// The derived function (`Function`); `derivative.function()`.
    pub fn function(&self) -> ExprFunctionHandle {
        self.function.clone()
    }
    /// The degree of derivation (`Degree`); `derivative.degree()`.
    pub fn degree(&self) -> i32 {
        self.degree
    }
    /// The variable of derivation (`DerivVariable`); `derivative.deriv_variable()`.
    pub fn deriv_variable(&self) -> Handle<ExprNamedUnknown> {
        self.derivate.clone()
    }
    /// The derived expression (`Expression`); `derivative.expression()`.
    pub fn expression(&self) -> ExprHandle {
        self.expression.read().clone()
    }
    /// Derive the expression of the function again (`UpdateExpression`), after updating
    /// an inner derivative first; a function that is neither named nor a derivative
    /// is a null dereference natively; `derivative.update_expression()?`.
    pub fn update_expression(&self) -> Result<(), Failure> {
        let inner: Handle<ExprFunctionDerivative> = self.function.cast();
        let source: ExprHandle = match inner.get() {
            Some(inner) => {
                inner.update_expression()?;
                inner.expression()
            }
            None => self.named_expression()?,
        };
        let derived: ExprHandle = deref(&source, "Expr_FunctionDerivative::UpdateExpression")?
            .n_derivative(&self.derivate, self.degree)?;
        *self.expression.write() = derived;
        Ok(())
    }
    /// The expression of the derived named function.
    fn named_expression(&self) -> Result<ExprHandle, Failure> {
        let named: Handle<ExprNamedFunction> = self.function.cast();
        named.get().map(ExprNamedFunction::expression).ok_or_else(|| {
            null_object(
                "Expr_FunctionDerivative::UpdateExpression: the function is neither an Expr_NamedFunction nor an Expr_FunctionDerivative",
            )
        })
    }
    /// The 1-based rank of the variable of derivation among the function's variables,
    /// compared by identity; 0 when absent.
    fn derivate_rank(&self) -> Result<i32, Failure> {
        for index in 1..=self.nb_of_variables() {
            if self.variable(index)? == self.derivate {
                return Ok(index);
            }
        }
        Ok(0)
    }
}
impl ExprGeneralFunction for ExprFunctionDerivative {
    fn nb_of_variables(&self) -> i32 {
        self.function
            .get()
            .map_or(0, |function| function.nb_of_variables())
    }
    fn variable(&self, index: i32) -> Result<Handle<ExprNamedUnknown>, Failure> {
        deref(&self.function, "Expr_FunctionDerivative::Variable")?.variable(index)
    }
    fn copy(&self) -> Result<ExprFunctionHandle, Failure> {
        let function: ExprFunctionHandle =
            deref(&self.function, "Expr_FunctionDerivative::Copy")?.copy()?;
        Ok(Self::new(&function, &self.derivate, self.degree)?.cast())
    }
    fn derivative(&self, var: &Handle<ExprNamedUnknown>) -> Result<ExprFunctionHandle, Failure> {
        self.derivative_of_degree(var, 1)
    }
    fn derivative_of_degree(
        &self,
        var: &Handle<ExprNamedUnknown>,
        deg: i32,
    ) -> Result<ExprFunctionHandle, Failure> {
        if *var == self.derivate {
            return Ok(Self::new(&self.function, var, self.degree + deg)?.cast());
        }
        let me: ExprFunctionHandle = self.link.me()?;
        Ok(Self::new(&me, var, deg)?.cast())
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        if vars.length() != vals.length() {
            return Err(out_of_range(&format!(
                "Expr_FunctionDerivative::Evaluate: {} variables and {} values; expected as many",
                vars.length(),
                vals.length()
            )));
        }
        deref(&self.expression(), "Expr_FunctionDerivative::Evaluate")?.evaluate(vars, vals)
    }
    fn is_identical(&self, func: &ExprFunctionHandle) -> Result<bool, Failure> {
        deref(func, "Expr_FunctionDerivative::IsIdentical")?;
        let other: Handle<ExprFunctionDerivative> = func.cast();
        let Some(other) = other.get() else {
            return Ok(false);
        };
        if self.degree != other.degree() {
            return Ok(false);
        }
        let derivate = deref(&self.derivate, "Expr_FunctionDerivative::IsIdentical")?;
        if !derivate.is_identical(&other.deriv_variable().cast())? {
            return Ok(false);
        }
        deref(&self.function, "Expr_FunctionDerivative::IsIdentical")?
            .is_identical(&other.function())
    }
    fn is_linear_on_variable(&self, _index: i32) -> Result<bool, Failure> {
        deref(
            &self.expression(),
            "Expr_FunctionDerivative::IsLinearOnVariable",
        )?
        .is_linear()
    }
    fn get_string_name(&self) -> Result<AsciiString, Failure> {
        let function_name: AsciiString =
            deref(&self.function, "Expr_FunctionDerivative::GetStringName")?.get_string_name()?;
        if self.nb_of_variables() == 1 {
            // f'' : one quote per degree.
            let mut text: AsciiString = function_name;
            text += &AsciiString::filled(usize::try_from(self.degree).unwrap_or(0), b'\'');
            return Ok(text);
        }
        // @2f/@2X1 : the degree, the function and the rank of the variable.
        let mut differential: AsciiString = AsciiString::new("@");
        if self.degree > 1 {
            differential += self.degree;
        }
        let mut text: AsciiString = differential.clone();
        text += &function_name;
        text += "/";
        text += &differential;
        text += "X";
        text += self.derivate_rank()?;
        Ok(text)
    }
}
