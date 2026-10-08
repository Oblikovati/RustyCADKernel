// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_NumericValue.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_NumericValue`: a real constant.
use super::errors::out_of_range;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, ExprSelfLink, adopt, deref};
use super::named_unknown::ExprNamedUnknown;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A real value; `ExprNumericValue::new(2.5).get_value() == 2.5`.
pub struct ExprNumericValue {
    parent: StandardTransient,
    value: SharedState<f64>,
    link: ExprSelfLink,
}
impl_transient!(
    ExprNumericValue,
    "Expr_NumericValue",
    dyn ExprGeneralExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression]
);
impl ExprNumericValue {
    /// `new Expr_NumericValue(val)`; `let two: Handle<ExprNumericValue> = ExprNumericValue::new(2.0);`.
    pub fn new(val: f64) -> Handle<Self> {
        let object: Self = Self {
            parent: StandardTransient::default(),
            value: SharedState::new(val),
            link: ExprSelfLink::default(),
        };
        adopt(object, |value| &value.link)
    }
    /// The same object as a general expression; `ExprNumericValue::expression(0.0)`.
    pub fn expression(val: f64) -> ExprHandle {
        Self::new(val).cast()
    }
    /// The value (`GetValue`); `value.get_value()`.
    pub fn get_value(&self) -> f64 {
        *self.value.read()
    }
    /// Replace the value (`SetValue`); `value.set_value(3.0)`.
    pub fn set_value(&self, val: f64) {
        *self.value.write() = val;
    }
}
impl ExprGeneralExpression for ExprNumericValue {
    fn nb_sub_expressions(&self) -> i32 {
        0
    }
    fn sub_expression(&self, index: i32) -> Result<ExprHandle, Failure> {
        Err(out_of_range(&format!(
            "Expr_NumericValue::SubExpression: I = {index}; a value has no sub-expression"
        )))
    }
    fn simplified(&self) -> Result<ExprHandle, Failure> {
        self.copy()
    }
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        self.link.me()
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        Ok(Self::expression(self.get_value()))
    }
    fn contains_unknowns(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn contains(&self, _exp: &ExprHandle) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        deref(other, "Expr_NumericValue::IsIdentical")?;
        let other: Handle<ExprNumericValue> = other.cast();
        Ok(other
            .get()
            .is_some_and(|other| self.get_value() == other.get_value()))
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn derivative(&self, _x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        Ok(Self::expression(0.0))
    }
    // Native quirk: the override accepts N <= 0 and still answers 0.
    fn n_derivative(&self, _x: &Handle<ExprNamedUnknown>, _n: i32) -> Result<ExprHandle, Failure> {
        Ok(Self::expression(0.0))
    }
    fn replace(&self, _var: &Handle<ExprNamedUnknown>, _with: &ExprHandle) -> Result<(), Failure> {
        Ok(())
    }
    fn evaluate(&self, _vars: &ExprVariables<'_>, _vals: &ExprValues<'_>) -> Result<f64, Failure> {
        Ok(self.get_value())
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        // `Sprintf(val, "%g", myValue)`, which TCollection_AsciiString(double) also writes.
        Ok(AsciiString::from_real(self.get_value()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_members_follow_occt() {
        let value: Handle<ExprNumericValue> = ExprNumericValue::new(2.5);
        let general: ExprHandle = value.cast();
        assert_eq!(general.string().unwrap(), "2.5");
        value.set_value(1e-7);
        assert_eq!(general.string().unwrap(), "1e-07");
        assert!(general.shallow_simplified().unwrap() == general);
        assert!(general.simplified().unwrap() != general);
        assert!(
            general
                .is_identical(&ExprNumericValue::expression(1e-7))
                .unwrap()
        );
        assert!(
            !general
                .is_identical(&ExprNumericValue::expression(1.0))
                .unwrap()
        );
        assert_eq!(
            general.sub_expression(1).err().unwrap().exception_type(),
            "Standard_OutOfRange"
        );
        assert_eq!(general.evaluate_numeric().unwrap(), 1e-7);
    }
}
