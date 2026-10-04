//! Floating conversions `f F e E g G a A` of the C `sprintf` subset.
use super::{CArg, Spec, pad, sign_prefix};

/// Formats one floating conversion; a missing or non-real argument formats as 0.0.
pub(super) fn format(spec: &Spec, argument: Option<&CArg<'_>>) -> Vec<u8> {
    let value = match argument {
        Some(CArg::Real(value)) => *value,
        _ => 0.0,
    };
    let prefix = sign_prefix(spec, value.is_sign_negative());
    if !value.is_finite() {
        return nonfinite(spec, value, prefix);
    }
    finite(spec, value.abs(), prefix)
}

fn nonfinite(spec: &Spec, value: f64, prefix: Vec<u8>) -> Vec<u8> {
    // glibc pads infinities and NaNs with spaces even under the `0` flag.
    let word = if value.is_nan() { "nan" } else { "inf" };
    let digits = cased(
        word.as_bytes().to_vec(),
        spec.conversion.is_ascii_uppercase(),
    );
    pad(spec, prefix, digits, false)
}

fn finite(spec: &Spec, magnitude: f64, prefix: Vec<u8>) -> Vec<u8> {
    if matches!(spec.conversion, b'a' | b'A') {
        return super::hexadecimal::format(spec, magnitude, prefix);
    }
    let digits = match spec.conversion.to_ascii_lowercase() {
        b'f' => fixed(magnitude, spec.precision.unwrap_or(6), spec.alternate),
        b'e' => exponential(magnitude, spec.precision.unwrap_or(6), spec.alternate),
        _ => general(magnitude, spec),
    };
    pad(
        spec,
        prefix,
        cased(digits, spec.conversion.is_ascii_uppercase()),
        true,
    )
}

fn cased(mut bytes: Vec<u8>, upper: bool) -> Vec<u8> {
    if upper {
        bytes.make_ascii_uppercase();
    }
    bytes
}

fn fixed(magnitude: f64, precision: usize, alternate: bool) -> Vec<u8> {
    let mut text = format!("{magnitude:.precision$}").into_bytes();
    if alternate && precision == 0 {
        text.push(b'.');
    }
    text
}

/// Rust's exact `{:.Ne}` split into mantissa and decimal exponent.
fn scientific(magnitude: f64, precision: usize) -> (String, i32) {
    let text = format!("{magnitude:.precision$e}");
    let (mantissa, exponent) = text.split_once('e').expect("Rust exponent form");
    (
        mantissa.to_owned(),
        exponent.parse().expect("decimal exponent"),
    )
}

fn exponential(magnitude: f64, precision: usize, alternate: bool) -> Vec<u8> {
    let (mut mantissa, exponent) = scientific(magnitude, precision);
    if alternate && precision == 0 {
        mantissa.push('.');
    }
    with_exponent(mantissa, exponent)
}

fn with_exponent(mantissa: String, exponent: i32) -> Vec<u8> {
    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{mantissa}e{sign}{:02}", exponent.unsigned_abs()).into_bytes()
}

/// `%g`: the style follows the exponent after rounding to `P` significant digits (C11 7.21.6.1).
///
/// glibc quirk kept: when the fixed-style candidate (chosen from the unrounded exponent
/// `P - 1`) rounds up to exponent `P`, glibc switches to the exponent style but keeps the
/// candidate's zero fraction digits, so `%#g` of 999999.875 prints `1.e+06`.
fn general(magnitude: f64, spec: &Spec) -> Vec<u8> {
    let significant = spec.precision.unwrap_or(6).max(1);
    let (_, exponent) = scientific(magnitude, significant - 1);
    let unrounded = if magnitude == 0.0 {
        0
    } else {
        scientific_shortest_exponent(magnitude)
    };
    if spec.alternate && unrounded == significant as i32 - 1 && exponent == significant as i32 {
        return with_exponent("1.".to_owned(), exponent);
    }
    let use_fixed = exponent >= -4 && exponent < significant as i32;
    if use_fixed {
        let decimals = (significant as i32 - 1 - exponent) as usize;
        let text = format!("{magnitude:.decimals$}");
        return trimmed(text, spec.alternate).into_bytes();
    }
    let (mantissa, exponent) = scientific(magnitude, significant - 1);
    with_exponent(trimmed(mantissa, spec.alternate), exponent)
}

/// Decimal exponent of the exact value: the shortest round-trip digits never cross a
/// power of ten that the value itself does not reach.
fn scientific_shortest_exponent(magnitude: f64) -> i32 {
    let text = format!("{magnitude:e}");
    let (_, exponent) = text.split_once('e').expect("Rust exponent form");
    exponent.parse().expect("decimal exponent")
}

/// Drops trailing fractional zeros and a bare point; `#` keeps them and forces the point.
fn trimmed(mut text: String, alternate: bool) -> String {
    if alternate && !text.contains('.') {
        text.push('.');
    }
    if alternate || !text.contains('.') {
        return text;
    }
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}
