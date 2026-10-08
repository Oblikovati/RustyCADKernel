//! Behaviour of the TKExpress port on cases whose results were read from the pinned
//! OCCT (tools/oracle/tkexpress_probe.cxx): simplification, derivation, the interpreter,
//! function derivatives and the native quirks the port keeps.
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkexpress::expr::handles::one_based;
use rustycad_tkexpress::expr::operators::{add_real, multiply_real, real_divide, real_subtract};
use rustycad_tkexpress::expr::{
    ExprDifference, ExprEqual, ExprFunctionHandle, ExprGeneralFunction, ExprGeneralRelation,
    ExprHandle, ExprNamedFunction, ExprNamedUnknown, ExprNumericValue, ExprPolyExpression,
    ExprProduct, ExprRelationHandle, ExprSine, ExprSquare, ExprSum, ExprSystemRelation,
    ExprUnaryExpression,
};
use rustycad_tkexpress::expr_intrp::ExprIntrpGenExp;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// The interpreter with its echo sent to a sink.
fn interpreter() -> Handle<ExprIntrpGenExp> {
    let interpreter = ExprIntrpGenExp::create();
    interpreter.set_scanner_output(Box::new(std::io::sink()));
    interpreter
}

/// The text of the expression of `text`.
fn parsed(text: &str) -> ExprHandle {
    let interpreter = interpreter();
    interpreter.process(&AsciiString::new(text));
    assert!(interpreter.is_done(), "{text} should parse");
    interpreter.expression().unwrap()
}

fn text_of(expression: &ExprHandle) -> String {
    String::from_utf8(expression.string().unwrap().as_bytes().to_vec()).unwrap()
}

fn unknown(name: &str) -> Handle<ExprNamedUnknown> {
    ExprNamedUnknown::new(&AsciiString::new(name))
}

#[test]
fn the_interpreter_simplifies_one_level_as_it_parses() {
    let cases = [
        ("2*x*3*4", "x*24"),
        ("-1*x", "-x"),
        ("x-(-y)", "x+y"),
        ("-x-y", "-(x+y)"),
        ("2^0.5", "1.41421"),
        ("Abs(-x)", "Abs(x)"),
        ("Sqrt(x)^2", "Sqrt(x)^2"),
        ("Sign(-0)", "-1"),
        ("1e400", "inf"),
        ("Sum[x^2, x, 1, 1, 4]", "(1^2)+(2^2)+(3^2)+(4^2)"),
        ("Prod[x+1, x, 1, 1, 3]", "(1+1)*(2+1)*(3+1)"),
        ("Deriv[x^3, x]", "(x^2)*3"),
        ("Sin''(x^2)", "(Cos(x^2)*2)+(-Sin(x^2)*x*x*4)"),
    ];
    for (text, expected) in cases {
        assert_eq!(text_of(&parsed(text)), expected, "{text}");
    }
}

#[test]
fn derivatives_and_simplification_follow_occt() {
    let x = unknown("x");
    let product = parsed("Sin(x)*Cos(x)");
    assert_eq!(
        text_of(&product.derivative(&x).unwrap()),
        "(Sin(x)*-Sin(x))+(Cos(x)*Cos(x))"
    );
    let arc = parsed("ATan(x^2)");
    assert_eq!(text_of(&arc.derivative(&x).unwrap()), "(x*2)/(1+(x^4))");
    assert_eq!(
        text_of(&parsed("(x^2)^2").derivative(&x).unwrap()),
        "(x^2)*x*4"
    );
    // Native quirk: the second derivative of -x is the value -0.
    assert_eq!(text_of(&parsed("-1*x").n_derivative(&x, 2).unwrap()), "-0");
    let series = parsed("Sum[x^2, x, 1, 1, 4]");
    assert_eq!(text_of(&series.simplified().unwrap()), "30");
    assert_eq!(series.evaluate_numeric().unwrap(), 30.0);
}

