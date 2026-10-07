//! `Draw::Atof`: the numeric expression language of DRAW arguments.
//!
//! Grammar (no precedence beyond `*`/`/` over `+`/`-`, evaluated left to right):
//! `expr = factor {(+|-) factor}`, `factor = value {(*|/) value}`,
//! `value = (expr) | +value | -value | number | variable | function(args)`.
//! A name that is a numeric DRAW variable gives its value; a name followed by `(...)` is
//! evaluated as the Tcl command `name arg1 arg2 ...` and its result is parsed. Any other
//! name sets the parse-failure flag and gives 0.
//!
//! The port works on a NUL-terminated byte buffer and edits it in place exactly as the
//! C++ does (it blanks the parentheses and commas of a function call), so that every
//! quirk of the original survives. One difference: after a missing `)` the C++ steps
//! past the terminating NUL and reads outside its buffer (undefined behavior); here the
//! position stops at the terminator.
use rustycad_tkernel::standard::strtod_bytes;

/// What the parser needs from the DRAW session.
pub trait ExpressionContext {
    /// `Draw::Get(name, double&)`: the value of a numeric variable.
    fn number_variable(&self, name: &str) -> Option<f64>;
    /// Evaluates `script` as a Tcl command and returns its numeric result, printing
    /// `Call of function <script> failed` (and returning 0) on a Tcl error; the
    /// interpreter result is saved and restored around the call.
    fn call_function(&self, script: &str) -> f64;
    /// `std::cout << text << '\n'`.
    fn print_line(&self, text: &str);
    /// Sets the session-wide `Draw_ParseFailed` flag.
    fn set_parse_failed(&self, failed: bool);
}

/// `Draw::Atof`: parses `text`, setting the parse-failure flag when the expression has
/// trailing text or an unknown name.
pub fn evaluate(text: &str, context: &dyn ExpressionContext) -> f64 {
    let mut parser = Parser::new(text, context);
    context.set_parse_failed(false);
    let value = parser.parse_sum();
    parser.skip_blanks();
    if parser.current() != 0 {
        context.set_parse_failed(true);
    }
    value
}

/// `Numeric`: a digit or a dot.
fn is_numeric(byte: u8) -> bool {
    byte == b'.' || byte.is_ascii_digit()
}

/// `Alphabetic`: a letter or an underscore.
fn is_alphabetic(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

/// The parser state: the edited buffer and the read position.
struct Parser<'a> {
    buffer: Vec<u8>,
    position: usize,
    context: &'a dyn ExpressionContext,
}

impl<'a> Parser<'a> {
    fn new(text: &str, context: &'a dyn ExpressionContext) -> Self {
        // `strcpy` into a buffer of strlen + 1 bytes: the text up to its first NUL.
        let mut buffer: Vec<u8> = text.bytes().take_while(|&b| b != 0).collect();
        buffer.push(0);
        Self {
            buffer,
            position: 0,
            context,
        }
    }

    /// The byte at `index`, 0 past the terminator.
    fn at(&self, index: usize) -> u8 {
        self.buffer.get(index).copied().unwrap_or(0)
    }

    fn current(&self) -> u8 {
        self.at(self.position)
    }

    /// Advances one byte, never past the terminator.
    fn advance(&mut self) {
        self.position = (self.position + 1).min(self.buffer.len() - 1);
    }

    fn skip_blanks(&mut self) {
        while matches!(self.current(), b' ' | b'\t') {
            self.advance();
        }
    }

    /// `Parse`: factors joined by `+` and `-`.
    fn parse_sum(&mut self) -> f64 {
        let mut value = self.parse_product();
        loop {
            match self.current() {
                b'+' => {
                    self.advance();
                    value += self.parse_product();
                }
                b'-' => {
                    self.advance();
                    value -= self.parse_product();
                }
                _ => return value,
            }
        }
    }

