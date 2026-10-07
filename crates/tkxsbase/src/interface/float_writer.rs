// Rust port of DataExchange/TKXSBase/Interface/Interface_FloatWriter (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_FloatWriter`: writes reals with a main C format, a second format for a
//! range of magnitudes, and an optional suppression of the trailing zeros.
use rustycad_tkernel::standard::{
    Failure,
    c_string::{FormatArg, sprintf_into},
};

/// `Interface_FloatWriter`; the formats are C `printf` formats of one `double`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceFloatWriter;
/// let writer = InterfaceFloatWriter::new(0);
/// let mut text = Vec::new();
/// assert_eq!(writer.write(2.5, &mut text).unwrap(), 3);
/// assert_eq!(text, b"2.5");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceFloatWriter {
    themainform: String,
    therange1: f64,
    therange2: f64,
    therangeform: String,
    thezerosup: bool,
}

/// `Options`: zero suppression, whether the range format applies, and the range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterfaceFloatWriterOptions {
    /// `zerosup`.
    pub zerosup: bool,
    /// `range`: R2 >= R1 >= 0.
    pub range: bool,
    /// `R1`.
    pub r1: f64,
    /// `R2`.
    pub r2: f64,
}

impl Default for InterfaceFloatWriter {
    fn default() -> Self {
        Self::new(0)
    }
}

impl InterfaceFloatWriter {
    /// `Interface_FloatWriter(chars = 0)`: the defaults of `SetDefaults(chars)`.
    pub fn new(chars: i32) -> Self {
        let mut writer = Self {
            themainform: String::new(),
            therange1: 0.0,
            therange2: 0.0,
            therangeform: String::new(),
            thezerosup: false,
        };
        writer.set_defaults(chars);
        writer
    }
    /// `SetFormat(form, reset = true)`: with `reset`, no range format and no zero
    /// suppression.
    pub fn set_format(&mut self, form: &str, reset: bool) {
        self.themainform = form.to_owned();
        if reset {
            self.therange1 = 0.0;
            self.therange2 = 0.0;
            self.thezerosup = false;
        }
    }
    /// `SetFormatForRange(form, R1, R2)`.
    pub fn set_format_for_range(&mut self, form: &str, r1: f64, r2: f64) {
        self.therangeform = form.to_owned();
        self.therange1 = r1;
        self.therange2 = r2;
    }
    /// `SetZeroSuppress(mode)`.
    pub fn set_zero_suppress(&mut self, mode: bool) {
        self.thezerosup = mode;
    }
    /// `SetDefaults(chars = 0)`: "%E" and "%f" (or `"%<chars+2>.<chars>E/f"`), range
    /// [0.1, 1000[, zero suppression.
    pub fn set_defaults(&mut self, chars: i32) {
        if chars <= 0 {
            self.themainform = String::from("%E");
            self.therangeform = String::from("%f");
        } else {
            self.themainform = format!("%{}.{chars}E", chars + 2);
            self.therangeform = format!("%{}.{chars}f", chars + 2);
        }
        self.therange1 = 0.1;
        self.therange2 = 1000.0;
        self.thezerosup = true;
    }
    /// `Options(zerosup, range, R1, R2)`.
    pub fn options(&self) -> InterfaceFloatWriterOptions {
        InterfaceFloatWriterOptions {
            zerosup: self.thezerosup,
            range: self.therange2 >= self.therange1 && self.therange1 >= 0.0,
            r1: self.therange1,
            r2: self.therange2,
        }
    }
    /// `MainFormat`.
    pub fn main_format(&self) -> &str {
        &self.themainform
    }
    /// `FormatForRange`.
    pub fn format_for_range(&self) -> &str {
        &self.therangeform
    }
    /// `Write(val, text)`: writes `val` into `text` (without NUL) and returns its length.
    pub fn write(&self, val: f64, text: &mut Vec<u8>) -> Result<i32, Failure> {
        Self::convert(
            val,
            text,
            self.thezerosup,
            self.therange1,
            self.therange2,
            &self.themainform,
            &self.therangeform,
        )
    }
    /// `Convert(val, text, zerosup, R1, R2, mainform, rangeform)`: the range format for
    /// R1 <= |val| < R2, else the main one; with `zerosup`, the trailing zeros of the
    /// mantissa and an exponent "E+00" are removed. A format the C formatter refuses is
    /// a `Standard_Failure`.
    pub fn convert(
        val: f64,
        text: &mut Vec<u8>,
        zerosup: bool,
        r1: f64,
        r2: f64,
        mainform: &str,
        rangeform: &str,
    ) -> Result<i32, Failure> {
        let in_range = (val >= r1 && val < r2) || (val <= -r1 && val > -r2);
        let form = if in_range { rangeform } else { mainform };
        let mut buffer = Vec::new();
        sprintf_into(&mut buffer, form.as_bytes(), &[FormatArg::Real(val)])
            .map_err(|e| Failure::new(&format!("Interface_FloatWriter: format {form:?}: {e}")))?;
        // The native text is a caller buffer: room past the NUL for the edits below.
        buffer.resize(buffer.len().max(16) + 8, 0);
        if zerosup {
            suppress_zeros(&mut buffer);
        }
        let length = buffer.iter().position(|b| *b == 0).unwrap_or(buffer.len());
        buffer.truncate(length);
        *text = buffer;
        Ok(length as i32)
    }
}

/// The zero suppression of `Convert`, byte for byte on the C string.
fn suppress_zeros(text: &mut [u8]) {
    let mut lxp = [0u8; 5];
    let mut i0 = 0;
    for i in 0..16 {
        i0 = i;
        if text[i] == b'e' || text[i] == b'E' {
            lxp = [b'E', text[i + 1], text[i + 2], text[i + 3], text[i + 4]];
            if lxp[1] == b'+' && lxp[2] == b'0' && lxp[3] == b'0' && lxp[4] == 0 {
                lxp[0] = 0;
            }
            text[i] = 0;
        }
        if text[i] == 0 {
            break;
        }
    }
    let mut j0 = 0;
    for j in (0..i0).rev() {
        j0 = j;
        if text[j] != b'0' {
            break;
        }
        text[j] = 0;
    }
    text[j0 + 1..j0 + 6].copy_from_slice(&lxp);
    text[j0 + 6] = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(writer: &InterfaceFloatWriter, val: f64) -> String {
        let mut text = Vec::new();
        writer.write(val, &mut text).unwrap();
        String::from_utf8(text).unwrap()
    }

    #[test]
    fn defaults_choose_the_format_by_magnitude() {
        let writer = InterfaceFloatWriter::default();
        assert_eq!(written(&writer, 12.0), "12.");
        assert_eq!(written(&writer, 0.05), "5.E-02");
        assert_eq!(written(&writer, 5000.0), "5.E+03");
        assert_eq!(written(&writer, -2.0), "-2.");
        assert_eq!(written(&writer, 0.0), "0.");
    }

    #[test]
    fn explicit_formats() {
        let mut writer = InterfaceFloatWriter::new(4);
        assert_eq!(writer.main_format(), "%6.4E");
        assert_eq!(writer.format_for_range(), "%6.4f");
        writer.set_format("%.3E", true);
        assert_eq!(written(&writer, 1.0), "1.000E+00");
        assert!(!writer.options().zerosup);
        writer.set_zero_suppress(true);
        writer.set_format_for_range("%g", 1.0, 2.0);
        assert_eq!(written(&writer, 1.5), "1.5");
        let options = writer.options();
        assert!(options.range && options.zerosup);
        assert_eq!((options.r1, options.r2), (1.0, 2.0));
    }
}
