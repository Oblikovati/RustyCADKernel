//! Binary64 `%a` layout matched to OCCT's Linux C-locale printf forwarding.
//! Rounding uses nearest/even, like the default native floating-point environment.
use super::{Spec, pad};

pub(super) fn format(spec: &Spec, magnitude: f64, mut prefix: Vec<u8>) -> Vec<u8> {
    let (significand, exponent) = binary_parts(magnitude);
    let mut digits = mantissa(significand, spec.precision, spec.alternate);
    digits.extend_from_slice(format!("p{exponent:+}").as_bytes());
    let upper = spec.conversion == b'A';
    prefix.extend_from_slice(if upper { b"0X" } else { b"0x" });
    if upper {
        digits.make_ascii_uppercase();
    }
    pad(spec, prefix, digits, true)
}

fn binary_parts(magnitude: f64) -> (u64, i32) {
    let bits = magnitude.to_bits();
    let exponent_bits = (bits >> 52) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    if exponent_bits != 0 {
        return (fraction | (1_u64 << 52), exponent_bits - 1023);
    }
    // glibc keeps subnormal values at the minimum normal exponent, with a leading zero.
    (fraction, if fraction == 0 { 0 } else { -1022 })
}

fn mantissa(significand: u64, precision: Option<usize>, alternate: bool) -> Vec<u8> {
    let rounded = round_significand(significand, precision.unwrap_or(13));
    let full = format!("{rounded:014x}");
    let count = precision.unwrap_or_else(|| full.trim_end_matches('0').len().saturating_sub(1));
    let mut digits = vec![full.as_bytes()[0]];
    if count != 0 || alternate {
        digits.push(b'.');
    }
    digits.extend_from_slice(&full.as_bytes()[1..1 + count.min(13)]);
    digits.resize(digits.len() + count.saturating_sub(13), b'0');
    digits
}

fn round_significand(significand: u64, precision: usize) -> u64 {
    if precision >= 13 {
        return significand;
    }
    let discarded = 52 - 4 * precision;
    let retained = significand >> discarded;
    let remainder = significand & ((1_u64 << discarded) - 1);
    let half = 1_u64 << (discarded - 1);
    let increment = remainder > half || (remainder == half && retained & 1 != 0);
    (retained + u64::from(increment)) << discarded
}