#[test]
fn unmatched_characters_are_echoed_and_the_text_is_refused() {
    let echo: Arc<Mutex<Vec<u8>>> = Arc::default();
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(data);
            Ok(data.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let interpreter = ExprIntrpGenExp::create();
    interpreter.set_scanner_output(Box::new(Capture(echo.clone())));
    interpreter.process(&AsciiString::new("x$y"));
    assert!(!interpreter.is_done());
    assert_eq!(*echo.lock().unwrap(), b"$");
    assert_eq!(
        interpreter.expression().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn the_bison_stack_holds_9999_states() {
    let interpreter = interpreter();
    interpreter.process(&AsciiString::new(&format!("{}x", "-".repeat(9997))));
    assert!(interpreter.is_done());
    interpreter.process(&AsciiString::new(&format!("{}x", "-".repeat(9998))));
    assert!(!interpreter.is_done());
}

#[test]
fn definitions_assignments_and_relations_leave_no_expression() {
    let interpreter = interpreter();
    for text in ["x <- 2", "f[x] <- x^2", "x=1;y=2"] {
        interpreter.process(&AsciiString::new(text));
        assert!(interpreter.is_done(), "{text}");
        assert!(interpreter.expression().unwrap().is_null(), "{text}");
    }
    assert_eq!(interpreter.get_functions().len(), 1);
    interpreter.process(&AsciiString::new("f(3)+x"));
    let expression = interpreter.expression().unwrap();
    assert_eq!(text_of(&expression), "f(3)+x");
    // Native quirk: the variable of f is a copy of the global x, assigned to 2 when f
    // was defined, so f(3) is 2^2 and the sum is 6 (OCCT answers the same).
    assert_eq!(text_of(&expression.simplified().unwrap()), "6");
}

#[test]
fn the_operators_with_reals_keep_the_native_order() {
    let x: ExprHandle = unknown("x").cast();
    assert_eq!(text_of(&add_real(&x, 2.0).cast()), "2+x");
    assert_eq!(text_of(&multiply_real(&x, 2.0).cast()), "2*x");
    assert_eq!(text_of(&real_subtract(2.0, &x).cast()), "2-x");
    assert_eq!(text_of(&real_divide(2.0, &x).cast()), "2/x");
}

#[test]
fn set_operand_refuses_cycles() {
    let x: ExprHandle = unknown("x").cast();
    let sine = ExprSine::new(&x);
    let opposite = ExprDifference::new(&ExprNumericValue::expression(0.0), &sine.cast());
    let failure = sine.set_operand(&opposite.cast()).unwrap_err();
    assert_eq!(failure.exception_type(), "Expr_InvalidOperand");
    let sum = ExprSum::new_pair(&x, &x);
    assert_eq!(
        sum.remove_operand(1).unwrap_err().exception_type(),
        "Standard_DimensionMismatch"
    );
    assert_eq!(
        sum.set_operand(&x, 3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn identical_products_match_each_operand_once() {
    let (x, y): (ExprHandle, ExprHandle) = (unknown("x").cast(), unknown("y").cast());
    let xxy: ExprHandle = ExprProduct::new(&[x.clone(), x.clone(), y.clone()]).cast();
    let xyx: ExprHandle = ExprProduct::new(&[x.clone(), y.clone(), x.clone()]).cast();
    let xyy: ExprHandle = ExprProduct::new(&[x.clone(), y.clone(), y.clone()]).cast();
    assert!(xxy.is_identical(&xyx).unwrap());
    assert!(!xxy.is_identical(&xyy).unwrap());
    assert!(!xyy.is_identical(&xxy).unwrap());
}

#[test]
fn function_derivatives_are_named_as_natively() {
    let (x, y) = (unknown("x"), unknown("y"));
    let body: ExprHandle = ExprSum::new_pair(
        &ExprProduct::new_pair(&x.cast(), &y.cast()).cast(),
        &ExprSquare::new(&y.cast()).cast(),
    )
    .cast();
    let g = ExprNamedFunction::new(
        &AsciiString::new("g"),
        &body,
        &one_based(vec![x.clone(), y.clone()]),
    );
    let second: ExprFunctionHandle = g.derivative_of_degree(&y, 2).unwrap();
    assert_eq!(second.get_string_name().unwrap(), "@2g/@2X2");
    let mixed = second.derivative(&x).unwrap();
    assert_eq!(mixed.get_string_name().unwrap(), "@@2g/@2X2/@X1");
    let vars = one_based(vec![x.clone(), y.clone()]);
    assert_eq!(
        mixed.evaluate(&vars, &one_based(vec![0.75, -1.5])).unwrap(),
        0.0
    );
    let f = ExprNamedFunction::new(
        &AsciiString::new("f"),
        &x.cast(),
        &one_based(vec![x.clone()]),
    );
    let fx: ExprFunctionHandle = f.cast();
    assert_eq!(
        fx.derivative_of_degree(&x, 2)
            .unwrap()
            .get_string_name()
            .unwrap(),
        "f''"
    );
    assert_eq!(
        fx.derivative_of_degree(&x, 0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn system_remove_never_removes() {
    let x: ExprHandle = unknown("x").cast();
    let first: ExprRelationHandle = ExprEqual::new(&x, &ExprNumericValue::expression(1.0)).cast();
    let system = ExprSystemRelation::new(&first);
    assert_eq!(
        system.remove(&first).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    let other: ExprRelationHandle = ExprEqual::new(&x, &x).cast();
    assert_eq!(
        system.remove(&other).unwrap_err().exception_type(),
        "Standard_DimensionMismatch"
    );
    system.add(&other);
    assert_eq!(
        system.remove(&first).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    let third: ExprRelationHandle = ExprEqual::new(&x, &x).cast();
    assert_eq!(
        system.remove(&third).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(system.nb_of_sub_relations(), 2);
    assert_eq!(text_of_relation(&system.cast()), "x = 1\nx = x");
}

fn text_of_relation(relation: &ExprRelationHandle) -> String {
    String::from_utf8(relation.string().unwrap().as_bytes().to_vec()).unwrap()
}
