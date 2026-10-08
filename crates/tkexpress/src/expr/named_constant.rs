// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_NamedConstant.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Expr_NamedConstant`: a constant known by its name (`Pi`, ...).
use super::errors::out_of_range;
use super::general_expression::{ExprGeneralExpression, ExprValues, ExprVariables};
use super::handles::{ExprHandle, ExprSelfLink, adopt};
use super::named_expression::{
    ExprNamedExpression, named_is_identical, named_is_shareable, named_string,
};
use super::named_unknown::ExprNamedUnknown;
use super::numeric_value::ExprNumericValue;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A named real constant;
/// `ExprNamedConstant::new(&AsciiString::new("Pi"), 3.14).get_value() == 3.14`.
pub struct ExprNamedConstant {
    parent: StandardTransient,
    name: SharedState<AsciiString>,
    value: f64,
    link: ExprSelfLink,
}
impl_transient!(
    ExprNamedConstant,
    "Expr_NamedConstant",
    dyn ExprNamedExpression,
    parent,
    interfaces: [dyn ExprGeneralExpression, dyn ExprNamedExpression]
);
impl ExprNamedConstant {
    /// `new Expr_NamedConstant(name, value)`;
    /// `ExprNamedConstant::new(&AsciiString::new("e"), 2.718)`.
    pub fn new(name: &AsciiString, value: f64) -> Handle<Self> {
        let object: Self = Self {
            parent: StandardTransient::default(),
            name: SharedState::new(name.clone()),
            value,
            link: ExprSelfLink::default(),
        };
        adopt(object, |constant| &constant.link)
    }
    /// The value (`GetValue`); `constant.get_value()`.
    pub fn get_value(&self) -> f64 {
        self.value
    }
}
impl ExprNamedExpression for ExprNamedConstant {
    fn name_state(&self) -> &SharedState<AsciiString> {
        &self.name
    }
}
impl ExprGeneralExpression for ExprNamedConstant {
    fn nb_sub_expressions(&self) -> i32 {
        0
    }
    fn sub_expression(&self, index: i32) -> Result<ExprHandle, Failure> {
        Err(out_of_range(&format!(
            "Expr_NamedConstant::SubExpression: I = {index}; a constant has no sub-expression"
        )))
    }
    fn simplified(&self) -> Result<ExprHandle, Failure> {
        Ok(ExprNumericValue::expression(self.value))
    }
    fn shallow_simplified(&self) -> Result<ExprHandle, Failure> {
        Ok(ExprNumericValue::expression(self.value))
    }
    fn copy(&self) -> Result<ExprHandle, Failure> {
        Ok(Self::new(&self.get_name(), self.value).cast())
    }
    fn contains_unknowns(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn contains(&self, _exp: &ExprHandle) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_linear(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_shareable(&self) -> bool {
        named_is_shareable()
    }
    fn is_identical(&self, other: &ExprHandle) -> Result<bool, Failure> {
        named_is_identical(self, other)
    }
    fn derivative(&self, _x: &Handle<ExprNamedUnknown>) -> Result<ExprHandle, Failure> {
        Ok(ExprNumericValue::expression(0.0))
    }
    // Native quirk: the override accepts N <= 0 and still answers 0.
    fn n_derivative(&self, _x: &Handle<ExprNamedUnknown>, _n: i32) -> Result<ExprHandle, Failure> {
        Ok(ExprNumericValue::expression(0.0))
    }
    fn replace(&self, _var: &Handle<ExprNamedUnknown>, _with: &ExprHandle) -> Result<(), Failure> {
        Ok(())
    }
    fn evaluate(&self, _vars: &ExprVariables<'_>, _vals: &ExprValues<'_>) -> Result<f64, Failure> {
        Ok(self.value)
    }
    fn string(&self) -> Result<AsciiString, Failure> {
        named_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_constant_simplifies_to_its_value_and_keeps_its_name() {
        let pi: ExprHandle = ExprNamedConstant::new(&AsciiString::new("Pi"), 3.5).cast();
        assert_eq!(pi.string().unwrap(), "Pi");
        assert_eq!(pi.simplified().unwrap().string().unwrap(), "3.5");
        assert!(pi.is_shareable());
        let other: ExprHandle = ExprNamedConstant::new(&AsciiString::new("Pi"), 1.0).cast();
        assert!(pi.is_identical(&other).unwrap());
        assert_eq!(
            pi.n_derivative(&Handle::default(), 0)
                .unwrap()
                .string()
                .unwrap(),
            "0"
        );
    }
}
