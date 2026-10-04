//! Differential public wrappers for the defined, bounded `Standard_CString` format subset.
use rustycad_tkernel::standard::c_string::{
    FormatArg, fprintf_to, printf_to, sprintf_into, sprintf_into_slice, vsprintf_into,
    vsprintf_into_slice,
};

const NATIVE_ROWS: &str = include_str!("../../../tools/oracle/tkernel_c_string_format.expected");
type FixedFormatter = for<'a> fn(&mut [u8], &[u8], &[FormatArg<'a>]) -> std::io::Result<usize>;

#[test]
fn standard_c_string_sprintf_matches_native_fixture() {
    assert_buffer_row("sprintf_integer", format_integer());
    assert_buffer_row("sprintf_float", format_float());
    assert_buffer_row("sprintf_text", format_text());
    assert_buffer_row("sprintf_c_text", format_c_text());
    assert_buffer_row("sprintf_unsigned", format_unsigned());
    assert_buffer_row("sprintf_uppercase", format_uppercase());
    assert_buffer_row("sprintf_star", format_star_width_precision());
    assert_buffer_row("sprintf_char", format_char());
    assert_buffer_row("vsprintf", format_variadic_slice());
}

#[test]
fn standard_c_string_printf_and_fprintf_write_native_bytes() {
    assert_writer_row("printf", write_printf());
    assert_writer_row("fprintf", write_fprintf());
}

#[test]
fn standard_c_string_fixed_buffers_preserve_native_suffix() {
    assert_fixed_row("sprintf_slice_tail", sprintf_fixed_buffer());
    assert_fixed_row("vsprintf_slice_tail", vsprintf_fixed_buffer());
}

#[test]
fn standard_c_string_fixed_buffer_rejects_short_capacity_without_mutation() {
    let mut output = [0x5a; 6];
    let original = output;
    let error = sprintf_into_slice(&mut output, b"%c:%04d", &fixed_args()).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(error.to_string().contains("7 bytes"));
    assert_eq!(output, original);
}

#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
#[test]
fn standard_c_string_linux_long_lengths_match_native_extremes() {
    assert_buffer_row("sprintf_long_lengths", format_long_integer_lengths());
}

#[test]
fn standard_c_string_long_aliases_follow_target_c_width() {
    let expected = format!(
        "{}|{}|{}",
        std::ffi::c_long::MIN,
        std::ffi::c_long::MAX,
        std::ffi::c_ulong::MAX
    );
    let actual = format_buffer(
        b"%ld|%ld|%lu",
        &[
            FormatArg::Long(std::ffi::c_long::MIN),
            FormatArg::Long(std::ffi::c_long::MAX),
            FormatArg::ULong(std::ffi::c_ulong::MAX),
        ],
    );
    assert_eq!(&actual.0[..actual.0.len() - 1], expected.as_bytes());
    assert_eq!(actual.1, expected.len());
}

#[test]
fn standard_c_string_long_long_lengths_match_native_extremes() {
    assert_buffer_row("sprintf_long_long_lengths", format_long_long_lengths());
}

#[test]
fn standard_c_string_formatter_rejects_unsupported_and_mismatched_arguments() {
    assert_format_error(b"%*d", &[FormatArg::Int(3)]);
    assert_format_error(b"%d", &[FormatArg::Text(b"three")]);
    assert_format_error(b"%u", &[FormatArg::Int(3)]);
    assert_format_error(b"%5%", &[]);
    assert_format_error(b"%l%", &[]);
    assert_format_error(b"%*d", &[FormatArg::UInt(3), FormatArg::Int(4)]);
    assert_format_error(
        b"%.*s",
        &[FormatArg::Text(b"bad"), FormatArg::Text(b"value")],
    );
    assert_format_error(b"%.1c", &[FormatArg::Int(65)]);
    assert_format_error(b"%*d", &[FormatArg::Int(4)]);
    assert_format_error(b"%+c", &[FormatArg::Int(65)]);
    assert_format_error(b"%ld", &[FormatArg::Int(7)]);
    assert_format_error(b"%lld", &[FormatArg::Long(7)]);
    assert_format_error(b"%lu", &[FormatArg::UInt(7)]);
    assert_format_error(b"%llu", &[FormatArg::ULong(7)]);
}

fn format_integer() -> (Vec<u8>, usize) {
    format_buffer(
        b"[%+06d|%#o|%#08X]",
        &[FormatArg::Int(-42), FormatArg::UInt(8), FormatArg::UInt(42)],
    )
}

fn format_float() -> (Vec<u8>, usize) {
    format_buffer(
        b"float=%+08.2f;exp=%.3e;gen=%.4g",
        &[
            FormatArg::Real(-1.25),
            FormatArg::Real(0.0000125),
            FormatArg::Real(12345.67),
        ],
    )
}

fn format_text() -> (Vec<u8>, usize) {
    format_buffer(
        b"%8.3s|%-5s|%%",
        &[FormatArg::Text(b"kernel"), FormatArg::Text(b"CAD")],
    )
}

fn format_c_text() -> (Vec<u8>, usize) {
    format_buffer(b"%s", &[FormatArg::Text(b"left\0ignored")])
}

fn format_unsigned() -> (Vec<u8>, usize) {
    format_buffer(
        b"%u|%#x",
        &[FormatArg::UInt(u32::MAX), FormatArg::UInt(0xdead_beef)],
    )
}

