// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_Generator.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExprIntrp_Generator`: the named expressions and functions an interpreter knows.
use crate::expr::{ExprNamedExpression, ExprNamedFunction};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// The definitions an interpreter has seen, in order of definition.
#[derive(Default)]
struct GeneratorDefinitions {
    functions: Vec<Handle<ExprNamedFunction>>,
    named: Vec<Handle<dyn ExprNamedExpression>>,
}

/// The base of the interpreters: the named expressions (unknowns and constants) and
/// the functions met so far, which later texts can use; `generator.get_named(&name)`.
#[derive(Default)]
pub struct ExprIntrpGenerator {
    parent: StandardTransient,
    definitions: SharedState<GeneratorDefinitions>,
}
impl_transient!(
    ExprIntrpGenerator,
    "ExprIntrp_Generator",
    StandardTransient,
    parent
);
impl ExprIntrpGenerator {
    /// An empty generator (`ExprIntrp_Generator()`, protected natively: the
    /// interpreters embed it); `ExprIntrpGenerator::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Make `func` known (`Use(occ::handle<Expr_NamedFunction>)`);
    /// `generator.use_function(&f)`.
    pub fn use_function(&self, func: &Handle<ExprNamedFunction>) {
        self.definitions.write().functions.push(func.clone());
    }
    /// Make `named` known (`Use(occ::handle<Expr_NamedExpression>)`);
    /// `generator.use_named(&x)`.
    pub fn use_named(&self, named: &Handle<dyn ExprNamedExpression>) {
        self.definitions.write().named.push(named.clone());
    }
    /// Every named expression, in order (`GetNamed()`); `generator.get_named_list()`.
    pub fn get_named_list(&self) -> Vec<Handle<dyn ExprNamedExpression>> {
        self.definitions.read().named.clone()
    }
    /// Every function, in order (`GetFunctions()`); `generator.get_functions()`.
    pub fn get_functions(&self) -> Vec<Handle<ExprNamedFunction>> {
        self.definitions.read().functions.clone()
    }
    /// The first named expression called `name`, else null (`GetNamed(name)`);
    /// `generator.get_named(&AsciiString::new("x"))`.
    pub fn get_named(&self, name: &AsciiString) -> Handle<dyn ExprNamedExpression> {
        first_named(&self.get_named_list(), name)
    }
    /// The first function called `name`, else null (`GetFunction(name)`);
    /// `generator.get_function(&AsciiString::new("f"))`.
    pub fn get_function(&self, name: &AsciiString) -> Handle<ExprNamedFunction> {
        first_function(&self.get_functions(), name)
    }
}

/// The first of `named` called `name`, else null: the search of `GetNamed(name)`;
/// a null entry is skipped, where native code would crash on it.
pub fn first_named(
    named: &[Handle<dyn ExprNamedExpression>],
    name: &AsciiString,
) -> Handle<dyn ExprNamedExpression> {
    named
        .iter()
        .find(|candidate| {
            candidate
                .get()
                .is_some_and(|candidate| *name == candidate.get_name())
        })
        .cloned()
        .unwrap_or_default()
}

/// The first of `functions` called `name`, else null: the search of
/// `GetFunction(name)`; a null entry is skipped, where native code would crash on it.
pub fn first_function(
    functions: &[Handle<ExprNamedFunction>],
    name: &AsciiString,
) -> Handle<ExprNamedFunction> {
    functions
        .iter()
        .find(|candidate| {
            candidate
                .get()
                .is_some_and(|candidate| *name == candidate.get_name())
        })
        .cloned()
        .unwrap_or_default()
}
