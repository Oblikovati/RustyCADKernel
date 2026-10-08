//! `ex_text`: the interpreter and the expression battery of tools/oracle/tkexpress_probe.cxx.
use super::common::{ProbeLine, c_text};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkexpress::expr::ExprNamedExpression;
use rustycad_tkexpress::expr::handles::one_based;
use rustycad_tkexpress::expr::package::nb_of_free_variables;
use rustycad_tkexpress::expr::{ExprHandle, ExprNamedUnknown, ExprUnknownIterator};
use rustycad_tkexpress::expr_intrp::ExprIntrpGenExp;
use std::io::Write;
use std::sync::{Arc, Mutex, PoisonError};

/// A scanner output whose bytes the probe reads back (the native memstream).
#[derive(Clone, Default)]
struct EchoBuffer {
    bytes: Arc<Mutex<Vec<u8>>>,
}
impl Write for EchoBuffer {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.bytes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Process the hexadecimal `text` and return the characters the scanner echoed
/// (`ex_process`).
pub fn process(interpreter: &ExprIntrpGenExp, text: &str) -> Vec<u8> {
    let echo: EchoBuffer = EchoBuffer::default();
    interpreter.set_scanner_output(Box::new(echo.clone()));
    interpreter.process(&c_text(text));
    let echoed: Vec<u8> = echo
        .bytes
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    echoed
}

/// The unknowns of an expression in iteration order (`ex_unknowns`).
fn unknowns(expression: &ExprHandle) -> Result<Vec<Handle<ExprNamedUnknown>>, Failure> {
    let mut iterator: ExprUnknownIterator = ExprUnknownIterator::new(expression)?;
    let mut all: Vec<Handle<ExprNamedUnknown>> = Vec::new();
    while iterator.more() {
        all.push(iterator.value()?);
        iterator.next()?;
    }
    Ok(all)
}

/// The structure and simplification part of `ex_battery`.
fn structure(line: &mut ProbeLine, e: &ExprHandle) {
    line.step(|line| Ok(line.text(&e.string()?)));
    line.step(|line| {
        line.int(e.nb_sub_expressions());
        line.boolean(e.is_shareable());
        Ok(())
    });
    line.step(|line| Ok(line.boolean(e.contains_unknowns()?)));
    line.step(|line| Ok(line.boolean(e.is_linear()?)));
    line.step(|line| Ok(line.text(&e.simplified()?.string()?)));
    line.step(|line| Ok(line.text(&e.shallow_simplified()?.string()?)));
    line.step(|line| {
        let copy: ExprHandle = e.copy()?;
        line.text(&copy.string()?);
        line.boolean(e.is_identical(&copy)?);
        Ok(())
    });
    line.step(|line| Ok(line.int(nb_of_free_variables(e)?)));
}

/// The derivation part of `ex_battery`, for the first three unknowns.
fn derivatives(line: &mut ProbeLine, e: &ExprHandle, vars: &[Handle<ExprNamedUnknown>]) {
    for x in vars.iter().take(3) {
        line.step(|line| {
            line.text(&x.get_name());
            line.boolean(e.contains(&x.cast())?);
            Ok(())
        });
        line.step(|line| Ok(line.text(&e.derivative(x)?.string()?)));
        line.step(|line| Ok(line.text(&e.n_derivative(x, 2)?.string()?)));
    }
}

/// The evaluation part of `ex_battery`: the unknown `i` is 0.5 + 0.25*i.
fn evaluations(line: &mut ProbeLine, e: &ExprHandle, vars: &[Handle<ExprNamedUnknown>]) {
    line.step(|line| Ok(line.real(e.evaluate_numeric()?)));
    line.step(|line| {
        if vars.is_empty() {
            return Ok(());
        }
        let values: Vec<f64> = (1..=vars.len()).map(|i| 0.5 + 0.25 * i as f64).collect();
        line.real(e.evaluate(&one_based(vars.to_vec()), &one_based(values))?);
        Ok(())
    });
}

/// `ex_battery`.
fn battery(line: &mut ProbeLine, e: &ExprHandle) {
    structure(line, e);
    let mut vars: Vec<Handle<ExprNamedUnknown>> = Vec::new();
    line.step(|line| {
        vars = unknowns(e)?;
        line.int(vars.len() as i32);
        Ok(())
    });
    derivatives(line, e, &vars);
    evaluations(line, e, &vars);
}

/// `ex_text <text>...`.
pub fn text_case(fields: &[&str]) {
    let mut line: ProbeLine = ProbeLine::default();
    let interpreter: Handle<ExprIntrpGenExp> = ExprIntrpGenExp::create();
    for text in fields {
        let echoed: Vec<u8> = process(&interpreter, text);
        line.token(if interpreter.is_done() { "d1" } else { "d0" });
        line.bytes(&echoed);
        if !interpreter.is_done() {
            continue;
        }
        match interpreter.expression() {
            Ok(e) if !e.is_null() => battery(&mut line, &e),
            _ => line.token("n"),
        }
    }
    line.int(interpreter.get_named_list().len() as i32);
    line.int(interpreter.get_functions().len() as i32);
    line.end();
}
