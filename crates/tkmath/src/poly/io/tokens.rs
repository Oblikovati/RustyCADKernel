//! `std::istream >>` extraction in the C locale, one whitespace-delimited token at a time.
//!
//! Like libstdc++, a token read stops at the first whitespace byte and leaves it in the
//! stream, so several objects can be read one after the other from one stream. A token
//! that the native extraction would reject sets failbit natively and leaves the output
//! variables uninitialized; the port reports it as a `Standard_Failure` that names it.
use rustycad_tkernel::standard::Failure;
use std::io::BufRead;
/// Whitespace-delimited reader over a buffered stream; `TokenReader::new(&mut input)`.
pub(super) struct TokenReader<'a, R: BufRead> {
    input: &'a mut R,
}
impl<'a, R: BufRead> TokenReader<'a, R> {
    pub(super) const fn new(input: &'a mut R) -> Self {
        Self { input }
    }
    /// The next token, or `None` at end of stream; `reader.token()?`.
    pub(super) fn token(&mut self) -> Result<Option<String>, Failure> {
        self.skip_space()?;
        let mut token = Vec::new();
        while let Some(byte) = self.peek()? {
            if byte.is_ascii_whitespace() {
                break;
            }
            token.push(byte);
            self.input.consume(1);
        }
        Ok((!token.is_empty()).then(|| String::from_utf8_lossy(&token).into_owned()))
    }
    fn skip_space(&mut self) -> Result<(), Failure> {
        while let Some(byte) = self.peek()? {
            if !byte.is_ascii_whitespace() {
                break;
            }
            self.input.consume(1);
        }
        Ok(())
    }
    fn peek(&mut self) -> Result<Option<u8>, Failure> {
        let buffer = self
            .input
            .fill_buf()
            .map_err(|error| Failure::new(&format!("Poly stream read failed: {error}")))?;
        Ok(buffer.first().copied())
    }
    /// A decimal `int`, as `operator>>(int&)`; `reader.integer("nbNodes")?`.
    pub(super) fn integer(&mut self, field: &str) -> Result<i32, Failure> {
        let token = self.required(field)?;
        token
            .parse()
            .map_err(|_| rejected(field, &token, "a decimal int"))
    }
    /// A boolean written as 0 or 1, as `operator>>(bool&)` without boolalpha; `reader.flag("hasUV")?`.
    pub(super) fn flag(&mut self, field: &str) -> Result<bool, Failure> {
        let token = self.required(field)?;
        match token.parse::<i64>() {
            Ok(0) => Ok(false),
            Ok(1) => Ok(true),
            _ => Err(rejected(field, &token, "0 or 1")),
        }
    }
    /// A decimal double; libstdc++ accepts no `inf`, `nan` or hexadecimal form; `reader.real("x")?`.
    pub(super) fn real(&mut self, field: &str) -> Result<f64, Failure> {
        let token = self.required(field)?;
        if !is_decimal_real(&token) {
            return Err(rejected(field, &token, "a decimal real"));
        }
        let value: f64 = token
            .parse()
            .map_err(|_| rejected(field, &token, "a decimal real"))?;
        if value.is_infinite() {
            // An out-of-range value sets failbit in libstdc++.
            return Err(rejected(field, &token, "a finite decimal real"));
        }
        Ok(value)
    }
    fn required(&mut self, field: &str) -> Result<String, Failure> {
        self.token()?.ok_or_else(|| {
            Failure::new(&format!(
                "Poly stream: end of stream at {field}; expected a value"
            ))
        })
    }
}
fn rejected(field: &str, token: &str, expected: &str) -> Failure {
    Failure::new(&format!(
        "Poly stream: {field} is {token:?}; expected {expected}"
    ))
}
fn is_decimal_real(token: &str) -> bool {
    let body = token.strip_prefix(['+', '-']).unwrap_or(token);
    let (mantissa, exponent) = match body.find(['e', 'E']) {
        Some(split) => (&body[..split], Some(&body[split + 1..])),
        None => (body, None),
    };
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let mantissa_ok = !digits.is_empty()
        && digits.chars().all(|c| c.is_ascii_digit())
        && mantissa.matches('.').count() <= 1;
    let exponent_ok = exponent.is_none_or(|text| {
        let text = text.strip_prefix(['+', '-']).unwrap_or(text);
        !text.is_empty() && text.chars().all(|c| c.is_ascii_digit())
    });
    mantissa_ok && exponent_ok
}
