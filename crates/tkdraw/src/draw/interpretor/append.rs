//! The value overloads of `Draw_Interpretor::Append` / `operator<<`.
use rustycad_tkernel::storage::format_general;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};

/// A value the interpreter result can be extended with, formatted as the C++ overload
/// for its type: `%d` for `int`, `%zu` for `size_t`, `%.17g` for `double`, UTF-8 for an
/// extended string, the bytes of an ASCII string.
///
/// ```
/// use rustycad_tkdraw::draw::DrawAppend;
/// assert_eq!(0.1.draw_text(), "0.10000000000000001");
/// assert_eq!((-3).draw_text(), "-3");
/// ```
pub trait DrawAppend {
    /// The text `Append` adds.
    fn draw_text(&self) -> String;
}

impl DrawAppend for &str {
    fn draw_text(&self) -> String {
        (*self).to_string()
    }
}

impl DrawAppend for String {
    fn draw_text(&self) -> String {
        self.clone()
    }
}

impl DrawAppend for &String {
    fn draw_text(&self) -> String {
        (*self).clone()
    }
}

impl DrawAppend for &AsciiString {
    fn draw_text(&self) -> String {
        String::from_utf8_lossy(self.as_bytes()).into_owned()
    }
}

/// `Append(const TCollection_ExtendedString&)`: converted to UTF-8 for Tcl (8.1+).
impl DrawAppend for &ExtendedString {
    fn draw_text(&self) -> String {
        self.to_utf8()
    }
}

/// `Append(const int)`: `Sprintf(c, "%d", i)`.
impl DrawAppend for i32 {
    fn draw_text(&self) -> String {
        self.to_string()
    }
}

/// `Append(const size_t)`: `Sprintf(c, "%zu", n)`.
impl DrawAppend for usize {
    fn draw_text(&self) -> String {
        self.to_string()
    }
}

/// `Append(const double)`: `Sprintf(s, "%.17g", r)`.
impl DrawAppend for f64 {
    fn draw_text(&self) -> String {
        format_general(*self, 17)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_use_the_c_formats() {
        assert_eq!(1.0.draw_text(), "1");
        assert_eq!(1e22.draw_text(), "1e+22");
        assert_eq!(f64::INFINITY.draw_text(), "inf");
        assert_eq!(usize::MAX.draw_text(), "18446744073709551615");
        assert_eq!(i32::MIN.draw_text(), "-2147483648");
    }

    #[test]
    fn strings_append_their_text() {
        assert_eq!("a b".draw_text(), "a b");
        assert_eq!((&AsciiString::new("ascii")).draw_text(), "ascii");
        let extended = ExtendedString::from_utf8("é");
        assert_eq!((&extended).draw_text(), "é");
    }
}
