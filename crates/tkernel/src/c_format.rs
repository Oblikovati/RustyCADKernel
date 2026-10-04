//! Shared C-style formatting engine used by Message helpers and safe `Standard_CString` wrappers.
//!
//! Message_Msg::Arg formats are user data (`%-10.3f`, `%05d`, `%s`) passed to native
//! Sprintf. Message::FillTime and OSD timing output also depend on its C-locale layout.
//!
//! It implements literal and dynamic width/precision, `c_format` numeric/text conversions,
//! and glibc layout rules. The public wrappers validate a narrower defined subset before render.
mod float;
mod integer;
mod validation;

/// One typed argument for the supported C `printf` conversions.
#[derive(Clone, Copy, Debug)]
pub enum FormatArg<'a> {
    /// An `int` conversion, dynamic width/precision, or `%c` argument.
    Int(i32),
    /// An `unsigned int` argument.
    UInt(u32),
    /// A platform C `long` argument, with the target's native width.
    Long(std::ffi::c_long),
    /// A platform C `unsigned long` argument, with the target's native width.
    ULong(std::ffi::c_ulong),
    /// A C `long long` argument.
    LongLong(i64),
    /// A C `unsigned long long` argument.
    ULongLong(u64),
    /// A `double` argument.
    Real(f64),
    /// A byte string used as a `const char*`, truncated at the first NUL.
    Text(&'a [u8]),
}
pub(crate) use FormatArg as CArg;

/// A format directive outside the faithfully supported argument and conversion subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FormatError {
    pub(crate) offset: usize,
    pub(crate) conversion: u8,
    pub(crate) expected: &'static str,
}

/// Parsed flags, width, precision and length modifier of one conversion.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Spec {
    pub(crate) left: bool,
    pub(crate) plus: bool,
    pub(crate) space: bool,
    pub(crate) alternate: bool,
    pub(crate) zero: bool,
    pub(crate) width: usize,
    pub(crate) dynamic_width: bool,
    pub(crate) precision: Option<usize>,
    pub(crate) dynamic_precision: bool,
    /// Length modifier: -2 `hh`, -1 `h`, 0 none, 1 `l`, 2 `ll`.
    pub(crate) length: i8,
    pub(crate) conversion: u8,
}

/// Formats `format` with `args` like C `sprintf`; `c_sprintf(b"%02d", &[CArg::Int(5)]) == b"05"`.
pub(crate) fn c_sprintf(format: &[u8], args: &[CArg<'_>]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut arguments = args.iter();
    let mut index = 0;
    while index < format.len() {
        if format[index] != b'%' {
            output.push(format[index]);
            index += 1;
            continue;
        }
        let (spec, end) = parse_spec(format, index + 1);
        index = end;
        let spec = resolve_dimensions(spec, &mut arguments);
        render(&spec, &mut arguments, &format[..end], &mut output);
    }
    output
}

pub(crate) use validation::checked_sprintf;

fn render(
    spec: &Spec,
    arguments: &mut std::slice::Iter<'_, CArg<'_>>,
    consumed: &[u8],
    output: &mut Vec<u8>,
) {
    let body = match spec.conversion {
        b'%' => vec![b'%'],
        b'd' | b'i' | b'o' | b'u' | b'x' | b'X' => integer::format(spec, arguments.next()),
        b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => float::format(spec, arguments.next()),
        b's' => text(spec, arguments.next()),
        b'c' => character(spec, arguments.next()),
        // An invalid conversion is undefined behavior in C; glibc echoes the directive.
        _ => consumed[consumed.iter().rposition(|&byte| byte == b'%').unwrap_or(0)..].to_vec(),
    };
    output.extend_from_slice(&body);
}

pub(super) fn parse_spec(format: &[u8], mut index: usize) -> (Spec, usize) {
    let mut spec = Spec::default();
    index = parse_flags(format, index, &mut spec);
    index = parse_width(format, index, &mut spec);
    if format.get(index) == Some(&b'.') {
        index = parse_precision(format, index + 1, &mut spec);
    }
    (spec.length, index) = length_modifier(format, index);
    spec.conversion = format.get(index).copied().unwrap_or(0);
    (spec, (index + 1).min(format.len()))
}

fn parse_flags(format: &[u8], mut index: usize, spec: &mut Spec) -> usize {
    while let Some(&flag) = format.get(index) {
        match flag {
            b'-' => spec.left = true,
            b'+' => spec.plus = true,
            b' ' => spec.space = true,
            b'#' => spec.alternate = true,
            b'0' => spec.zero = true,
            _ => break,
        }
        index += 1;
    }
    index
}

fn parse_width(format: &[u8], index: usize, spec: &mut Spec) -> usize {
    if format.get(index) == Some(&b'*') {
        spec.dynamic_width = true;
        return index + 1;
    }
    let (width, end) = digits(format, index);
    spec.width = width;
    end
}

fn parse_precision(format: &[u8], index: usize, spec: &mut Spec) -> usize {
    if format.get(index) == Some(&b'*') {
        spec.dynamic_precision = true;
        return index + 1;
    }
    let (precision, end) = digits(format, index);
    spec.precision = Some(precision);
    end
}

fn resolve_dimensions(mut spec: Spec, arguments: &mut std::slice::Iter<'_, FormatArg<'_>>) -> Spec {
    if spec.dynamic_width {
        spec = apply_dynamic_width(spec, next_int(arguments));
    }
    if spec.dynamic_precision {
        spec = apply_dynamic_precision(spec, next_int(arguments));
    }
    spec
}

fn next_int(arguments: &mut std::slice::Iter<'_, FormatArg<'_>>) -> i32 {
    match arguments.next() {
        Some(FormatArg::Int(value)) => *value,
        _ => 0,
    }
}

fn apply_dynamic_width(mut spec: Spec, width: i32) -> Spec {
    spec.dynamic_width = false;
    spec.width = width.unsigned_abs() as usize;
    spec.left |= width < 0;
    spec
}

fn apply_dynamic_precision(mut spec: Spec, precision: i32) -> Spec {
    spec.dynamic_precision = false;
    spec.precision = (precision >= 0).then_some(precision as usize);
    spec
}

fn character(spec: &Spec, argument: Option<&CArg<'_>>) -> Vec<u8> {
    let byte = match argument {
        Some(CArg::Int(value)) => *value as u8,
        _ => 0,
    };
    pad(spec, Vec::new(), vec![byte], false)
}

fn digits(format: &[u8], mut index: usize) -> (usize, usize) {
    let mut value: usize = 0;
    while let Some(digit) = format.get(index).filter(|byte| byte.is_ascii_digit()) {
        value = value
            .saturating_mul(10)
            .saturating_add(usize::from(digit - b'0'));
        index += 1;
    }
    (value, index)
}

fn length_modifier(format: &[u8], index: usize) -> (i8, usize) {
    match (format.get(index), format.get(index + 1)) {
        (Some(b'h'), Some(b'h')) => (-2, index + 2),
        (Some(b'h'), _) => (-1, index + 1),
        (Some(b'l'), Some(b'l')) => (2, index + 2),
        (Some(b'l'), _) => (1, index + 1),
        _ => (0, index),
    }
}

fn text(spec: &Spec, argument: Option<&CArg<'_>>) -> Vec<u8> {
    let bytes: &[u8] = match argument {
        Some(CArg::Text(bytes)) => bytes,
        _ => b"(null)",
    };
    let bytes = &bytes[..bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len())];
    let shown = spec
        .precision
        .map_or(bytes, |limit| &bytes[..limit.min(bytes.len())]);
    pad(spec, Vec::new(), shown.to_vec(), false)
}

