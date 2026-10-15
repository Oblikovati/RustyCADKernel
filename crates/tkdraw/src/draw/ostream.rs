//! `Standard_OStream` as the drawables use it: a text buffer with the `std::ostream`
//! precision, `scientific` flag and field width that `operator<<(double)` reads.
use super::shared_buffer::SharedBuffer;
use rustycad_tkernel::storage::{IoStream, format_general};

/// An in-memory output stream with `std::ostream` number formatting.
///
/// ```
/// use rustycad_tkdraw::draw::DrawOStream;
/// let mut stream = DrawOStream::new();
/// stream.put_real(0.1).put_str(" ").put_int(3);
/// assert_eq!(stream.text(), "0.1 3");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawOStream {
    text: String,
    precision: usize,
    scientific: bool,
    width: usize,
}

impl Default for DrawOStream {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawOStream {
    /// An empty stream with the `std::ostream` defaults: precision 6, general format.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            precision: 6,
            scientific: false,
            width: 0,
        }
    }

    /// The text written so far.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Takes the text out, leaving the stream empty (formatting state kept).
    pub fn take_text(&mut self) -> String {
        std::mem::take(&mut self.text)
    }

    /// `precision()`.
    pub fn precision(&self) -> usize {
        self.precision
    }

    /// `precision(n)`.
    pub fn set_precision(&mut self, precision: usize) -> &mut Self {
        self.precision = precision;
        self
    }

    /// `setf(std::ios::scientific)` (true) or the default float field (false).
    pub fn set_scientific(&mut self, scientific: bool) -> &mut Self {
        self.scientific = scientific;
        self
    }

    /// `width(n)`: applies to the next insertion only, as in C++.
    pub fn set_width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }

    /// `operator<<(const char*)`.
    pub fn put_str(&mut self, text: &str) -> &mut Self {
        self.pad_and_push(text.to_string())
    }

    /// `operator<<(int)`.
    pub fn put_int(&mut self, value: i64) -> &mut Self {
        self.pad_and_push(value.to_string())
    }

    /// `operator<<(double)`: `%.{precision}g`, or `%.{precision}e` when scientific.
    pub fn put_real(&mut self, value: f64) -> &mut Self {
        let text = self.real_text()(value);
        self.pad_and_push(text)
    }

    /// The text `operator<<(double)` gives on this stream in its current state, for writers
    /// that format their own numbers (the Poly writers).
    ///
    /// ```
    /// use rustycad_tkdraw::draw::DrawOStream;
    /// let mut stream = DrawOStream::new();
    /// stream.set_scientific(true).set_precision(2);
    /// assert_eq!(stream.real_text()(1.5), "1.50e+00");
    /// ```
    pub fn real_text(&self) -> Box<dyn Fn(f64) -> String> {
        let (scientific, precision) = (self.scientific, self.precision);
        Box::new(move |value| {
            if scientific {
                format_scientific(value, precision)
            } else {
                format_general(value, precision)
            }
        })
    }

    /// Lets `write` print on this stream through the TKernel `std::ostream` model (the
    /// GeomTools and Poly printers take an [`IoStream`]) at this stream's precision;
    /// what it wrote is appended, also when it fails.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::DrawOStream;
    /// let mut stream = DrawOStream::new();
    /// stream.set_precision(3);
    /// let written: Result<(), ()> = stream.write_through(|io| {
    ///     io.insert_real(0.123456);
    ///     Ok(())
    /// });
    /// assert!(written.is_ok());
    /// assert_eq!(stream.text(), "0.123");
    /// ```
    pub fn write_through<E>(
        &mut self,
        write: impl FnOnce(&mut IoStream) -> Result<(), E>,
    ) -> Result<(), E> {
        let buffer = SharedBuffer::default();
        let mut stream = buffer.stream(self.precision);
        let written = write(&mut stream);
        let (bytes, _) = buffer.finish(stream);
        self.text.push_str(&String::from_utf8_lossy(&bytes));
        written
    }

    /// Lets `write` print raw bytes on this stream (the Poly writers take a
    /// [`std::io::Write`] and the number text of [`DrawOStream::real_text`]); what it
    /// wrote is appended, also when it fails.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::DrawOStream;
    /// use std::io::Write;
    /// let mut stream = DrawOStream::new();
    /// stream.write_bytes_through(|out| out.write_all(b"raw")).unwrap();
    /// assert_eq!(stream.text(), "raw");
    /// ```
    pub fn write_bytes_through<E>(
        &mut self,
        write: impl FnOnce(&mut Vec<u8>) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut bytes = Vec::new();
        let written = write(&mut bytes);
        self.text.push_str(&String::from_utf8_lossy(&bytes));
        written
    }

    /// Right-aligns `text` in the pending width (the default `std::ios::right` fill).
    fn pad_and_push(&mut self, text: String) -> &mut Self {
        let width = std::mem::take(&mut self.width);
        let padding = width.saturating_sub(text.chars().count());
        self.text.extend(std::iter::repeat_n(' ', padding));
        self.text.push_str(&text);
        self
    }
}

/// `printf("%.*e", precision, value)`: Rust's `{:e}` with C's signed two-digit exponent.
///
/// ```
/// use rustycad_tkdraw::draw::format_scientific;
/// assert_eq!(format_scientific(1.5, 3), "1.500e+00");
/// assert_eq!(format_scientific(-2.0e-123, 1), "-2.0e-123");
/// ```
pub fn format_scientific(value: f64, precision: usize) -> String {
    if !value.is_finite() {
        return format_general(value, precision);
    }
    let text = format!("{value:.precision$e}");
    let (mantissa, exponent) = text.split_once('e').unwrap_or((&text, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{mantissa}e{sign}{:02}", exponent.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_applies_once_and_right_aligns() {
        let mut stream = DrawOStream::new();
        stream.set_width(5).put_int(42).put_int(1);
        assert_eq!(stream.text(), "   421");
    }

    #[test]
    fn scientific_and_precision() {
        let mut stream = DrawOStream::new();
        stream
            .set_precision(15)
            .set_scientific(true)
            .set_width(30)
            .put_real(3.0);
        assert_eq!(stream.text(), "         3.000000000000000e+00");
        stream.take_text();
        stream.set_scientific(false).put_real(1.0 / 3.0);
        assert_eq!(stream.text(), "0.333333333333333");
    }
}
