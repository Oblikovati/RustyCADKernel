// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_Analysis.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExprIntrp_Analysis`: the stacks of the semantic actions during one parse.
//!
//! Natively one global object (`ExprIntrp_Recept`) serves every parse; the port gives
//! each parse its own analysis, which `set_master` resets as the global one is.
use super::generator::{ExprIntrpGenerator, first_function, first_named};
use crate::expr::{
    ExprFunctionHandle, ExprHandle, ExprNamedExpression, ExprNamedFunction, ExprRelationHandle,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;

/// The stacks of expressions, relations, functions, names and integers that the
/// semantic actions push and pop, and the definitions known to the parse;
/// `ExprIntrpAnalysis::new()`.
#[derive(Default)]
pub struct ExprIntrpAnalysis {
    expressions: Vec<ExprHandle>,
    relations: Vec<ExprRelationHandle>,
    functions_stack: Vec<ExprFunctionHandle>,
    names: Vec<AsciiString>,
    values: Vec<i32>,
    functions: Vec<Handle<ExprNamedFunction>>,
    named: Vec<Handle<dyn ExprNamedExpression>>,
    master: Handle<ExprIntrpGenerator>,
}
impl ExprIntrpAnalysis {
    /// Empty stacks and no master (`ExprIntrp_Analysis()`); `ExprIntrpAnalysis::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Start a parse for `agen` (`SetMaster`): empty the stacks and copy the
    /// definitions the generator knows; `analysis.set_master(&generator)`.
    pub fn set_master(&mut self, agen: &Handle<ExprIntrpGenerator>) {
        self.reset_all();
        self.master = agen.clone();
        if let Some(master) = agen.get() {
            self.functions = master.get_functions();
            self.named = master.get_named_list();
        }
    }
    /// Push an expression (`Push`); `analysis.push(&x)`.
    pub fn push(&mut self, exp: &ExprHandle) {
        self.expressions.push(exp.clone());
    }
    /// Push a relation (`PushRelation`); `analysis.push_relation(&equal)`.
    pub fn push_relation(&mut self, rel: &ExprRelationHandle) {
        self.relations.push(rel.clone());
    }
    /// Push a name (`PushName`); `analysis.push_name(&name)`.
    pub fn push_name(&mut self, name: &AsciiString) {
        self.names.push(name.clone());
    }
    /// Push an integer (`PushValue`); `analysis.push_value(2)`.
    pub fn push_value(&mut self, degree: i32) {
        self.values.push(degree);
    }
    /// Push a function (`PushFunction`); `analysis.push_function(&f)`.
    pub fn push_function(&mut self, func: &ExprFunctionHandle) {
        self.functions_stack.push(func.clone());
    }
    /// Pop the last expression, null when empty (`Pop`); `analysis.pop()`.
    pub fn pop(&mut self) -> ExprHandle {
        self.expressions.pop().unwrap_or_default()
    }
    /// Pop the last relation, null when empty (`PopRelation`); `analysis.pop_relation()`.
    pub fn pop_relation(&mut self) -> ExprRelationHandle {
        self.relations.pop().unwrap_or_default()
    }
    /// Pop the last name, empty when none (`PopName`); `analysis.pop_name()`.
    pub fn pop_name(&mut self) -> AsciiString {
        self.names.pop().unwrap_or_else(|| AsciiString::new(""))
    }
    /// Pop the last integer, 0 when none (`PopValue`); `analysis.pop_value()`.
    pub fn pop_value(&mut self) -> i32 {
        self.values.pop().unwrap_or(0)
    }
    /// Pop the last function, null when empty (`PopFunction`); `analysis.pop_function()`.
    pub fn pop_function(&mut self) -> ExprFunctionHandle {
        self.functions_stack.pop().unwrap_or_default()
    }
    /// True when no expression is stacked (`IsExpStackEmpty`); `analysis.is_exp_stack_empty()`.
    pub fn is_exp_stack_empty(&self) -> bool {
        self.expressions.is_empty()
    }
    /// True when no relation is stacked (`IsRelStackEmpty`); `analysis.is_rel_stack_empty()`.
    pub fn is_rel_stack_empty(&self) -> bool {
        self.relations.is_empty()
    }
    /// Empty every stack and forget the definitions (`ResetAll`); `analysis.reset_all()`.
    pub fn reset_all(&mut self) {
        self.expressions.clear();
        self.relations.clear();
        self.functions_stack.clear();
        self.names.clear();
        self.values.clear();
        self.functions.clear();
        self.named.clear();
    }
    /// Make `func` known to the parse and to the master (`Use(Expr_NamedFunction)`);
    /// `analysis.use_function(&f)`.
    pub fn use_function(&mut self, func: &Handle<ExprNamedFunction>) {
        self.functions.push(func.clone());
        if let Some(master) = self.master.get() {
            master.use_function(func);
        }
    }
    /// Make `named` known to the parse and to the master (`Use(Expr_NamedExpression)`);
    /// `analysis.use_named(&x)`.
    pub fn use_named(&mut self, named: &Handle<dyn ExprNamedExpression>) {
        self.named.push(named.clone());
        if let Some(master) = self.master.get() {
            master.use_named(named);
        }
    }
    /// The first named expression called `name`, else null (`GetNamed`);
    /// `analysis.get_named(&name)`.
    pub fn get_named(&self, name: &AsciiString) -> Handle<dyn ExprNamedExpression> {
        first_named(&self.named, name)
    }
    /// The first function called `name`, else null (`GetFunction`);
    /// `analysis.get_function(&name)`.
    pub fn get_function(&self, name: &AsciiString) -> Handle<ExprNamedFunction> {
        first_function(&self.functions, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::ExprNumericValue;

    #[test]
    fn stacks_pop_the_last_push_and_answer_defaults_when_empty() {
        let mut analysis: ExprIntrpAnalysis = ExprIntrpAnalysis::new();
        assert!(analysis.pop().is_null());
        assert_eq!(analysis.pop_value(), 0);
        assert_eq!(analysis.pop_name(), "");
        analysis.push_value(1);
        analysis.push_value(2);
        assert_eq!(analysis.pop_value(), 2);
        let value: ExprHandle = ExprNumericValue::expression(1.0);
        analysis.push(&value);
        assert!(!analysis.is_exp_stack_empty());
        assert!(analysis.pop() == value);
        analysis.reset_all();
        assert_eq!(analysis.pop_value(), 0);
    }
}
