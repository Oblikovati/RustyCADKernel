// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_NamedFunction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_NamedFunction`: a function defined by a name, an expression and the
//! variables of the expression.
use super::errors::out_of_range;
use super::function_derivative::ExprFunctionDerivative;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::general_function::{ExprFunctionHandle, ExprGeneralFunction};
use super::handles::{ExprHandle, ExprSelfLink, adopt, deref};
use super::named_unknown::ExprNamedUnknown;
use super::package::copy_share;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// The variables of a function with the bounds of the array they came from, since
/// `Variable(index)` reads them by those bounds.
struct FunctionVariables {
    lower: i32,
    unknowns: Vec<Handle<ExprNamedUnknown>>,
}

/// The mutable fields: the name and the expression.
struct NamedFunctionState {
    name: AsciiString,
    expression: ExprHandle,
}

/// A named function `f(x, y) = expression`;
/// `ExprNamedFunction::new(&AsciiString::new("f"), &body, &vars).get_name() == "f"`.
pub struct ExprNamedFunction {
    parent: StandardTransient,
    state: SharedState<NamedFunctionState>,
    variables: FunctionVariables,
    link: ExprSelfLink,
}
impl_transient!(
    ExprNamedFunction,
    "Expr_NamedFunction",
    dyn ExprGeneralFunction,
    parent,
    interfaces: [dyn ExprGeneralFunction]
);
impl ExprNamedFunction {
    /// `new Expr_NamedFunction(name, exp, vars)`: the variables keep the bounds of
    /// `vars`; `ExprNamedFunction::new(&AsciiString::new("f"), &body, &vars)`.
    pub fn new(name: &AsciiString, exp: &ExprHandle, vars: &ExprVariables<'_>) -> Handle<Self> {
        let object: Self = Self {
            parent: StandardTransient::default(),
            state: SharedState::new(NamedFunctionState {
                name: name.clone(),
                expression: exp.clone(),
            }),
            variables: FunctionVariables {
                lower: vars.lower(),
                unknowns: vars.iter().cloned().collect(),
            },
            link: ExprSelfLink::default(),
        };
        adopt(object, |function| &function.link)
    }
    /// Rename the function (`SetName`); `function.set_name(&AsciiString::new("g"))`.
    pub fn set_name(&self, newname: &AsciiString) {
        self.state.write().name = newname.clone();
    }
    /// The name (`GetName`); `function.get_name()`.
    pub fn get_name(&self) -> AsciiString {
        self.state.read().name.clone()
    }
    /// The defining expression (`Expression`); `function.expression()`.
    pub fn expression(&self) -> ExprHandle {
        self.state.read().expression.clone()
    }
    /// Replace the defining expression (`SetExpression`); `function.set_expression(&body)`.
    pub fn set_expression(&self, exp: &ExprHandle) {
        self.state.write().expression = exp.clone();
    }
    /// The variables as the array the function was built from.
    fn variable_array(&self) -> ExprVariables<'static> {
        let mut array: ExprVariables<'static> =
            super::handles::one_based(self.variables.unknowns.clone());
        array.update_lower_bound(self.variables.lower);
        array
    }
}
impl ExprGeneralFunction for ExprNamedFunction {
    fn nb_of_variables(&self) -> i32 {
        i32::try_from(self.variables.unknowns.len()).unwrap_or(i32::MAX)
    }
    fn variable(&self, index: i32) -> Result<Handle<ExprNamedUnknown>, Failure> {
        let offset: Option<usize> = index
            .checked_sub(self.variables.lower)
            .and_then(|offset| usize::try_from(offset).ok());
        offset
            .and_then(|offset| self.variables.unknowns.get(offset))
            .cloned()
            .ok_or_else(|| {
                out_of_range(&format!(
                    "Expr_NamedFunction::Variable: index = {index}; expected in [{}, {}]",
                    self.variables.lower,
                    self.variables.lower + self.nb_of_variables() - 1
                ))
            })
    }
    fn copy(&self) -> Result<ExprFunctionHandle, Failure> {
        let expression: ExprHandle = copy_share(&self.expression())?;
        Ok(Self::new(&self.get_name(), &expression, &self.variable_array()).cast())
    }
    fn derivative(&self, var: &Handle<ExprNamedUnknown>) -> Result<ExprFunctionHandle, Failure> {
        self.derivative_of_degree(var, 1)
    }
    fn derivative_of_degree(
        &self,
        var: &Handle<ExprNamedUnknown>,
        deg: i32,
    ) -> Result<ExprFunctionHandle, Failure> {
        let me: ExprFunctionHandle = self.link.me()?;
        Ok(ExprFunctionDerivative::new(&me, var, deg)?.cast())
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        if vars.length() != vals.length() {
            return Err(out_of_range(&format!(
                "Expr_NamedFunction::Evaluate: {} variables and {} values; expected as many",
                vars.length(),
                vals.length()
            )));
        }
        deref(&self.expression(), "Expr_NamedFunction::Evaluate")?.evaluate(vars, vals)
    }
    fn is_identical(&self, func: &ExprFunctionHandle) -> Result<bool, Failure> {
        let function = deref(func, "Expr_NamedFunction::IsIdentical")?;
        let named: Handle<ExprNamedFunction> = func.cast();
        let Some(named) = named.get() else {
            return Ok(false);
        };
        if self.get_name() != named.get_name()
            || self.nb_of_variables() != function.nb_of_variables()
        {
            return Ok(false);
        }
        for index in 1..=self.nb_of_variables() {
            let variable: Handle<ExprNamedUnknown> = self.variable(index)?;
            let other: ExprHandle = function.variable(index)?.cast();
            if !deref(&variable, "Expr_NamedFunction::IsIdentical")?.is_identical(&other)? {
                return Ok(false);
            }
        }
        deref(&self.expression(), "Expr_NamedFunction::IsIdentical")?
            .is_identical(&named.expression())
    }
    fn is_linear_on_variable(&self, _index: i32) -> Result<bool, Failure> {
        deref(&self.expression(), "Expr_NamedFunction::IsLinearOnVariable")?.is_linear()
    }
    fn get_string_name(&self) -> Result<AsciiString, Failure> {
        Ok(self.get_name())
    }
}
