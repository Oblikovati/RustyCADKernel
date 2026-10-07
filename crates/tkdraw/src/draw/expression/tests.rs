use super::*;
use std::cell::{Cell, RefCell};

/// A session stand-in: a few numeric variables, functions that double their first
/// argument (or fail for `bad`), and recorded output.
#[derive(Default)]
struct FakeContext {
    failed: Cell<bool>,
    calls: RefCell<Vec<String>>,
    lines: RefCell<Vec<String>>,
}

impl ExpressionContext for FakeContext {
    fn number_variable(&self, name: &str) -> Option<f64> {
        match name {
            "x" => Some(4.0),
            "pi" => Some(std::f64::consts::PI),
            _ => None,
        }
    }

    fn call_function(&self, script: &str) -> f64 {
        self.calls.borrow_mut().push(script.to_string());
        let mut words = script.split_whitespace();
        if words.next() == Some("bad") {
            self.print_line(&format!("Call of function {script} failed"));
            return 0.0;
        }
        words
            .next()
            .and_then(|w| w.parse::<f64>().ok())
            .map_or(0.0, |v| 2.0 * v)
    }

    fn print_line(&self, text: &str) {
        self.lines.borrow_mut().push(text.to_string());
    }

    fn set_parse_failed(&self, failed: bool) {
        self.failed.set(failed);
    }
}

fn eval(text: &str) -> (f64, bool, FakeContext) {
    let context = FakeContext::default();
    let value = evaluate(text, &context);
    let failed = context.failed.get();
    (value, failed, context)
}

#[test]
fn arithmetic_is_left_to_right_with_two_levels() {
    assert_eq!(eval("1+2*3").0, 7.0);
    assert_eq!(eval("8/2/2").0, 2.0);
    assert_eq!(eval("2-3-4").0, -5.0);
    assert_eq!(eval(" -(1 + 2) * 2 ").0, -6.0);
    assert_eq!(eval("--3").0, 3.0);
}

#[test]
fn literals_follow_strtod_on_the_token() {
    assert_eq!(eval("1.5e2").0, 150.0);
    assert_eq!(eval("2e-1").0, 0.2);
    // The token "12abc" is consumed whole; strtod reads 12 and nothing fails.
    let (value, failed, _) = eval("12abc");
    assert_eq!((value, failed), (12.0, false));
}

#[test]
fn variables_and_unknown_names() {
    assert_eq!(eval("x*2").0, 8.0);
    let (value, failed, _) = eval("y+1");
    assert_eq!((value, failed), (1.0, true));
}

#[test]
fn function_calls_become_tcl_commands() {
    let (value, failed, context) = eval("dbl(3, 4)+1");
    assert_eq!((value, failed), (7.0, false));
    assert_eq!(context.calls.borrow().as_slice(), ["dbl 3  4"]);
    let (value, _, context) = eval("bad(1)");
    assert_eq!(value, 0.0);
    assert_eq!(
        context.lines.borrow().as_slice(),
        ["Call of function bad 1 failed"]
    );
}

#[test]
fn nested_calls_keep_inner_parentheses_and_commas() {
    let (_, _, context) = eval("f(g(1,2),3)");
    assert_eq!(context.calls.borrow().as_slice(), ["f g(1,2) 3"]);
}

#[test]
fn malformed_expressions_report_like_upstream() {
    let (value, failed, context) = eval("f(1");
    assert_eq!((value, failed), (0.0, true));
    assert_eq!(context.lines.borrow().as_slice(), ["Unclosed parenthesis"]);
    let (value, _, context) = eval("(1+2");
    assert_eq!(value, 3.0);
    assert_eq!(
        context.lines.borrow().as_slice(),
        ["Mismatched parenthesis"]
    );
    assert_eq!(eval("2**3").0, 0.0);
    assert!(eval("1 2").1);
    assert_eq!(eval("").0, 0.0);
}
