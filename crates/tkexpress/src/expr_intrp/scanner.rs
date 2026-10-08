// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp.lex (the flex scanner
// lex.ExprIntrp.c) and ExprIntrp_yacclex.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The scanner of the expression interpreter.
//!
//! flex picks, at each position, the longest match among the rules of
//! ExprIntrp.lex, the first rule winning a tie; a character no rule matches is
//! copied to the scanner output (`ECHO` to `yyout`, the standard output by default)
//! and skipped. The scanner reads a C string, so the input ends at its first NUL.
//! `ExprIntrp_SetResult` and `ExprIntrp_SetDegree` keep the text of the last value or
//! identifier and the length of the last run of quotes; they are members here.
use super::token::ExprIntrpTokenKind;
use rustycad_tkernel::tcollection::AsciiString;
use std::io::Write;

/// The rules of ExprIntrp.lex, in priority order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScannerRule {
    /// `" "`: skipped.
    Blank,
    /// A fixed operator or keyword, with its token.
    Fixed(ExprIntrpTokenKind),
    /// `[0-9.]+e[+|-]?[0-9]+` and `[0-9.]+`.
    Value,
    /// `[a-zA-Z][a-zA-Z0-9_]*`.
    Identifier,
    /// `"'"+`.
    Quotes,
    /// The default rule: `ECHO` one character.
    Echo,
}

/// The fixed strings of the rules 1 to 20 and 24 of ExprIntrp.lex, in rule order.
const FIXED_RULES: [(&[u8], ExprIntrpTokenKind); 20] = [
    (b"+", ExprIntrpTokenKind::Sumop),
    (b"-", ExprIntrpTokenKind::Minusop),
    (b"/", ExprIntrpTokenKind::Divideop),
    (b"^", ExprIntrpTokenKind::Expop),
    (b"**", ExprIntrpTokenKind::Expop),
    (b"*", ExprIntrpTokenKind::Multop),
    (b"(", ExprIntrpTokenKind::Parenthesis),
    (b"[", ExprIntrpTokenKind::Bracket),
    (b")", ExprIntrpTokenKind::Endparenthesis),
    (b"]", ExprIntrpTokenKind::Endbracket),
    (b",", ExprIntrpTokenKind::Comma),
    (b"@", ExprIntrpTokenKind::Differential),
    (b"<-", ExprIntrpTokenKind::Assignop),
    (b"=", ExprIntrpTokenKind::Equalop),
    (b"Deassign", ExprIntrpTokenKind::Deassignkey),
    (b"Deriv", ExprIntrpTokenKind::Derivkey),
    (b"Const", ExprIntrpTokenKind::Constkey),
    (b"Sum", ExprIntrpTokenKind::Sumkey),
    (b"Prod", ExprIntrpTokenKind::Prodkey),
    (b";", ExprIntrpTokenKind::Relseparator),
];

/// Length of the run of bytes satisfying `accept` at the start of `text`.
fn run_length(text: &[u8], accept: impl Fn(u8) -> bool) -> usize {
    text.iter().take_while(|&&byte| accept(byte)).count()
}

/// Length of `[0-9.]+e[+|-]?[0-9]+`, else of `[0-9.]+`, at the start of `text`; 0
/// when neither matches. The `|` in the sign class is the native pattern's own.
fn value_length(text: &[u8]) -> usize {
    let mantissa: usize = run_length(text, |byte| byte.is_ascii_digit() || byte == b'.');
    if mantissa == 0 || text.get(mantissa) != Some(&b'e') {
        return mantissa;
    }
    let mut exponent: usize = mantissa + 1;
    if matches!(text.get(exponent), Some(b'+' | b'|' | b'-')) {
        exponent += 1;
    }
    let digits: usize = run_length(&text[exponent..], |byte| byte.is_ascii_digit());
    if digits == 0 {
        return mantissa;
    }
    exponent + digits
}

/// Length of `[a-zA-Z][a-zA-Z0-9_]*` at the start of `text`, 0 when it does not match.
fn identifier_length(text: &[u8]) -> usize {
    if !text.first().is_some_and(u8::is_ascii_alphabetic) {
        return 0;
    }
    1 + run_length(&text[1..], |byte| {
        byte.is_ascii_alphanumeric() || byte == b'_'
    })
}

/// The longest match at the start of `text` (not empty): the rule and its length,
/// the earlier rule winning a tie.
fn longest_match(text: &[u8]) -> (ScannerRule, usize) {
    let mut best: (ScannerRule, usize) = (ScannerRule::Echo, 1);
    let mut consider = |rule: ScannerRule, length: usize| {
        if length > best.1 || (length == best.1 && best.0 == ScannerRule::Echo && length > 0) {
            best = (rule, length);
        }
    };
    if text[0] == b' ' {
        consider(ScannerRule::Blank, 1);
    }
    for (pattern, token) in FIXED_RULES {
        if text.starts_with(pattern) {
            consider(ScannerRule::Fixed(token), pattern.len());
        }
    }
    consider(ScannerRule::Value, value_length(text));
    consider(ScannerRule::Identifier, identifier_length(text));
    consider(ScannerRule::Quotes, run_length(text, |byte| byte == b'\''));
    best
}

