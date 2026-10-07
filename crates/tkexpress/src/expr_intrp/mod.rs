// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `ExprIntrp` package: the interpreter that builds expressions from text.
//!
//! OCCT generates its scanner with flex (ExprIntrp.lex, lex.ExprIntrp.c) and its parser
//! with Bison (ExprIntrp.yacc, ExprIntrp.tab.c); both generated files are part of the
//! toolkit's sources. The port writes the scanner by hand from the flex rules
//! ([`scanner`]) and drives the Bison tables unchanged ([`tables`], [`parser`]); the
//! semantic actions of ExprIntrp_yaccintrf.cxx are members of [`context`]'s
//! parse context.
mod actions_derivatives;
pub mod actions_functions;
mod actions_relations;
mod actions_values;
pub mod analysis;
pub mod context;
pub mod errors;
pub mod gen_exp;
pub mod generator;
pub mod parse;
pub mod parser;
pub mod scanner;
pub mod tables;
pub mod token;

pub use analysis::ExprIntrpAnalysis;
pub use gen_exp::ExprIntrpGenExp;
pub use generator::ExprIntrpGenerator;
pub use parser::ExprIntrpParser;
pub use scanner::ExprIntrpScanner;
pub use token::ExprIntrpTokenKind;
