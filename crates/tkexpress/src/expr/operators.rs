// Rust translation of ModelingAlgorithms/TKExpress/Expr/Expr_Operators.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The arithmetic operators on expression handles.
//!
//! Rust cannot implement `+` for the foreign `Handle` type, so each C++ operator is a
//! named function. The mixed forms keep the native operand order: `x + 2.0` builds
//! `2 + x` (`operator+(x, y)` returns `y + x`), and so does `x * 2.0`.
use super::difference::ExprDifference;
use super::division::ExprDivision;
use super::handles::ExprHandle;
use super::numeric_value::ExprNumericValue;
use super::product::ExprProduct;
use super::sum::ExprSum;
use super::unary_minus::ExprUnaryMinus;
use rustycad_tkernel::standard::Handle;

/// `operator+(x, y)`: `new Expr_Sum(x, y)`; `expr_add(&x, &y)`.
pub fn expr_add(x: &ExprHandle, y: &ExprHandle) -> Handle<ExprSum> {
    ExprSum::new_pair(x, y)
}
/// `operator+(double x, y)`: `x` as a value, then `y`; `real_add(1.0, &y)`.
pub fn real_add(x: f64, y: &ExprHandle) -> Handle<ExprSum> {
    ExprSum::new_pair(&ExprNumericValue::expression(x), y)
}
/// `operator+(x, double y)`: `y + x`, the value first; `add_real(&x, 1.0)`.
pub fn add_real(x: &ExprHandle, y: f64) -> Handle<ExprSum> {
    real_add(y, x)
}
/// `operator-(x, y)`: `new Expr_Difference(x, y)`; `expr_subtract(&x, &y)`.
pub fn expr_subtract(x: &ExprHandle, y: &ExprHandle) -> Handle<ExprDifference> {
    ExprDifference::new(x, y)
}
/// `operator-(double x, y)`; `real_subtract(1.0, &y)`.
pub fn real_subtract(x: f64, y: &ExprHandle) -> Handle<ExprDifference> {
    ExprDifference::new(&ExprNumericValue::expression(x), y)
}
/// `operator-(x, double y)`; `subtract_real(&x, 1.0)`.
pub fn subtract_real(x: &ExprHandle, y: f64) -> Handle<ExprDifference> {
    ExprDifference::new(x, &ExprNumericValue::expression(y))
}
/// `operator-(x)`: `new Expr_UnaryMinus(x)`; `expr_negate(&x)`.
pub fn expr_negate(x: &ExprHandle) -> Handle<ExprUnaryMinus> {
    ExprUnaryMinus::new(x)
}
/// `operator*(x, y)`: `new Expr_Product(x, y)`; `expr_multiply(&x, &y)`.
pub fn expr_multiply(x: &ExprHandle, y: &ExprHandle) -> Handle<ExprProduct> {
    ExprProduct::new_pair(x, y)
}
/// `operator*(double x, y)`: `x` as a value, then `y`; `real_multiply(2.0, &y)`.
pub fn real_multiply(x: f64, y: &ExprHandle) -> Handle<ExprProduct> {
    ExprProduct::new_pair(&ExprNumericValue::expression(x), y)
}
/// `operator*(x, double y)`: `y * x`, the value first; `multiply_real(&x, 2.0)`.
pub fn multiply_real(x: &ExprHandle, y: f64) -> Handle<ExprProduct> {
    real_multiply(y, x)
}
/// `operator/(x, y)`: `new Expr_Division(x, y)`; `expr_divide(&x, &y)`.
pub fn expr_divide(x: &ExprHandle, y: &ExprHandle) -> Handle<ExprDivision> {
    ExprDivision::new(x, y)
}
/// `operator/(double x, y)`; `real_divide(1.0, &y)`.
pub fn real_divide(x: f64, y: &ExprHandle) -> Handle<ExprDivision> {
    ExprDivision::new(&ExprNumericValue::expression(x), y)
}
/// `operator/(x, double y)`; `divide_real(&x, 2.0)`.
pub fn divide_real(x: &ExprHandle, y: f64) -> Handle<ExprDivision> {
    ExprDivision::new(x, &ExprNumericValue::expression(y))
}
