// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_GenExp.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExprIntrp_GenExp`: the interpreter that builds an expression from a text.
use super::generator::ExprIntrpGenerator;
use super::parse::{ExprIntrpParseOutcome, parse};
use crate::expr::ExprHandle;
use crate::expr::errors::no_such_object;
use crate::expr::handles::ExprSelfLink;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkernel::tcollection::AsciiString;
use std::io::Write;
use std::ops::Deref;

/// The result of the last `Process`.
#[derive(Default)]
struct GenExpResult {
    done: bool,
    expression: ExprHandle,
}

/// Where the scanner copies the characters it cannot match: the standard output, as
/// flex's `yyout`, unless [`ExprIntrpGenExp::set_scanner_output`] replaced it.
type ScannerOutput = Box<dyn Write + Send + Sync>;

/// The expression interpreter: `Process` parses a text, `Expression` gives the
/// result. Unknowns, constants and functions met in one text stay known for the next
/// ones (they are kept by the embedded [`ExprIntrpGenerator`]).
///
/// ```
/// use rustycad_tkexpress::expr_intrp::ExprIntrpGenExp;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let interpreter = ExprIntrpGenExp::create();
/// interpreter.process(&AsciiString::new("2*x+x"));
/// assert!(interpreter.is_done());
/// assert_eq!(interpreter.expression().unwrap().string().unwrap(), "(2*x)+x");
/// ```
pub struct ExprIntrpGenExp {
    parent: ExprIntrpGenerator,
    result: SharedState<GenExpResult>,
    output: SharedState<Option<ScannerOutput>>,
    link: ExprSelfLink,
}
impl_transient!(
    ExprIntrpGenExp,
    "ExprIntrp_GenExp",
    ExprIntrpGenerator,
    parent
);
impl Deref for ExprIntrpGenExp {
    type Target = ExprIntrpGenerator;
    /// The embedded generator, as the C++ base class; `interpreter.get_named(&name)`.
    fn deref(&self) -> &ExprIntrpGenerator {
        &self.parent
    }
}
impl ExprIntrpGenExp {
    /// A new interpreter (`Create`); `ExprIntrpGenExp::create()`.
    pub fn create() -> Handle<Self> {
        let object: Self = Self {
            parent: ExprIntrpGenerator::new(),
            result: SharedState::new(GenExpResult::default()),
            output: SharedState::new(None),
            link: ExprSelfLink::default(),
        };
        crate::expr::handles::adopt(object, |interpreter| &interpreter.link)
    }
    /// Send the characters the scanner cannot match to `output` instead of the
    /// standard output (the port's form of flex's `ExprIntrpset_out`);
    /// `interpreter.set_scanner_output(Box::new(std::io::sink()))`.
    pub fn set_scanner_output(&self, output: Box<dyn Write + Send + Sync>) {
        *self.output.write() = Some(output);
    }
    /// Parse `str` (`Process`): done when the text is accepted; the expression is the
    /// last one stacked, null after a definition, an assignment or a relation;
    /// `interpreter.process(&AsciiString::new("Sin(x)"))`.
    pub fn process(&self, str: &AsciiString) {
        let outcome: ExprIntrpParseOutcome = self.run_parse(str);
        let mut result = self.result.write();
        *result = GenExpResult::default();
        if !outcome.accepted {
            return;
        }
        let mut recept = outcome.recept;
        if !recept.is_exp_stack_empty() {
            result.expression = recept.pop();
        }
        result.done = true;
    }
    /// True when the last text was accepted (`IsDone`); `interpreter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.result.read().done
    }
    /// The expression of the last text (`Expression`); Standard_NoSuchObject when the
    /// last text was not accepted; `interpreter.expression()?`.
    pub fn expression(&self) -> Result<ExprHandle, Failure> {
        let result = self.result.read();
        if !result.done {
            return Err(no_such_object(
                "ExprIntrp_GenExp::Expression: the last Process failed; expected IsDone()",
            ));
        }
        Ok(result.expression.clone())
    }
    /// `ExprIntrp::Parse(me, str)` with the scanner output of this interpreter.
    fn run_parse(&self, str: &AsciiString) -> ExprIntrpParseOutcome {
        let me: Handle<ExprIntrpGenerator> = self.link.me().unwrap_or_default();
        let mut output = self.output.write();
        match output.as_mut() {
            Some(output) => parse(&me, str, output.as_mut()),
            None => parse(&me, str, &mut std::io::stdout()),
        }
    }
}
