// Rust translation of ModelingAlgorithms/TKExpress at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKExpress: symbolic mathematical expressions (`Expr`) and the interpreter that
//! builds them from text (`ExprIntrp`).
//!
//! TKExpress depends on TKernel only (its EXTERNLIB). Each of the 2 OCCT packages is
//! one module named after it in snake case (`Expr` is `expr`, `ExprIntrp` is
//! `expr_intrp`).
//!
//! The class hierarchy follows the rules of the Geom port (docs/geom-design.md):
//! abstract classes are traits reached through `Handle<dyn Trait>`, concrete classes
//! are structs that keep their mutable fields behind a `SharedState`, so that a change
//! through one handle is seen through every other handle, as in OCCT. Native
//! exceptions are `Failure` values with the native exception type; a call through a
//! null handle, which crashes natively, is the `Standard_NullObject` failure.
//!
//! Many OCCT members take a handle to the object itself (`occ::handle<T> me = this`):
//! every Expr object therefore keeps a non-owning pointer to its own handle, and its
//! constructors return the handle (`ExprSum::new(a, b)` is `new Expr_Sum(a, b)`).
pub mod expr;
pub mod expr_intrp;
