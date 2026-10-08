// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_NamedUnknown.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_NamedUnknown`: a variable, which an expression can be assigned to.
use super::errors::{
    invalid_assignment, invalid_operand, not_assigned, not_evaluable, out_of_range,
};
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, ExprSelfLink, adopt, deref, is_kind, lossy};
use super::named_expression::{
    ExprNamedExpression, named_is_identical, named_is_shareable, named_string,
};
use super::numeric_value::ExprNumericValue;
use super::package::copy_share;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A named unknown, free or assigned to an expression;
/// `let x: Handle<ExprNamedUnknown> = ExprNamedUnknown::new(&AsciiString::new("x"));`.
pub struct ExprNamedUnknown {
    parent: StandardTransient,
    name: SharedState<AsciiString>,
    expression: SharedState<ExprHandle>,
    link: ExprSelfLink,
}
impl_transient!(
    ExprNamedUnknown,
    "Expr_NamedUnknown",
    dyn ExprNamedExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression, dyn ExprNamedExpression]
);
impl ExprNamedUnknown {
    /// `new Expr_NamedUnknown(name)`: a free unknown; `ExprNamedUnknown::new(&AsciiString::new("x"))`.
    pub fn new(name: &AsciiString) -> Handle<Self> {
        let object: Self = Self {
            parent: StandardTransient::default(),
            name: SharedState::new(name.clone()),
            expression: SharedState::new(Handle::default()),
            link: ExprSelfLink::default(),
        };
        adopt(object, |unknown| &unknown.link)
    }
    /// True when an expression is assigned (`IsAssigned`); `x.is_assigned()`.
    pub fn is_assigned(&self) -> bool {
        !self.expression.read().is_null()
    }
    /// The assigned expression (`AssignedExpression`); Expr_NotAssigned when free;
    /// `x.assigned_expression()?`.
    pub fn assigned_expression(&self) -> Result<ExprHandle, Failure> {
        let expression: ExprHandle = self.expression.read().clone();
        if expression.is_null() {
            return Err(not_assigned(&format!(
                "Expr_NamedUnknown::AssignedExpression: {} is not assigned",
                lossy(&self.get_name())
            )));
        }
        Ok(expression)
    }
    /// Assign `exp` (`Assign`); Expr_InvalidAssignment when `exp` contains the unknown;
    /// `x.assign(&ExprNumericValue::expression(2.0))?`.
    pub fn assign(&self, exp: &ExprHandle) -> Result<(), Failure> {
        let me: ExprHandle = self.link.me()?;
        if deref(exp, "Expr_NamedUnknown::Assign")?.contains(&me)? {
            return Err(invalid_assignment(&format!(
                "Expr_NamedUnknown::Assign: the expression contains {}; expected an expression without it",
                lossy(&self.get_name())
            )));
        }
        *self.expression.write() = exp.clone();
        Ok(())
    }
    /// Make the unknown free again (`Deassign`); `x.deassign()`.
    pub fn deassign(&self) {
        *self.expression.write() = Handle::default();
    }
    /// The assigned expression or null, without the native check; `x.assigned_or_null()`.
    fn assigned_or_null(&self) -> ExprHandle {
        self.expression.read().clone()
    }
    /// `Contains` of a free unknown: `exp` is a free unknown identical to this one.
    fn free_contains(&self, exp: &ExprHandle) -> Result<bool, Failure> {
        let named: Handle<ExprNamedUnknown> = exp.cast();
        let Some(unknown) = named.get() else {
            return Ok(false);
        };
        if unknown.is_assigned() {
            return Ok(false);
        }
        named_is_identical(self, exp)
    }
    /// `Evaluate` of a free unknown: the value whose variable has the same name.
    fn free_evaluate(
        &self,
        vars: &ExprVariables<'_>,
        vals: &ExprValues<'_>,
    ) -> Result<f64, Failure> {
        let name: AsciiString = self.get_name();
        for (offset, variable) in vars.iter().enumerate() {
            if name == deref(variable, "Expr_NamedUnknown::Evaluate")?.get_name() {
                return value_at(vals, offset);
            }
        }
        Err(not_evaluable(&format!(
            "Expr_NamedUnknown::Evaluate: {} is not among the variables; expected it there",
            lossy(&name)
        )))
    }
    /// `Replace` of an assigned unknown whose expression is `var`.
    fn replace_assignment(&self, with: &ExprHandle) -> Result<(), Failure> {
        let me: ExprHandle = self.link.me()?;
        if deref(with, "Expr_NamedUnknown::Replace")?.contains(&me)? {
            return Err(invalid_operand(&format!(
                "Expr_NamedUnknown::Replace: the replacement contains {}",
                lossy(&self.get_name())
            )));
        }
        self.assign(with)
    }
}
/// `vals(i - vars.Lower() + vals.Lower())`: the value at `offset` from the lower bound;
/// Standard_OutOfRange past the values, as the checked native access.
fn value_at(vals: &ExprValues<'_>, offset: usize) -> Result<f64, Failure> {
    vals.at(offset).copied().map_err(|error| {
        out_of_range(&format!(
            "Expr_NamedUnknown::Evaluate: {error}; expected a value per variable"
        ))
    })
}
impl ExprNamedExpression for ExprNamedUnknown {
    fn name_state(&self) -> &SharedState<AsciiString> {
        &self.name
    }
}
impl ExprGeneralExpression for ExprNamedUnknown {
    fn nb_sub_expressions(&self) -> i32 {
        i32::from(self.is_assigned())
    }
    fn sub_expression(&self, index: i32) -> Result<ExprHandle, Failure> {
        if !self.is_assigned() || index != 1 {
            return Err(out_of_range(&format!(
                "Expr_NamedUnknown::SubExpression: I = {index}, assigned = {}; expected I = 1 on an assigned unknown",
                self.is_assigned()
            )));
        }
        self.assigned_expression()
    }
    fn simplified(&self) -> Result<ExprHandle, Failure> {
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return self.link.me();
        }
        deref(&expression, "Expr_NamedUnknown::Simplified")?.simplified()
    }
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return self.link.me();
        }
        Ok(expression)
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        let copy: Handle<ExprNamedUnknown> = Self::new(&self.get_name());
        let expression: ExprHandle = self.assigned_or_null();
        if !expression.is_null() {
            deref(&copy, "Expr_NamedUnknown::Copy")?.assign(&copy_share(&expression)?)?;
        }
        Ok(copy.cast())
    }
    fn contains_unknowns(&self) -> Result<bool, Failure> {
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return Ok(false);
        }
        if is_kind::<ExprNamedUnknown>(&expression) {
            return Ok(true);
        }
        deref(&expression, "Expr_NamedUnknown::ContainsUnknowns")?.contains_unknowns()
    }
    fn contains(&self, exp: &ExprHandle) -> Result<bool, Failure> {
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return self.free_contains(exp);
        }
        if expression == *exp {
            return Ok(true);
        }
        deref(&expression, "Expr_NamedUnknown::Contains")?.contains(exp)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return Ok(true);
        }
        deref(&expression, "Expr_NamedUnknown::IsLinear")?.is_linear()
    }
    fn is_shareable(&self) -> bool {
        named_is_shareable()
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        named_is_identical(self, other)
    }
    fn derivative(&self, x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        if named_is_identical(self, &x.cast())? {
            return Ok(ExprNumericValue::expression(1.0));
        }
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return Ok(ExprNumericValue::expression(0.0));
        }
        deref(&expression, "Expr_NamedUnknown::Derivative")?.derivative(x)
    }
    fn replace(&self, var: &Handle<ExprNamedUnknown>, with: &ExprHandle) -> Result<(), Failure> {
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return Ok(());
        }
        if expression == *var {
            return self.replace_assignment(with);
        }
        let assigned: &dyn ExprGeneralExpression =
            deref(&expression, "Expr_NamedUnknown::Replace")?;
        if assigned.contains(&var.cast())? {
            assigned.replace(var, with)?;
        }
        Ok(())
    }
    fn evaluate(&self, vars: &ExprVariables<'_>, vals: &ExprValues<'_>) -> Result<f64, Failure> {
        let expression: ExprHandle = self.assigned_or_null();
        if expression.is_null() {
            return self.free_evaluate(vars, vals);
        }
        deref(&expression, "Expr_NamedUnknown::Evaluate")?.evaluate(vars, vals)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        named_string(self)
    }
}