/// The scanner of one parse over a C string (`ExprIntrp_start_string`), with the
/// lexical results the semantic actions read; `ExprIntrpScanner::new(b"x+1", &mut out)`.
pub struct ExprIntrpScanner<'a> {
    input: &'a [u8],
    position: usize,
    output: &'a mut dyn Write,
    result: AsciiString,
    degree: i32,
}
impl<'a> ExprIntrpScanner<'a> {
    /// Scan `text` up to its first NUL (`ExprIntrp_start_string`); unmatched characters
    /// go to `output` (`yyout`); `ExprIntrpScanner::new(b"x+1", &mut std::io::sink())`.
    pub fn new(text: &'a [u8], output: &'a mut dyn Write) -> Self {
        let end: usize = text
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(text.len());
        Self {
            input: &text[..end],
            position: 0,
            output,
            result: AsciiString::new(""),
            degree: 0,
        }
    }
    /// Release the input (`ExprIntrp_stop_string`); later calls see the end of the
    /// input; `scanner.stop()`.
    pub fn stop(&mut self) {
        self.position = self.input.len();
    }
    /// The next token (`ExprIntrplex`): its external number, 0 at the end of the
    /// input; `scanner.lex() == ExprIntrpTokenKind::Identifier as i32`.
    pub fn lex(&mut self) -> i32 {
        while self.position < self.input.len() {
            let text: &[u8] = &self.input[self.position..];
            let (rule, length) = longest_match(text);
            let lexeme: &[u8] = &text[..length];
            self.position += length;
            if let Some(token) = self.act(rule, lexeme) {
                return token as i32;
            }
        }
        ExprIntrpTokenKind::Yyeof as i32
    }
    /// The action of `rule` on `lexeme`: the token it returns, `None` for a skip.
    fn act(&mut self, rule: ScannerRule, lexeme: &[u8]) -> Option<ExprIntrpTokenKind> {
        match rule {
            ScannerRule::Blank => None,
            ScannerRule::Echo => {
                // `ECHO`: fwrite ignores a failed write, and so does the port.
                let _ = self.output.write_all(lexeme);
                None
            }
            ScannerRule::Fixed(token) => Some(token),
            ScannerRule::Value => {
                self.set_result(lexeme);
                Some(ExprIntrpTokenKind::Value)
            }
            ScannerRule::Identifier => {
                self.set_result(lexeme);
                Some(ExprIntrpTokenKind::Identifier)
            }
            ScannerRule::Quotes => {
                self.set_degree(lexeme);
                Some(ExprIntrpTokenKind::Derivate)
            }
        }
    }
    /// Keep the text of a value or an identifier (`ExprIntrp_SetResult`);
    /// `scanner.set_result(b"x")`.
    pub fn set_result(&mut self, lexeme: &[u8]) {
        self.result = AsciiString::from_bytes(lexeme);
    }
    /// Keep the number of quotes of a derivation mark (`ExprIntrp_SetDegree`);
    /// `scanner.set_degree(b"''")` gives degree 2.
    pub fn set_degree(&mut self, lexeme: &[u8]) {
        self.degree = i32::try_from(lexeme.len()).unwrap_or(i32::MAX);
    }
    /// The text of the last value or identifier (`ExprIntrp_GetResult`);
    /// `scanner.get_result()`.
    pub fn get_result(&self) -> &AsciiString {
        &self.result
    }
    /// The number of quotes of the last derivation mark (`ExprIntrp_GetDegree`);
    /// `scanner.get_degree()`.
    pub fn get_degree(&self) -> i32 {
        self.degree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every token of `text`, with the bytes echoed.
    fn tokens(text: &[u8]) -> (Vec<i32>, Vec<u8>) {
        let mut echoed: Vec<u8> = Vec::new();
        let mut scanner = ExprIntrpScanner::new(text, &mut echoed);
        let mut found: Vec<i32> = Vec::new();
        loop {
            let token: i32 = scanner.lex();
            found.push(token);
            if token == 0 {
                break;
            }
        }
        (found, echoed)
    }

    #[test]
    fn the_longest_match_wins_and_keywords_win_ties() {
        assert_eq!(tokens(b"x**2").0, [268, 261, 267, 0]);
        assert_eq!(tokens(b"Sum Sums").0, [278, 268, 0]);
        assert_eq!(tokens(b"1e5 1e x1.e|3").0, [267, 267, 268, 268, 267, 0]);
    }

    #[test]
    fn unmatched_characters_are_echoed_and_skipped() {
        let (found, echoed) = tokens(b"a$\t<-<b");
        assert_eq!(found, [268, 273, 268, 0]);
        assert_eq!(echoed, b"$\t<");
    }

    #[test]
    fn the_input_ends_at_nul_and_results_are_kept() {
        let mut sink: Vec<u8> = Vec::new();
        let mut scanner = ExprIntrpScanner::new(b"f''\0x", &mut sink);
        assert_eq!(scanner.lex(), 268);
        assert_eq!(scanner.get_result().as_bytes(), b"f");
        assert_eq!(scanner.lex(), 271);
        assert_eq!(scanner.get_degree(), 2);
        assert_eq!(scanner.lex(), 0);
        assert_eq!(scanner.lex(), 0);
    }
}