/// Joins sign/prefix and digits, padding to the width with spaces or (numeric) zeros.
pub(crate) fn pad(spec: &Spec, prefix: Vec<u8>, digits: Vec<u8>, zero_allowed: bool) -> Vec<u8> {
    let length = prefix.len() + digits.len();
    let fill = spec.width.saturating_sub(length);
    if spec.left {
        return [prefix, digits, vec![b' '; fill]].concat();
    }
    if spec.zero && zero_allowed {
        return [prefix, vec![b'0'; fill], digits].concat();
    }
    [vec![b' '; fill], prefix, digits].concat()
}

/// The `+`/space sign prefix of a signed conversion.
pub(crate) fn sign_prefix(spec: &Spec, negative: bool) -> Vec<u8> {
    match (negative, spec.plus, spec.space) {
        (true, _, _) => b"-".to_vec(),
        (false, true, _) => b"+".to_vec(),
        (false, false, true) => b" ".to_vec(),
        _ => Vec::new(),
    }
}

/// `printf("%.*e", precision, value)`: `OSD::RealToCString`; `format_e(1.5, 2) == "1.50e+00"`.
pub(crate) fn format_e(value: f64, precision: usize) -> String {
    format_real(&format!("%.{precision}e"), value)
}

/// `printf("%.*g", precision, value)`, the default `ostream << double` spelling at
/// `precision(N)`; `format_g(5.5, 12) == "5.5"`.
pub(crate) fn format_g(value: f64, precision: usize) -> String {
    format_real(&format!("%.{precision}g"), value)
}

fn format_real(format: &str, value: f64) -> String {
    let bytes: Vec<u8> = c_sprintf(format.as_bytes(), &[CArg::Real(value)]);
    String::from_utf8(bytes).expect("C float formatting is ASCII")
}

#[cfg(test)]
mod real_tests {
    use super::*;
    #[test]
    fn c_spellings_of_finite_and_special_values() {
        assert_eq!(format_e(1.5, 2), "1.50e+00");
        assert_eq!(format_e(-1e-300, 1), "-1.0e-300");
        assert_eq!(format_e(-0.0, 0), "-0e+00");
        assert_eq!(format_e(f64::NAN, 17), "nan");
        assert_eq!(format_e(f64::NEG_INFINITY, 17), "-inf");
        assert_eq!(format_g(5.5, 12), "5.5");
        assert_eq!(format_g(0.0, 12), "0");
        assert_eq!(format_g(1e12, 12), "1e+12");
        assert_eq!(format_g(123456789012.0, 12), "123456789012");
        assert_eq!(format_g(0.0001, 12), "0.0001");
        assert_eq!(format_g(0.00001, 12), "1e-05");
        assert_eq!(format_g(0.125, 2), "0.12");
    }
}
