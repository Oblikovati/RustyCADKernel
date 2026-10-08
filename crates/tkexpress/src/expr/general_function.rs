// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_GeneralFunction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_GeneralFunction`: a function of named variables.
use super::general_expression::{ExprValues, ExprVariables};
use super::named_unknown::ExprNamedUnknown;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;

/// `occ::handle<Expr_GeneralFunction>`; `let f: ExprFunctionHandle = named.cast();`.
pub type ExprFunctionHandle = Handle<dyn ExprGeneralFunction>;

/// A function of named variables, defined by an expression or derived from one;
/// `function.evaluate(&vars, &vals)?`.
pub trait ExprGeneralFunction: Transient {
    /// Number of variables (`NbOfVariables`); `function.nb_of_variables()`.
    fn nb_of_variables(&self) -> i32;
    /// The `index`-th variable (`Variable`); Standard_OutOfRange outside the bounds;
    /// `function.variable(1)?`.
    fn variable(&self, index: i32) -> Result<Handle<ExprNamedUnknown>, Failure>;
    /// A copy (`Copy`); `function.copy()?`.
    fn copy(&self) -> Result<ExprFunctionHandle, Failure>;
    /// The derivative on `var` (`Derivative(var)`); `function.derivative(&x)?`.
    fn derivative(&self, var: &Handle<ExprNamedUnknown>) -> Result<ExprFunctionHandle, Failure>;
    /// The `deg`-th derivative on `var` (`Derivative(var, deg)`); Standard_OutOfRange
    /// for `deg <= 0`; `function.derivative_of_degree(&x, 2)?`.
    fn derivative_of_degree(
        &self,
        var: &Handle<ExprNamedUnknown>,
        deg: i32,
    ) -> Result<ExprFunctionHandle, Failure>;
    /// The value with `vars` replaced by `vals` (`Evaluate`); Standard_OutOfRange when
    /// the arrays differ in length; `function.evaluate(&vars, &vals)?`.
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure>;
    /// True when `func` is the same function (`IsIdentical`); `function.is_identical(&other)?`.
    fn is_identical(&self, func: &ExprFunctionHandle) -> Result<bool, Failure>;
    /// True when linear on the `index`-th variable (`IsLinearOnVariable`; the native
    /// classes answer for the whole expression); `function.is_linear_on_variable(1)?`.
    fn is_linear_on_variable(&self, index: i32) -> Result<bool, Failure>;
    /// The name used in texts (`GetStringName`); `function.get_string_name()?`.
    fn get_string_name(&self) -> Result<AsciiString, Failure>;
}
impl_transient_interface!(
    dyn ExprGeneralFunction,
    "Expr_GeneralFunction",
    StandardTransient
);
