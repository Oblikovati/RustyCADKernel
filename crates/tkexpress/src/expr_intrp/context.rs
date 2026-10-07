// Rust translation of the parser state of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_yaccintrf.cxx
// (its file-static variables and the global ExprIntrp_Recept) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The state that the semantic actions share during one parse.
//!
//! Natively it is global: `ExprIntrp_Recept`, the scanner buffer and the statics of
//! ExprIntrp_yaccintrf.cxx. Every parse starts by resetting the parts it reads, so a
//! context per parse behaves the same and lets parses run on several threads.
use super::analysis::ExprIntrpAnalysis;
use super::errors::syntax_error;
use super::scanner::ExprIntrpScanner;
use crate::expr::{ExprHandle, ExprNamedExpression, ExprNamedUnknown};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// The scanner, the analysis stacks and the statics of the semantic actions;
/// `ExprIntrpParseContext::new(scanner, analysis)`.
pub struct ExprIntrpParseContext<'a> {
    /// The scanner and its lexical results (`ExprIntrp_GetResult`, `ExprIntrp_GetDegree`).
    pub scanner: ExprIntrpScanner<'a>,
    /// The stacks of the actions (`ExprIntrp_Recept`).
    pub recept: ExprIntrpAnalysis,
    /// `ExprIntrp_assname`: the unknown of an assignment.
    pub(super) assname: AsciiString,
    /// `ExprIntrp_funcdefname`: the function of a definition.
    pub(super) funcdefname: AsciiString,
    /// `ExprIntrp_nbargs`: the arguments counted since the last function start.
    pub(super) nbargs: i32,
    /// `ExprIntrp_nbdiff`: the differentials of a `@f/@X1` function.
    pub(super) nbdiff: i32,
}
impl<'a> ExprIntrpParseContext<'a> {
    /// The context of one parse over `scanner` with the analysis `recept`;
    /// `ExprIntrpParseContext::new(scanner, analysis)`.
    pub fn new(scanner: ExprIntrpScanner<'a>, recept: ExprIntrpAnalysis) -> Self {
        Self {
            scanner,
            recept,
            assname: AsciiString::new(""),
            funcdefname: AsciiString::new(""),
            nbargs: 0,
            nbdiff: 0,
        }
    }
    /// The text of the last value or identifier (`ExprIntrp_GetResult`).
    pub(super) fn result(&self) -> AsciiString {
        self.scanner.get_result().clone()
    }
    /// The named expression the parse knows as the last identifier read, which must be
    /// an unknown; null when unknown; ExprIntrp_SyntaxError for a named constant.
    pub(super) fn named_unknown(
        &self,
        name: &AsciiString,
    ) -> Result<Handle<ExprNamedUnknown>, Failure> {
        let named: Handle<dyn ExprNamedExpression> = self.recept.get_named(name);
        if named.is_null() {
            return Ok(Handle::default());
        }
        let unknown: Handle<ExprNamedUnknown> = named.cast();
        if unknown.is_null() {
            return Err(syntax_error(&format!(
                "{} is not an unknown; expected an Expr_NamedUnknown",
                String::from_utf8_lossy(name.as_bytes())
            )));
        }
        Ok(unknown)
    }
    /// Pop an expression that must exist: a null pop is the native null dereference
    /// (or the explicit ExprIntrp_SyntaxError where native code checks it).
    pub(super) fn pop_required(&mut self, member: &str) -> Result<ExprHandle, Failure> {
        let expression: ExprHandle = self.recept.pop();
        if expression.is_null() {
            return Err(syntax_error(&format!(
                "{member}: no expression on the stack; expected an operand"
            )));
        }
        Ok(expression)
    }
}

/// `ExprIntrperror`: the scanner is released and the error becomes an
/// ExprIntrp_SyntaxError carrying the parser message; `expr_intrp_error("syntax error")`.
pub fn expr_intrp_error(msg: &str) -> Failure {
    syntax_error(msg)
}

/// `atoi` of the text after the first byte of `text`: optional blanks, a sign and
/// digits, the `long` result of `strtol` truncated to `int`; `atoi_after_first(b"X12") == 12`.
pub fn atoi_after_first(text: &[u8]) -> i32 {
    let rest: &[u8] = text.get(1..).unwrap_or_default();
    let rest: &[u8] = &rest[rest
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count()..];
    let (negative, digits) = match rest.first() {
        Some(b'-') => (true, &rest[1..]),
        Some(b'+') => (false, &rest[1..]),
        _ => (false, rest),
    };
    let mut value: i64 = 0;
    for byte in digits.iter().take_while(|byte| byte.is_ascii_digit()) {
        // strtol saturates at LONG_MAX / LONG_MIN.
        value = value
            .saturating_mul(10)
            .saturating_add(i64::from(byte - b'0'));
    }
    let value: i64 = if negative {
        value.saturating_neg()
    } else {
        value
    };
    // `(int)` keeps the low 32 bits of the `long`.
    value as i32
}

/// `(int)value`, the C conversion as the x86-64 `cvttsd2si` instruction performs it:
/// truncation toward zero, and `INT_MIN` for NaN and out-of-range values;
/// `c_int_of(2.7) == 2` and `c_int_of(1e20) == i32::MIN`.
pub fn c_int_of(value: f64) -> i32 {
    if value.is_nan() || value >= 2_147_483_648.0 || value <= -2_147_483_649.0 {
        return i32::MIN;
    }
    value as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atoi_reads_the_digits_after_the_first_byte() {
        assert_eq!(atoi_after_first(b"X12"), 12);
        assert_eq!(atoi_after_first(b"x3ab"), 3);
        assert_eq!(atoi_after_first(b"X"), 0);
        assert_eq!(atoi_after_first(b"Xa1"), 0);
        assert_eq!(atoi_after_first(b"X99999999999999999999"), -1);
    }

    #[test]
    fn c_int_of_truncates_and_saturates_to_int_min() {
        assert_eq!(c_int_of(2.7), 2);
        assert_eq!(c_int_of(-2.7), -2);
        assert_eq!(c_int_of(1e20), i32::MIN);
        assert_eq!(c_int_of(f64::NAN), i32::MIN);
        assert_eq!(c_int_of(-2_147_483_648.5), -2_147_483_648);
    }
}
