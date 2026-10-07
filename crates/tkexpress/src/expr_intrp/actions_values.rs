// Rust translation of the operator, value and assignment actions of
// ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_yaccintrf.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The semantic actions of operators, values, unknowns, constants and assignments.
use super::context::ExprIntrpParseContext;
use super::errors::syntax_error;
use crate::expr::handles::deref;
use crate::expr::operators::{expr_add, expr_divide, expr_multiply, expr_subtract};
use crate::expr::{
    ExprExponentiate, ExprGeneralExpression, ExprHandle, ExprNamedConstant, ExprNamedExpression,
    ExprNamedUnknown, ExprNumericValue, ExprUnaryMinus,
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};

/// `handle->ShallowSimplified()` of a freshly built operation.
fn simplified<T: ?Sized + RuntimeTyped + ExprGeneralExpression>(
    operation: &Handle<T>,
) -> Result<ExprHandle, Failure> {
    deref(operation, "ExprIntrp")?.shallow_simplified()
}

impl ExprIntrpParseContext<'_> {
    /// Pop the two operands of a binary operator: the second, then the first.
    fn pop_operands(&mut self) -> (ExprHandle, ExprHandle) {
        let second: ExprHandle = self.recept.pop();
        let first: ExprHandle = self.recept.pop();
        (first, second)
    }
    /// `ExprIntrp_SumOperator`: `op1 + op2`, simplified one level.
    pub(super) fn sum_operator(&mut self) -> Result<(), Failure> {
        let (first, second) = self.pop_operands();
        let result: ExprHandle = simplified(&expr_add(&first, &second))?;
        self.recept.push(&result);
        Ok(())
    }
    /// `ExprIntrp_MinusOperator`: `op1 - op2`, simplified one level.
    pub(super) fn minus_operator(&mut self) -> Result<(), Failure> {
        let (first, second) = self.pop_operands();
        let result: ExprHandle = simplified(&expr_subtract(&first, &second))?;
        self.recept.push(&result);
        Ok(())
    }
    /// `ExprIntrp_DivideOperator`: `op1 / op2`, simplified one level.
    pub(super) fn divide_operator(&mut self) -> Result<(), Failure> {
        let (first, second) = self.pop_operands();
        let result: ExprHandle = simplified(&expr_divide(&first, &second))?;
        self.recept.push(&result);
        Ok(())
    }
    /// `ExprIntrp_ExpOperator`: `op1 ^ op2`, simplified one level.
    pub(super) fn exp_operator(&mut self) -> Result<(), Failure> {
        let (first, second) = self.pop_operands();
        let result: ExprHandle = simplified(&ExprExponentiate::new(&first, &second))?;
        self.recept.push(&result);
        Ok(())
    }
    /// `ExprIntrp_ProductOperator`: `op1 * op2`, simplified one level.
    pub(super) fn product_operator(&mut self) -> Result<(), Failure> {
        let (first, second) = self.pop_operands();
        let result: ExprHandle = simplified(&expr_multiply(&first, &second))?;
        self.recept.push(&result);
        Ok(())
    }
    /// `ExprIntrp_UnaryMinusOperator`: `-op`, simplified one level.
    pub(super) fn unary_minus_operator(&mut self) -> Result<(), Failure> {
        let operand: ExprHandle = self.recept.pop();
        let result: ExprHandle = simplified(&ExprUnaryMinus::new(&operand))?;
        self.recept.push(&result);
        Ok(())
    }
    /// `ExprIntrp_UnaryPlusOperator`: `+op` is `op`.
    pub(super) fn unary_plus_operator(&mut self) -> Result<(), Failure> {
        let operand: ExprHandle = self.recept.pop();
        self.recept.push(&operand);
        Ok(())
    }
    /// `ExprIntrp_VariableIdentifier`: the named expression of the last identifier; a
    /// new unknown, made known, the first time.
    pub(super) fn variable_identifier(&mut self) -> Result<(), Failure> {
        let name = self.result();
        let mut named: Handle<dyn ExprNamedExpression> = self.recept.get_named(&name);
        if named.is_null() {
            named = ExprNamedUnknown::new(&name).cast();
            self.recept.use_named(&named);
        }
        self.recept.push(&named.cast());
        Ok(())
    }
    /// `ExprIntrp_NumValue`: the last value read, through `RealValue` (which raises
    /// Standard_NumericError on a text such as `.`).
    pub(super) fn num_value(&mut self) -> Result<(), Failure> {
        let value: f64 = self.result().real_value()?;
        self.recept.push(&ExprNumericValue::expression(value));
        Ok(())
    }
    /// `ExprIntrp_AssignVariable`: remember the unknown of the assignment.
    pub(super) fn assign_variable(&mut self) -> Result<(), Failure> {
        self.assname = self.result();
        Ok(())
    }
    /// `ExprIntrp_EndOfAssign`: assign the expression to the unknown, created and made
    /// known when new; ExprIntrp_SyntaxError when the name is a constant.
    pub(super) fn end_of_assign(&mut self) -> Result<(), Failure> {
        let name = self.assname.clone();
        let mut unknown: Handle<ExprNamedUnknown> = self.named_unknown(&name)?;
        if unknown.is_null() {
            unknown = ExprNamedUnknown::new(&name);
            self.recept.use_named(&unknown.cast());
        }
        let expression: ExprHandle = self.recept.pop();
        deref(&unknown, "ExprIntrp_EndOfAssign")?.assign(&expression)
    }
    /// `ExprIntrp_Deassign`: free the unknown of the last identifier;
    /// ExprIntrp_SyntaxError when it is not a known unknown.
    pub(super) fn deassign(&mut self) -> Result<(), Failure> {
        let name = self.result();
        let unknown: Handle<ExprNamedUnknown> = self.named_unknown(&name)?;
        let unknown: &ExprNamedUnknown = unknown.get().ok_or_else(|| {
            syntax_error(&format!(
                "Deassign: {} is not known; expected a known unknown",
                String::from_utf8_lossy(name.as_bytes())
            ))
        })?;
        unknown.deassign();
        Ok(())
    }
    /// `ExprIntrp_ConstantIdentifier`: remember the name of the constant.
    pub(super) fn constant_identifier(&mut self) -> Result<(), Failure> {
        let name = self.result();
        self.recept.push_name(&name);
        Ok(())
    }
    /// `ExprIntrp_ConstantDefinition`: a new named constant, made known and pushed.
    pub(super) fn constant_definition(&mut self) -> Result<(), Failure> {
        let name = self.recept.pop_name();
        let value: f64 = self.result().real_value()?;
        let constant: Handle<ExprNamedConstant> = ExprNamedConstant::new(&name, value);
        self.recept.use_named(&constant.cast());
        self.recept.push(&constant.cast());
        Ok(())
    }
}