    /// `ParseFactor`: values joined by `*` and `/`.
    fn parse_product(&mut self) -> f64 {
        let mut value = self.parse_value();
        loop {
            match self.current() {
                b'*' => {
                    self.advance();
                    value *= self.parse_value();
                }
                b'/' => {
                    self.advance();
                    value /= self.parse_value();
                }
                _ => return value,
            }
        }
    }

    /// `ParseValue`: one operand, with the blanks around it.
    fn parse_value(&mut self) -> f64 {
        self.skip_blanks();
        let value = match self.current() {
            0 => 0.0,
            b'(' => self.parse_parenthesized(),
            b'+' => {
                self.advance();
                self.parse_value()
            }
            b'-' => {
                self.advance();
                -self.parse_value()
            }
            _ => self.parse_name_or_number(),
        };
        self.skip_blanks();
        value
    }

    /// `(expr)`; a missing `)` is reported and skipped over anyway.
    fn parse_parenthesized(&mut self) -> f64 {
        self.advance();
        let value = self.parse_sum();
        if self.current() != b')' {
            self.context.print_line("Mismatched parenthesis");
        }
        self.advance();
        value
    }

    /// The end of the token at the position: digits and dots, an exponent marker with
    /// its first character, then letters, digits, dots and underscores.
    fn token_end(&self) -> usize {
        let mut end = self.position;
        while is_numeric(self.at(end)) {
            end += 1;
        }
        let next = self.at(end + 1);
        if matches!(self.at(end), b'e' | b'E') && (is_numeric(next) || next == b'+' || next == b'-')
        {
            end += 2;
        }
        while is_numeric(self.at(end)) || is_alphabetic(self.at(end)) {
            end += 1;
        }
        end
    }

    /// A number literal, a numeric variable, or a function call.
    fn parse_name_or_number(&mut self) -> f64 {
        let start = self.position;
        let end = self.token_end();
        let token = self.buffer[start..end].to_vec();
        let value = if token.first().is_some_and(|&byte| is_numeric(byte)) {
            // OCCT `Atof` is `Strtod`: the longest numeric prefix of the token.
            strtod_bytes(&token).0
        } else if let Some(value) = self
            .context
            .number_variable(&String::from_utf8_lossy(&token))
        {
            value
        } else {
            return self.parse_function_call(start, end);
        };
        self.position = end;
        value
    }

    /// `name(arg, ...)`: counts the arguments, then evaluates `name arg ...`; a name
    /// without parentheses sets the failure flag and gives 0.
    fn parse_function_call(&mut self, start: usize, end: usize) -> f64 {
        let mut open = end;
        while matches!(self.at(open), b' ' | b'\t') {
            open += 1;
        }
        if self.at(open) != b'(' {
            self.context.set_parse_failed(true);
            self.position = end;
            return 0.0;
        }
        if !self.parentheses_close(open) {
            self.context.print_line("Unclosed parenthesis");
            self.position = end;
            return 0.0;
        }
        let close = self.blank_call_syntax(end);
        let script = String::from_utf8_lossy(&self.buffer[start..close]).into_owned();
        self.position = close + 1;
        self.context.call_function(&script)
    }

    /// True when the parenthesis at `open` is closed before the terminator.
    fn parentheses_close(&self, open: usize) -> bool {
        let (mut depth, mut index) = (1, open + 1);
        while depth > 0 && self.at(index) != 0 {
            match self.at(index) {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        depth == 0
    }

    /// Turns `name(a,b)` into `name a b` in place: the opening parenthesis and the
    /// top-level commas become blanks and the closing parenthesis ends the string.
    /// Returns the index of that terminator.
    fn blank_call_syntax(&mut self, mut index: usize) -> usize {
        while self.buffer[index] != b'(' {
            index += 1;
        }
        self.buffer[index] = b' ';
        index += 1;
        let mut depth = 1;
        while depth > 0 {
            match self.buffer[index] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b',' if depth == 1 => self.buffer[index] = b' ',
                _ => {}
            }
            index += 1;
        }
        self.buffer[index - 1] = 0;
        index - 1
    }
}

#[cfg(test)]
mod tests;