fn format_uppercase() -> (Vec<u8>, usize) {
    format_buffer(
        b"%i|%.1F|%.2E|%.4G",
        &[
            FormatArg::Int(-12),
            FormatArg::Real(1.25),
            FormatArg::Real(0.0000125),
            FormatArg::Real(12345.67),
        ],
    )
}

fn format_star_width_precision() -> (Vec<u8>, usize) {
    format_buffer(
        b"%*.*f|%*d|%.*s",
        &[
            FormatArg::Int(-8),
            FormatArg::Int(2),
            FormatArg::Real(1.25),
            FormatArg::Int(5),
            FormatArg::Int(-7),
            FormatArg::Int(-2),
            FormatArg::Text(b"native"),
        ],
    )
}

fn format_char() -> (Vec<u8>, usize) {
    format_buffer(
        b"%cX%3c|%-4c",
        &[FormatArg::Int(0), FormatArg::Int(65), FormatArg::Int(-1)],
    )
}

fn format_variadic_slice() -> (Vec<u8>, usize) {
    let mut output = Vec::new();
    let count = vsprintf_into(
        &mut output,
        b"%04d:%s",
        &[FormatArg::Int(7), FormatArg::Text(b"edge")],
    )
    .unwrap();
    (output, count)
}

fn format_long_integer_lengths() -> (Vec<u8>, usize) {
    format_buffer(
        b"%ld|%ld|%lu|%#018lx",
        &[
            FormatArg::Long(std::ffi::c_long::MIN),
            FormatArg::Long(std::ffi::c_long::MAX),
            FormatArg::ULong(std::ffi::c_ulong::MAX),
            FormatArg::ULong(0xdead_beef),
        ],
    )
}

fn format_long_long_lengths() -> (Vec<u8>, usize) {
    format_buffer(
        b"%lld|%lld|%.20llu|%#020llX",
        &[
            FormatArg::LongLong(i64::MIN),
            FormatArg::LongLong(i64::MAX),
            FormatArg::ULongLong(u64::MAX),
            FormatArg::ULongLong(42),
        ],
    )
}

fn sprintf_fixed_buffer() -> (Vec<u8>, usize) {
    format_fixed_buffer(sprintf_into_slice)
}

fn vsprintf_fixed_buffer() -> (Vec<u8>, usize) {
    format_fixed_buffer(vsprintf_into_slice)
}

fn format_fixed_buffer(format: FixedFormatter) -> (Vec<u8>, usize) {
    let mut output = vec![0xa5; 16];
    let count = format(&mut output, b"%c:%04d", &fixed_args()).unwrap();
    (output, count)
}

fn fixed_args() -> [FormatArg<'static>; 2] {
    [FormatArg::Int(0), FormatArg::Int(7)]
}

fn format_buffer(format: &[u8], args: &[FormatArg<'_>]) -> (Vec<u8>, usize) {
    let mut output = Vec::new();
    let count = sprintf_into(&mut output, format, args).unwrap();
    (output, count)
}

fn write_printf() -> (Vec<u8>, usize) {
    let mut output = Vec::new();
    let count = printf_to(&mut output, b"printf:%04d\n", &[FormatArg::Int(7)]).unwrap();
    (output, count)
}

fn write_fprintf() -> (Vec<u8>, usize) {
    let mut output = Vec::new();
    let count = fprintf_to(&mut output, b"fprintf:%#x\n", &[FormatArg::UInt(42)]).unwrap();
    (output, count)
}

fn assert_buffer_row(label: &str, actual: (Vec<u8>, usize)) {
    let (buffer, count) = actual;
    assert_native_row(label, count, &buffer[..buffer.len() - 1]);
    assert_eq!(
        buffer.last(),
        Some(&0),
        "Sprintf buffer needs native NUL terminator"
    );
}

fn assert_writer_row(label: &str, actual: (Vec<u8>, usize)) {
    let (bytes, count) = actual;
    assert_native_row(label, count, &bytes);
}

fn assert_fixed_row(label: &str, actual: (Vec<u8>, usize)) {
    let (buffer, count) = actual;
    let row = native_row(label);
    let mut fields = row.split_ascii_whitespace();
    assert_eq!(fields.next(), Some(label));
    assert_eq!(fields.next().unwrap().parse::<usize>().unwrap(), count);
    assert_eq!(decode_hex(fields.next().unwrap()), buffer);
}

fn assert_native_row(label: &str, count: usize, bytes: &[u8]) {
    let row = native_row(label);
    let mut fields = row.split_ascii_whitespace();
    assert_eq!(fields.next(), Some(label));
    assert_eq!(fields.next().unwrap().parse::<usize>().unwrap(), count);
    assert_eq!(decode_hex(fields.next().unwrap()), bytes);
}

fn native_row(label: &str) -> &str {
    NATIVE_ROWS
        .lines()
        .find(|row| row.starts_with(label))
        .unwrap()
}

fn assert_format_error(format: &[u8], args: &[FormatArg<'_>]) {
    let mut output = Vec::new();
    output.extend_from_slice(b"prior");
    assert!(sprintf_into(&mut output, format, args).is_err());
    assert_eq!(output, b"prior");
}

fn decode_hex(hex: &str) -> Vec<u8> {
    let (pairs, rest) = hex.as_bytes().as_chunks::<2>();
    assert!(rest.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
