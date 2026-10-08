//! 1:1 translation of src/ModelingAlgorithms/TKExpress/GTests/Expr_GeneralExpression_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkexpress::expr::{ExprHandle, ExprNamedUnknown};
use rustycad_tkexpress::expr_intrp::ExprIntrpGenExp;

#[test]
fn Expr_GeneralExpression_Test__OCC902_ExpressionDerivative() {
    // Bug OCC902: Bad derivative computation
    // This test verifies that the derivative of Exp(5*x) is correctly computed
    // The expected result is either "Exp(5*x)*5" or "5*Exp(5*x)"

    let mut an_exp_str: AsciiString = AsciiString::new("5");
    an_exp_str.assign_cat("*x");
    an_exp_str.prepend(&AsciiString::new("Exp("));
    an_exp_str.assign_cat(")");

    let an_expr_intrp: Handle<ExprIntrpGenExp> = ExprIntrpGenExp::create();

    // Create the expression
    an_expr_intrp.process(&an_exp_str);

    assert!(
        an_expr_intrp.is_done(),
        "Expression interpretation should succeed"
    );

    let an_expr: ExprHandle = an_expr_intrp.expression().unwrap();
    assert!(!an_expr.is_null(), "Expression should not be null");

    let a_var: Handle<ExprNamedUnknown> = ExprNamedUnknown::new(&AsciiString::new("x"));
    let a_new_expr: ExprHandle = an_expr.derivative(&a_var).unwrap();

    assert!(!a_new_expr.is_null(), "Derivative should not be null");

    let a_derivative_str: AsciiString = a_new_expr.string().unwrap();

    // The derivative of Exp(5*x) should be either "Exp(5*x)*5" or "5*Exp(5*x)"
    let is_correct: bool = a_derivative_str == "Exp(5*x)*5" || a_derivative_str == "5*Exp(5*x)";

    assert!(
        is_correct,
        "Derivative result was: {}, expected either 'Exp(5*x)*5' or '5*Exp(5*x)'",
        String::from_utf8_lossy(a_derivative_str.as_bytes())
    );
}

// Test OCC31697: Expr_GeneralExpression::Derivative for Exp(2*Sin(x^2))
// Expected derivative: Exp(2*Sin(x^2))*Cos(x^2)*x*4
#[test]
fn Expr_GeneralExpression_Test__OCC31697_DerivativeOfComplexExpression() {
    let an_expr_intrp: Handle<ExprIntrpGenExp> = ExprIntrpGenExp::create();
    an_expr_intrp.process(&AsciiString::new("Exp(2*Sin(x^2))"));

    assert!(an_expr_intrp.is_done(), "Expression parsing should succeed");

    let an_expr: ExprHandle = an_expr_intrp.expression().unwrap();
    assert!(!an_expr.is_null(), "Expression should not be null");

    let a_var: Handle<ExprNamedUnknown> = ExprNamedUnknown::new(&AsciiString::new("x"));
    assert!(
        an_expr.contains(&a_var.cast()).unwrap(),
        "Expression should contain variable x"
    );

    let a_der: ExprHandle = an_expr.derivative(&a_var).unwrap();
    assert!(!a_der.is_null(), "Derivative should not be null");

    let a_der_str: AsciiString = a_der.string().unwrap();
    assert_eq!(
        a_der_str,
        AsciiString::new("Exp(2*Sin(x^2))*Cos(x^2)*x*4"),
        "Derivative result was: {}",
        String::from_utf8_lossy(a_der_str.as_bytes())
    );
}

// Test OCC22611: ExprIntrp_GenExp must not leak and must parse numeric literal correctly.
// Migrated from QABugs_19.cxx OCC22611
#[test]
fn Expr_GeneralExpression_Test__OCC22611_ParseNumericLiteral() {
    let a_gen: Handle<ExprIntrpGenExp> = ExprIntrpGenExp::create();
    for i in 0..10 {
        a_gen.process(&AsciiString::new("0.1214343"));
        assert!(a_gen.is_done(), "Parsing should succeed on iteration {i}");
        let a_expr: ExprHandle = a_gen.expression().unwrap();
        assert!(
            !a_expr.is_null(),
            "Expression should not be null on iteration {i}"
        );
    }
}

// fclasses/bugs/bug23403: a lexical-error input must not escape the expression
// parser as an exception.
#[test]
fn Expr_GeneralExpression_Test__FClassesBug_23403_TabInputDoesNotThrow() {
    let a_generator: Handle<ExprIntrpGenExp> = ExprIntrpGenExp::create();
    // The scanner echoes the tab to its output, the standard output by default as flex's
    // `yyout`. GoogleTest reads its results from XML, but the Rust harness parses its own
    // standard output, which the echo would corrupt; the echo goes to a sink instead.
    a_generator.set_scanner_output(Box::new(std::io::sink()));
    // EXPECT_NO_THROW: Process returns nothing and cannot fail in Rust; it must not panic.
    a_generator.process(&AsciiString::new("\t"));
}

// fclasses/bugs/bug24897: a long invalid expression must also be rejected
// without an exception or uncontrolled exit.
#[test]
fn Expr_GeneralExpression_Test__FClassesBug_24897_InvalidInputDoesNotThrow() {
    let a_generator: Handle<ExprIntrpGenExp> = ExprIntrpGenExp::create();
    // EXPECT_NO_THROW: Process returns nothing and cannot fail in Rust; it must not panic.
    a_generator.process(&AsciiString::new(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ));
}
