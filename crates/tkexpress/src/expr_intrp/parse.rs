// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExprIntrp::Parse`: one run of the interpreter over a text.
use super::analysis::ExprIntrpAnalysis;
use super::context::ExprIntrpParseContext;
use super::generator::ExprIntrpGenerator;
use super::parser::ExprIntrpParser;
use super::scanner::ExprIntrpScanner;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use std::io::Write;

/// The result of [`parse`]: whether the text was accepted, and the analysis stacks the
/// interpreter reads its result from (`ExprIntrp_Recept` after the parse).
pub struct ExprIntrpParseOutcome {
    /// True when the whole text parsed and every action succeeded.
    pub accepted: bool,
    /// The stacks left by the semantic actions.
    pub recept: ExprIntrpAnalysis,
}

/// `ExprIntrp::Parse` (private natively, for `ExprIntrp_GenExp`): parse `text` for
/// `generator`; an empty text, a syntax error or any failure of an action gives
/// `accepted == false`, as the native `catch (Standard_Failure)`. Characters no token
/// matches go to `output`;
/// `parse(&generator, &AsciiString::new("x+1"), &mut std::io::sink()).accepted`.
pub fn parse(
    generator: &Handle<ExprIntrpGenerator>,
    text: &AsciiString,
    output: &mut dyn Write,
) -> ExprIntrpParseOutcome {
    let mut recept: ExprIntrpAnalysis = ExprIntrpAnalysis::new();
    recept.set_master(generator);
    if text.is_empty() {
        return ExprIntrpParseOutcome {
            accepted: false,
            recept,
        };
    }
    let scanner: ExprIntrpScanner<'_> = ExprIntrpScanner::new(text.as_bytes(), output);
    let mut context: ExprIntrpParseContext<'_> = ExprIntrpParseContext::new(scanner, recept);
    let accepted: bool = ExprIntrpParser::new(&mut context).parse().is_ok();
    context.scanner.stop();
    ExprIntrpParseOutcome {
        accepted,
        recept: context.recept,
    }
}
